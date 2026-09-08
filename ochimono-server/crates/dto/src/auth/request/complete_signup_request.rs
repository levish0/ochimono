use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Deserialize, ToSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct CompleteSignupRequest {
    pub pending_token: String,
    pub handle: String,
}
