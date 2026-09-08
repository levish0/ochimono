use super::AuthState;
use crate::repository::user::{
    repository_advance_security_generation, repository_get_user_by_id,
    repository_get_user_by_id_for_update,
};
use auth_core::{
    session::Session,
    session_store::{self, IssueOutcome},
    token::{generate_secure_token, hash_token},
};
use chrono::Utc;
use entity::users::Model as UserModel;
use errors::errors::{Errors, ServiceResult};
use sea_orm::TransactionTrait;
use uuid::Uuid;

/// Data structure for session service.
pub struct SessionService;

pub struct IssuedSession {
    pub token: String,
    pub expires_at: chrono::DateTime<Utc>,
    pub persistent: bool,
}

/// Maps a store failure onto the application error type. The store is deliberately free of
/// application error types, so the mapping lives at its call site.
fn store_error(error: session_store::SessionStoreError) -> Errors {
    Errors::SysInternalError(error.to_string())
}

impl SessionService {
    /// Creates a new session and stores it in Redis.
    ///
    /// # Returns
    /// The **raw** 256-bit bearer token to put in the
    /// cookie (returned to the caller and never stored). The stored [`Session`]
    /// has a `session_id` that is the *hash* of that token. Everything in Redis is keyed
    /// by the hash, so a store leak yields no usable session tokens.
    pub async fn create_session(
        state: &AuthState,
        user: &UserModel,
        persistent: bool,
    ) -> ServiceResult<IssuedSession> {
        let raw_token = generate_secure_token();
        let generation = session_store::current_generation(&state.redis, &user.id.to_string())
            .await
            .map_err(store_error)?;
        let session = Session::new(
            hash_token(&raw_token),
            user.id.to_string(),
            generation,
            user.security_generation,
            state.policy,
        )
        .map_err(|error| Errors::SysInternalError(error.to_string()))?
        .with_persistence(persistent);
        if session_store::issue(
            &state.redis,
            &session,
            state.policy,
            (session.expires_at - Utc::now()).num_seconds() as u64,
        )
        .await
        .map_err(store_error)?
            != IssueOutcome::Issued
        {
            return Err(Errors::AuthProofStale);
        }
        Ok(IssuedSession {
            token: raw_token,
            expires_at: session.expires_at,
            persistent,
        })
    }

    /// Resolve an opaque session and check the current authoritative account state.
    pub async fn authenticate(
        state: &AuthState,
        raw_token: &str,
    ) -> ServiceResult<(UserModel, Session)> {
        let session = session_store::get(&state.redis, &hash_token(raw_token))
            .await
            .map_err(store_error)?
            .ok_or(Errors::UserUnauthorized)?;
        if session.expires_at <= Utc::now() || session.max_expires_at <= Utc::now() {
            return Err(Errors::SessionExpired);
        }
        let user_id =
            Uuid::parse_str(&session.user_id).map_err(|_| Errors::SessionInvalidUserId)?;
        let user = repository_get_user_by_id(&state.db, user_id).await?;
        if user.disabled || user.security_generation != session.security_generation {
            return Err(Errors::UserUnauthorized);
        }
        Ok((user, session))
    }

    /// Extend a live session without changing its bearer token or absolute lifetime.
    /// Account locking serializes renewal with durable account-wide revocation.
    pub async fn refresh_session(
        state: &AuthState,
        raw_token: &str,
    ) -> ServiceResult<IssuedSession> {
        let (discovered, _) = Self::authenticate(state, raw_token).await?;
        let transaction = state.db.begin().await?;
        let user = repository_get_user_by_id_for_update(&transaction, discovered.id).await?;
        let old = session_store::get(&state.redis, &hash_token(raw_token))
            .await
            .map_err(store_error)?
            .ok_or(Errors::UserUnauthorized)?;
        if old.security_generation != user.security_generation {
            return Err(Errors::AuthProofStale);
        }
        let mut refreshed = old.clone();
        refreshed.expires_at = old
            .refreshed_expiry(state.policy)
            .ok_or(Errors::SessionExpired)?;
        if !session_store::refresh(&state.redis, &old, &refreshed, state.policy)
            .await
            .map_err(store_error)?
        {
            return Err(Errors::UserUnauthorized);
        }
        transaction.commit().await?;
        Ok(IssuedSession {
            token: raw_token.to_owned(),
            expires_at: refreshed.expires_at,
            persistent: refreshed.persistent,
        })
    }

    pub async fn revoke_user_session(
        state: &AuthState,
        user_id: Uuid,
        management_id: &str,
    ) -> ServiceResult<()> {
        let transaction = state.db.begin().await?;
        repository_get_user_by_id_for_update(&transaction, user_id).await?;
        let session = session_store::get_managed(&state.redis, &user_id.to_string(), management_id)
            .await
            .map_err(store_error)?
            .ok_or(Errors::SessionNotFound)?;
        session_store::delete(&state.redis, &session.session_id)
            .await
            .map_err(store_error)?;
        transaction.commit().await?;
        Ok(())
    }

    pub async fn revoke_all_user_sessions(state: &AuthState, user_id: Uuid) -> ServiceResult<()> {
        let transaction = state.db.begin().await?;
        let user = repository_get_user_by_id_for_update(&transaction, user_id).await?;
        repository_advance_security_generation(&transaction, user).await?;
        // Hold the account lock until cleanup completes so a new login cannot be revoked.
        session_store::revoke_all(&state.redis, &user_id.to_string())
            .await
            .map_err(store_error)?;
        transaction.commit().await?;
        Ok(())
    }

    pub async fn logout(state: &AuthState, raw_token: &str) -> ServiceResult<()> {
        session_store::delete(&state.redis, &hash_token(raw_token))
            .await
            .map_err(store_error)?;
        Ok(())
    }
}
