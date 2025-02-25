use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct AppConfig {
    pub port: u16,
}