use anyhow::{Context, Result};
use std::net::SocketAddr;

/// Process configuration. Environment loading belongs to the executable.
#[derive(Clone, Debug)]
pub struct ServerConfig {
    pub bind_address: SocketAddr,
}

impl ServerConfig {
    pub fn from_env() -> Result<Self> {
        let bind_address = match std::env::var("SERVER_BIND_ADDRESS") {
            Ok(value) => value,
            Err(std::env::VarError::NotPresent) => "127.0.0.1:8000".to_owned(),
            Err(error) => return Err(error).context("Invalid SERVER_BIND_ADDRESS encoding"),
        };
        Ok(Self {
            bind_address: bind_address
                .parse()
                .context("Invalid SERVER_BIND_ADDRESS")?,
        })
    }
}
