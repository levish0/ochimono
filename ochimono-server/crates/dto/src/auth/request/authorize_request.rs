use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Deserialize, ToSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct AuthorizeRequest {
    #[serde(default)]
    pub remember_me: bool,
}
