//! Generation-fenced Redis primitives for the opaque session contract.
//!
//! Every multi-key mutation, plus issuer-only list-and-prune, is a Lua script executed by the
//! Redis primary. Single-session lookup uses read commands only, including an MGET snapshot of its
//! payload and generation. Auth always uses the primary: a disconnected replica can serve a
//! revoked positive result for its full local TTL. A missing generation key means generation 1;
//! mutation scripts materialize the key without an expiry.

use crate::session::{
    INITIAL_SESSION_GENERATION, MAX_SESSION_GENERATION, MAX_SESSION_TTL_SECONDS, Session,
    SessionGeneration, SessionPolicy, keys,
};
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use std::fmt;
use std::sync::LazyLock;

static ISSUE_SCRIPT: LazyLock<redis::Script> =
    LazyLock::new(|| redis::Script::new(include_str!("session_lua/issue.lua")));
static DELETE_SCRIPT: LazyLock<redis::Script> =
    LazyLock::new(|| redis::Script::new(include_str!("session_lua/delete.lua")));
static REVOKE_ALL_SCRIPT: LazyLock<redis::Script> =
    LazyLock::new(|| redis::Script::new(include_str!("session_lua/revoke_all.lua")));
static REVOKE_OTHERS_SCRIPT: LazyLock<redis::Script> =
    LazyLock::new(|| redis::Script::new(include_str!("session_lua/revoke_others.lua")));
static LIST_USER_SESSIONS_SCRIPT: LazyLock<redis::Script> =
    LazyLock::new(|| redis::Script::new(include_str!("session_lua/list_user_sessions.lua")));

/// Why a session store operation failed.
#[derive(Debug)]
pub enum SessionStoreError {
    /// The store was unreachable or rejected the command.
    Redis(String),
    /// A stored payload could not be read or violated the session-key contract.
    Payload(String),
    /// The per-user generation key was corrupt or exhausted its Lua-safe integer range.
    Generation(String),
}

impl fmt::Display for SessionStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Redis(message) => write!(f, "session store command failed: {message}"),
            Self::Payload(message) => write!(f, "session payload is not readable: {message}"),
            Self::Generation(message) => write!(f, "session generation is invalid: {message}"),
        }
    }
}

impl std::error::Error for SessionStoreError {}

pub type StoreResult<T> = Result<T, SessionStoreError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IssueOutcome {
    Issued,
    StaleGeneration,
}

