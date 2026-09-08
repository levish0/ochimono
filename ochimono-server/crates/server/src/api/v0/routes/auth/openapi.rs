use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::v0::routes::auth::oauth::google::google_authorize::auth_google_authorize,
        crate::api::v0::routes::auth::oauth::google::google_login::auth_google_login,
        crate::api::v0::routes::auth::session::complete_signup::auth_complete_signup,
        crate::api::v0::routes::auth::session::refresh::auth_refresh,
        crate::api::v0::routes::auth::session::logout::auth_logout,
        crate::api::v0::routes::auth::session::me::auth_me,
        crate::api::v0::routes::auth::session::list_sessions::auth_list_sessions,
        crate::api::v0::routes::auth::session::revoke_session::auth_revoke_session,
        crate::api::v0::routes::auth::session::revoke_all_sessions::auth_revoke_all_sessions
    ),
    components(schemas(errors::ErrorResponse))
)]
pub struct AuthApiDoc;
