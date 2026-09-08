use super::super::{
    google, provider,
    state::{OAuthStateData, key, store},
};
use crate::service::auth::AuthState;
use auth_core::token::generate_secure_token;
use errors::errors::ServiceResult;

pub async fn service_generate_google_oauth_url(
    state: &AuthState,
    browser: &str,
    remember_me: bool,
) -> ServiceResult<String> {
    // Generate state — 256-bit CSPRNG token (not a time-ordered UUID, which would leak
    // issuance time and carry less entropy) matching the codebase's token standard.
    let token = generate_secure_token();
    let (url, _, pkce_verifier) = provider::client::generate_auth_url::<google::GoogleProvider>(
        &state.config.google,
        token.clone(),
    )?;
    // Key by the hashed state (hash-at-rest, like every other short-lived credential).
    store(
        state,
        key("state", &token, browser),
        &OAuthStateData {
            pkce_verifier,
            remember_me,
        },
    )
    .await?;
    Ok(url)
}
