use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, ToSchema)]
pub struct SessionResponse {
    pub id: Uuid,
    pub current: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}
