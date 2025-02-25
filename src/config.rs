use serde::Deserialize;

#[derive(Clone, Deserialize)]
pub struct AppConfig {
    pub port: u16,
    pub configcat_sdk_key: Option<String>,
}