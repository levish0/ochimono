//! Deterministic integration tests for the generation-fenced session store.
//!
//! Run with a disposable Redis:
//! `cargo test -p auth-core --test session_store_redis -- --ignored --test-threads=1`.

use auth_core::session::{
    INITIAL_SESSION_GENERATION, MAX_SESSION_GENERATION, Session, SessionPolicy, keys,
};
use auth_core::session_store::{
    DeleteOutcome, IssueOutcome, RevokeOthersOutcome, current_generation, delete, get, issue,
    list_user_sessions, revoke_all, revoke_others,
};
use chrono::{Duration, Utc};
use redis::AsyncCommands;
use redis::aio::ConnectionManager;
use std::sync::Arc;
use tokio::sync::Barrier;
use uuid::Uuid;

fn redis_url() -> String {
    std::env::var("SESSION_REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379/".to_string())
}

async fn test_redis() -> ConnectionManager {
    let client = redis::Client::open(redis_url()).expect("parse disposable Redis URL");
    ConnectionManager::new(client)
        .await
        .expect("connect to disposable Redis")
}

fn redis_url_for_user(username: &str, password: &str) -> String {
    let base = redis_url();
    let authority = base
        .strip_prefix("redis://")
        .expect("SESSION_REDIS_URL uses the redis:// scheme");
    assert!(
        !authority.contains('@'),
        "read-only Redis test expects SESSION_REDIS_URL without credentials"
    );
    format!("redis://{username}:{password}@{authority}")
}

async fn test_read_only_redis(username: &str, password: &str) -> ConnectionManager {
    let client = redis::Client::open(redis_url_for_user(username, password))
        .expect("parse read-only Redis URL");
    ConnectionManager::new(client)
        .await
        .expect("connect with read-only Redis ACL user")
}

fn policy() -> SessionPolicy {
    SessionPolicy {
        sliding_ttl_hours: 1,
        max_lifetime_hours: 24,
    }
}

#[tokio::test]
#[ignore = "requires disposable Redis"]
async fn renewal_cannot_restore_revoked_or_changed_sessions() {
    use auth_core::session_store::refresh;
    let redis = test_redis().await;
    let user = Uuid::now_v7().to_string();
    let original = session(&user, 1, "renewal");
    issue_ok(&redis, &original).await;
    let mut renewed = original.clone();
    renewed.expires_at += Duration::minutes(10);
    assert!(
        refresh(&redis, &original, &renewed, policy())
            .await
            .unwrap()
    );
    assert!(
        !refresh(&redis, &original, &renewed, policy())
            .await
            .unwrap()
    );
    assert_eq!(
        get(&redis, &original.session_id)
            .await
            .unwrap()
            .unwrap()
            .expires_at,
        renewed.expires_at
    );
    let mut invalid = renewed.clone();
    invalid.security_generation += 1;
    assert!(refresh(&redis, &renewed, &invalid, policy()).await.is_err());
    invalid = renewed.clone();
    invalid.expires_at = invalid.max_expires_at + Duration::seconds(1);
    assert!(refresh(&redis, &renewed, &invalid, policy()).await.is_err());
    delete(&redis, &renewed.session_id).await.unwrap();
    assert!(!refresh(&redis, &renewed, &renewed, policy()).await.unwrap());
    assert!(get(&redis, &renewed.session_id).await.unwrap().is_none());
    let another = session(&user, 1, "renewal-revoke-all");
    issue_ok(&redis, &another).await;
    revoke_all(&redis, &user).await.unwrap();
    assert!(!refresh(&redis, &another, &another, policy()).await.unwrap());
    let concurrent = session(&user, 2, "concurrent-renewal");
    issue_ok(&redis, &concurrent).await;
    let mut extended = concurrent.clone();
    extended.expires_at += Duration::minutes(5);
    let (renewal, deletion) = tokio::join!(
        refresh(&redis, &concurrent, &extended, policy()),
        delete(&redis, &concurrent.session_id)
    );
    renewal.unwrap();
    deletion.unwrap();
    assert!(get(&redis, &concurrent.session_id).await.unwrap().is_none());
}

