//! Session payloads, Redis keys and expiry policy.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Keys for the session store.
pub mod keys {
    /// Session payload key.
    pub fn session(session_id: &str) -> String {
        format!("session:{session_id}")
    }

    /// Management id → bearer session id lookup key.
    pub fn management(management_id: &str) -> String {
        format!("session_mgmt:{management_id}")
    }

    /// Per-user active session index (sorted set, scored by expiry).
    pub fn user_index(user_id: &str) -> String {
        format!("user_sessions:{user_id}")
    }

    /// Per-user monotonic session generation. This key never expires; a missing
    /// key is the initial generation for a UUID that has never been revoked.
    pub fn generation(user_id: &str) -> String {
        format!("session_generation:{user_id}")
    }
}

/// A generation is represented as an integer that remains exact when Redis Lua
/// passes it through both its IEEE-754 number type and `cjson`. Generation zero
/// is never valid. One trillion revocations per account is far beyond a
/// realistic service lifetime, while still staying below cjson's 14-digit
/// precision boundary and Lua's scientific-notation formatting threshold.
pub type SessionGeneration = u64;
pub const INITIAL_SESSION_GENERATION: SessionGeneration = 1;
pub const MAX_SESSION_GENERATION: SessionGeneration = 1_000_000_000_000;

/// Conservative Redis-compatible TTL ceiling. Redis converts relative expiry
/// to an absolute signed-millisecond timestamp, so the theoretical `i64`
/// seconds range is not safe after adding the current epoch.
pub const MAX_SESSION_TTL_SECONDS: u64 = i32::MAX as u64;
pub const MAX_SESSION_TTL_HOURS: i64 = (MAX_SESSION_TTL_SECONDS / 3600) as i64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionPolicyError {
    field: &'static str,
    value: i64,
    expected: &'static str,
}

impl fmt::Display for SessionPolicyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} must be {} (got {})",
            self.field, self.expected, self.value
        )
    }
}

impl std::error::Error for SessionPolicyError {}

/// How long sessions live and when they slide. Supplied by the caller's configuration.
#[derive(Debug, Clone, Copy)]
pub struct SessionPolicy {
    /// Sliding TTL granted on each refresh.
    pub sliding_ttl_hours: i64,
    /// Absolute lifetime; a session is never extended past its `max_expires_at`.
    pub max_lifetime_hours: i64,
}

impl SessionPolicy {
    /// Reject policy values before any duration or seconds arithmetic occurs.
    pub fn validate(&self) -> Result<(), SessionPolicyError> {
        for (field, value) in [
            ("sliding TTL hours", self.sliding_ttl_hours),
            ("maximum lifetime hours", self.max_lifetime_hours),
        ] {
            if !(1..=MAX_SESSION_TTL_HOURS).contains(&value) {
                return Err(SessionPolicyError {
                    field,
                    value,
                    expected: "within 1..=596523",
                });
            }
        }

        Ok(())
    }

    pub fn sliding_ttl_seconds(&self) -> Result<u64, SessionPolicyError> {
        self.validate()?;
        Ok(self.sliding_ttl_hours as u64 * 3600)
    }

    /// TTL for the per-user index key: it must outlive every session it can hold.
    pub fn user_index_ttl_seconds(&self) -> u64 {
        let max_lifetime = (self.max_lifetime_hours.max(0) as u64)
            .saturating_mul(3600)
            .max(1);
        let sliding_ttl = (self.sliding_ttl_hours.max(0) as u64)
            .saturating_mul(3600)
            .max(1);
        max_lifetime.max(sliding_ttl)
    }
}

/// The resolved identity of an authenticated caller.
///
/// Not a stored shape — this is what a validated session resolves to in process.
#[derive(Debug, Clone)]
pub struct SessionContext {
    pub user_id: Uuid,
    /// Public logical-session UUID used for listing and revocation.
    /// This is intentionally distinct from the stored session-token hash.
    pub management_id: String,
}

/// The stored session payload.
///
/// `session_id` is the server-side identifier — the **hash** of the raw session token, never
/// the raw credential itself, so Redis never persists a replayable token. `management_id` is the
/// separate, non-secret logical-session UUID used for
/// ownership-checked listing/revocation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    pub management_id: String,
    pub user_id: String,
    pub generation: SessionGeneration,
    /// Durable account generation captured when this session was issued.
    pub security_generation: i64,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub max_expires_at: DateTime<Utc>,
    pub user_agent: Option<String>,
    pub ip_address: Option<String>,
    /// Public trusted-device management ID associated with this session, when present.
    pub device_management_id: Option<String>,
    /// Whether a browser session cookie should survive a browser restart.
    pub persistent: bool,
}

impl Session {
    /// Builds a session starting now, with its sliding and absolute expiries derived from
    /// `policy`.
    pub fn new(
        session_id: String,
        user_id: String,
        generation: SessionGeneration,
        security_generation: i64,
        policy: SessionPolicy,
    ) -> Result<Self, SessionPolicyError> {
        policy.validate()?;
        let now = Utc::now();
        let expires_at = now
            .checked_add_signed(Duration::hours(policy.sliding_ttl_hours))
            .ok_or(SessionPolicyError {
                field: "sliding TTL hours",
                value: policy.sliding_ttl_hours,
                expected: "representable as an expiry timestamp",
            })?;
        let max_expires_at = now
            .checked_add_signed(Duration::hours(policy.max_lifetime_hours))
            .ok_or(SessionPolicyError {
                field: "maximum lifetime hours",
                value: policy.max_lifetime_hours,
                expected: "representable as an expiry timestamp",
            })?;

        Ok(Self {
            session_id,
            management_id: Uuid::now_v7().to_string(),
            user_id,
            generation,
            security_generation,
            created_at: now,
            expires_at,
            max_expires_at,
            user_agent: None,
            ip_address: None,
            device_management_id: None,
            persistent: false,
        })
    }

