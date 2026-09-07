//! Integration contracts for Photon embedded wiring.

use std::sync::Arc;

use photon_uf_embedded::{build_runtime, SQLITE_PATH_ENV};
use valence::{Error, Result, Valence, ValenceFactory};

static ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

struct StubValenceFactory;

impl ValenceFactory for StubValenceFactory {
    fn build(&self, _actor_json: &serde_json::Value) -> Result<Valence> {
        Err(Error::Internal("stub valence factory".into()))
    }
}

#[tokio::test]
async fn build_runtime_temp_sqlite_happy_path() {
    let _guard = ENV_LOCK.lock().await;
    std::env::set_var(
        "PHOTON_TRANSPORT_KEY",
        "cGhvdG9uLWRldi10cmFuc3BvcnQta2V5LTMyYnl0ZXM=",
    );
    let dir = tempfile::tempdir().expect("tempdir");
    std::env::set_var(SQLITE_PATH_ENV, dir.path().join("integ-photon.sqlite3"));

    let factory: Arc<dyn ValenceFactory> = Arc::new(StubValenceFactory);
    let runtime = build_runtime(factory).await.expect("build_runtime");
    let _ = runtime.photon.as_ref();
}

#[tokio::test]
async fn build_runtime_missing_transport_key_sad() {
    let _guard = ENV_LOCK.lock().await;
    std::env::remove_var("PHOTON_TRANSPORT_KEY");
    std::env::remove_var("PHOTON_ALLOW_DEV_TRANSPORT_KEY");
    let dir = tempfile::tempdir().expect("tempdir");
    std::env::set_var(SQLITE_PATH_ENV, dir.path().join("integ-photon-sad.sqlite3"));

    let factory: Arc<dyn ValenceFactory> = Arc::new(StubValenceFactory);
    let Err(err) = build_runtime(factory).await else {
        panic!("missing transport key");
    };
    let msg = err.to_string().to_lowercase();
    assert!(
        msg.contains("transport") || msg.contains("photon") || msg.contains("key"),
        "unexpected error: {msg}"
    );
}
