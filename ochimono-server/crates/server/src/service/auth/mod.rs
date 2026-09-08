pub mod session;
use anyhow::Result;
use auth_core::session::SessionPolicy;
use config::auth::AuthConfig;
use redis::aio::ConnectionManager;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, EntityTrait, QuerySelect};
use std::{sync::Arc, time::Duration};

pub struct AuthState {
    pub db: DatabaseConnection,
    pub redis: ConnectionManager,
    pub http: reqwest::Client,
    pub config: AuthConfig,
    pub policy: SessionPolicy,
}
impl AuthState {
    pub async fn connect(config: AuthConfig) -> Result<Arc<Self>> {
        let mut options = ConnectOptions::new(config.database_url.clone());
        options
            .max_connections(20)
            .connect_timeout(Duration::from_secs(5))
            .sqlx_logging(false);
        let db = Database::connect(options)
            .await
            .map_err(|_| anyhow::anyhow!("Cannot connect to authentication PostgreSQL"))?;
        entity::users::Entity::find()
            .limit(1)
            .all(&db)
            .await
            .map_err(|_| {
                anyhow::anyhow!("Run authentication migrations before starting the server")
            })?;
        let redis = ConnectionManager::new(
            redis::Client::open(config.redis_url.as_str())
                .map_err(|_| anyhow::anyhow!("Invalid REDIS_URL"))?,
        )
        .await
        .map_err(|_| anyhow::anyhow!("Cannot connect to authentication Redis"))?;
        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .timeout(Duration::from_secs(10))
            .build()?;
        Ok(Arc::new(Self {
            db,
            redis,
            http,
            config,
            policy: SessionPolicy {
                sliding_ttl_hours: 168,
                max_lifetime_hours: 720,
            },
        }))
    }
}
