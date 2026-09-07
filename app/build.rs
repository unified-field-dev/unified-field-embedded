//! Build script.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_root = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?)
        .parent()
        .ok_or("CARGO_MANIFEST_DIR has no parent")?
        .to_path_buf();
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);

    let extra_packages = vec![
        "counter-app".to_string(),
        "uf-apps".to_string(),
        "uf-component-preview".to_string(),
        "tag-app".to_string(),
        "uf-welcome".to_string(),
        "uf-notifications".to_string(),
        "lepton-app".to_string(),
        "valence-app".to_string(),
        "chronon-app".to_string(),
        "photon-app".to_string(),
        "boson-app".to_string(),
        "spectra-app".to_string(),
        "gauge-app".to_string(),
        "neutrino-app".to_string(),
        "record-history-leptos".to_string(),
    ];
    // Marketing stays on unified-field-site. Gluon/Nucleus/setup-wizard stay out.
    let excluded_packages = vec![
        "setup-wizard-app".to_string(),
        "gluon-app".to_string(),
        "nucleus-app".to_string(),
        "uf-marketing".to_string(),
    ];

    uf_codegen::generate_registered_routes(&uf_codegen::RoutesCodegenConfig {
        workspace_root,
        out_dir,
        extra_packages,
        excluded_packages,
    })?;
    Ok(())
}