fn session(user_id: &str, generation: u64, label: &str) -> Session {
    let now = Utc::now();
    Session {
        session_id: format!("session-test-{label}-{}", Uuid::now_v7()),
        management_id: format!("management-test-{label}-{}", Uuid::now_v7()),
        user_id: user_id.to_string(),
        generation,
        security_generation: generation as i64,
        created_at: now,
        expires_at: now + Duration::hours(1),
        max_expires_at: now + Duration::hours(24),
        user_agent: Some("session integrity test".to_string()),
        ip_address: Some("192.0.2.1".to_string()),
        device_management_id: None,
        persistent: false,
    }
}

async fn issue_ok(redis: &ConnectionManager, session: &Session) {
    assert_eq!(
        issue(redis, session, policy(), 3_600)
            .await
            .expect("issue session"),
        IssueOutcome::Issued
    );
}

async fn assert_session_keys_absent(redis: &ConnectionManager, session: &Session) {
    let mut conn = redis.clone();
    let exists: i64 = redis::cmd("EXISTS")
        .arg(keys::session(&session.session_id))
        .arg(keys::management(&session.management_id))
        .query_async(&mut conn)
        .await
        .expect("inspect session keys");
    assert_eq!(exists, 0);
    let indexed: Option<f64> = redis::cmd("ZSCORE")
        .arg(keys::user_index(&session.user_id))
        .arg(&session.management_id)
        .query_async(&mut conn)
        .await
        .expect("inspect session index");
    assert!(indexed.is_none());
}

async fn cleanup(redis: &ConnectionManager, user_id: &str, sessions: &[&Session]) {
    let mut conn = redis.clone();
    let mut pipe = redis::pipe();
    for session in sessions {
        pipe.del(keys::session(&session.session_id)).ignore();
        pipe.del(keys::management(&session.management_id)).ignore();
    }
    pipe.del(keys::user_index(user_id)).ignore();
    pipe.del(keys::generation(user_id)).ignore();
    pipe.query_async::<()>(&mut conn)
        .await
        .expect("clean exact integration-test keys");
}

#[tokio::test]
#[ignore = "requires a disposable Redis"]
async fn issue_and_revoke_are_linearizable_in_both_orderings() {
    let redis = test_redis().await;
    let user_id = Uuid::now_v7().to_string();

    // create-before-revoke: the revoker observes and removes the complete issued tuple.
    let before = session(&user_id, INITIAL_SESSION_GENERATION, "before-revoke");
    issue_ok(&redis, &before).await;
    let first_revoke = revoke_all(&redis, &user_id).await.expect("first revoke");
    assert_eq!(first_revoke.deleted_count, 1);
    assert_session_keys_absent(&redis, &before).await;

    // revoke-before-create: a proof captured at generation 2 cannot write any partial key after
    // the next revoke advances the user to generation 3.
    let captured = current_generation(&redis, &user_id)
        .await
        .expect("capture generation");
    assert_eq!(captured, 2);
    let stale_issue = session(&user_id, captured, "after-revoke");
    let second_revoke = revoke_all(&redis, &user_id).await.expect("second revoke");
    assert_eq!(second_revoke.generation, 3);
    assert_eq!(
        issue(&redis, &stale_issue, policy(), 3_600)
            .await
            .expect("stale issue result"),
        IssueOutcome::StaleGeneration
    );
    assert_session_keys_absent(&redis, &stale_issue).await;

    cleanup(&redis, &user_id, &[&before, &stale_issue]).await;
}

