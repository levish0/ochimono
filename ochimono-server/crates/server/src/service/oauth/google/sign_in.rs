use super::super::{
    google, provider,
    resolve_identity::service_resolve_google_identity,
    state::{OAuthStateData, SignIn, consume, key},
};
use crate::service::auth::AuthState;
use errors::errors::{Errors, ServiceResult};

pub async fn service_google_sign_in(
    state: &AuthState,
    browser: &str,
    code: &str,
    token: &str,
) -> ServiceResult<SignIn> {
    let proof: OAuthStateData = consume(state, key("state", token, browser)).await?;
    let access_token = provider::client::exchange_code::<google::GoogleProvider>(
        &state.http,
        &state.config.google,
        code,
        &proof.pkce_verifier,
    )
    .await?;
    let info = google::client::fetch_google_user_info(&state.http, &access_token).await?;
    if !info.verified_email || info.id.is_empty() || info.id.len() > 255 || info.email.len() > 320 {
        return Err(Errors::BadRequestError("Invalid or expired request".into()));
    }
    service_resolve_google_identity(state, browser, &info.id, &info.email, proof.remember_me).await
}
