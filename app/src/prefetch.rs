//! Host-side WASM chunk prefetch for app families (`cargo leptos --split`).
//!
//! Registers a [`PrefetchAppFamily`] context so the
//! apps directory can warm family chunks on hover/focus. Customize the match arms
//! in [`prefetch_route_family`] when you add a split uf-app; call
//! [`provide_app_family_prefetcher`] once from [`crate::App`] at mount.

use std::sync::Arc;

use leptos::prelude::*;
use leptos::task::spawn_local;
use uf_apps::PrefetchAppFamily;

/// Register a context callback so the apps directory can prefetch family chunks on hover.
///
/// Call once inside [`crate::App`] after auth/appearance context. Customize by
/// extending [`prefetch_route_family`]; do not register a second prefetcher.
///
/// # Examples
///
/// ```rust,ignore
/// use app::prefetch::provide_app_family_prefetcher;
///
/// // Inside App() after provide_auth_context / appearance:
/// provide_app_family_prefetcher();
/// println!("PrefetchAppFamily context registered for split chunks");
/// ```
pub fn provide_app_family_prefetcher() {
    provide_context(PrefetchAppFamily(Arc::new(|route_path: &str| {
        let route_path = route_path.to_string();
        spawn_local(async move {
            prefetch_route_family(&route_path).await;
        });
    })));
}

/// Prefetch the WASM family for a known app route path.
///
/// Invoked from the [`PrefetchAppFamily`] callback.
/// Add a match arm when you introduce a new `--split` uf-app; unknown paths are
/// no-ops. Customize arms to match your route table, not the stock set alone.
///
/// # Examples
///
/// ```rust,ignore
/// use app::prefetch::prefetch_route_family;
///
/// async fn warm_counter() {
///     prefetch_route_family("/counter").await;
///     println!("counter family prefetch requested");
/// }
/// ```
pub async fn prefetch_route_family(route_path: &str) {
    match route_path {
        "/counter" => counter_app::prefetch_family().await,
        "/apps" => uf_apps::prefetch_family().await,
        "/tag" => tag_app::prefetch_family().await,
        "/orbital" => uf_component_preview::prefetch_family().await,
        "/welcome" => uf_welcome::prefetch_family().await,
        "/notifications" => uf_notifications::prefetch_family().await,
        "/user" => lepton_app::prefetch_family().await,
        "/valence" => valence_app::prefetch_family().await,
        "/chronon" => chronon_app::prefetch_family().await,
        "/photon" => photon_app::prefetch_family().await,
        "/spectra" => spectra_app::prefetch_family().await,
        "/boson" => boson_app::prefetch_family().await,
        "/permission" => gauge_app::prefetch_family().await,
        "/secrets" => neutrino_app::prefetch_family().await,
        _ => {}
    }
}
