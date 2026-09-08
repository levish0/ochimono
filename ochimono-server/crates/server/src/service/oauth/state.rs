use crate::service::auth::{AuthState, session::IssuedSession};
use auth_core::token::hash_token;
use dto::auth::LoginResponse;
use errors::errors::{Errors, ServiceResult};
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub(super) struct OAuthStateData {
    pub(super) pkce_verifier: String,
    pub(super) remember_me: bool,
}
#[derive(Serialize, Deserialize)]
pub(super) struct PendingSignupData {
    pub(super) subject: String,
    pub(super) email: String,
    pub(super) remember_me: bool,
}

pub type SignIn = (LoginResponse, Option<IssuedSession>);

pub(super) fn key(kind: &str, token: &str, browser: &str) -> String {
    format!("oauth:{kind}:{}:{}", hash_token(token), hash_token(browser))
}

pub(super) async fn store<T: Serialize>(
    state: &AuthState,
    key: String,
    value: &T,
) -> ServiceResult<()> {
    let _: () = state
        .redis
        .clone()
        .set_ex(key, serde_json::to_string(value)?, 600)
        .await?;
    Ok(())
}
pub(super) async fn consume<T: serde::de::DeserializeOwned>(
    state: &AuthState,
    key: String,
) -> ServiceResult<T> {
    // Single-use consumption and browser binding are both enforced by the lookup key.
    let data: Option<String> = state.redis.clone().get_del(key).await?;
    Ok(serde_json::from_str(&data.ok_or(
        Errors::BadRequestError("Invalid or expired request".into()),
    )?)?)
}
