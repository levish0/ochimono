use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct AuthorizeResponse {
    pub auth_url: String,
}
