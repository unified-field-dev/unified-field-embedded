//! Process-scoped [`ValenceFactory`] / [`HiggsValenceFactory`] for host boot.
//!
//! - **Internal** factories ([`ProcessValenceFactory::as_internal_valence_factory`],
//!   [`ProcessValenceFactory::as_higgs_factory`]) allow process-local System minting
//!   (Chronon default jobs, Boson/Photon runtimes, Higgs SSR escape hatch).
//! - **External** factories ([`ProcessValenceFactory::as_external_valence_factory`])
//!   apply [`RejectExternalSystemActor`] so untrusted actor JSON cannot mint
//!   `Actor::System`.
//!
//! Wire internal/Higgs factories inside [`crate::platform::compose_embedded_platform`].
//! When you add a worker bridge or other untrusted actor-JSON path, call
//! [`ProcessValenceFactory::as_external_valence_factory`] at that call site — do not
//! hand the internal factory across that boundary.

use std::sync::Arc;

use higgs::actor_policy::external_actor_json_policy;
use higgs::HiggsValenceFactory;
use valence::{
    Actor, ActorJsonPolicy, ActorTrust, DatabaseRouter, RejectExternalSystemActor, Valence,
    ValenceFactory,
};

/// Whether actor JSON is treated as process-local or untrusted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ActorTrustMode {
    /// Chronon/Boson/Photon boot and Higgs SSR — System mint allowed.
    Internal,
    /// Worker bridges / external actor JSON — System-shaped actors rejected.
    External,
}

/// Builds per-request [`Valence`] from the process router handle.
///
/// Holds the shared [`DatabaseRouter`], default backend key, and
/// [`ActorTrustMode`]. Construct via the `as_*` helpers rather than
/// [`ProcessValenceFactory::new`] unless you are extending trust modes in-tree.
///
/// # Examples
///
/// See crate [Wire process Valence factories](../index.html#wire-process-valence-factories).
#[derive(Clone)]
pub struct ProcessValenceFactory {
    router: Arc<DatabaseRouter>,
    default_backend_key: String,
    trust: ActorTrustMode,
}

impl ProcessValenceFactory {
    /// Construct a factory with internal trust (System mint allowed).
    ///
    /// Prefer [`Self::as_internal_valence_factory`] / [`Self::as_higgs_factory`] /
    /// [`Self::as_external_valence_factory`] at call sites. Use `new` only when
    /// composing a custom trust mode in host code.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use std::sync::Arc;
    /// use server::process_valence_factory::ProcessValenceFactory;
    /// use valence::DatabaseRouter;
    ///
    /// fn demo(router: Arc<DatabaseRouter>, key: String) {
    ///     let factory = ProcessValenceFactory::new(router, key);
    ///     println!("ProcessValenceFactory ready (internal trust)");
    ///     let _ = factory;
    /// }
    /// ```
    pub fn new(router: Arc<DatabaseRouter>, default_backend_key: impl Into<String>) -> Self {
        Self {
            router,
            default_backend_key: default_backend_key.into(),
            trust: ActorTrustMode::Internal,
        }
    }

    fn with_external_trust(mut self) -> Self {
        self.trust = ActorTrustMode::External;
        self
    }

    /// Trusted process-local factory (Chronon/Boson/Photon boot, default jobs).
    ///
    /// Call at platform compose time for runtimes that must mint System. Do not
    /// pass this handle to untrusted actor JSON paths — use
    /// [`Self::as_external_valence_factory`] there instead.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use std::sync::Arc;
    /// use server::process_valence_factory::ProcessValenceFactory;
    /// use valence::{DatabaseRouter, ValenceFactory};
    ///
    /// fn wire(router: Arc<DatabaseRouter>, key: String) -> Arc<dyn ValenceFactory> {
    ///     let factory = ProcessValenceFactory::as_internal_valence_factory(router, key);
    ///     assert!(Arc::strong_count(&factory) >= 1);
    ///     factory
    /// }
    /// ```
    pub fn as_internal_valence_factory(
        router: Arc<DatabaseRouter>,
        default_backend_key: impl Into<String>,
    ) -> Arc<dyn ValenceFactory> {
        Arc::new(Self::new(router, default_backend_key))
    }

