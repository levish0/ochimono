use super::UserResponse;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum LoginResponse {
    SignedIn { user: UserResponse },
    PendingSignup { pending_token: String },
}
