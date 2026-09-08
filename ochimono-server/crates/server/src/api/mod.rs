pub mod health;
pub mod openapi;
pub mod v0;

use crate::state::AppState;
use axum::{Router, routing::get};
use tower_http::trace::TraceLayer;
use utoipa_swagger_ui::SwaggerUi;

pub fn router(state: AppState) -> Router {
    let mut router = Router::new()
        .route("/health/live", get(health::live))
        .nest("/v0", v0::routes::auth::routes::auth_routes(state.clone()))
        .merge(SwaggerUi::new("/swagger-ui").url("/swagger.json", openapi::ApiDoc::merged()))
        .fallback(errors::handler_404)
        .layer(TraceLayer::new_for_http())
        .with_state(state.clone());
    if let Some(auth) = state.auth {
        use axum::http::{HeaderValue, Method, header};
        use tower_http::cors::CorsLayer;
        router = router.layer(
            CorsLayer::new()
                .allow_origin(
                    auth.config
                        .browser_origin
                        .parse::<HeaderValue>()
                        .expect("validated origin"),
                )
                .allow_credentials(true)
                .allow_methods([Method::GET, Method::POST, Method::DELETE])
                .allow_headers([header::CONTENT_TYPE]),
        );
    }
    router
}
