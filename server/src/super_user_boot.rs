//! Boot-time Super User group ensure + env email seed.
//!
//! Ensures the well-known `super_user_group`, then promotes addresses listed in
//! `UF_SUPER_USER_EMAILS` when those Lepton users already exist. Missing emails
//! are soft-failed so boot can restart after signup.

use gauge::super_user::{ensure_super_user_group, seed_super_user_member_by_email};
use valence::Valence;

const UF_SUPER_USER_EMAILS: &str = "UF_SUPER_USER_EMAILS";

/// Counts from a best-effort Super User email seed (host boot logs these only).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SuperUserBootStats {
    /// Number of email addresses configured.
    pub configured: usize,
    /// Addresses that resolved and were ensured as members.
    pub seeded: usize,
    /// Addresses with no matching Lepton user.
    pub missing_user: usize,
    /// Addresses that failed for another reason.
    pub failed: usize,
}

/// Parse `UF_SUPER_USER_EMAILS` (comma / semicolon / whitespace separated).
///
/// Empty tokens are dropped. Unset or blank env yields an empty list.
#[must_use]
pub fn parse_uf_super_user_emails(raw: Option<&str>) -> Vec<String> {
    let Some(raw) = raw.map(str::trim).filter(|s| !s.is_empty()) else {
        return Vec::new();
    };
    raw.split(|c: char| c == ',' || c == ';' || c.is_whitespace())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect()
}

/// Read [`UF_SUPER_USER_EMAILS`] from the process environment.
#[must_use]
pub fn parse_uf_super_user_emails_from_env() -> Vec<String> {
    parse_uf_super_user_emails(std::env::var(UF_SUPER_USER_EMAILS).ok().as_deref())
}

/// Ensure Super User group and seed configured emails (best-effort).
///
/// Uses Gauge [`ensure_super_user_group`] plus per-email
/// [`seed_super_user_member_by_email`], soft-failing missing users so boot can
/// restart after signup. When the linked gauge exports
/// `seed_super_user_members_from_emails`, hosts may switch to that helper; this
/// path stays compatible with published gauge crates that only have the
/// single-email API.
///
/// # Errors
///
/// Returns when ensuring the group fails. Missing emails do not fail the call.
pub async fn ensure_and_seed_super_users_for_valence(
    valence: &Valence,
) -> anyhow::Result<SuperUserBootStats> {
    let group = ensure_super_user_group(valence).await?;
    let emails = parse_uf_super_user_emails_from_env();
    let mut stats = SuperUserBootStats {
        configured: emails.len(),
        ..SuperUserBootStats::default()
    };
    for email in &emails {
        match seed_super_user_member_by_email(valence, &group, email).await {
            Ok(()) => stats.seeded += 1,
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("no user found for email") {
                    stats.missing_user += 1;
                } else {
                    stats.failed += 1;
                    log::warn!("[server] super_user_boot: seed failed (details redacted)");
                }
            }
        }
    }
    log::info!(
        "[server] super_user_boot: configured={} seeded={} missing_user={} failed={}",
        stats.configured,
        stats.seeded,
        stats.missing_user,
        stats.failed
    );
    if stats.missing_user > 0 {
        log::warn!(
            "[server] super_user_boot: {} configured email(s) have no Lepton user yet; sign up, then restart",
            stats.missing_user
        );
    }
    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::parse_uf_super_user_emails;

    #[test]
    fn parse_uf_super_user_emails_splits_and_trims_happy_path() {
        let emails = parse_uf_super_user_emails(Some("a@x.com, b@y.com;c@z.com\td@w.com"));
        assert_eq!(
            emails,
            vec![
                "a@x.com".to_string(),
                "b@y.com".to_string(),
                "c@z.com".to_string(),
                "d@w.com".to_string(),
            ]
        );
    }

    #[test]
    fn parse_uf_super_user_emails_unset_or_blank_sad() {
        assert!(parse_uf_super_user_emails(None).is_empty());
        assert!(parse_uf_super_user_emails(Some("")).is_empty());
        assert!(parse_uf_super_user_emails(Some("  , ;  ")).is_empty());
    }
}
