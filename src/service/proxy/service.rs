use crate::state::AppState;
use axum::http::{HeaderMap, Method, StatusCode};
use configcat::User;
use derust::http_clientx::{HttpClient, Response};
use derust::httpx::json::JsonResponse;
use derust::httpx::{AppContext, HttpError, HttpResponse, HttpTags};
use serde::Deserialize;
use serde_json::Value;
use std::fmt::format;
use tracing::info;
use uuid::Uuid;

#[derive(Deserialize)]
struct Feature {
    response_status_code: u16,
    body: Value,
}

pub struct ProxyService;

impl ProxyService {
    pub async fn proxy_or_mock(
        context: &AppContext<AppState>,
        method: Method,
        headers: &HeaderMap,
        path: String,
        body: Option<Value>,
    ) -> Result<JsonResponse<Value>, HttpError> {
        let mock = get_mock(context, &method, headers, &path, &body).await?;
        if let Some(found_mock) = mock {
            return Ok(found_mock);
        }

        let original_url = get_header("X-MOCK-ORIGINAL-URL", &headers)?;
        let timeout = get_header("X-MOCK-TIMEOUT", &headers).ok().unwrap_or("1000".to_string()).parse::<u64>().unwrap_or(1000);
        let gateway = HttpClient::new("http-proxy-client", &original_url, timeout, 1000).await.map_err(|error| HttpError::without_body(
            StatusCode::BAD_REQUEST,
            format!("Failed to create proxy gateway: {error}"),
            HttpTags::default(),
        ))?;
        let headers = header_map_to_vec(headers);

        let response_result: Result<Response<Value>, HttpError> = match method {
            Method::GET => gateway.get(context, &format!("/{path}"), None, Some(headers), &HttpTags::default()).await,
            Method::POST => gateway.post(context, &format!("/{path}"), &body.unwrap_or(Value::Null), None, Some(headers), &HttpTags::default()).await,
            Method::PUT => gateway.put(context, &format!("/{path}"), &body.unwrap_or(Value::Null), None, Some(headers), &HttpTags::default()).await,
            Method::DELETE => gateway.delete::<Value, Value, AppState>(context, &format!("/{path}"), None, Some(headers), &HttpTags::default()).await,
            Method::PATCH => gateway.patch(context, &format!("/{path}"), &body.unwrap_or(Value::Null), None, Some(headers), &HttpTags::default()).await,
            _ => Err(HttpError::without_body(
                StatusCode::NOT_IMPLEMENTED,
                format!("Unsupported method: {method}"),
                HttpTags::default(),
            ))
        };

        match response_result {
            Ok(response) => {
                Ok(JsonResponse::new(response.status_code, response.body.unwrap_or(Value::Null), HttpTags::default()))
            },
            Err(error) => {
                Ok(JsonResponse::new(error.status_code(), error.response_json().unwrap_or(error.response_body().map(Value::String).unwrap_or(Value::Null)), HttpTags::default()))
            }
        }
    }
}

async fn get_mock(
    context: &AppContext<AppState>,
    method: &Method,
    headers: &HeaderMap,
    path: &str,
    body: &Option<Value>,
) -> Result<Option<JsonResponse<Value>>, HttpError> {
    if let Some(feature_name) = get_header("X-MOCK-FEATURE", &headers).ok() {
        if let Some(configcat) = context.state().configcat.as_ref() {
            let json_body = if let Some(request_body) = body {
                serde_json::to_string(&request_body).map_err(|error| HttpError::without_body(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to serialize request body: {error}"),
                    HttpTags::default(),
                ))?
            } else {
                "".to_string()
            };

            let configcat_user = User::new(Uuid::now_v7().to_string().as_str())
                .custom("method", format!("{method}").as_str())
                .custom("path", format!("/{path}").as_str())
                .custom("body", json_body.as_str());

            let mock = configcat.get_value(&feature_name, "None".to_string(), Some(configcat_user)).await;
            if !mock.is_empty() && mock != "None" {
                info!("Mocking feature {feature_name}");

                let feature = serde_json::from_str::<Feature>(&mock).map_err(|error| HttpError::without_body(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to deserialize feature {feature_name} with mock: {error}"),
                    HttpTags::default(),
                ))?;
                let response_status_code = StatusCode::from_u16(feature.response_status_code).map_err(|error| HttpError::without_body(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Invalid mock response status code {} for feature {feature_name}: {error}", feature.response_status_code),
                    HttpTags::default(),
                ))?;

                return Ok(Some(JsonResponse::new(response_status_code, feature.body, HttpTags::default())));
            }
        }
    }

    Ok(None)
}

fn get_header(header_name: &str, headers: &HeaderMap) -> Result<String, HttpError> {
    headers.get(header_name).ok_or(HttpError::without_body(
        StatusCode::BAD_REQUEST,
        format!("{header_name} header is missing"),
        HttpTags::default(),
    ))?.to_str()
        .map(|value| value.to_string())
        .map_err(|error| HttpError::without_body(
            StatusCode::BAD_REQUEST,
            format!("X-FEATURE header is invalid: {error}"),
            HttpTags::default(),
        ))
}

fn header_map_to_vec(header_map: &HeaderMap) -> Vec<(&str, &str)> {
    header_map
        .iter()
        .filter(|(key, _)| key.as_str().to_uppercase() != "HOST")
        .filter_map(|(key, value)| {
            value.to_str().ok().map(|v| (key.as_str(), v))
        })
        .collect()
}
