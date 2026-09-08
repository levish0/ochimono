use crate::state::AppState;
use axum::{
    extract::{ConnectInfo, Request, State},
    http::{HeaderValue, Method, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use errors::errors::Errors;
use std::net::SocketAddr;

async fn guard_request(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let Some(auth) = state.auth.as_ref() else {
        return Errors::AuthUnavailable.into_response();
    };
    if request.method() != Method::GET
        && request
            .headers()
            .get(header::ORIGIN)
            .and_then(|v| v.to_str().ok())
            != Some(auth.config.browser_origin.as_str())
    {
        return Errors::ForbiddenError("Request is not allowed".into()).into_response();
    }
    // Trust the transport peer, never an unverified forwarding header.
    let Some(peer) = request.extensions().get::<ConnectInfo<SocketAddr>>() else {
        return Errors::AuthUnavailable.into_response();
    };
    let key = format!(
        "auth:rate:{}:{}",
        auth_core::token::hash_token(&peer.0.ip().to_string()),
        chrono::Utc::now().timestamp() / 60
    );
    let count: Result<i64,_> = redis::Script::new("local n=redis.call('INCR',KEYS[1]); if n==1 then redis.call('EXPIRE',KEYS[1],120) end; return n")
        .key(key).invoke_async(&mut auth.redis.clone()).await;
    match count {
        Ok(count) if count <= 120 => {}
        Ok(_) => return Errors::RateLimitExceeded.into_response(),
        Err(_) => return Errors::AuthUnavailable.into_response(),
    }
    next.run(request).await
}

pub async fn guard(state: State<AppState>, request: Request, next: Next) -> Response {
    let mut response = guard_request(state, request, next).await;
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}
