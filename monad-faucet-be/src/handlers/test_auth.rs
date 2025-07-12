use axum::{http::StatusCode, Json};

use super::{middleware::AuthUser, response::Response};

#[axum::debug_handler]
pub async fn test_auth(
    auth_user: AuthUser,
) -> Result<Json<Response<String>>, (StatusCode, Json<Response<()>>)> {
    Ok(Response::ok(format!("Hello, {}", auth_user.user_id)))
}
