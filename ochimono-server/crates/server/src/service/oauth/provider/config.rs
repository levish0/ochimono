use entity::OAuthProvider;

/// Trait defining OAuth provider configuration.
/// Implemented by OAuth providers for use in generic functions.
pub trait OAuthProviderConfig {
    const AUTH_URL: &'static str;
    const TOKEN_URL: &'static str;
    const SCOPES: &'static [&'static str];
    const PROVIDER: OAuthProvider;
}
