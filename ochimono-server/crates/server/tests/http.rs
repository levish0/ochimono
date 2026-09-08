use serde_json::Value;
use std::time::Duration;

#[tokio::test]
async fn http_contract_and_shutdown() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let (shutdown, received) = tokio::sync::oneshot::channel::<()>();
    let task = tokio::spawn(async move {
        axum::serve(listener, server::api::router(Default::default()))
            .with_graceful_shutdown(async {
                let _ = received.await;
            })
            .await
            .unwrap();
    });
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap();
    let response = client
        .get(format!("{base}/health/live"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    let health: dto::health::HealthResponse = response.json().await.unwrap();
    assert_eq!(health.status, "ok");
    assert_eq!(health.version, env!("CARGO_PKG_VERSION"));
    let schema: Value = client
        .get(format!("{base}/swagger.json"))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap()
        .json()
        .await
        .unwrap();
    let expected = serde_json::to_value(server::api::openapi::ApiDoc::merged()).unwrap();
    assert_eq!(schema, expected);
    assert!(schema["paths"]["/health/live"]["get"].is_object());
    let docs = client
        .get(format!("{base}/swagger-ui/"))
        .send()
        .await
        .unwrap();
    assert_eq!(docs.status(), 200);
    assert!(docs.text().await.unwrap().contains("swagger-ui"));
    let missing = client.get(format!("{base}/v0/rooms")).send().await.unwrap();
    assert_eq!(missing.status(), 404);
    assert_eq!(
        missing.json::<Value>().await.unwrap()["code"],
        "http:not_found"
    );
    shutdown.send(()).unwrap();
    tokio::time::timeout(Duration::from_secs(5), task)
        .await
        .unwrap()
        .unwrap();
}
