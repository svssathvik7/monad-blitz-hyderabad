use axum::{extract::State, Json};
use reqwest::StatusCode;
use tracing::error;

use crate::executor::ErrorResponse;
use crate::store::{Store, Token};
use crate::AppState;

use super::response::{res_err, Response};

pub async fn tokens(
    State(state): State<AppState>,
) -> Result<Json<Response<Vec<Token>>>, (StatusCode, Json<Response<ErrorResponse>>)> {
    let tokens = match state.store.get_all_tokens().await {
        Ok(tokens) => tokens,
        Err(e) => {
            error!("Error fetching tokens {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, res_err(&e.to_string())));
        }
    };
    Ok(Response::ok(tokens))
}
