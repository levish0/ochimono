mod boundary;
mod cookies;
pub mod oauth;
pub mod openapi;
pub mod routes;
pub mod session;

use crate::{service::auth::AuthState, state::AppState};
use errors::errors::{Errors, ServiceResult};

fn auth(state: &AppState) -> ServiceResult<&AuthState> {
    state.auth.as_deref().ok_or(Errors::AuthUnavailable)
}

/// Ordinary sign-in must not silently replace an authenticated browser identity.
async fn require_anonymous(
    state: &AuthState,
    cookies: &tower_cookies::Cookies,
) -> ServiceResult<()> {
    let token = match cookies::read(state, cookies, "session") {
        Ok(token) => token,
        Err(Errors::UserUnauthorized) => return Ok(()),
        Err(error) => return Err(error),
    };
    match crate::service::auth::session::SessionService::authenticate(state, &token).await {
        Ok(_) => Err(Errors::AuthAlreadyAuthenticated),
        Err(Errors::UserUnauthorized | Errors::SessionExpired | Errors::UserNotFound) => Ok(()),
        Err(error) => Err(error),
    }
}