/// The result of deleting the bearer key for one session.
///
/// `PayloadOnlyRemoved` deliberately leaves related management/index keys in place: when their
/// relationship to the bearer payload is corrupt, deleting them could destroy a different valid
/// session. A later list/revocation can safely prune those orphaned keys under its own fence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteOutcome {
    Deleted,
    Missing,
    PayloadOnlyRemoved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RevocationResult {
    pub generation: SessionGeneration,
    pub deleted_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RevokeOthersOutcome {
    Preserved(RevocationResult),
    CurrentSessionUnavailable,
}

fn redis_error(error: redis::RedisError) -> SessionStoreError {
    let message = error.to_string();
    if message.contains("SESSION_GENERATION") {
        SessionStoreError::Generation(message)
    } else if message.contains("SESSION_PAYLOAD") || message.contains("SESSION_INVARIANT") {
        SessionStoreError::Payload(message)
    } else {
        SessionStoreError::Redis(message)
    }
}

fn payload_error(message: impl Into<String>) -> SessionStoreError {
    SessionStoreError::Payload(message.into())
}

fn validate_redis_ttl(name: &str, ttl_seconds: u64) -> StoreResult<()> {
    if ttl_seconds == 0 || ttl_seconds > MAX_SESSION_TTL_SECONDS {
        return Err(payload_error(format!(
            "{name} must be within 1..={MAX_SESSION_TTL_SECONDS} seconds"
        )));
    }
    Ok(())
}

fn validated_user_index_ttl(policy: SessionPolicy) -> StoreResult<u64> {
    let ttl_seconds = policy.user_index_ttl_seconds();
    validate_redis_ttl("user index TTL", ttl_seconds)?;
    Ok(ttl_seconds)
}

fn parse_generation(raw: Option<String>) -> StoreResult<SessionGeneration> {
    let Some(raw) = raw else {
        return Ok(INITIAL_SESSION_GENERATION);
    };
    let generation = raw.parse::<SessionGeneration>().map_err(|_| {
        SessionStoreError::Generation(format!("'{raw}' is not a positive decimal integer"))
    })?;
    if generation == 0 || generation > MAX_SESSION_GENERATION || generation.to_string() != raw {
        return Err(SessionStoreError::Generation(format!(
            "'{raw}' is outside the canonical Lua-safe range"
        )));
    }
    Ok(generation)
}

fn validate_session(session: &Session, requested_session_id: &str) -> StoreResult<()> {
    if session.session_id != requested_session_id {
        return Err(payload_error(
            "stored session_id does not match its Redis key",
        ));
    }
    if session.user_id.is_empty() || session.management_id.is_empty() {
        return Err(payload_error("stored session identity is incomplete"));
    }
    if session.generation == 0 || session.generation > MAX_SESSION_GENERATION {
        return Err(payload_error(
            "stored session generation is outside the valid range",
        ));
    }
    if session.security_generation < 1 {
        return Err(payload_error("stored security generation must be positive"));
    }
    Ok(())
}

/// Reads the user's current generation. The missing key is the initial generation 1.
pub async fn current_generation(
    conn: &ConnectionManager,
    user_id: &str,
) -> StoreResult<SessionGeneration> {
    let mut conn = conn.clone();
    let raw: Option<String> = conn
        .get(keys::generation(user_id))
        .await
        .map_err(redis_error)?;
    parse_generation(raw)
}

/// Reads a payload and its generation in one internally consistent Redis snapshot.
/// A payload from an older generation is indistinguishable from a revoked/missing session.
pub async fn get(conn: &ConnectionManager, session_id: &str) -> StoreResult<Option<Session>> {
    let mut conn = conn.clone();
    let session_key = keys::session(session_id);
    let payload: Option<String> = conn.get(&session_key).await.map_err(redis_error)?;
    let Some(payload) = payload else {
        return Ok(None);
    };

    // The user id is required to derive the generation key. Parse once, then atomically re-read
    // both values so a revoke between the first payload lookup and the snapshot cannot authenticate
    // that payload. The first read is only key discovery and is never returned to the caller.
    let discovered: Session =
        serde_json::from_str(&payload).map_err(|error| payload_error(error.to_string()))?;
    validate_session(&discovered, session_id)?;

    let (payload, generation): (Option<String>, Option<String>) = redis::cmd("MGET")
        .arg(&session_key)
        .arg(keys::generation(&discovered.user_id))
        .query_async(&mut conn)
        .await
        .map_err(redis_error)?;
    let Some(payload) = payload else {
        return Ok(None);
    };
    let session: Session =
        serde_json::from_str(&payload).map_err(|error| payload_error(error.to_string()))?;
    validate_session(&session, session_id)?;
    if session.user_id != discovered.user_id {
        return Err(payload_error("stored session user changed during lookup"));
    }

    let generation = parse_generation(generation)?;
    if session.generation != generation {
        return Ok(None);
    }

    // A current-generation orphan is still not a session. These checks use read commands only,
    // but callers making authorization decisions must use an authoritative primary connection.
    let (managed_session_id, indexed_score): (Option<String>, Option<f64>) = redis::pipe()
        .get(keys::management(&session.management_id))
        .cmd("ZSCORE")
        .arg(keys::user_index(&session.user_id))
        .arg(&session.management_id)
        .query_async(&mut conn)
        .await
        .map_err(redis_error)?;
    if managed_session_id.as_deref() != Some(session_id) || indexed_score.is_none() {
        return Ok(None);
    }
    Ok(Some(session))
}

/// Resolves an opaque refresh session by its public management UUID.
///
/// The second payload read revalidates the management mapping, index membership,
/// and Redis generation, so a concurrent rotation/revocation cannot return a
/// detached payload. Ownership mismatch is indistinguishable from absence.
pub async fn get_managed(
    conn: &ConnectionManager,
    user_id: &str,
    management_id: &str,
) -> StoreResult<Option<Session>> {
    let mut redis = conn.clone();
    let Some(session_id): Option<String> = redis
        .get(keys::management(management_id))
        .await
        .map_err(redis_error)?
    else {
        return Ok(None);
    };
    let Some(session) = get(conn, &session_id).await? else {
        return Ok(None);
    };
    if session.user_id != user_id {
        return Ok(None);
    }
    if session.management_id != management_id {
        return Err(payload_error(
            "managed session lookup resolved a different management id",
        ));
    }
    Ok(Some(session))
}

/// Issues a session only if its expected generation is still current. All session, management,
/// index, and generation keys are written atomically; a stale generation writes nothing.
pub async fn issue(
    conn: &ConnectionManager,
    session: &Session,
    policy: SessionPolicy,
    ttl_seconds: u64,
) -> StoreResult<IssueOutcome> {
    validate_session(session, &session.session_id)?;
    validate_redis_ttl("session TTL", ttl_seconds)?;
    let user_index_ttl_seconds = validated_user_index_ttl(policy)?;
    let payload =
        serde_json::to_string(session).map_err(|error| payload_error(error.to_string()))?;

    let mut conn = conn.clone();
    let issued: i64 = ISSUE_SCRIPT
        .key(keys::generation(&session.user_id))
        .key(keys::session(&session.session_id))
        .key(keys::management(&session.management_id))
        .key(keys::user_index(&session.user_id))
        .arg(session.generation)
        .arg(payload)
        .arg(&session.session_id)
        .arg(&session.management_id)
        .arg(session.expires_at.timestamp())
        .arg(ttl_seconds)
        .arg(user_index_ttl_seconds)
        .invoke_async(&mut conn)
        .await
        .map_err(redis_error)?;

    Ok(if issued == 1 {
        IssueOutcome::Issued
    } else {
        IssueOutcome::StaleGeneration
    })
}

/// Deletes one bearer session and its matching management/index entries atomically.
///
/// If the bearer payload or its related keys are corrupt, the bearer payload is still removed,
/// but no related key is trusted or modified. This makes logout fail closed without letting a
/// cache inconsistency turn a cookie-clearing operation into a 500.
pub async fn delete(conn: &ConnectionManager, session_id: &str) -> StoreResult<DeleteOutcome> {
    let mut conn = conn.clone();
    let deleted: i64 = DELETE_SCRIPT
        .key(keys::session(session_id))
        .arg(session_id)
        .arg("session_mgmt:")
        .arg("user_sessions:")
        .invoke_async(&mut conn)
        .await
        .map_err(redis_error)?;
    match deleted {
        0 => Ok(DeleteOutcome::Missing),
        1 => Ok(DeleteOutcome::Deleted),
        2 => Ok(DeleteOutcome::PayloadOnlyRemoved),
        _ => Err(payload_error("unexpected delete script response")),
    }
}

/// Reads and prunes a user's session index under one Redis Lua linearization point.
///
/// The former read-then-prune sequence could observe generation N, let `revoke_others` rebind the
/// kept session to N+1, and then delete that now-valid session's management/index entries. The
/// script validates each index entry and removes only stale entries as part of the same snapshot.
pub async fn list_user_sessions(
    conn: &ConnectionManager,
    user_id: &str,
) -> StoreResult<Vec<Session>> {
    let mut conn = conn.clone();
    let payloads: Vec<String> = LIST_USER_SESSIONS_SCRIPT
        .key(keys::generation(user_id))
        .key(keys::user_index(user_id))
        .arg(user_id)
        .arg(MAX_SESSION_GENERATION)
        .arg("session_mgmt:")
        .arg("session:")
        .arg(chrono::Utc::now().timestamp())
        .invoke_async(&mut conn)
        .await
        .map_err(redis_error)?;

    payloads
        .into_iter()
        .map(|payload| {
            let session: Session =
                serde_json::from_str(&payload).map_err(|error| payload_error(error.to_string()))?;
            validate_session(&session, &session.session_id)?;
            if session.user_id != user_id {
                return Err(payload_error("listed session belongs to a different user"));
            }
            Ok(session)
        })
        .collect()
}

/// Atomically advances the generation and deletes every session visible in the user's index.
pub async fn revoke_all(conn: &ConnectionManager, user_id: &str) -> StoreResult<RevocationResult> {
    let mut conn = conn.clone();
    let result: Vec<i64> = REVOKE_ALL_SCRIPT
        .key(keys::generation(user_id))
        .key(keys::user_index(user_id))
        .arg(user_id)
        .arg(MAX_SESSION_GENERATION)
        .arg("session_mgmt:")
        .arg("session:")
        .invoke_async(&mut conn)
        .await
        .map_err(redis_error)?;
    decode_revocation(result)
}

/// Atomically advances the generation, rebinds one exact current session to it, and removes all
/// other indexed sessions. Nothing changes when the current session cannot be preserved.
pub async fn revoke_others(
    conn: &ConnectionManager,
    user_id: &str,
    current_session_id: &str,
    security_generation: i64,
) -> StoreResult<RevokeOthersOutcome> {
    if !(1..=9_007_199_254_740_991).contains(&security_generation) {
        return Err(SessionStoreError::Generation(
            "security generation is outside the Lua-safe positive range".to_owned(),
        ));
    }
    let mut conn = conn.clone();
    let result: Vec<i64> = REVOKE_OTHERS_SCRIPT
        .key(keys::generation(user_id))
        .key(keys::user_index(user_id))
        .key(keys::session(current_session_id))
        .arg(user_id)
        .arg(current_session_id)
        .arg(MAX_SESSION_GENERATION)
        .arg("session_mgmt:")
        .arg("session:")
        .arg(security_generation)
        .invoke_async(&mut conn)
        .await
        .map_err(redis_error)?;

    match result.as_slice() {
        [0, _, 0] => Ok(RevokeOthersOutcome::CurrentSessionUnavailable),
        [1, generation, deleted_count] => Ok(RevokeOthersOutcome::Preserved(RevocationResult {
            generation: (*generation).try_into().map_err(|_| {
                SessionStoreError::Generation("script returned a negative generation".to_string())
            })?,
            deleted_count: (*deleted_count).try_into().map_err(|_| {
                SessionStoreError::Payload("script returned a negative delete count".to_string())
            })?,
        })),
        [-1, generation, 0] => Err(SessionStoreError::Generation(format!(
            "generation {generation} cannot be incremented safely"
        ))),
        _ => Err(payload_error("unexpected revoke-others script response")),
    }
}

fn decode_revocation(result: Vec<i64>) -> StoreResult<RevocationResult> {
    match result.as_slice() {
        [1, generation, deleted_count] => Ok(RevocationResult {
            generation: (*generation).try_into().map_err(|_| {
                SessionStoreError::Generation("script returned a negative generation".to_string())
            })?,
            deleted_count: (*deleted_count).try_into().map_err(|_| {
                SessionStoreError::Payload("script returned a negative delete count".to_string())
            })?,
        }),
        [-1, generation, 0] => Err(SessionStoreError::Generation(format!(
            "generation {generation} cannot be incremented safely"
        ))),
        _ => Err(payload_error("unexpected revoke-all script response")),
    }
}

/// Extend a live session with a compare-and-set operation. Revocation, expiry, or a
/// concurrent payload change makes this operation a no-op instead of resurrecting it.
pub async fn refresh(
    conn: &ConnectionManager,
    expected: &Session,
    updated: &Session,
    policy: SessionPolicy,
) -> StoreResult<bool> {
    validate_session(expected, &expected.session_id)?;
    validate_session(updated, &expected.session_id)?;
    let mut unchanged = updated.clone();
    unchanged.expires_at = expected.expires_at;
    if serde_json::to_value(&unchanged).map_err(|e| payload_error(e.to_string()))?
        != serde_json::to_value(expected).map_err(|e| payload_error(e.to_string()))?
        || expected.expires_at <= chrono::Utc::now()
        || updated.expires_at > updated.max_expires_at
        || updated.expires_at < expected.expires_at
    {
        return Err(payload_error("invalid session renewal"));
    }
    let ttl = (updated.expires_at - chrono::Utc::now()).num_seconds();
    if ttl <= 0 {
        return Ok(false);
    }
    validate_redis_ttl("session TTL", ttl as u64)?;
    let result: i64 = redis::Script::new(include_str!("session_lua/refresh.lua"))
        .key(keys::session(&expected.session_id))
        .key(keys::generation(&expected.user_id))
        .key(keys::management(&expected.management_id))
        .key(keys::user_index(&expected.user_id))
        .arg(serde_json::to_string(expected).map_err(|e| payload_error(e.to_string()))?)
        .arg(expected.generation)
        .arg(&expected.session_id)
        .arg(&expected.management_id)
        .arg(serde_json::to_string(updated).map_err(|e| payload_error(e.to_string()))?)
        .arg(ttl)
        .arg(updated.expires_at.timestamp())
        .arg(validated_user_index_ttl(policy)?)
        .invoke_async(&mut conn.clone())
        .await
        .map_err(redis_error)?;
    Ok(result == 1)
}

#[cfg(test)]
mod tests {
    use super::parse_generation;
    use crate::session::{INITIAL_SESSION_GENERATION, MAX_SESSION_GENERATION};

    #[test]
    fn missing_generation_is_one_and_noncanonical_values_are_rejected() {
        assert_eq!(
            parse_generation(None).expect("missing generation"),
            INITIAL_SESSION_GENERATION
        );
        assert!(parse_generation(Some("0".to_string())).is_err());
        assert!(parse_generation(Some("01".to_string())).is_err());
        assert!(parse_generation(Some((MAX_SESSION_GENERATION + 1).to_string())).is_err());
    }
}
