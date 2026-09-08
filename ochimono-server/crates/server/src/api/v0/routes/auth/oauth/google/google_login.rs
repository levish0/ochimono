use crate::api::v0::routes::auth::{auth, cookies, require_anonymous};
use crate::service::oauth;
use crate::state::AppState;
use axum::Json;
use axum::extract::State;
use dto::auth::{GoogleLoginRequest, LoginResponse};
use dto::validator::json_validator::ValidatedJson;
use errors::errors::Errors;
use errors::{ErrorResponse, errors::ServiceResult};
use tower_cookies::Cookies;

#[utoipa::path(
    post,
    path = "/v0/auth/oauth/google/login",
    summary = "Exchange Google code and browser-bound state",
    description = "Browser API using HttpOnly session cookies. Mutating requests require the configured Origin.",
    request_body = GoogleLoginRequest,
    responses(
        (status = 200, description = "Success", body = LoginResponse),
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
pub async fn auth_google_login(
    State(state): State<AppState>,
    cookies: Cookies,
    ValidatedJson(request): ValidatedJson<GoogleLoginRequest>,
) -> ServiceResult<Json<LoginResponse>> {
    let auth = auth(&state)?;
    require_anonymous(auth, &cookies).await?;
    if request.code.is_empty() || request.code.len() > 4096 || request.state.len() != 43 {
        return Err(Errors::BadRequestError("Invalid or expired request".into()));
    }
    let browser = cookies::read(auth, &cookies, "browser")?;
    let (response, session) =
        oauth::google::service_google_sign_in(auth, &browser, &request.code, &request.state)
            .await?;
    if let Some(session) = session {
        cookies::issue(auth, &cookies, session);
    }
    Ok(Json(response))
}
