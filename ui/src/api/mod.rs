//! HTTP client helpers for the 4iiz REST API.
//!
//! All requests target `/api/v1/` and include the stored auth token.

pub mod types;

use gloo_net::http::{RequestBuilder, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;

use types::ApiResponse;

/// Base path for all API requests.
const API_BASE: &str = "/api/v1";

/// Get the stored auth token from localStorage (set at login).
fn auth_token() -> Option<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item("iiz_token").ok().flatten())
}

/// Add auth header to a request builder if a token exists.
fn with_auth(builder: RequestBuilder) -> RequestBuilder {
    match auth_token() {
        Some(token) => builder.header("Authorization", &format!("Bearer {}", token)),
        None => builder,
    }
}

/// Parse error body from a non-OK response.
async fn parse_error(resp: Response) -> String {
    let status = resp.status();
    resp.json::<ApiResponse<()>>()
        .await
        .unwrap_or_else(|_| ApiResponse::error(status, "Request failed"))
        .message()
        .to_string()
}

/// GET request returning parsed JSON.
pub async fn api_get<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    let url = format!("{}{}", API_BASE, path);
    let resp = with_auth(gloo_net::http::Request::get(&url))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !resp.ok() {
        return Err(parse_error(resp).await);
    }

    resp.json::<T>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

/// POST request with JSON body, returning parsed JSON.
pub async fn api_post<B: Serialize, T: DeserializeOwned>(
    path: &str,
    body: &B,
) -> Result<T, String> {
    let url = format!("{}{}", API_BASE, path);
    let resp = with_auth(gloo_net::http::Request::post(&url))
        .json(body)
        .map_err(|e| format!("Serialize error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !resp.ok() {
        return Err(parse_error(resp).await);
    }

    resp.json::<T>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

/// PUT request with JSON body, returning parsed JSON.
pub async fn api_put<B: Serialize, T: DeserializeOwned>(
    path: &str,
    body: &B,
) -> Result<T, String> {
    let url = format!("{}{}", API_BASE, path);
    let resp = with_auth(gloo_net::http::Request::put(&url))
        .json(body)
        .map_err(|e| format!("Serialize error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !resp.ok() {
        return Err(parse_error(resp).await);
    }

    resp.json::<T>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

/// DELETE request, returning parsed JSON (or unit for 204).
pub async fn api_delete<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    let url = format!("{}{}", API_BASE, path);
    let resp = with_auth(gloo_net::http::Request::delete(&url))
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !resp.ok() {
        return Err(parse_error(resp).await);
    }

    resp.json::<T>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}
