//! Hygiene: embedded full-product host always mounts platform apps; no flag dualism / marketing / Coming Soon.

#[test]
fn embedded_app_requires_platform_apps_happy_path() {
    let manifest = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));
    for required in [
        "valence-app",
        "chronon-app",
        "photon-app",
        "boson-app",
        "spectra-app",
        "gauge-app",
        "neutrino-app",
        "lepton-app",
        "uf-notifications",
        "record-history-leptos",
        "uf-welcome",
        "uf-component-preview",
        "counter-app",
        "lepton-auth",
        "server-embedded",
    ] {
        assert!(
            manifest.contains(required),
            "embedded app/Cargo.toml missing required full-product surface `{required}`"
        );
    }
}

#[test]
fn embedded_app_forbids_preview_dualism_and_marketing_sad_if_present() {
    let manifest = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"));
    for forbidden in ["uf-marketing", "server-full", "\nfull ="] {
        assert!(
            !manifest.contains(forbidden),
            "embedded app/Cargo.toml must not contain `{forbidden}`"
        );
    }
}

#[test]
fn embedded_routes_mount_platform_apps_without_coming_soon_happy_path() {
    let not_found = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/not_found.rs"));
    let routes = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/main_app_routes.rs"
    ));
    assert!(
        !not_found.contains("HostComingSoonPage"),
        "embedded must not define HostComingSoonPage"
    );
    assert!(
        !routes.contains("HostComingSoonPage") && !routes.contains("coming-soon"),
        "embedded routes must not mount Coming Soon"
    );
    assert!(
        routes.contains("ValenceRoutes")
            && routes.contains("ChrononRoutes")
            && routes.contains("BosonRoutes")
            && routes.contains("HostNotFoundPage"),
        "embedded must mount platform routes with 404 catch-all"
    );
}
