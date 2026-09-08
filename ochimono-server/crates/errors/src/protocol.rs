//! Error code constants

pub mod auth {
    pub const AUTH_ALREADY_AUTHENTICATED: &str = "auth:already_authenticated";
    pub const AUTH_INVALID_CREDENTIALS: &str = "auth:invalid_credentials";
    pub const AUTH_PROOF_STALE: &str = "auth:proof_stale";
    pub const AUTH_SIGNUP_RESTRICTED: &str = "auth:signup_restricted";
}

pub mod user {
    pub const USER_INVALID_PASSWORD: &str = "user:invalid_password";
    pub const USER_PASSWORD_NOT_SET: &str = "user:password_not_set";
    pub const REAUTHENTICATION_REQUIRED: &str = "user:reauthentication_required";
    pub const USER_INVALID_SESSION: &str = "user:invalid_session";
    pub const USER_NOT_VERIFIED: &str = "user:not_verified";
    pub const USER_NOT_FOUND: &str = "user:not_found";
    pub const USER_UNAUTHORIZED: &str = "user:unauthorized";
    pub const USER_BANNED: &str = "user:banned";
    pub const USER_PERMISSION_INSUFFICIENT: &str = "user:permission_insufficient";
    pub const USER_HANDLE_ALREADY_EXISTS: &str = "user:handle_already_exists";
    pub const USER_EMAIL_ALREADY_EXISTS: &str = "user:email_already_exists";
    pub const USER_TOKEN_EXPIRED: &str = "user:token_expired";
    pub const USER_NO_REFRESH_TOKEN: &str = "user:no_refresh_token";
    pub const USER_INVALID_TOKEN: &str = "user:invalid_token";
    pub const USER_NOT_BANNED: &str = "user:not_banned";
    pub const USER_ALREADY_BANNED: &str = "user:already_banned";
    pub const USER_DOES_NOT_HAVE_ROLE: &str = "user:does_not_have_role";
    pub const USER_ALREADY_HAS_ROLE: &str = "user:already_has_role";
    pub const USER_CANNOT_MANAGE_SELF: &str = "user:cannot_manage_self";
    pub const USER_CANNOT_MANAGE_HIGHER_OR_EQUAL_ROLE: &str =
        "user:cannot_manage_higher_or_equal_role";
}

pub mod oauth {
    pub const OAUTH_INVALID_AUTH_URL: &str = "oauth:invalid_auth_url";
    pub const OAUTH_INVALID_TOKEN_URL: &str = "oauth:invalid_token_url";
    pub const OAUTH_INVALID_REDIRECT_URL: &str = "oauth:invalid_redirect_url";
    pub const OAUTH_TOKEN_EXCHANGE_FAILED: &str = "oauth:token_exchange_failed";
    pub const OAUTH_USER_INFO_FETCH_FAILED: &str = "oauth:user_info_fetch_failed";
    pub const OAUTH_USER_INFO_PARSE_FAILED: &str = "oauth:user_info_parse_failed";
    pub const OAUTH_ACCOUNT_ALREADY_LINKED: &str = "oauth:account_already_linked";
    pub const OAUTH_CONNECTION_NOT_FOUND: &str = "oauth:connection_not_found";
    pub const OAUTH_CANNOT_UNLINK_LAST_CONNECTION: &str = "oauth:cannot_unlink_last_connection";
    pub const OAUTH_INVALID_IMAGE_URL: &str = "oauth:invalid_image_url";
    pub const OAUTH_INVALID_STATE: &str = "oauth:invalid_state";
    pub const OAUTH_STATE_EXPIRED: &str = "oauth:state_expired";
    pub const OAUTH_HANDLE_REQUIRED: &str = "oauth:handle_required";
    pub const OAUTH_EMAIL_ALREADY_EXISTS: &str = "oauth:email_already_exists";
    pub const OAUTH_EMAIL_NOT_VERIFIED: &str = "oauth:email_not_verified";
    pub const GOOGLE_INVALID_ID_TOKEN: &str = "google:invalid_id_token";
    pub const GOOGLE_ONE_TAP_NONCE_INVALID: &str = "google:one_tap_nonce_invalid";
    pub const GOOGLE_JWKS_FETCH_FAILED: &str = "google:jwks_fetch_failed";
    pub const GOOGLE_JWKS_PARSE_FAILED: &str = "google:jwks_parse_failed";
}

pub mod general {
    pub const GLOBAL_GRANT_REVISION_CONFLICT: &str = "global:grant_revision_conflict";
    pub const BAD_REQUEST: &str = "general:bad_request";
    pub const IDEMPOTENCY_CONFLICT: &str = "general:idempotency_conflict";
    pub const ACCESS_SECURITY_SNAPSHOT_CONFLICT: &str = "security:snapshot_conflict";
    pub const ACCESS_SECURITY_CHANGE_FEED_GONE: &str = "security:change_feed_gone";
    pub const VALIDATION_ERROR: &str = "general:validation_error";
    pub const INVALID_IP_ADDRESS: &str = "general:invalid_ip_address";
    pub const FORBIDDEN: &str = "general:forbidden";
    pub const FILE_TOO_LARGE: &str = "general:file_too_large";
}

pub mod session {
    pub const SESSION_INVALID_USER_ID: &str = "session:invalid_user_id";
    pub const SESSION_EXPIRED: &str = "session:expired";
    pub const SESSION_NOT_FOUND: &str = "session:not_found";
}

pub mod system {
    pub const SECURITY_PROJECTION_STALE: &str = "security:projection_stale";
    pub const RENDER_CAPACITY_EXCEEDED: &str = "render:capacity_exceeded";
    pub const SYS_INTERNAL_ERROR: &str = "system:internal_error";
    pub const SYS_HASHING_ERROR: &str = "system:hashing_error";
    pub const SYS_NOT_FOUND: &str = "system:not_found";
    pub const SYS_TRANSACTION_ERROR: &str = "system:transaction_error";
    pub const SYS_DATABASE_ERROR: &str = "system:database_error";
    pub const SYS_TOKEN_CREATION_ERROR: &str = "system:token_creation_error";
}

pub mod rate_limit {
    pub const RATE_LIMIT_EXCEEDED: &str = "rate_limit:exceeded";
}