#[tokio::test]
#[ignore = "requires a disposable Redis"]
async fn wrong_type_user_index_cannot_leave_a_partially_issued_session() {
    let redis = test_redis().await;
    let user_id = Uuid::now_v7().to_string();
    let candidate = session(&user_id, INITIAL_SESSION_GENERATION, "wrong-type-issue");
    let mut conn = redis.clone();
    let index_key = keys::user_index(&user_id);
    let _: () = conn
        .set(&index_key, "not-a-zset")
        .await
        .expect("inject wrong-type user index");

    assert!(
        issue(&redis, &candidate, policy(), 3_600).await.is_err(),
        "issue must reject a corrupt user index before writing any tuple member"
    );
    let partial_keys: i64 = redis::cmd("EXISTS")
        .arg(keys::generation(&user_id))
        .arg(keys::session(&candidate.session_id))
        .arg(keys::management(&candidate.management_id))
        .query_async(&mut conn)
        .await
        .expect("inspect rejected issue keys");
    assert_eq!(partial_keys, 0, "failed issue must leave no partial tuple");

    cleanup(&redis, &user_id, &[&candidate]).await;
}

#[tokio::test]
#[ignore = "requires a disposable Redis"]
async fn oversized_ttls_fail_before_issue_mutates_session_state() {
    let redis = test_redis().await;
    let user_id = Uuid::now_v7().to_string();
    let rejected = session(&user_id, INITIAL_SESSION_GENERATION, "oversized-issue-ttl");

    assert!(
        issue(&redis, &rejected, policy(), u64::MAX).await.is_err(),
        "an unrepresentable Redis TTL must be rejected before Lua writes"
    );
    assert_session_keys_absent(&redis, &rejected).await;

    let extreme_policy = SessionPolicy {
        max_lifetime_hours: i64::MAX,
        ..policy()
    };
    let rejected_index = session(&user_id, INITIAL_SESSION_GENERATION, "oversized-index-ttl");
    assert!(
        issue(&redis, &rejected_index, extreme_policy, 3_600)
            .await
            .is_err(),
        "an unrepresentable user-index TTL must be rejected before Lua writes"
    );
    assert_session_keys_absent(&redis, &rejected_index).await;

    cleanup(&redis, &user_id, &[&rejected, &rejected_index]).await;
}

#[tokio::test]
#[ignore = "requires a disposable Redis"]
async fn listing_wrong_type_error_cannot_partially_prune_an_earlier_entry() {
    let redis = test_redis().await;
    let user_id = Uuid::now_v7().to_string();
    let first_management_id = format!("management-list-first-{}", Uuid::now_v7());
    let corrupt_management_id = format!("management-list-corrupt-{}", Uuid::now_v7());
    let index_key = keys::user_index(&user_id);
    let corrupt_management_key = keys::management(&corrupt_management_id);
    let mut conn = redis.clone();
    let _: () = redis::pipe()
        .cmd("ZADD")
        .arg(&index_key)
        .arg(1)
        .arg(&first_management_id)
        .ignore()
        .cmd("ZADD")
        .arg(&index_key)
        .arg(2)
        .arg(&corrupt_management_id)
        .ignore()
        .cmd("RPUSH")
        .arg(&corrupt_management_key)
        .arg("wrong-type")
        .ignore()
        .query_async(&mut conn)
        .await
        .expect("inject list corruption");

    assert!(
        list_user_sessions(&redis, &user_id).await.is_err(),
        "a wrong-type management key must fail the listing"
    );
    let indexed: Vec<String> = redis::cmd("ZRANGE")
        .arg(&index_key)
        .arg(0)
        .arg(-1)
        .query_async(&mut conn)
        .await
        .expect("inspect index after failed listing");
    assert_eq!(
        indexed,
        vec![first_management_id, corrupt_management_id],
        "a later read error must not leave earlier cleanup writes committed"
    );

    let _: i64 = redis::cmd("DEL")
        .arg(&corrupt_management_key)
        .query_async(&mut conn)
        .await
        .expect("clean corrupt management key");
    cleanup(&redis, &user_id, &[]).await;
}

