use crate::service::oauth::provider::config::OAuthProviderConfig;
use entity::OAuthProvider;

/// Data structure for google provider.
pub struct GoogleProvider;

impl OAuthProviderConfig for GoogleProvider {
    const AUTH_URL: &'static str = "https://accounts.google.com/o/oauth2/v2/auth";
    const TOKEN_URL: &'static str = "https://oauth2.googleapis.com/token";
    const SCOPES: &'static [&'static str] = &["email", "profile"];
    const PROVIDER: OAuthProvider = OAuthProvider::Google;
}
