pub mod health;
pub mod openapi;

use crate::state::AppState;
use axum::{Router, routing::get};
use tower_http::trace::TraceLayer;
use utoipa_swagger_ui::SwaggerUi;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health/live", get(health::live))
        .merge(SwaggerUi::new("/swagger-ui").url("/swagger.json", openapi::ApiDoc::merged()))
        .fallback(errors::not_found)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
