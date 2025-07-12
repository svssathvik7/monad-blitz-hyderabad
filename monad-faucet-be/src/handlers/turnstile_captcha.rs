use axum::{extract::State, http::StatusCode, Json};
use reqwest::Client;
use serde_json::json;
use tracing::error;

use crate::{executor::ErrorResponse, handlers::response::Response, AppState};

use super::response::res_err;

#[derive(serde::Deserialize, Debug)]
pub struct TurnstilePayload {
    pub token: String,
}

#[derive(serde::Serialize)]
pub struct TurnstileResponse {
    pub success: bool,
}

pub async fn verify_turnstile_captcha(
    State(state): State<AppState>,
    Json(body): Json<TurnstilePayload>,
) -> Result<Json<Response<TurnstileResponse>>, (StatusCode, Json<Response<ErrorResponse>>)> {
    if body.token.is_empty() {
        return Err((StatusCode::BAD_REQUEST, res_err("Invalid request")));
    }

    let success = verify(&state.config.captcha_secret_key, &body.token).await;
    Ok(Response::ok(TurnstileResponse { success }))
}

async fn verify(secret_key: &str, token: &str) -> bool {
    let request = Client::new();
    let response = match request
        .post("https://challenges.cloudflare.com/turnstile/v0/siteverify")
        .json(&json!({
            "secret": secret_key,
            "response": token
        }))
        .send()
        .await
    {
        Ok(response) => response,
        Err(e) => {
            error!("Error verifying user {}", e);
            return false;
        }
    };

    let body = match response.text().await {
        Ok(body) => body,
        Err(e) => {
            error!("Error parsing the verify response body {}", e);
            return false;
        }
    };
    let body_json: serde_json::Value = match serde_json::from_str(&body) {
        Ok(json_data) => json_data,
        Err(e) => {
            error!("Error serializing the body to json value {}", e);
            return false;
        }
    };

    body_json["success"].as_bool().unwrap_or(false)
}
