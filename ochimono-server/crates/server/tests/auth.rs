//! Requires a migrated, disposable PostgreSQL database and Redis.
use config::auth::{AuthConfig, OAuthProviderCredentials};
use dto::auth::LoginResponse;
use reqwest::{Client, Response, header};
use serde_json::{Value, json};
use server::{
    service::{
        auth::{AuthState, session::SessionService},
        oauth,
    },
    state::AppState,
};
use std::{net::SocketAddr, time::Duration};
use uuid::Uuid;

fn session_cookie(response: &Response) -> String {
    let cookie = response
        .headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .map(|value| value.to_str().unwrap())
        .find(|value| value.starts_with("ochimono_session="))
        .expect("session cookie");
    assert!(cookie.contains("HttpOnly"));
    assert!(cookie.contains("SameSite=Lax"));
    assert!(cookie.contains("Path=/"));
    cookie.split(';').next().unwrap().to_owned()
}

#[tokio::test]
#[ignore = "requires disposable PostgreSQL and Redis"]
async fn browser_auth_and_session_lifecycle() {
    let origin = "http://localhost:5173";
    let auth = AuthState::connect(AuthConfig {
        database_url: std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL"),
        redis_url: std::env::var("SESSION_REDIS_URL").expect("SESSION_REDIS_URL"),
        browser_origin: origin.into(),
        secure_cookies: false,
        google: OAuthProviderCredentials {
            client_id: "test-client".into(),
            client_secret: "test-secret".into(),
            redirect_uri: format!("{origin}/auth/callback/google"),
        },
    })
    .await
    .unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let app = server::api::router(AppState {
        auth: Some(auth.clone()),
        ..Default::default()
    });
    let (shutdown, received) = tokio::sync::oneshot::channel();
    let task = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(async {
            let _ = received.await;
        })
        .await
        .unwrap();
    });
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap();
    let denied = client
        .post(format!("{base}/v0/auth/oauth/google/authorize"))
        .json(&json!({}))
        .send()
        .await
        .unwrap();
    assert_eq!(denied.status(), 403);
    let malformed = client
        .post(format!("{base}/v0/auth/oauth/google/authorize"))
        .header(header::ORIGIN, origin)
        .json(&json!({"unknown":true}))
        .send()
        .await
        .unwrap();
    assert_eq!(malformed.status(), 400);
    assert!(malformed.json::<Value>().await.unwrap()["code"].is_string());
    let authorize = client
        .post(format!("{base}/v0/auth/oauth/google/authorize"))
        .header(header::ORIGIN, origin)
        .json(&json!({"remember_me":true}))
        .send()
        .await
        .unwrap();
    assert_eq!(authorize.status(), 200);
    assert_eq!(authorize.headers()[header::CACHE_CONTROL], "no-store");
    let url = url::Url::parse(
        authorize.json::<Value>().await.unwrap()["auth_url"]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    let params: std::collections::HashMap<_, _> = url.query_pairs().into_owned().collect();
    assert_eq!(params["code_challenge_method"], "S256");
    assert_eq!(params["state"].len(), 43);
    assert!(
        oauth::google::service_google_sign_in(
            &auth,
            "another-browser",
            "unused-code",
            &params["state"]
        )
        .await
        .is_err()
    );

    // Enter at the verified identity boundary; no Google credentials or network are used.
    let subject = Uuid::now_v7().to_string();
    let browser = auth_core::token::generate_secure_token();
    let (pending, _) = oauth::resolve_identity::service_resolve_google_identity(
        &auth,
        &browser,
        &subject,
        "test@example.invalid",
        true,
    )
    .await
    .unwrap();
    let LoginResponse::PendingSignup { pending_token } = pending else {
        panic!("new identity")
    };
    assert!(
        oauth::complete_signup::service_complete_signup(
            &auth,
            "another-browser",
            &pending_token,
            "unused"
        )
        .await
        .is_err()
    );
    let handle = format!("u{}", &Uuid::now_v7().simple().to_string()[12..]);
    let response = client
        .post(format!("{base}/v0/auth/complete-signup"))
        .header(header::ORIGIN, origin)
        .header(header::COOKIE, format!("ochimono_browser={browser}"))
        .json(&json!({"pending_token":pending_token,"handle":handle}))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let cookie = session_cookie(&response);
    let replacement = client
        .post(format!("{base}/v0/auth/oauth/google/authorize"))
        .header(header::ORIGIN, origin)
        .header(header::COOKIE, &cookie)
        .json(&json!({}))
        .send()
        .await
        .unwrap();
    assert_eq!(replacement.status(), 409);
    assert!(
        response
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .any(|value| value.to_str().unwrap().contains("Max-Age="))
    );
    let user = response.json::<Value>().await.unwrap();
    let user_id = Uuid::parse_str(user["user"]["id"].as_str().unwrap()).unwrap();
    assert!(
        oauth::complete_signup::service_complete_signup(&auth, &browser, &pending_token, &handle)
            .await
            .is_err()
    );
    let me = client
        .get(format!("{base}/v0/auth/me"))
        .header(header::COOKIE, &cookie)
        .send()
        .await
        .unwrap();
    assert_eq!(me.status(), 200);
    assert_eq!(me.json::<Value>().await.unwrap()["handle"], handle);
    let sessions = client
        .get(format!("{base}/v0/auth/sessions"))
        .header(header::COOKIE, &cookie)
        .send()
        .await
        .unwrap();
    assert_eq!(sessions.status(), 200);
    let sessions = sessions.json::<Value>().await.unwrap();
    assert_eq!(sessions.as_array().unwrap().len(), 1);
    assert_eq!(sessions[0]["current"], true);
    let refreshed = client
        .post(format!("{base}/v0/auth/refresh"))
        .header(header::ORIGIN, origin)
        .header(header::COOKIE, &cookie)
        .send()
        .await
        .unwrap();
    assert_eq!(refreshed.status(), 204);
    assert_eq!(session_cookie(&refreshed), cookie);
    let (_, second) = oauth::resolve_identity::service_resolve_google_identity(
        &auth,
        &browser,
        &subject,
        "changed@example.invalid",
        false,
    )
    .await
    .unwrap();
    let second = second.unwrap();
    let transient = client
        .post(format!("{base}/v0/auth/refresh"))
        .header(header::ORIGIN, origin)
        .header(header::COOKIE, format!("ochimono_session={}", second.token))
        .send()
        .await
        .unwrap();
    assert_eq!(transient.status(), 204);
    session_cookie(&transient);
    assert!(
        transient
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .all(|value| !value.to_str().unwrap().contains("Max-Age="))
    );
    let (_, second_session) = SessionService::authenticate(&auth, &second.token)
        .await
        .unwrap();
    // Ownership is checked even if another account's management ID is known.
    let (other, _) = oauth::resolve_identity::service_resolve_google_identity(
        &auth,
        &browser,
        &Uuid::now_v7().to_string(),
        "test@example.invalid",
        false,
    )
    .await
    .unwrap();
    let LoginResponse::PendingSignup { pending_token } = other else {
        panic!("emails must not link identities")
    };
    let (_, other) = oauth::complete_signup::service_complete_signup(
        &auth,
        &browser,
        &pending_token,
        &format!("v{}", &handle[1..]),
    )
    .await
    .unwrap();
    let other = other.unwrap();
    let (other_user, _) = SessionService::authenticate(&auth, &other.token)
        .await
        .unwrap();
    assert!(
        SessionService::revoke_user_session(&auth, other_user.id, &second_session.management_id)
            .await
            .is_err()
    );
    SessionService::revoke_user_session(&auth, user_id, &second_session.management_id)
        .await
        .unwrap();
    assert!(
        SessionService::authenticate(&auth, &second.token)
            .await
            .is_err()
    );
    assert!(
        SessionService::refresh_session(&auth, &second.token)
            .await
            .is_err()
    );
    let revoked = client
        .delete(format!("{base}/v0/auth/sessions"))
        .header(header::ORIGIN, origin)
        .header(header::COOKIE, &cookie)
        .send()
        .await
        .unwrap();
    assert_eq!(revoked.status(), 204);
    let stale = client
        .get(format!("{base}/v0/auth/me"))
        .header(header::COOKIE, &cookie)
        .send()
        .await
        .unwrap();
    assert_eq!(stale.status(), 401);
    let (_, new_session) = oauth::resolve_identity::service_resolve_google_identity(
        &auth,
        &browser,
        &subject,
        "test@example.invalid",
        false,
    )
    .await
    .unwrap();
    let new_session = new_session.unwrap();
    SessionService::authenticate(&auth, &new_session.token)
        .await
        .unwrap();
    SessionService::logout(&auth, &new_session.token)
        .await
        .unwrap();
    assert!(
        SessionService::refresh_session(&auth, &new_session.token)
            .await
            .is_err()
    );
    SessionService::revoke_all_user_sessions(&auth, other_user.id)
        .await
        .unwrap();
    use sea_orm::EntityTrait;
    for id in [user_id, other_user.id] {
        entity::users::Entity::delete_by_id(id)
            .exec(&auth.db)
            .await
            .unwrap();
    }
    shutdown.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(5), task)
        .await
        .unwrap()
        .unwrap();
}
