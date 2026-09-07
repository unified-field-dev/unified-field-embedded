//! Integration contracts for Spectra embedded wiring.

use spectra_uf_embedded::{install_embedded_sqlite, EVENTS_PATH_ENV, METRICS_PATH_ENV};

static ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[tokio::test]
async fn install_embedded_sqlite_temp_paths_happy_path() {
    let _guard = ENV_LOCK.lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    std::env::set_var(METRICS_PATH_ENV, dir.path().join("integ-m.sqlite3"));
    std::env::set_var(EVENTS_PATH_ENV, dir.path().join("integ-e.sqlite3"));
    let spectra = install_embedded_sqlite()
        .await
        .expect("install_embedded_sqlite");
    let _ = spectra.as_ref();
}

#[tokio::test]
async fn install_embedded_sqlite_parent_not_dir_sad() {
    let _guard = ENV_LOCK.lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let blocker = dir.path().join("blocker");
    std::fs::write(&blocker, b"x").expect("write");
    std::env::set_var(METRICS_PATH_ENV, blocker.join("m.sqlite3"));
    std::env::set_var(EVENTS_PATH_ENV, dir.path().join("e.sqlite3"));

    let Err(err) = install_embedded_sqlite().await else {
        panic!("metrics parent is a file");
    };
    assert_ne!(err.to_string(), "");
}
