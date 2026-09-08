use crate::api::v0::routes::auth::{auth, cookies};
use crate::service::auth::session::SessionService;
use crate::state::AppState;
use axum::Json;
use axum::extract::State;
use dto::auth::UserResponse;
use errors::{ErrorResponse, errors::ServiceResult};
use tower_cookies::Cookies;

#[utoipa::path(
    get,
    path = "/v0/auth/me",
    summary = "Read the authenticated account",
    description = "Browser API using HttpOnly session cookies. Mutating requests require the configured Origin.",
    responses(
        (status = 200, description = "Success", body = UserResponse),
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
pub async fn auth_me(
    State(state): State<AppState>,
    cookies: Cookies,
) -> ServiceResult<Json<UserResponse>> {
    let auth = auth(&state)?;
    let (user, _) =
        SessionService::authenticate(auth, &cookies::read(auth, &cookies, "session")?).await?;
    Ok(Json(UserResponse {
        id: user.id,
        handle: user.handle,
    }))
}
