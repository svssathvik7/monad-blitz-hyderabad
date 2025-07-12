use axum::body::Bytes;
use std::time::Duration;

use crate::executor::ErrorResponse;
use crate::executor::TokenDeployRequest;
use crate::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Json;
use uuid::Uuid;

use super::middleware::AuthUser;
use super::response::res_err;
use super::response::Response;
use super::response::ResponseStatus;
use axum::extract::Multipart;

use tracing::error;

#[derive(Debug, serde::Serialize)]
pub struct DeployErc20Response {
    pub contract_address: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct TokenDeployRequestData {
    pub name: String,
    pub symbol: String,
    pub total_supply: String,
    pub decimals: u8,
    pub deployer_address: String,
    pub ip: ipnetwork::IpNetwork,
}

#[allow(unused_variables)]
pub async fn deploy_erc20(
    auth_user: AuthUser,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Response<DeployErc20Response>>, (StatusCode, Json<Response<ErrorResponse>>)> {
    if !auth_user.is_github_authenticated {
        return Err((
            StatusCode::UNAUTHORIZED,
            res_err("Please authenticate with Github to deploy your tokens"),
        ));
    }
    let mut token_data: Option<TokenDeployRequestData> = None;
    let mut file_data: Option<Bytes> = None;
    let mut unique_file_name: Option<String> = None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        let field_name = field.name().unwrap_or("").to_string();

        if field_name == "file" {
            let _file_name = field.file_name().unwrap().to_string();
            let content_type = field.content_type().unwrap().to_string();
            let data = field.bytes().await.unwrap();

            let file_ext = _file_name.split('.').last().unwrap();

            unique_file_name = Some(format!("{}.{}", Uuid::new_v4(), file_ext));
            file_data = Some(data);
        } else if field_name == "data" {
            let json_data = match field.text().await {
                Ok(text) => text,
                Err(e) => {
                    error!("Error deploying erc20 {}", e);
                    return Err((
                        StatusCode::BAD_REQUEST,
                        res_err(&format!("Failed to read JSON data: {}", e)),
                    ));
                }
            };

            token_data = match serde_json::from_str(&json_data) {
                Ok(data) => Some(data),
                Err(e) => {
                    error!("Error deploying erc20 {}", e);
                    return Err((
                        StatusCode::BAD_REQUEST,
                        res_err(&format!("Invalid JSON data: {}", e)),
                    ));
                }
            };
        }
    }

    if let (Some(token), Some(file_name), Some(data)) = (token_data, unique_file_name, file_data) {
        let (tx, rx) = tokio::sync::oneshot::channel();

        {
            let supply = token.total_supply.parse::<u128>().unwrap_or_default();
            let mut queue = state.executor.deploy_queue.lock().map_err(|e| {
                error!("Error deploying erc20 {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    res_err(&format!("Something went wrong")),
                )
            })?;
            let token = TokenDeployRequest {
                file_name,
                file_data: data.to_vec(),
                name: token.name,
                symbol: token.symbol,
                total_supply: supply,
                decimals: token.decimals,
                deployer_address: token.deployer_address,
                ip: token.ip,
            };
            queue.push_back((token, tx));
        }

        return match tokio::time::timeout(Duration::from_secs(300), rx).await {
            Ok(response) => match response {
                Ok(response) => match response.clone().status {
                    ResponseStatus::Success => match response.clone().data {
                        Some(data) => Ok(Response::ok(DeployErc20Response {
                            contract_address: data.tx_hash,
                        })),
                        None => Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            res_err(&response.error.unwrap().message),
                        )),
                    },
                    ResponseStatus::Error => Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        res_err(&response.error.unwrap().message),
                    )),
                },
                Err(e) => {
                    error!("Error deploying erc20 {}", e);
                    Err((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        res_err("Executor dropped the response channel"),
                    ))
                }
            },
            Err(e) => {
                error!("Error deploying erc20 {}", e);
                Err((StatusCode::REQUEST_TIMEOUT, res_err(&"Request timed out")))
            }
        };
    }

    return Err((
        StatusCode::INTERNAL_SERVER_ERROR,
        res_err("No file uploaded"),
    ));
}
