use std::sync::Arc;
use configcat::{Client, PollingMode};
use derust::envx::{load_app_config, Environment};
use derust::httpx::{start, AppContext};
use derust::metricx::PrometheusConfig;
use derust::tracex;
use http_proxy_mock::config::AppConfig;
use http_proxy_mock::routes::Routes;
use http_proxy_mock::state::AppState;
use regex::Regex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _guard = tracex::init();

    let env = Environment::detect().ok().unwrap_or(Environment::Local);

    let app_config: AppConfig = load_app_config(env, Some("APP")).await?;

    let configcat = if let Some(configcat_sdk_key) = app_config.configcat_sdk_key {
        Client::builder(&configcat_sdk_key).build().ok()
    } else {
        None
    };

    let app_state = AppState {
        configcat: Arc::new(configcat),
    };

    let application_name = "http-proxy-mock";

    let prometheus_config = PrometheusConfig {
        denied_metric_tags: vec![],
        denied_metric_tags_by_regex: vec![Regex::new(".+_id$").unwrap()],
    };

    let context = AppContext::new(
        application_name,
        env,
        prometheus_config,
        app_state,
    )?;
    let router = Routes::routes().await;

    start(app_config.port, context, router).await
}
