use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(title = "ochimono API", license(name = "AGPL-3.0-only")),
    paths(crate::api::health::live,),
    components(schemas(dto::health::HealthResponse, errors::ErrorResponse))
)]
pub struct ApiDoc;

impl ApiDoc {
    pub fn merged() -> utoipa::openapi::OpenApi {
        let mut document = Self::openapi();
        document.merge(crate::api::v0::routes::auth::openapi::AuthApiDoc::openapi());
        document
    }
}
