use super::boundary;
use super::oauth::google::google_authorize::auth_google_authorize;
use super::oauth::google::google_login::auth_google_login;
use super::session::complete_signup::auth_complete_signup;
use super::session::list_sessions::auth_list_sessions;
use super::session::logout::auth_logout;
use super::session::me::auth_me;
use super::session::refresh::auth_refresh;
use super::session::revoke_all_sessions::auth_revoke_all_sessions;
use super::session::revoke_session::auth_revoke_session;
use crate::state::AppState;
use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware,
    routing::{delete, get, post},
};
use tower_cookies::CookieManagerLayer;

pub fn auth_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/auth/oauth/google/authorize", post(auth_google_authorize))
        .route("/auth/oauth/google/login", post(auth_google_login))
        .route("/auth/complete-signup", post(auth_complete_signup))
        .route("/auth/refresh", post(auth_refresh))
        .route("/auth/logout", post(auth_logout))
        .route("/auth/me", get(auth_me))
        .route(
            "/auth/sessions",
            get(auth_list_sessions).delete(auth_revoke_all_sessions),
        )
        .route("/auth/sessions/{id}", delete(auth_revoke_session))
        .layer(DefaultBodyLimit::max(16 * 1024))
        .layer(middleware::from_fn_with_state(state, boundary::guard))
        .layer(CookieManagerLayer::new())
}