#[tokio::test]
#[ignore = "requires a disposable Redis"]
async fn revoke_others_bumps_generation_and_preserves_exactly_current_session() {
    let redis = test_redis().await;
    let user_id = Uuid::now_v7().to_string();
    let current = session(&user_id, INITIAL_SESSION_GENERATION, "current");
    let other_a = session(&user_id, INITIAL_SESSION_GENERATION, "other-a");
    let other_b = session(&user_id, INITIAL_SESSION_GENERATION, "other-b");
    issue_ok(&redis, &current).await;
    issue_ok(&redis, &other_a).await;
    issue_ok(&redis, &other_b).await;

    let outcome = revoke_others(&redis, &user_id, &current.session_id, 42)
        .await
        .expect("revoke other sessions");
    let RevokeOthersOutcome::Preserved(result) = outcome else {
        panic!("current session should be preserved");
    };
    assert_eq!(result.generation, 2);
    assert_eq!(result.deleted_count, 2);

    let rebound = get(&redis, &current.session_id)
        .await
        .expect("read current session")
        .expect("current session remains");
    assert_eq!(rebound.generation, 2);
    assert_eq!(rebound.security_generation, 42);
    assert!(get(&redis, &other_a.session_id).await.unwrap().is_none());
    assert!(get(&redis, &other_b.session_id).await.unwrap().is_none());
    assert_session_keys_absent(&redis, &other_a).await;
    assert_session_keys_absent(&redis, &other_b).await;

    let mut conn = redis.clone();
    let management_target: Option<String> = conn
        .get(keys::management(&current.management_id))
        .await
        .expect("read current management mapping");
    assert_eq!(
        management_target.as_deref(),
        Some(current.session_id.as_str())
    );
    let indexed: Vec<String> = redis::cmd("ZRANGE")
        .arg(keys::user_index(&user_id))
        .arg(0)
        .arg(-1)
        .query_async(&mut conn)
        .await
        .expect("read final user index");
    assert_eq!(indexed, vec![current.management_id.clone()]);

    cleanup(&redis, &user_id, &[&current, &other_a, &other_b]).await;
}

#[tokio::test]
#[ignore = "requires a disposable Redis"]
async fn concurrent_listing_and_revoke_others_never_remove_the_rebound_session_tuple() {
    // No sleeps: each iteration starts both real Redis operations behind one barrier. Redis may
    // linearize list before or after revoke-others, but either valid order must retain the
    // re-bound current session's payload, management mapping, and index membership.
    for iteration in 0..32 {
        let redis = test_redis().await;
        let user_id = Uuid::now_v7().to_string();
        let current = session(
            &user_id,
            INITIAL_SESSION_GENERATION,
            &format!("race-current-{iteration}"),
        );
        let other = session(
            &user_id,
            INITIAL_SESSION_GENERATION,
            &format!("race-other-{iteration}"),
        );
        issue_ok(&redis, &current).await;
        issue_ok(&redis, &other).await;

        let start = Arc::new(Barrier::new(3));
        let list_start = Arc::clone(&start);
        let list_redis = redis.clone();
        let list_user_id = user_id.clone();
        let listing = tokio::spawn(async move {
            list_start.wait().await;
            list_user_sessions(&list_redis, &list_user_id).await
        });

        let revoke_start = Arc::clone(&start);
        let revoke_redis = redis.clone();
        let revoke_user_id = user_id.clone();
        let current_session_id = current.session_id.clone();
        let revoking = tokio::spawn(async move {
            revoke_start.wait().await;
            revoke_others(&revoke_redis, &revoke_user_id, &current_session_id, 2).await
        });

        start.wait().await;
        listing
            .await
            .expect("listing task completes")
            .expect("listing succeeds");
        let outcome = revoking
            .await
            .expect("revocation task completes")
            .expect("revocation succeeds");
        assert!(matches!(outcome, RevokeOthersOutcome::Preserved(_)));

        let rebound = get(&redis, &current.session_id)
            .await
            .expect("read re-bound current session")
            .expect("current session survives concurrent listing");
        assert_eq!(rebound.generation, 2);
        let mut conn = redis.clone();
        let mapping: Option<String> = conn
            .get(keys::management(&current.management_id))
            .await
            .expect("read current management mapping");
        assert_eq!(mapping.as_deref(), Some(current.session_id.as_str()));
        let indexed: Option<f64> = redis::cmd("ZSCORE")
            .arg(keys::user_index(&user_id))
            .arg(&current.management_id)
            .query_async(&mut conn)
            .await
            .expect("read current index membership");
        assert!(indexed.is_some());
        assert_session_keys_absent(&redis, &other).await;

        cleanup(&redis, &user_id, &[&current, &other]).await;
    }
}

