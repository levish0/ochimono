use crate::api::v0::routes::auth::{auth, cookies};
use crate::service::auth::session::SessionService;
use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use errors::{ErrorResponse, errors::ServiceResult};
use tower_cookies::Cookies;

#[utoipa::path(
    post,
    path = "/v0/auth/refresh",
    summary = "Extend the current session",
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
pub async fn auth_refresh(
    State(state): State<AppState>,
    cookies: Cookies,
) -> ServiceResult<StatusCode> {
    let auth = auth(&state)?;
    let token = cookies::read(auth, &cookies, "session")?;
    let session = SessionService::refresh_session(auth, &token).await?;
    cookies::issue(auth, &cookies, session);
    Ok(StatusCode::NO_CONTENT)
}
