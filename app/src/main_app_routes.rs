//! Shared Leptos route children for the embedded full-product host UI shell.
//!
//! Counter, welcome, auth, apps, tags, orbital preview, and all platform admin
//! apps. Unknown paths are 404. Marketing and Coming Soon live on
//! `unified-field-site`. **This is the primary customize surface** for adding or
//! removing uf-apps: edit the children below and matching Cargo features/deps.

use leptos::prelude::*;
use leptos_router::{components::*, path};

use boson_app::BosonRoutes;
use chronon_app::ChrononRoutes;
use counter_app::CounterRoutes;
use gauge_app::PermissionRoutes;
use lepton_app::UserAppRoutes;
use lepton_auth_app::LeptonAuthRoutes;
use neutrino_app::NeutrinoRoutes;
use photon_app::PhotonRoutes;
use spectra_app::SpectraRoutes;
use tag_app::TagRoutes;
use uf_apps::UfAppsRoutes;
use uf_component_preview::OrbitalDevRoutes;
use uf_notifications::NotificationsRoutes;
use uf_product::{AppearanceThemeController, PageViewTracker};
use uf_welcome::UfWelcomeRoutes;
use valence_app::ValenceRoutes;

use crate::not_found::HostNotFoundPage;

/// Full product routes and 404 catch-all.
///
/// Mount under `<Routes>` from [`crate::App`]. Add a uf-app by inserting its
/// `*Routes` component (and enabling the crate feature/dep); remove by deleting
/// the child and unused deps. Keep the `/*any` → [`HostNotFoundPage`] catch-all
/// so unknown paths stay a clean product 404.
///
/// # Examples
///
/// See crate [Customize uf-app routes](../index.html#customize-uf-app-routes).
#[component(transparent)]
pub fn MainAppRouteChildren() -> impl leptos_router::MatchNestedRoutes + Clone {
    // Primary customize surface: add/remove `*Routes` + matching Cargo features/deps.
    // See docs/add-or-remove-uf-app.md. Do not edit generated OUT_DIR route tables.
    view! {
        <ParentRoute path=path!("") view=move || view! {
            <AppearanceThemeController routes=crate::uf_app_route_table::UF_APP_ROUTE_TABLE />
            <PageViewTracker routes=crate::uf_app_route_table::UF_APP_ROUTE_TABLE />
            <Outlet />
        }>
            // Demo / product apps
            <CounterRoutes />
            <UfWelcomeRoutes />

            <Route path=path!("") view=|| view! { <Redirect path="/apps"/> } />

            <Route path=path!("404") view=HostNotFoundPage />

            // Auth UI (signup/signin always available on this host)
            <LeptonAuthRoutes />

            <UfAppsRoutes />
            <TagRoutes />
            // Orbital component catalog (dev preview); remove if you do not want /orbital
            <OrbitalDevRoutes />

            <NotificationsRoutes />
            <UserAppRoutes />
            // Platform admin apps — trim for a thinner product surface
            <ValenceRoutes />
            <ChrononRoutes />
            <PhotonRoutes />
            <SpectraRoutes />
            <BosonRoutes />
            <PermissionRoutes />
            <NeutrinoRoutes />

            // Keep catch-all last so unknown paths stay a clean product 404
            <Route path=path!("/*any") view=HostNotFoundPage />
        </ParentRoute>
    }
    .into_inner()
}
