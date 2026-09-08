use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use sea_orm::{DbErr, TransactionError};
use serde::Serialize;
use tracing::error;
use utoipa::ToSchema;

pub type ServiceResult<T> = Result<T, Errors>;
type ErrorMapping = (StatusCode, &'static str, Option<String>);

macro_rules! domain_error_handlers {
    ($($handler:ident),+ $(,)?) => {
        fn log_domain_error(error: &Errors) {
            $(
                crate::handlers::$handler::log_error(error);
            )+
        }

        fn map_domain_response(error: &Errors) -> Option<ErrorMapping> {
            [
                $(
                    crate::handlers::$handler::map_response as fn(&Errors) -> Option<ErrorMapping>,
                )+
            ]
            .into_iter()
            .find_map(|map_response| map_response(error))
        }
    };
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub status: u16,
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

impl IntoResponse for ErrorResponse {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}

impl From<DbErr> for Errors {
    fn from(err: DbErr) -> Self {
        Errors::DatabaseError(err.to_string())
    }
}

impl From<TransactionError<DbErr>> for Errors {
    fn from(err: TransactionError<DbErr>) -> Self {
        Errors::TransactionError(err.to_string())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Errors {
    InvalidCredentials,
    /// Ordinary sign-in must not silently replace an already-authenticated
    /// browser identity. Explicit account switching is a separate flow.
    AuthAlreadyAuthenticated,
    /// A valid login/signup proof was captured under an older session generation and may no
    /// longer mint a session. The client must restart the authentication flow.
    AuthProofStale,
    UserInvalidPassword,
    UserPasswordNotSet,
    /// A sensitive action (e.g. account deletion) requires a re-authentication factor
    /// that the request did not supply.
    ReauthenticationRequired,
    UserInvalidSession,
    UserNotVerified,
    UserNotFound,
    UserUnauthorized,
    UserBanned,
    UserPermissionInsufficient,
    UserHandleAlreadyExists,
    UserEmailAlreadyExists,
    UserNotBanned,
    UserAlreadyBanned,
    UserDoesNotHaveRole,
    UserAlreadyHasRole,
    CannotManageSelf,
    CannotManageHigherOrEqualRole,
    UserTokenExpired,
    UserNoRefreshToken,
    UserInvalidToken,
    SessionInvalidUserId,
    SessionExpired,
    SessionNotFound,
    ForbiddenError(String),
    OauthInvalidAuthUrl,
    OauthInvalidTokenUrl,
    OauthInvalidRedirectUrl,
    OauthTokenExchangeFailed,
    OauthUserInfoFetchFailed,
    OauthUserInfoParseFailed(String),
    OauthAccountAlreadyLinked,
    OauthConnectionNotFound,
    OauthCannotUnlinkLastConnection,
    OauthInvalidImageUrl,
    OauthInvalidState,
    OauthStateExpired,
    OauthHandleRequired,
    OauthEmailAlreadyExists,
    OauthEmailNotVerified,
    GoogleInvalidIdToken,
    GoogleOneTapNonceInvalid,
    GoogleJwksFetchFailed,
    GoogleJwksParseFailed,
    /// Account creation refused for the client's IP by the projected
    /// `auth:signup` ACL policy.
    SignupRestricted,
    BadRequestError(String),
    ValidationError(String),
    SysInternalError(String),
    DatabaseError(String),
    TransactionError(String),
    NotFound(String),
    RateLimitExceeded,
    AuthUnavailable,
}

domain_error_handlers!(
    user_handler,
    oauth_handler,
    session_handler,
    rate_limit_handler,
    system_handler,
    general_handler
);

fn response_body(error: &Errors) -> (StatusCode, ErrorResponse) {
    log_domain_error(error);

    let (status, code, details) = map_domain_response(error).unwrap_or_else(|| {
        error!(error = ?error, "Unhandled error");
        (StatusCode::INTERNAL_SERVER_ERROR, "UNKNOWN_ERROR", None)
    });

    // A 5xx detail is an internal diagnostic, not an API contract. The error
    // itself is logged above; clients receive only the stable code regardless
    // of which environment rendered the response. 4xx details remain useful
    // client-facing validation/action feedback.
    let details = if status.is_server_error() {
        None
    } else {
        details
    };

    (
        status,
        ErrorResponse {
            status: status.as_u16(),
            code: code.to_string(),
            details,
        },
    )
}

impl IntoResponse for Errors {
    fn into_response(self) -> Response {
        let (status, body) = response_body(&self);
        (status, Json(body)).into_response()
    }
}

pub async fn handler_404<B>(req: axum::extract::Request<B>) -> impl IntoResponse {
    let path = req.uri().path();
    let method = req.method().to_string();

    Errors::NotFound(format!("Path {} with method {} not found", path, method))
}

impl From<redis::RedisError> for Errors {
    fn from(_: redis::RedisError) -> Self {
        Self::AuthUnavailable
    }
}
impl From<serde_json::Error> for Errors {
    fn from(_: serde_json::Error) -> Self {
        Self::SysInternalError("Invalid stored payload".into())
    }
}

#[cfg(test)]
mod tests {
    use super::{Errors, response_body};
    use axum::http::StatusCode;

    #[test]
    fn server_error_details_are_never_exposed() {
        let (status, body) = response_body(&Errors::SysInternalError(
            "database connection failed at private-host".into(),
        ));

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(body.code, "system:internal_error");
        assert_eq!(body.details, None);
    }

    #[test]
    fn client_error_details_remain_exposed() {
        let (status, body) = response_body(&Errors::BadRequestError("invalid handle".into()));

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body.code, "general:bad_request");
        assert_eq!(body.details.as_deref(), Some("invalid handle"));
    }

    #[test]
    fn already_authenticated_is_a_stable_conflict() {
        let (status, body) = response_body(&Errors::AuthAlreadyAuthenticated);

        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body.code, "auth:already_authenticated");
        assert_eq!(body.details, None);
    }
}
