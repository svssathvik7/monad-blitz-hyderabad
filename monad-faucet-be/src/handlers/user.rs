use axum::{extract::State, Json};
use tracing::error;

use crate::{
    executor::ErrorResponse,
    handlers::response::Response,
    store::{Store, User},
    AppState,
};

use super::middleware::AuthUser;

#[axum::debug_handler]
pub async fn user(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<Json<Response<User>>, Json<Response<ErrorResponse>>> {
    let user = state
        .store
        .get_user_by_id(auth_user.user_id.to_string())
        .await;

    match user {
        Ok(user) => Ok(Response::ok(user)),
        Err(e) => {
            error!("Error at users handler {}", e);
            return Err(Response::error(ErrorResponse {
                message: format!("Failed to get user: {:?}", e),
                next_access: None,
            }));
        }
    }
}
