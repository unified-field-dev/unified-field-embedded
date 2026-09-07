//! Integration contracts for Chronon embedded wiring.

use std::sync::Arc;

use chronon_uf_embedded::{build_chronon_runtime, SQLITE_PATH_ENV};
use valence::{Error, Result, Valence, ValenceFactory};

static ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

struct StubValenceFactory;

impl ValenceFactory for StubValenceFactory {
    fn build(&self, _actor_json: &serde_json::Value) -> Result<Valence> {
        Err(Error::Internal("stub valence factory".into()))
    }
}

#[tokio::test]
async fn build_chronon_runtime_temp_sqlite_happy_path() {
    let _guard = ENV_LOCK.lock().await;
    std::env::set_var("CHRONON_TELEMETRY", "off");
    let dir = tempfile::tempdir().expect("tempdir");
    std::env::set_var(SQLITE_PATH_ENV, dir.path().join("integ-chronon.sqlite3"));

    let factory: Arc<dyn ValenceFactory> = Arc::new(StubValenceFactory);
    let runtime = build_chronon_runtime(factory)
        .await
        .expect("build_chronon_runtime");
    runtime.shutdown.notify_waiters();
    runtime.run_loop.abort();
}

#[tokio::test]
async fn build_chronon_runtime_parent_not_dir_sad() {
    let _guard = ENV_LOCK.lock().await;
    std::env::set_var("CHRONON_TELEMETRY", "off");
    let dir = tempfile::tempdir().expect("tempdir");
    let blocker = dir.path().join("blocker");
    std::fs::write(&blocker, b"x").expect("write");
    std::env::set_var(SQLITE_PATH_ENV, blocker.join("chronon.sqlite3"));

    let factory: Arc<dyn ValenceFactory> = Arc::new(StubValenceFactory);
    let Err(err) = build_chronon_runtime(factory).await else {
        panic!("parent is a file");
    };
    assert_ne!(err.to_string(), "");
}
