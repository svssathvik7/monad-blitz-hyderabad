use axum::Json;
use serde::{Deserialize, Serialize};

use crate::executor::ErrorResponse;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ResponseStatus {
    Success,
    Error,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Response<T> {
    pub status: ResponseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<T>,
}

impl<T> Response<T> {
    pub fn ok(data: T) -> Json<Self> {
        Json(Response {
            status: ResponseStatus::Success,
            data: Some(data),
            error: None,
        })
    }

    pub fn error(data: T) -> Json<Self> {
        Json(Response {
            status: ResponseStatus::Error,
            data: None,
            error: Some(data),
        })
    }
}

pub fn res_err(data: &str) -> Json<Response<ErrorResponse>> {
    Response::error(ErrorResponse {
        message: data.to_string(),
        next_access: None,
    })
}
