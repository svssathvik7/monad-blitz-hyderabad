use crate::AppState;
use crate::{executor::ErrorResponse, handlers::auth::validate_and_decode_jwt};
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    Json,
};

use super::response::{res_err, Response};

#[derive(Clone, Debug)]
pub struct AuthUser {
    pub user_id: String,
    pub is_github_authenticated: bool,
}

#[async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<Response<ErrorResponse>>);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let state: &AppState = match parts.extensions.get() {
            Some(state) => state,
            None => {
                return Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    res_err("Failed to get state"),
                ));
            }
        };

        if let Some(auth_header) = parts.headers.get("Authorization") {
            if let Ok(token) = auth_header.to_str() {
                if let Some(token) = token.strip_prefix("Bearer ") {
                    if let Ok(claims) = validate_and_decode_jwt(token, &state.config.jwt_secret) {
                        return Ok(AuthUser {
                            user_id: claims.sub,
                            is_github_authenticated: true,
                        });
                    }
                }
            }
        }

        Ok(AuthUser {
            user_id: String::new(),
            is_github_authenticated: false,
        })
    }
}
