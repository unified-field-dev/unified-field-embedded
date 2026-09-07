//! Binary entry for the embedded full-product host.
//!
//! Initializes logging and SQLite ownership-fetch defaults, then calls
//! [`server::run`] so listen address, platform compose, graceful Axum serve, and
//! Chronon drain stay in the library. Customize logging level here; leave boot
//! order and HTTP wiring in `server` lib / `platform`.
//!
//! # Examples
//!
//! ```rust,ignore
//! use server::run;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Same shape as this binary: run → listen_addr → build_host → serve.
//!     run().await?;
//!     Ok(())
//! }
//! ```

use server::run;

fn ensure_embedded_env_defaults() {
    // SQLite embedded does not support Valence's ownership JOIN fetch path yet;
    // keep the legacy two-trip read unless the operator explicitly enables it.
    if std::env::var_os("VALENCE_OWNERSHIP_UNIFIED_FETCH").is_none() {
        std::env::set_var("VALENCE_OWNERSHIP_UNIFIED_FETCH", "0");
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    simple_logger::init_with_level(log::Level::Info).ok();
    ensure_embedded_env_defaults();
    run().await
}
