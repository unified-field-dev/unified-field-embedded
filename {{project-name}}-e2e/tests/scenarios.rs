//! Layer 2 `IsolatedLab` scenarios for embedded host composition.
//!
//! Scenario IDs:
//! - [`{{crate_name}}_e2e::scenarios::HOST_COMPOSITION_BOOT_HAPPY`]
//! - [`{{crate_name}}_e2e::scenarios::HOST_COMPOSITION_PHOTON_KEY_SAD`]

use {{crate_name}}_e2e::boot_isolated_lab;
use {{crate_name}}_e2e::scenarios::{HOST_COMPOSITION_BOOT_HAPPY, HOST_COMPOSITION_PHOTON_KEY_SAD};

#[tokio::test(flavor = "multi_thread")]
async fn host_composition_boot_happy() {
    let scenario_id = HOST_COMPOSITION_BOOT_HAPPY;
    let lab = boot_isolated_lab(true)
        .await
        .unwrap_or_else(|e| panic!("scenario {scenario_id} boot failed: {e:#}"));

    lab.higgs
        .photon()
        .unwrap_or_else(|e| panic!("scenario {scenario_id}: photon missing: {e}"));
    lab.higgs
        .boson_backend()
        .unwrap_or_else(|e| panic!("scenario {scenario_id}: boson missing: {e}"));
    lab.higgs
        .chronon_backend()
        .unwrap_or_else(|e| panic!("scenario {scenario_id}: chronon missing: {e}"));
    let _ = lab.spectra.as_ref();
    let _ = lab.photon.photon.as_ref();
    let _ = lab.boson.boson.as_ref();
}

#[tokio::test(flavor = "multi_thread")]
async fn host_composition_photon_key_sad() {
    let scenario_id = HOST_COMPOSITION_PHOTON_KEY_SAD;
    let Err(err) = boot_isolated_lab(false).await else {
        panic!("scenario {scenario_id} must fail without PHOTON_TRANSPORT_KEY");
    };
    let msg = err.to_string().to_lowercase();
    assert!(
        msg.contains("transport") || msg.contains("photon") || msg.contains("key"),
        "scenario {scenario_id}: unexpected error: {msg}"
    );
}
