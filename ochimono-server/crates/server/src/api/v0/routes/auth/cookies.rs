use crate::service::auth::{AuthState, session::IssuedSession};
use errors::errors::{Errors, ServiceResult};
use tower_cookies::{
    Cookie, Cookies,
    cookie::{SameSite, time::Duration},
};

fn name(state: &AuthState, kind: &str) -> String {
    let prefix = if state.config.secure_cookies {
        "__Host-"
    } else {
        ""
    };
    format!("{prefix}ochimono_{kind}")
}
pub fn read(state: &AuthState, cookies: &Cookies, kind: &str) -> ServiceResult<String> {
    let value = cookies
        .get(&name(state, kind))
        .ok_or(Errors::UserUnauthorized)?
        .value()
        .to_owned();
    if value.is_empty() || value.len() > 4096 {
        return Err(Errors::UserUnauthorized);
    }
    Ok(value)
}
fn set(state: &AuthState, cookies: &Cookies, kind: &str, value: String, max_age: Option<i64>) {
    let mut cookie = Cookie::build((name(state, kind), value))
        .path("/")
        .http_only(true)
        .secure(state.config.secure_cookies)
        .same_site(SameSite::Lax);
    if let Some(seconds) = max_age {
        cookie = cookie.max_age(Duration::seconds(seconds));
    }
    cookies.add(cookie.build());
}
pub fn browser(state: &AuthState, cookies: &Cookies) -> String {
    if let Ok(value) = read(state, cookies, "browser")
        && value.len() == 43
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-')
    {
        return value;
    }
    let value = auth_core::token::generate_secure_token();
    set(state, cookies, "browser", value.clone(), Some(3600));
    value
}
pub fn issue(state: &AuthState, cookies: &Cookies, session: IssuedSession) {
    set(
        state,
        cookies,
        "session",
        session.token,
        if session.persistent {
            Some(
                (session.expires_at - chrono::Utc::now())
                    .num_seconds()
                    .max(0),
            )
        } else {
            None
        },
    );
}
pub fn clear(state: &AuthState, cookies: &Cookies) {
    set(state, cookies, "session", String::new(), Some(0));
}
