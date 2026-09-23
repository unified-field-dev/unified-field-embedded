//! Boot-time sync of `uf_app!` permission manifests into Gauge Valence tables.
//!
//! Collects every registered app's `permission_manifest`, then calls
//! [`gauge::manifest_sync::sync_permission_manifests`]. Idempotent — safe on every
//! process start. Gluon operator group seed is best-effort when that domain exists.

use gauge::manifest_sync::{PermissionDomainInput, PermissionInput, PermissionManifestInput};
use uf_product::AppRegistry;
use valence::Valence;

/// Build Gauge sync inputs from every discovered app that declares a manifest.
pub fn collect_registered_app_permission_manifests() -> Vec<PermissionManifestInput> {
    let mut manifests = Vec::new();
    for registration in AppRegistry::auto_discover().iter() {
        let Some(manifest_provider) = registration.permission_manifest else {
            continue;
        };
        let manifest = manifest_provider();

        let mut domains = Vec::new();
        for domain in manifest.domains {
            let permissions = domain
                .permissions
                .iter()
                .map(|permission| PermissionInput {
                    name: permission.name.to_string(),
                    description: permission.description.to_string(),
                })
                .collect::<Vec<_>>();
            domains.push(PermissionDomainInput {
                key: domain.key.to_string(),
                name: domain.name.to_string(),
                description: domain.description.to_string(),
                permissions,
            });
        }

        manifests.push(PermissionManifestInput {
            app_id: manifest.app_id.to_string(),
            domains,
        });
    }
    manifests
}

/// Upsert domains/permissions/manifest owner groups for all registered apps.
///
/// # Errors
///
/// Returns when Gauge Valence create/lookup fails during sync.
pub async fn sync_permission_manifests_for_valence(valence: &Valence) -> anyhow::Result<()> {
    let manifests = collect_registered_app_permission_manifests();
    let app_count = manifests.len();
    let stats = gauge::manifest_sync::sync_permission_manifests(valence, &manifests).await?;
    log::info!(
        "[server] permission manifest sync: apps={app_count} domains_created={} permissions_created={} domains_existing={} permissions_existing={}",
        stats.domains_created,
        stats.permissions_created,
        stats.domains_existing,
        stats.permissions_existing
    );
    if let Err(e) = gauge::gluon_operator_groups::ensure_gluon_default_operator_groups(valence).await
    {
        log::warn!("[server] Gluon operator group seed skipped or failed: {e}");
    }
    Ok(())
}
