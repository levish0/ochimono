#[derive(Clone)]
pub struct AppState {
    pub version: &'static str,
}
impl Default for AppState {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION"),
        }
    }
}