    /// Factory for untrusted / external actor JSON (workers, bridges).
    ///
    /// Rejects System-shaped actors via [`RejectExternalSystemActor`]. Use for
    /// any path that deserializes actor JSON from outside the process control
    /// plane. Not held on [`crate::platform::EmbeddedPlatform`] until such a
    /// consumer is wired — call this at the bridge call site.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use std::sync::Arc;
    /// use server::process_valence_factory::ProcessValenceFactory;
    /// use valence::{DatabaseRouter, ValenceFactory};
    ///
    /// fn wire(router: Arc<DatabaseRouter>, key: String) -> Arc<dyn ValenceFactory> {
    ///     let factory = ProcessValenceFactory::as_external_valence_factory(router, key);
    ///     assert!(Arc::strong_count(&factory) >= 1);
    ///     factory
    /// }
    /// ```
    pub fn as_external_valence_factory(
        router: Arc<DatabaseRouter>,
        default_backend_key: impl Into<String>,
    ) -> Arc<dyn ValenceFactory> {
        Arc::new(Self::new(router, default_backend_key).with_external_trust())
    }

    /// Factory for Higgs SSR (`valence()` / `unsafe_system_valence`).
    ///
    /// Same internal trust as [`Self::as_internal_valence_factory`], typed for
    /// [`HiggsValenceFactory`]. Wire into [`higgs::HiggsConfig`] at platform compose.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use std::sync::Arc;
    /// use server::process_valence_factory::ProcessValenceFactory;
    /// use higgs::HiggsValenceFactory;
    /// use valence::DatabaseRouter;
    ///
    /// fn wire(router: Arc<DatabaseRouter>, key: String) -> Arc<dyn HiggsValenceFactory> {
    ///     let factory = ProcessValenceFactory::as_higgs_factory(router, key);
    ///     assert!(Arc::strong_count(&factory) >= 1);
    ///     factory
    /// }
    /// ```
    pub fn as_higgs_factory(
        router: Arc<DatabaseRouter>,
        default_backend_key: impl Into<String>,
    ) -> Arc<dyn HiggsValenceFactory> {
        Arc::new(Self::new(router, default_backend_key))
    }

    fn build_valence(&self, actor_json: &serde_json::Value) -> valence::Result<Valence> {
        if self.trust == ActorTrustMode::External {
            let policy: RejectExternalSystemActor = external_actor_json_policy();
            policy.validate(ActorTrust::External, actor_json)?;
        }
        let actor: Actor = serde_json::from_value(actor_json.clone())
            .map_err(|e| valence::Error::Internal(format!("Failed to deserialize actor: {e}")))?;
        Valence::builder()
            .database_router(self.router.clone())
            .default_backend_key(self.default_backend_key.clone())
            .with_actor(actor)
            .build()
    }
}

impl ValenceFactory for ProcessValenceFactory {
    fn build(&self, actor_json: &serde_json::Value) -> valence::Result<Valence> {
        self.build_valence(actor_json)
    }
}

impl HiggsValenceFactory for ProcessValenceFactory {
    fn build(&self, actor_json: &serde_json::Value) -> anyhow::Result<Valence> {
        self.build_valence(actor_json).map_err(anyhow::Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_policy_rejects_system_actor_json_happy_path() {
        let policy: RejectExternalSystemActor = external_actor_json_policy();
        let system = serde_json::json!({"System":{"operation":"boot"}});
        let err = policy
            .validate(ActorTrust::External, &system)
            .expect_err("System actor must be rejected for External trust");
        let _ = err;
    }

    #[test]
    fn external_policy_allows_user_actor_json_happy_path() {
        let policy: RejectExternalSystemActor = external_actor_json_policy();
        let user = serde_json::json!({"User":{"user_id":"user:1"}});
        policy
            .validate(ActorTrust::External, &user)
            .expect("User actor allowed for External trust");
    }

    #[test]
    fn internal_trust_allows_system_for_escape_hatch_happy_path() {
        let policy: RejectExternalSystemActor = external_actor_json_policy();
        let system = serde_json::json!({"System":{"operation":"ops.control"}});
        policy
            .validate(ActorTrust::Internal, &system)
            .expect("Internal trust may mint System for escape hatch");
    }
}
