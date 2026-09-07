//! Root UI composition for the embedded {{app_title}} full-product host.
//!
//! This template ships a stock Orbital document shell and [`App`] root. For a
//! GitHub-template fork, the customize surface is the uf-app route table
//! ([`MainAppRouteChildren`]), not Leptos/Orbital presentation patterns. Deep UI
//! work belongs in uf-app crates and Orbital docs; keep this crate thin.
//!
//! Unmatched paths render [`HostNotFoundPage`](not_found::HostNotFoundPage).
//! Marketing and Coming Soon live on `unified-field-site`.
//!
//! ## Features
//!
//! - **uf-app route table** — Mounts product and platform admin routes via
//!   [`MainAppRouteChildren`]; edit that table to add or remove uf-apps.
//!   [Get started](#customize-uf-app-routes)
//!
//! ## Customize uf-app routes
//!
//! [`MainAppRouteChildren`] is the primary customize surface for which uf-apps
//! appear in the host. Edit it once when adding or removing a uf-app (and matching
//! Cargo features/deps), then rebuild so the route appears or 404s cleanly via
//! [`HostNotFoundPage`](not_found::HostNotFoundPage). Do not patch generated
//! `OUT_DIR` tables. Leave [`shell`] / [`App`] alone unless you intentionally
//! change document chrome or auth/appearance providers (see Orbital / `uf-product`).
//!
//! **Prerequisites:** `hydrate` or `server-embedded`; uf-app crates linked with
//! matching features; `uf_app!` codegen producing `OUT_DIR` route tables.
//!
//! ```rust,ignore
//! use leptos::prelude::*;
//! use leptos_router::components::*;
//! use leptos_router::path;
//! use app::main_app_routes::MainAppRouteChildren;
//! use app::not_found::HostNotFoundPage;
//!
//! // Sketch of product_shell_routes(): MainAppRouteChildren + HostNotFoundPage.
//! fn product_shell_routes() -> impl IntoView {
//!     view! {
//!         <Routes fallback=|| view! { <HostNotFoundPage /> }>
//!             <MainAppRouteChildren />
//!         </Routes>
//!     }
//! }
//!
//! let _ = product_shell_routes;
//! println!("route table mounts MainAppRouteChildren; unknown paths → HostNotFoundPage");
//! ```
//!
//! On success matched paths render the uf-app; unknown paths render
//! [`HostNotFoundPage`](not_found::HostNotFoundPage). Missing Cargo features fail
//! at compile time. Variant: trim platform admin routes for a thinner product
//! surface, keeping auth + at least one demo app.
//!
//! ## Further reading (stock shell)
//!
//! [`shell`] and [`App`] are the stock document entry and interactive root wired
//! by `server::build_router`. Change them only when you mean to alter title/favicon
//! chrome, auth/appearance context, or Orbital template slots — not for ordinary
//! uf-app mounts.
//!
//! ## Feature flags
//!
//! | Flag | Role |
//! |------|------|
//! | `ssr` | Server-side Leptos + Axum helpers |
//! | `hydrate` | WASM hydration of [`App`] |
//! | `server-embedded` | Full-product host profile (required with `ssr`) |

use cfg_if::cfg_if;
use leptos::prelude::*;
use leptos_router::components::*;

use orbital::{orbital_shell_with_meta, OrbitalDocumentMeta, OrbitalTemplate};