#[tokio::test]
#[ignore = "requires a disposable Redis"]
async fn corrupt_index_cannot_remove_another_users_management_mapping() {
    let redis = test_redis().await;
    let owner_id = Uuid::now_v7().to_string();
    let corrupted_index_user_id = Uuid::now_v7().to_string();
    let victim = session(&owner_id, INITIAL_SESSION_GENERATION, "cross-user-index");
    issue_ok(&redis, &victim).await;

    let mut conn = redis.clone();
    let _: () = redis::cmd("ZADD")
        .arg(keys::user_index(&corrupted_index_user_id))
        .arg(victim.expires_at.timestamp())
        .arg(&victim.management_id)
        .query_async(&mut conn)
        .await
        .expect("inject corrupted foreign index entry");

    assert!(
        list_user_sessions(&redis, &corrupted_index_user_id)
            .await
            .expect("list corrupted index")
            .is_empty()
    );
    let foreign_membership: Option<f64> = redis::cmd("ZSCORE")
        .arg(keys::user_index(&corrupted_index_user_id))
        .arg(&victim.management_id)
        .query_async(&mut conn)
        .await
        .expect("read pruned corrupt index membership");
    assert!(foreign_membership.is_none());
    let mapping: Option<String> = conn
        .get(keys::management(&victim.management_id))
        .await
        .expect("read victim management mapping");
    assert_eq!(mapping.as_deref(), Some(victim.session_id.as_str()));
    assert!(
        get(&redis, &victim.session_id)
            .await
            .expect("read victim session")
            .is_some(),
        "a corrupted foreign index must not evict the victim session"
    );

    cleanup(&redis, &owner_id, &[&victim]).await;
    let _: () = conn
        .del(keys::user_index(&corrupted_index_user_id))
        .await
        .expect("clean corrupted test index");
}

#[tokio::test]
#[ignore = "requires a disposable Redis"]
async fn generation_is_monotonic_nonexpiring_and_overflow_fails_without_mutation() {
    let redis = test_redis().await;
    let user_id = Uuid::now_v7().to_string();
    assert_eq!(
        current_generation(&redis, &user_id).await.unwrap(),
        INITIAL_SESSION_GENERATION
    );

    assert_eq!(revoke_all(&redis, &user_id).await.unwrap().generation, 2);
    assert_eq!(revoke_all(&redis, &user_id).await.unwrap().generation, 3);
    let mut conn = redis.clone();
    let ttl: i64 = conn
        .ttl(keys::generation(&user_id))
        .await
        .expect("read generation TTL");
    assert_eq!(ttl, -1, "generation key must never expire");

    // Lua must be able to re-encode the exact maximum through cjson. The old 2^53-1 limit is
    // mathematically exact as a double but Redis cjson emits it in rounded scientific notation.
    let _: () = conn
        .set(keys::generation(&user_id), MAX_SESSION_GENERATION - 1)
        .await
        .expect("set generation immediately below the maximum");
    let boundary = session(
        &user_id,
        MAX_SESSION_GENERATION - 1,
        "cjson-safe-generation-boundary",
    );
    issue_ok(&redis, &boundary).await;
    let RevokeOthersOutcome::Preserved(boundary_revoke) =
        revoke_others(&redis, &user_id, &boundary.session_id, 99)
            .await
            .expect("rebind session at cjson-safe maximum")
    else {
        panic!("current boundary session should be preserved");
    };
    assert_eq!(boundary_revoke.generation, MAX_SESSION_GENERATION);
    assert_eq!(
        current_generation(&redis, &user_id).await.unwrap(),
        MAX_SESSION_GENERATION
    );
    assert_eq!(
        get(&redis, &boundary.session_id)
            .await
            .expect("read cjson-rebound boundary session")
            .expect("boundary session survives")
            .generation,
        MAX_SESSION_GENERATION
    );
    assert!(
        revoke_all(&redis, &user_id).await.is_err(),
        "incrementing the maximum must fail without mutation"
    );

    cleanup(&redis, &user_id, &[&boundary]).await;
}

