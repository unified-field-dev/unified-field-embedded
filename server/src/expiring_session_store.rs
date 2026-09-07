//! File-backed session store with periodic expired-row deletion.
//!
//! Keeps tower-sessions records as one JSON file per session id so a process
//! restart does not wipe logins. Root directory comes from `UF_SESSION_DIR`
//! (default `data/sessions`, mode `0700`; files `0600`). A host sweeper calls
//! [`ExpiredDeletion::delete_expired`] about once a minute.
//!
//! This store is for a single-process host. Prefer a shared SQL or Redis
//! session backend when multiple processes share one cookie domain.
//!
//! Disk IO runs on `spawn_blocking` so the Tokio worker is not blocked on
//! filesystem calls. Failures surface as [`tower_sessions::session_store::Error`];
//! there is no silent fallback to an in-memory store.
//!
//! Operator guide: repository `SECURITY.md` and `docs/deploy.md` (session and
//! CSRF sections).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use time::OffsetDateTime;
use tower_sessions::session_store::Error as StoreError;
use tower_sessions::session_store::{ExpiredDeletion, Result as StoreResult, SessionStore};
use tower_sessions::{session::Id, session::Record};

/// Durable session store: one JSON file per session id under `root`.
#[derive(Clone, Debug)]
pub struct FileSessionStore {
    root: PathBuf,
    /// Serialize directory mutations (create/save/delete/sweep).
    lock: Arc<Mutex<()>>,
}

impl FileSessionStore {
    /// Create the store rooted at `root`, creating the directory if needed.
    pub fn new(root: impl Into<PathBuf>) -> std::io::Result<Self> {
        let root = root.into();
        std::fs::create_dir_all(&root)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700));
        }
        Ok(Self {
            root,
            lock: Arc::new(Mutex::new(())),
        })
    }

    fn path_for(root: &Path, id: &Id) -> PathBuf {
        root.join(format!("{id}.json"))
    }

    fn is_active(expiry_date: OffsetDateTime) -> bool {
        expiry_date > OffsetDateTime::now_utc()
    }

    fn read_record(path: &Path) -> StoreResult<Option<Record>> {
        if !path.exists() {
            return Ok(None);
        }
        let bytes = std::fs::read(path).map_err(|e| StoreError::Backend(e.to_string()))?;
        let record: Record =
            serde_json::from_slice(&bytes).map_err(|e| StoreError::Decode(e.to_string()))?;
        Ok(Some(record))
    }

    fn write_record(path: &Path, record: &Record) -> StoreResult<()> {
        let bytes = serde_json::to_vec(record).map_err(|e| StoreError::Encode(e.to_string()))?;
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, &bytes).map_err(|e| StoreError::Backend(e.to_string()))?;
        std::fs::rename(&tmp, path).map_err(|e| StoreError::Backend(e.to_string()))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
        }
        Ok(())
    }

    async fn with_lock_blocking<T, F>(&self, op: F) -> StoreResult<T>
    where
        F: FnOnce(&Path) -> StoreResult<T> + Send + 'static,
        T: Send + 'static,
    {
        let root = self.root.clone();
        let lock = Arc::clone(&self.lock);
        tokio::task::spawn_blocking(move || {
            let _guard = lock
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            op(&root)
        })
        .await
        .map_err(|e| StoreError::Backend(format!("session store task join: {e}")))?
    }
}

#[async_trait]
impl SessionStore for FileSessionStore {
    async fn create(&self, record: &mut Record) -> StoreResult<()> {
        loop {
            let candidate = record.clone();
            let created = self
                .with_lock_blocking(move |root| {
                    let path = Self::path_for(root, &candidate.id);
                    if path.exists() {
                        return Ok(None);
                    }
                    Self::write_record(&path, &candidate)?;
                    Ok(Some(candidate.id))
                })
                .await?;
            match created {
                Some(id) => {
                    record.id = id;
                    return Ok(());
                }
                None => {
                    record.id = Id::default();
                }
            }
        }
    }

    async fn save(&self, record: &Record) -> StoreResult<()> {
        let record = record.clone();
        self.with_lock_blocking(move |root| {
            Self::write_record(&Self::path_for(root, &record.id), &record)
        })
        .await
    }

    async fn load(&self, session_id: &Id) -> StoreResult<Option<Record>> {
        let session_id = *session_id;
        self.with_lock_blocking(move |root| {
            let path = Self::path_for(root, &session_id);
            match Self::read_record(&path)? {
                Some(record) if Self::is_active(record.expiry_date) => Ok(Some(record)),
                Some(_) => {
                    let _ = std::fs::remove_file(&path);
                    Ok(None)
                }
                None => Ok(None),
            }
        })
        .await
    }

    async fn delete(&self, session_id: &Id) -> StoreResult<()> {
        let session_id = *session_id;
        self.with_lock_blocking(move |root| {
            let path = Self::path_for(root, &session_id);
            if path.exists() {
                std::fs::remove_file(&path).map_err(|e| StoreError::Backend(e.to_string()))?;
            }
            Ok(())
        })
        .await
    }
}

#[async_trait]
impl ExpiredDeletion for FileSessionStore {
    async fn delete_expired(&self) -> StoreResult<()> {
        self.with_lock_blocking(|root| {
            let entries =
                std::fs::read_dir(root).map_err(|e| StoreError::Backend(e.to_string()))?;
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("json") {
                    continue;
                }
                if let Ok(Some(record)) = Self::read_record(&path) {
                    if !Self::is_active(record.expiry_date) {
                        let _ = std::fs::remove_file(&path);
                    }
                }
            }
            Ok(())
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Duration;
    use tower_sessions::session::Record;

    #[tokio::test]
    async fn file_store_round_trip_and_expiry_sweep() {
        let dir = std::env::temp_dir().join(format!("uf-sess-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let store = FileSessionStore::new(&dir).expect("store");
        let mut record = Record {
            id: Id::default(),
            data: Default::default(),
            expiry_date: OffsetDateTime::now_utc() + Duration::minutes(30),
        };
        store.create(&mut record).await.expect("create");
        let loaded = store.load(&record.id).await.expect("load");
        assert_eq!(loaded.as_ref().map(|r| r.id), Some(record.id));

        let mut expired = Record {
            id: Id::default(),
            data: Default::default(),
            expiry_date: OffsetDateTime::now_utc() - Duration::minutes(1),
        };
        store.create(&mut expired).await.expect("create expired");
        store.delete_expired().await.expect("sweep");
        assert!(store
            .load(&expired.id)
            .await
            .expect("load expired")
            .is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
