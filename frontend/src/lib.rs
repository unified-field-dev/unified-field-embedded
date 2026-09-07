//! WASM hydrate entry for the embedded full-product host.
//!
//! Stock `wasm_bindgen` entry: console logging, panic hook, then
//! `hydrate_lazy(app::App)` for `cargo leptos --split`. Leave this crate alone
//! for ordinary template customization — mount uf-apps in
//! [`app::main_app_routes::MainAppRouteChildren`], and change host boot in
//! `server`. Touch [`hydrate`] only for client logging / panic-hook tweaks.
//!
//! ## Feature flags
//!
//! | Flag | Role |
//! |------|------|
//! | `hydrate` | Enables the WASM mount body inside [`hydrate`] |

/// Browser entry: hydrate the product [`app::App`] shell.
///
/// Exported via `wasm_bindgen` for the Leptos frontend pkg. Call only from the
/// WASM client (never SSR). Customize logging/panic hooks; leave App composition
/// and uf-app mounts in the `app` crate.
///
/// # Examples
///
/// Stock WASM entry only — see `app::MainAppRouteChildren` to customize routes.
/// Call `hydrate` from the browser pkg (`wasm_bindgen`), never from SSR.
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    _ = console_log::init_with_level(log::Level::Debug);

    #[cfg(feature = "hydrate")]
    {
        std::panic::set_hook(Box::new(|info| {
            orbital::hide_boot_loader();
            console_error_panic_hook::hook(info);
        }));
        // hydrate_lazy supports #[lazy]/#[lazy_route] WASM chunks from `cargo leptos --split`.
        leptos::mount::hydrate_lazy(app::App);
        orbital::hide_boot_loader();
    }
}
