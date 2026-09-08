use crate::api::v0::routes::auth::{auth, cookies};
use crate::service::auth::session::SessionService;
use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use errors::{ErrorResponse, errors::ServiceResult};
use tower_cookies::Cookies;

#[utoipa::path(
    delete,
    path = "/v0/auth/sessions",
    summary = "Revoke all sessions",
    description = "Browser API using HttpOnly session cookies. Mutating requests require the configured Origin.",
    responses(
        (status = 204, description = "Success"),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 401, description = "Authentication required", body = ErrorResponse),
        (status = 403, description = "Request denied", body = ErrorResponse),
        (status = 404, description = "Account or session not found", body = ErrorResponse),
        (status = 409, description = "Account conflict or stale proof", body = ErrorResponse),
        (status = 429, description = "Rate limit exceeded", body = ErrorResponse),
        (status = 500, description = "Internal error", body = ErrorResponse),
        (status = 503, description = "Authentication unavailable", body = ErrorResponse),
    ),
    tag = "Auth"
)]
pub async fn auth_revoke_all_sessions(
    State(state): State<AppState>,
    cookies: Cookies,
) -> ServiceResult<StatusCode> {
    let auth = auth(&state)?;
    let (user, _) =
        SessionService::authenticate(auth, &cookies::read(auth, &cookies, "session")?).await?;
    SessionService::revoke_all_user_sessions(auth, user.id).await?;
    cookies::clear(auth, &cookies);
    Ok(StatusCode::NO_CONTENT)
}
