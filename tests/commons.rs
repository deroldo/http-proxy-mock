use axum::Router;
use axum::response::Response;
use configcat::Client;
use derust::envx::{Environment, load_app_config};
use derust::httpx::AppContext;
use http_proxy_mock::config::AppConfig;
use http_proxy_mock::routes::Routes;
use http_proxy_mock::state::AppState;
use jsonpath_rust::JsonPathQuery;
use rand::Rng;
use serde_json::Value;
use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;
use test_context::AsyncTestContext;
use wiremock::MockServer;

#[allow(dead_code)]
pub struct TestContext {
    pub app_state: AppState,
    pub router: Router<AppContext<AppState>>,
    pub wiremock: MockServer,
}

impl AsyncTestContext for TestContext {
    #[allow(dead_code)]
    async fn setup() -> Self {
        let env = Environment::detect().ok().unwrap_or(Environment::Local);

        let app_config: AppConfig = load_app_config(env, Some("APP")).await.unwrap();

        let configcat = if let Some(configcat_sdk_key) = app_config.configcat_sdk_key {
            Client::builder(&configcat_sdk_key).build().ok()
        } else {
            None
        };

        let app_state = AppState { configcat: Arc::new(configcat) };

        let router = Routes::routes().await;

        let wiremock = create_mock_server().await;

        Self { app_state, router, wiremock }
    }
}

async fn create_mock_server() -> MockServer {
    for _ in 1..10 {
        // try to start mock server in a random port 10 times
        let port = rand::thread_rng().gen_range(51000..54000);
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        if let Ok(listener) = TcpListener::bind(addr) {
            let mock_server = MockServer::builder().listener(listener).start().await;
            return mock_server;
        }
    }
    panic!("Failed to create mock server");
}

pub trait CommonsJson {
    #[allow(dead_code)]
    fn path_as_string(
        &self,
        path: &str,
    ) -> String;
}

impl CommonsJson for Value {
    #[allow(dead_code)]
    fn path_as_string(
        &self,
        path: &str,
    ) -> String {
        self.clone().path(path).unwrap().as_array().unwrap()[0].as_str().unwrap().to_string()
    }
}

pub trait ResponseJson {
    #[allow(dead_code)]
    fn to_value(self) -> impl std::future::Future<Output = Result<Value, Box<dyn std::error::Error>>> + Send;
}

impl ResponseJson for Response {
    #[allow(dead_code)]
    async fn to_value(self) -> Result<Value, Box<dyn std::error::Error>> {
        let (_, body) = self.into_parts();
        let bytes = body.collect().await.unwrap().to_bytes();

        Ok(serde_json::from_slice(&bytes).unwrap())
    }
}