#[tokio::test]
#[ignore = "requires a disposable Redis"]
async fn corrupt_bearer_payload_is_removed_without_trusting_related_keys() {
    let redis = test_redis().await;
    let user_id = Uuid::now_v7().to_string();
    let stored = session(
        &user_id,
        INITIAL_SESSION_GENERATION,
        "corrupt-bearer-delete",
    );
    issue_ok(&redis, &stored).await;

    let mut conn = redis.clone();
    let _: () = conn
        .set(keys::session(&stored.session_id), "not valid session JSON")
        .await
        .expect("corrupt only the bearer payload");
    assert_eq!(
        delete(&redis, &stored.session_id)
            .await
            .expect("safe delete result"),
        DeleteOutcome::PayloadOnlyRemoved
    );
    let bearer_exists: bool = conn
        .exists(keys::session(&stored.session_id))
        .await
        .expect("read removed bearer payload");
    assert!(!bearer_exists);
    let mapping: Option<String> = conn
        .get(keys::management(&stored.management_id))
        .await
        .expect("read untouched management mapping");
    assert_eq!(mapping.as_deref(), Some(stored.session_id.as_str()));
    let indexed: Option<f64> = redis::cmd("ZSCORE")
        .arg(keys::user_index(&user_id))
        .arg(&stored.management_id)
        .query_async(&mut conn)
        .await
        .expect("read untouched index entry");
    assert!(indexed.is_some());

    cleanup(&redis, &user_id, &[&stored]).await;
}

#[tokio::test]
#[ignore = "requires a disposable Redis"]
async fn read_only_connection_can_classify_a_current_session_without_writes() {
    let redis = test_redis().await;
    let user_id = Uuid::now_v7().to_string();
    let stored = session(&user_id, INITIAL_SESSION_GENERATION, "read-only-store");
    issue_ok(&redis, &stored).await;

    let reader_name = format!("session_reader_{}", Uuid::now_v7().simple());
    let reader_password = format!("reader_{}", Uuid::now_v7().simple());
    let mut admin = redis.clone();
    let _: String = redis::cmd("ACL")
        .arg("SETUSER")
        .arg(&reader_name)
        .arg("on")
        .arg(format!(">{reader_password}"))
        .arg("~session:*")
        .arg("~session_mgmt:*")
        .arg("~user_sessions:*")
        .arg("~session_generation:*")
        .arg("+get")
        .arg("+mget")
        .arg("+zscore")
        .arg("+client")
        .arg("+ping")
        .arg("+hello")
        .arg("+select")
        .query_async(&mut admin)
        .await
        .expect("create read-only Redis ACL user");

    let reader = test_read_only_redis(&reader_name, &reader_password).await;
    assert!(
        get(&reader, &stored.session_id)
            .await
            .expect("read-only session classification")
            .is_some(),
        "non-authoritative classification uses only commands allowed to a read-only connection"
    );
    let mut reader_for_write_check = reader.clone();
    let denied_write: redis::RedisResult<String> = redis::cmd("SET")
        .arg("session:reader-write-probe")
        .arg("nope")
        .query_async(&mut reader_for_write_check)
        .await;
    assert!(
        denied_write.is_err(),
        "the classification connection has no write privilege"
    );

    let _: i64 = redis::cmd("ACL")
        .arg("DELUSER")
        .arg(&reader_name)
        .query_async(&mut admin)
        .await
        .expect("remove read-only Redis ACL user");
    cleanup(&redis, &user_id, &[&stored]).await;
}
