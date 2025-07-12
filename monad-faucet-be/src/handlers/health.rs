use axum::Json;
use reqwest::StatusCode;
use serde::Serialize;

use crate::handlers::response::Response;

#[derive(Debug, Serialize)]
pub struct HealthCheckResponse {
    status: String,
}

pub async fn health_check(
) -> Result<Json<Response<HealthCheckResponse>>, (StatusCode, Json<Response<()>>)> {
    Ok(Response::ok(HealthCheckResponse {
        status: "OK".to_string(),
    }))
}
