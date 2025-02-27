use crate::service::proxy::service::ProxyService;
use crate::state::AppState;
use axum::Router;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::routing::{delete, get, patch, post, put};
use derust::httpx::json::JsonResponse;
use derust::httpx::{AppContext, HttpError, HttpTags};
use serde_json::Value;

pub struct ProxyRoutes;

impl ProxyRoutes {
    pub fn routes() -> Router<AppContext<AppState>> {
        Router::new()
            .route("/", get(get_handler))
            .route("/{*path}", get(get_handler_with_path))
            .route("/", post(post_handler))
            .route("/{*path}", post(post_handler_with_path))
            .route("/", put(put_handler))
            .route("/{*path}", put(put_handler_with_path))
            .route("/", delete(delete_handler))
            .route("/{*path}", delete(delete_handler_with_path))
            .route("/", patch(patch_handler))
            .route("/{*path}", patch(patch_handler_with_path))
    }
}

async fn get_handler(
    State(context): State<AppContext<AppState>>,
    headers: HeaderMap,
) -> Result<JsonResponse<Value>, HttpError> {
    ProxyService::proxy_or_mock(&context, Method::GET, &headers, String::new(), None).await
}

async fn get_handler_with_path(
    State(context): State<AppContext<AppState>>,
    Path(path): Path<String>,
    headers: HeaderMap,
) -> Result<JsonResponse<Value>, HttpError> {
    ProxyService::proxy_or_mock(&context, Method::GET, &headers, path, None).await
}

async fn post_handler(
    State(context): State<AppContext<AppState>>,
    headers: HeaderMap,
    body: String,
) -> Result<JsonResponse<Value>, HttpError> {
    ProxyService::proxy_or_mock(&context, Method::POST, &headers, String::new(), parse_body_to_json(&body)?).await
}

async fn post_handler_with_path(
    State(context): State<AppContext<AppState>>,
    Path(path): Path<String>,
    headers: HeaderMap,
    body: String,
) -> Result<JsonResponse<Value>, HttpError> {
    ProxyService::proxy_or_mock(&context, Method::POST, &headers, path, parse_body_to_json(&body)?).await
}

async fn put_handler(
    State(context): State<AppContext<AppState>>,
    headers: HeaderMap,
    body: String,
) -> Result<JsonResponse<Value>, HttpError> {
    ProxyService::proxy_or_mock(&context, Method::PUT, &headers, String::new(), parse_body_to_json(&body)?).await
}

async fn put_handler_with_path(
    State(context): State<AppContext<AppState>>,
    Path(path): Path<String>,
    headers: HeaderMap,
    body: String,
) -> Result<JsonResponse<Value>, HttpError> {
    ProxyService::proxy_or_mock(&context, Method::PUT, &headers, path, parse_body_to_json(&body)?).await
}

async fn delete_handler(
    State(context): State<AppContext<AppState>>,
    headers: HeaderMap,
) -> Result<JsonResponse<Value>, HttpError> {
    ProxyService::proxy_or_mock(&context, Method::DELETE, &headers, String::new(), None).await
}

async fn delete_handler_with_path(
    State(context): State<AppContext<AppState>>,
    Path(path): Path<String>,
    headers: HeaderMap,
) -> Result<JsonResponse<Value>, HttpError> {
    ProxyService::proxy_or_mock(&context, Method::DELETE, &headers, path, None).await
}

async fn patch_handler(
    State(context): State<AppContext<AppState>>,
    headers: HeaderMap,
    body: String,
) -> Result<JsonResponse<Value>, HttpError> {
    ProxyService::proxy_or_mock(&context, Method::PATCH, &headers, String::new(), parse_body_to_json(&body)?).await
}

async fn patch_handler_with_path(
    State(context): State<AppContext<AppState>>,
    Path(path): Path<String>,
    headers: HeaderMap,
    body: String,
) -> Result<JsonResponse<Value>, HttpError> {
    ProxyService::proxy_or_mock(&context, Method::PATCH, &headers, path, parse_body_to_json(&body)?).await
}

fn parse_body_to_json(body: &str) -> Result<Option<Value>, HttpError> {
    if body.is_empty() {
        return Ok(None);
    }

    serde_json::from_str(body)
        .map(Some)
        .map_err(|error| HttpError::without_body(StatusCode::BAD_REQUEST, format!("Invalid JSON: {error}"), HttpTags::default()))
}
