use crate::api::v0::routes::auth::{auth, cookies, require_anonymous};
use crate::service::oauth;
use crate::state::AppState;
use axum::Json;
use axum::extract::State;
use dto::auth::{AuthorizeRequest, AuthorizeResponse};
use dto::validator::json_validator::ValidatedJson;
use errors::{ErrorResponse, errors::ServiceResult};
use tower_cookies::Cookies;

#[utoipa::path(
    post,
    path = "/v0/auth/oauth/google/authorize",
    summary = "Start Google OAuth",
    description = "Browser API using HttpOnly session cookies. Mutating requests require the configured Origin.",
    request_body = AuthorizeRequest,
    responses(
        (status = 200, description = "Success", body = AuthorizeResponse),
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
pub async fn auth_google_authorize(
    State(state): State<AppState>,
    cookies: Cookies,
    ValidatedJson(request): ValidatedJson<AuthorizeRequest>,
) -> ServiceResult<Json<AuthorizeResponse>> {
    let auth = auth(&state)?;
    require_anonymous(auth, &cookies).await?;
    let browser = cookies::browser(auth, &cookies);
    let auth_url =
        oauth::google::service_generate_google_oauth_url(auth, &browser, request.remember_me)
            .await?;
    Ok(Json(AuthorizeResponse { auth_url }))
}