cfg_if! {
    if #[cfg(any(feature = "hydrate", feature = "server-embedded"))] {
        use lepton_shell::AppBarUserMenu;
        use uf_integrations::provide_shell_auth_menu;
        use uf_notifications::ensure_notification_bell_linked;
        use uf_product::{
            init_appearance_resource, init_auth_resource, provide_appearance_context,
            provide_auth_context, AuthSession,
        };

        pub(crate) mod uf_registered_routes {
            #![allow(
                dead_code,
                unused_imports,
                missing_docs,
                clippy::all,
                clippy::pedantic,
                clippy::nursery,
                clippy::restriction,
            )]

            include!(concat!(env!("OUT_DIR"), "/uf_registered_routes.rs"));
        }

        pub(crate) mod uf_app_route_table {
            #![allow(
                dead_code,
                unused_imports,
                missing_docs,
                clippy::all,
                clippy::pedantic,
                clippy::nursery,
                clippy::restriction,
            )]

            include!(concat!(env!("OUT_DIR"), "/uf_app_route_table.rs"));
        }

        pub mod main_app_routes;
        use main_app_routes::MainAppRouteChildren;
        pub mod not_found;
        pub mod prefetch;
    }
}

#[cfg(all(feature = "ssr", not(feature = "server-embedded")))]
compile_error!("Missing app runtime profile feature. Enable server-embedded");

/// SSR document entry: Orbital meta chrome wrapping [`App`].
///
/// Passed to Leptos from `server::build_router` at process startup. Customize
/// title/favicon via [`OrbitalDocumentMeta`]; leave route mounting to
/// [`MainAppRouteChildren`].
///
/// # Examples
///
/// See [Mount document shell](index.html#mount-document-shell).
pub fn shell(options: LeptosOptions) -> impl IntoView {
    orbital_shell_with_meta(
        options,
        OrbitalDocumentMeta {
            title: "{{app_title}}",
            favicon_href: "/favicon.ico",
            apple_touch_icon_href: Some("/apple-touch-icon.png"),
        },
        || view! { <App /> },
    )
}

cfg_if! {
    if #[cfg(any(feature = "hydrate", feature = "server-embedded"))] {
        fn product_shell_routes() -> impl IntoView {
            use not_found::HostNotFoundPage;
            view! {
                <Routes fallback=|| view! { <HostNotFoundPage /> }>
                    <MainAppRouteChildren />
                </Routes>
            }
        }
    }
}

/// Embedded full-product UI shell (auth, appearance, product + platform routes).
///
/// Mount once under [`shell`] for SSR or via `frontend::hydrate` for WASM.
/// Customize chrome (auth menu, notification bell) here; add/remove uf-apps in
/// [`MainAppRouteChildren`].
///
/// # Examples
///
/// See [Compose App root](index.html#compose-app-root).
#[cfg(any(feature = "hydrate", feature = "server-embedded"))]
#[component]
pub fn App() -> impl IntoView {
    let auth = provide_auth_context(AuthSession::default());
    let _auth_resource = init_auth_resource(&auth);
    provide_shell_auth_menu(|| view! { <AppBarUserMenu /> });
    ensure_notification_bell_linked();
    let appearance = provide_appearance_context();
    let _appearance_resource = init_appearance_resource(appearance);
    let (is_routing, set_is_routing) = signal(false);
    prefetch::provide_app_family_prefetcher();

    view! {
        <OrbitalTemplate>
            <Show when=move || is_routing.get()>
                <div
                    data-testid="host-routing-indicator"
                    style="position:fixed;top:0;left:0;right:0;z-index:9999;height:3px;background:var(--colorBrandBackground, #0f6cbd);opacity:0.85;"
                    aria-live="polite"
                    aria-busy="true"
                />
            </Show>
            <Router set_is_routing>
                {product_shell_routes()}
            </Router>
        </OrbitalTemplate>
    }
}

/// Minimal fallback when no UI profile feature is enabled.
#[cfg(not(any(feature = "hydrate", feature = "server-embedded")))]
#[component]
pub fn App() -> impl IntoView {
    use leptos_router::path;
    use orbital::components::NotFoundPage;

    view! {
        <OrbitalTemplate>
            <Router>
                <Routes fallback=|| view! { <NotFoundPage /> }>
                    <Route path=path!("404") view=NotFoundPage />
                </Routes>
            </Router>
        </OrbitalTemplate>
    }
}
