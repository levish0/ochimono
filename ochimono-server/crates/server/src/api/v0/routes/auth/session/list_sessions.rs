use crate::api::v0::routes::auth::{auth, cookies};
use crate::service::auth::session::SessionService;
use crate::state::AppState;
use axum::Json;
use axum::extract::State;
use dto::auth::SessionResponse;
use errors::errors::Errors;
use errors::{ErrorResponse, errors::ServiceResult};
use tower_cookies::Cookies;
use uuid::Uuid;

#[utoipa::path(
    get,
    path = "/v0/auth/sessions",
    summary = "List active sessions",
    description = "Browser API using HttpOnly session cookies. Mutating requests require the configured Origin.",
    responses(
        (status = 200, description = "Success", body = [SessionResponse]),
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
pub async fn auth_list_sessions(
    State(state): State<AppState>,
    cookies: Cookies,
) -> ServiceResult<Json<Vec<SessionResponse>>> {
    let auth = auth(&state)?;
    let (user, current) =
        SessionService::authenticate(auth, &cookies::read(auth, &cookies, "session")?).await?;
    let rows = auth_core::session_store::list_user_sessions(&auth.redis, &user.id.to_string())
        .await
        .map_err(|error| Errors::SysInternalError(error.to_string()))?;
    Ok(Json(
        rows.into_iter()
            .filter(|row| {
                row.security_generation == user.security_generation
                    && row.expires_at > chrono::Utc::now()
            })
            .map(|row| {
                Ok(SessionResponse {
                    id: Uuid::parse_str(&row.management_id).map_err(|_| {
                        Errors::SysInternalError("Invalid session management ID".into())
                    })?,
                    current: row.management_id == current.management_id,
                    created_at: row.created_at.with_timezone(&chrono::Utc),
                    expires_at: row.expires_at.with_timezone(&chrono::Utc),
                })
            })
            .collect::<ServiceResult<Vec<_>>>()?,
    ))
}
