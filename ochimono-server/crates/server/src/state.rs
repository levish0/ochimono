#[derive(Clone)]
pub struct AppState {
    pub version: &'static str,
    pub auth: Option<std::sync::Arc<crate::service::auth::AuthState>>,
}
impl Default for AppState {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION"),
            auth: None,
        }
    }
}
