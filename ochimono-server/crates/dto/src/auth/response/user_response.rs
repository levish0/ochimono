use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct UserResponse {
    pub id: Uuid,
    pub handle: String,
}
