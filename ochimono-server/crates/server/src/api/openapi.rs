use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(title = "ochimono API", license(name = "AGPL-3.0-only")),
    paths(crate::api::health::live),
    components(schemas(dto::health::HealthResponse, errors::ErrorResponse))
)]
pub struct ApiDoc;

impl ApiDoc {
    pub fn merged() -> utoipa::openapi::OpenApi {
        Self::openapi()
    }
}
