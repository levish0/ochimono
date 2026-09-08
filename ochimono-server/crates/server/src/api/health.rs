use crate::state::AppState;
use axum::{Json, extract::State};
use dto::health::HealthResponse;

#[utoipa::path(
    get,
    path = "/health/live",
    tag = "Health",
    summary = "Check HTTP process liveness",
    description = "Reports process liveness only. Does not verify PostgreSQL, Redis, authentication or game availability.",
    responses((status = 200, description = "HTTP process is alive", body = HealthResponse))
)]
pub async fn live(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".into(),
        version: state.version.into(),
    })
}
