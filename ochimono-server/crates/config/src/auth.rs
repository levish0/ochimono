use anyhow::{Context, Result, bail};
use url::Url;

/// OAuth provider credentials are deliberately not Debug.
pub struct OAuthProviderCredentials {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

/// Secrets are read by the executable and never included in diagnostic output.
pub struct AuthConfig {
    pub database_url: String,
    pub redis_url: String,
    pub browser_origin: String,
    pub secure_cookies: bool,
    pub google: OAuthProviderCredentials,
}

fn required(name: &str) -> Result<String> {
    let value = std::env::var(name).with_context(|| format!("{name} is required"))?;
    if value.trim().is_empty() {
        bail!("{name} must not be empty");
    }
    Ok(value)
}

impl AuthConfig {
    pub fn from_env() -> Result<Option<Self>> {
        match std::env::var("AUTH_ENABLED").as_deref() {
            Err(std::env::VarError::NotPresent) | Ok("false") => return Ok(None),
            Ok("true") => {}
            _ => bail!("AUTH_ENABLED must be true or false"),
        }
        let origin = required("BROWSER_ORIGIN")?;
        let parsed = Url::parse(&origin).context("Invalid BROWSER_ORIGIN")?;
        if parsed.origin().ascii_serialization() != origin
            || !parsed.username().is_empty()
            || parsed.password().is_some()
        {
            bail!("BROWSER_ORIGIN must be an exact HTTP(S) origin without a path");
        }
        let local = matches!(parsed.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
        let secure_cookies = parsed.scheme() == "https";
        if !secure_cookies && !(local && parsed.scheme() == "http") {
            bail!("BROWSER_ORIGIN requires HTTPS except on loopback");
        }
        let redirect_uri = required("GOOGLE_REDIRECT_URI")?;
        if Url::parse(&redirect_uri)?.origin() != parsed.origin() {
            bail!("GOOGLE_REDIRECT_URI must belong to BROWSER_ORIGIN");
        }
        Ok(Some(Self {
            database_url: required("DATABASE_URL")?,
            redis_url: required("REDIS_URL")?,
            browser_origin: origin,
            secure_cookies,
            google: OAuthProviderCredentials {
                client_id: required("GOOGLE_CLIENT_ID")?,
                client_secret: required("GOOGLE_CLIENT_SECRET")?,
                redirect_uri,
            },
        }))
    }
}
