//! Integration contracts for Boson embedded wiring.

use std::sync::Arc;

use boson_uf_embedded::{build_boson_runtime, SQLITE_PATH_ENV};
use valence::{Error, Result, Valence, ValenceFactory};

static ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

struct StubValenceFactory;

impl ValenceFactory for StubValenceFactory {
    fn build(&self, _actor_json: &serde_json::Value) -> Result<Valence> {
        Err(Error::Internal("stub valence factory".into()))
    }
}

#[tokio::test]
async fn build_boson_runtime_temp_sqlite_happy_path() {
    let _guard = ENV_LOCK.lock().await;
    std::env::set_var("BOSON_TELEMETRY", "off");
    let dir = tempfile::tempdir().expect("tempdir");
    std::env::set_var(SQLITE_PATH_ENV, dir.path().join("integ-boson.sqlite3"));

    let factory: Arc<dyn ValenceFactory> = Arc::new(StubValenceFactory);
    let runtime = build_boson_runtime(factory)
        .await
        .expect("build_boson_runtime");
    let _ = runtime.boson.as_ref();
    let _ = runtime.backend.as_ref();
}

#[tokio::test]
async fn build_boson_runtime_parent_not_dir_sad() {
    let _guard = ENV_LOCK.lock().await;
    std::env::set_var("BOSON_TELEMETRY", "off");
    let dir = tempfile::tempdir().expect("tempdir");
    let blocker = dir.path().join("blocker");
    std::fs::write(&blocker, b"x").expect("write");
    std::env::set_var(SQLITE_PATH_ENV, blocker.join("boson.sqlite3"));

    let factory: Arc<dyn ValenceFactory> = Arc::new(StubValenceFactory);
    let Err(err) = build_boson_runtime(factory).await else {
        panic!("parent is a file");
    };
    assert_ne!(err.to_string(), "");
}
