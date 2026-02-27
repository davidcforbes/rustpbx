//! JWT authentication and dev-mode header bypass.
//!
//! The `AuthContext` axum extractor checks:
//! 1. `Authorization: Bearer <jwt>` — decoded with the configured secret.
//! 2. Dev-mode headers (`X-Account-Id`, `X-User-Id`) — accepted only when
//!    `jwt_secret` is empty (no production secret configured).
//!
//! Handlers receive `AuthContext` to access the authenticated user's identity.

use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::iiz::api::IizState;

/// JWT claims structure — encoded in every access token.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Subject — the user's UUID.
    pub sub: Uuid,
    /// The account (tenant) this user belongs to.
    pub account_id: Uuid,
    /// Role slug (e.g. "admin", "manager", "agent").
    pub role: String,
    /// Expiry (seconds since epoch).
    pub exp: usize,
}

/// Authenticated context available to handlers after extraction.
#[derive(Debug, Clone)]
pub struct AuthContext {
    pub user_id: Uuid,
    pub account_id: Uuid,
    pub role: String,
}

/// Axum extractor — pulls `AuthContext` from the request.
///
/// Checks JWT first, then falls back to dev headers when `jwt_secret` is empty.
impl<S> FromRequestParts<S> for AuthContext
where
    IizState: axum::extract::FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let iiz_state = IizState::from_ref(state);

        // --- Try JWT bearer token ---
        if let Some(auth_header) = parts.headers.get("authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                if let Some(token) = auth_str.strip_prefix("Bearer ") {
                    if !iiz_state.jwt_secret.is_empty() {
                        match decode::<Claims>(
                            token,
                            &DecodingKey::from_secret(&iiz_state.jwt_secret),
                            &Validation::new(Algorithm::HS256),
                        ) {
                            Ok(data) => {
                                return Ok(AuthContext {
                                    user_id: data.claims.sub,
                                    account_id: data.claims.account_id,
                                    role: data.claims.role,
                                });
                            }
                            Err(_) => {
                                return Err((
                                    StatusCode::UNAUTHORIZED,
                                    Json(serde_json::json!({
                                        "error": "unauthorized",
                                        "message": "Invalid token"
                                    })),
                                )
                                    .into_response());
                            }
                        }
                    }
                }
            }
        }

        // --- Dev-mode: accept X-Account-Id + X-User-Id headers ---
        // Only when jwt_secret is empty (no production secret configured).
        if iiz_state.jwt_secret.is_empty() {
            if let (Some(account_id), Some(user_id)) = (
                parts
                    .headers
                    .get("x-account-id")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| Uuid::parse_str(s).ok()),
                parts
                    .headers
                    .get("x-user-id")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| Uuid::parse_str(s).ok()),
            ) {
                let role = parts
                    .headers
                    .get("x-user-role")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("admin")
                    .to_string();
                return Ok(AuthContext {
                    user_id,
                    account_id,
                    role,
                });
            }
        }

        Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "unauthorized",
                "message": "Missing or invalid authentication"
            })),
        )
            .into_response())
    }
}

/// Create a signed JWT token from claims.
///
/// Used by login endpoints (Phase 1) to issue access tokens.
pub fn create_token(
    secret: &[u8],
    claims: &Claims,
) -> Result<String, jsonwebtoken::errors::Error> {
    jsonwebtoken::encode(
        &jsonwebtoken::Header::new(Algorithm::HS256),
        claims,
        &jsonwebtoken::EncodingKey::from_secret(secret),
    )
}
