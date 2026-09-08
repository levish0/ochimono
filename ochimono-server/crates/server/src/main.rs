use anyhow::Context;
use config::ServerConfig;
use server::{api::router, state::AppState};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    match dotenvy::dotenv() {
        Ok(_) => {}
        Err(error) if error.not_found() => {}
        Err(error) => return Err(error.into()),
    }
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .json()
        .init();
    let config = ServerConfig::from_env()?;
    let auth = match config::auth::AuthConfig::from_env()? {
        Some(auth_config) => {
            if !auth_config.secure_cookies && !config.bind_address.ip().is_loopback() {
                anyhow::bail!("HTTP development authentication must bind to loopback");
            }
            Some(server::service::auth::AuthState::connect(auth_config).await?)
        }
        None => None,
    };
    let state = AppState {
        auth,
        ..Default::default()
    };
    let listener = tokio::net::TcpListener::bind(config.bind_address)
        .await
        .context("Failed to bind HTTP listener")?;
    #[cfg(unix)]
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    tracing::info!(address = %listener.local_addr()?, "HTTP server listening");
    let shutdown = async move {
        let interrupt = async {
            if let Err(error) = tokio::signal::ctrl_c().await {
                tracing::error!(%error, "Failed to listen for interrupt");
            }
        };
        #[cfg(unix)]
        tokio::select! {
            _ = interrupt => {},
            _ = terminate.recv() => {},
        }
        #[cfg(not(unix))]
        interrupt.await;
        tracing::info!("Shutting down HTTP server");
    };
    axum::serve(
        listener,
        router(state).into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown)
    .await
    .context("HTTP server failed")?;
    Ok(())
}