    /// Attaches the client identifiers kept for session listing and audit.
    pub fn with_client_info(
        mut self,
        user_agent: Option<String>,
        ip_address: Option<String>,
    ) -> Self {
        self.user_agent = user_agent;
        self.ip_address = ip_address;
        self
    }

    pub fn with_device_management_id(mut self, management_id: String) -> Self {
        self.device_management_id = Some(management_id);
        self
    }

    pub fn with_persistence(mut self, persistent: bool) -> Self {
        self.persistent = persistent;
        self
    }

    /// The extended expiry for a refresh, or `None` when the session can no longer be extended
    /// (absolute lifetime reached, or the extension would be zero-length).
    ///
    /// Separate from the store write so the rule — never past `max_expires_at` — is stated once
    /// and can be unit-tested without Redis.
    pub fn refreshed_expiry(&self, policy: SessionPolicy) -> Option<DateTime<Utc>> {
        let now = Utc::now();
        if now >= self.max_expires_at {
            return None;
        }

        policy.validate().ok()?;
        let sliding_expiry = now.checked_add_signed(Duration::hours(policy.sliding_ttl_hours))?;
        let new_expires_at = sliding_expiry.min(self.max_expires_at);

        if (new_expires_at - now).num_seconds() <= 0 {
            return None;
        }
        Some(new_expires_at)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> SessionPolicy {
        SessionPolicy {
            sliding_ttl_hours: 168,
            max_lifetime_hours: 720,
        }
    }

    fn make_session(expires_in_hours: i64) -> Session {
        let now = Utc::now();
        Session {
            session_id: "test-session".to_string(),
            management_id: "test-management".to_string(),
            user_id: "test-user".to_string(),
            generation: INITIAL_SESSION_GENERATION,
            security_generation: INITIAL_SESSION_GENERATION as i64,
            created_at: now,
            expires_at: now + Duration::hours(expires_in_hours),
            max_expires_at: now + Duration::hours(720),
            user_agent: None,
            ip_address: None,
            device_management_id: None,
            persistent: false,
        }
    }

    #[test]
    fn keys_are_derived_from_ids() {
        assert_eq!(keys::session("abc"), "session:abc");
        assert_eq!(keys::management("mgmt"), "session_mgmt:mgmt");
        assert_eq!(keys::user_index("user"), "user_sessions:user");
        assert_eq!(keys::generation("user"), "session_generation:user");
    }

    #[test]
    fn refreshed_expiry_never_exceeds_absolute_lifetime() {
        let mut session = make_session(1);
        // Absolute lifetime ends in an hour; the sliding TTL would go far past it.
        session.max_expires_at = Utc::now() + Duration::hours(1);

        let expiry = session
            .refreshed_expiry(policy())
            .expect("still refreshable");
        assert_eq!(expiry, session.max_expires_at);
    }

    #[test]
    fn refreshed_expiry_is_none_past_absolute_lifetime() {
        let mut session = make_session(1);
        session.max_expires_at = Utc::now() - Duration::seconds(1);

        assert!(session.refreshed_expiry(policy()).is_none());
    }

    #[test]
    fn user_index_ttl_covers_the_longest_lived_session() {
        assert_eq!(policy().user_index_ttl_seconds(), 720 * 3600);
    }

    #[test]
    fn policy_rejects_ttls_that_cannot_be_stored_safely() {
        for invalid_hours in [0, MAX_SESSION_TTL_HOURS + 1, i64::MAX] {
            let invalid = SessionPolicy {
                sliding_ttl_hours: invalid_hours,
                ..policy()
            };
            assert!(invalid.validate().is_err());
            assert!(
                Session::new(
                    "session".to_string(),
                    "user".to_string(),
                    INITIAL_SESSION_GENERATION,
                    1,
                    invalid,
                )
                .is_err()
            );
        }
    }

    #[test]
    fn policy_accepts_the_exact_redis_safe_ttl_boundary() {
        let boundary = SessionPolicy {
            sliding_ttl_hours: MAX_SESSION_TTL_HOURS,
            max_lifetime_hours: MAX_SESSION_TTL_HOURS,
        };

        assert_eq!(
            boundary.sliding_ttl_seconds().expect("valid boundary"),
            MAX_SESSION_TTL_HOURS as u64 * 3600
        );
        assert!(
            Session::new(
                "session".to_string(),
                "user".to_string(),
                INITIAL_SESSION_GENERATION,
                1,
                boundary,
            )
            .is_ok()
        );
    }

    #[test]
    fn stored_session_requires_generation_in_serde_contract() {
        let session = make_session(1);
        let mut value = serde_json::to_value(&session).expect("serialize session");
        assert_eq!(
            value.get("generation").and_then(serde_json::Value::as_u64),
            Some(INITIAL_SESSION_GENERATION)
        );

        value
            .as_object_mut()
            .expect("session is a JSON object")
            .remove("generation");
        assert!(
            serde_json::from_value::<Session>(value).is_err(),
            "generation is a required session field"
        );
    }
}
