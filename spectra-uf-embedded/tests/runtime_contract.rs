//! Integration contracts for Spectra embedded wiring.

use spectra_uf_embedded::{install_embedded_sqlite, SPECTRA_STORE_BASE_PATH_ENV};

static ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[tokio::test]
async fn install_embedded_sqlite_temp_paths_happy_path() {
    let _guard = ENV_LOCK.lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    std::env::set_var(SPECTRA_STORE_BASE_PATH_ENV, dir.path());
    let spectra = install_embedded_sqlite()
        .await
        .expect("install_embedded_sqlite");
    let _ = spectra.as_ref();
}

#[tokio::test]
async fn install_embedded_sqlite_parent_not_dir_sad() {
    let _guard = ENV_LOCK.lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    // "default" always exists among distinct store names; blocking its subdirectory with
    // a plain file reproduces a per-store open failure.
    let blocker = dir.path().join("default");
    std::fs::write(&blocker, b"x").expect("write");
    std::env::set_var(SPECTRA_STORE_BASE_PATH_ENV, dir.path());

    let Err(err) = install_embedded_sqlite().await else {
        panic!("default store's directory is a file, not a directory");
    };
    assert_ne!(err.to_string(), "");
}
