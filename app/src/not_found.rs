//! Host 404 page — {{app_title}} shell chrome + account menu.
//!
//! Used as the `<Routes>` fallback and the `/*any` catch-all in
//! [`crate::main_app_routes::MainAppRouteChildren`]. Customize the slot content
//! when your product needs a different account menu; keep the shared
//! `UnifiedFieldNotFoundPage` chrome unless you intentionally fork branding.

use lepton_shell::AppBarUserMenu;
use leptos::prelude::*;
use uf_integrations::{ShellAuthMenu, UnifiedFieldNotFoundPage};

/// Product 404 with the shared app bar, theme surface, and auth menu.
///
/// Renders when no uf-app route matches. Prefer this over a bare Orbital
/// `NotFoundPage` so signed-in users still see the account menu. Customize the
/// `auth_menu` slot; leave the outer page chrome alone for brand consistency.
///
/// # Examples
///
/// ```rust,ignore
/// use leptos::prelude::*;
/// use app::not_found::HostNotFoundPage;
///
/// view! { <HostNotFoundPage /> };
/// println!("404 view mounts UnifiedFieldNotFoundPage + AppBarUserMenu");
/// ```
#[component]
pub fn HostNotFoundPage() -> impl IntoView {
    view! {
        <UnifiedFieldNotFoundPage>
            <ShellAuthMenu slot:auth_menu>
                <AppBarUserMenu />
            </ShellAuthMenu>
        </UnifiedFieldNotFoundPage>
    }
}
