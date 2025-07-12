use std::time::Duration;

use crate::{
    executor::{ErrorResponse, TokenTransferRequest},
    faucet::DripResponse,
    utils::magnify_faucet_drip,
    AppState,
};
use axum::{extract::State, Json};

use super::{
    middleware::AuthUser,
    response::{Response, ResponseStatus},
};

use tracing::error;

#[derive(serde::Serialize, Debug)]
pub struct WithdrawErc20Response {
    pub tx_hash: String,
}

#[axum::debug_handler]
#[allow(unused_variables)]
pub async fn withdraw(
    auth_user: AuthUser,
    State(state): State<AppState>,
    Json(mut payload): Json<TokenTransferRequest>,
) -> Result<Json<Response<DripResponse>>, Json<Response<ErrorResponse>>> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    // maliciously setting magnification in req payload doesn't show any effect
    let magnification = magnify_faucet_drip(
        state.config.orderbook_url.clone(),
        auth_user.clone(),
        payload.to.clone(),
    )
    .await;
    payload.magnification = Some(magnification);
    {
        let mut queue = state.executor.withdraw_queue.lock().map_err(|e| {
            error!("Failed to lock withdraw queue {}", e);
            Response::error(ErrorResponse {
                message: "Something went wrong".to_string(),
                next_access: None,
            })
        })?;
        queue.push_back((payload, tx));
    }

    match tokio::time::timeout(Duration::from_secs(60), rx).await {
        Ok(response) => match response {
            Ok(response) => match response.clone().status {
                ResponseStatus::Success => Ok(Response::ok(DripResponse {
                    tx_hash: response.data.clone().unwrap().tx_hash,
                    amount: response.data.unwrap().amount,
                    magnification,
                })),
                ResponseStatus::Error => Err(Response::error(response.error.unwrap())),
            },
            Err(e) => Err(Response::error(ErrorResponse {
                message: "Executor dropped the response channel".to_string(),
                next_access: None,
            })),
        },
        Err(e) => {
            error!("Error at /withdraw {}", e);
            Err(Response::error(ErrorResponse {
                message: "Request timed out".to_string(),
                next_access: None,
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::{extract::State, Json};

    use crate::{
        executor::TokenTransferRequest,
        handlers::{middleware::AuthUser, withdraw::withdraw},
        store::TokenType,
        utils::setup,
    };

    #[tokio::test]
    async fn test_erc20_withdraw_github_auth() {
        let state = setup().await;
        let executor_clone = state.executor.clone();
        tokio::spawn(async move {
            executor_clone.process_queue().await;
        });
        let auth_user = AuthUser {
            is_github_authenticated: true,
            user_id: "7313fc8b-491c-4275-b247-a7c489f88441".to_string(),
        };
        let payload = Json(TokenTransferRequest {
            to: "0xDda173bd23b07007394611D789EF789a9Aae5CF5".to_string(),
            token_address: "0xb1baC9E12095043045d19F3E7a988D0C63dC2523".to_string(),
            token_type: TokenType::ERC20,
            ip: ipnetwork::IpNetwork::V4("60.243.163.4".parse().unwrap()),
            magnification: None,
        });
        let response = withdraw(auth_user, State(state), payload).await.unwrap();
        println!("{:?}", response);
    }
    #[tokio::test]
    async fn test_erc20_withdraw_no_auth() {
        let state = setup().await;
        let executor_clone = state.executor.clone();
        tokio::spawn(async move {
            executor_clone.process_queue().await;
        });
        let auth_user = AuthUser {
            is_github_authenticated: false,
            user_id: "7313fc8b-491c-4275-b247-a7c489f88441".to_string(),
        };
        let payload = Json(TokenTransferRequest {
            to: "0xd53D4f100AaBA314bF033f99f86a312BfbdDF113".to_string(),
            token_address: "0xb1baC9E12095043045d19F3E7a988D0C63dC2523".to_string(),
            token_type: TokenType::ERC20,
            ip: ipnetwork::IpNetwork::V4("60.143.163.20".parse().unwrap()),
            magnification: None,
        });
        let response = withdraw(auth_user, State(state), payload).await.unwrap();
        println!("{:?}", response);
    }
    #[tokio::test]
    async fn test_erc20_withdraw_garden_user() {
        let state = setup().await;
        let executor_clone = state.executor.clone();
        tokio::spawn(async move {
            executor_clone.process_queue().await;
        });
        let auth_user = AuthUser {
            is_github_authenticated: true,
            user_id: "7313fc8b-491c-4275-b247-a7c489f88441".to_string(),
        };
        let payload = Json(TokenTransferRequest {
            to: "0x41154d8D32dA87A7c565e964CD191243B728EDF7".to_string(),
            token_address: "0xb1baC9E12095043045d19F3E7a988D0C63dC2523".to_string(),
            token_type: TokenType::ERC20,
            ip: ipnetwork::IpNetwork::V4("60.243.163.20".parse().unwrap()),
            magnification: None,
        });
        let response = withdraw(auth_user, State(state), payload).await.unwrap();
        println!("{:?}", response);
    }

    #[tokio::test]
    async fn test_native_token_withdraw_github_auth() {
        let state = setup().await;
        let executor_clone = state.executor.clone();
        tokio::spawn(async move {
            executor_clone.process_queue().await;
        });
        let auth_user = AuthUser {
            is_github_authenticated: true,
            user_id: "7313fc8b-491c-4275-b247-a7c489f88441".to_string(),
        };
        let payload = Json(TokenTransferRequest {
            to: "0xDda173bd23b07007394611D789EF789a9Aae5CF5".to_string(),
            // need to test on native token by replacing the address
            token_address: "0xb1baC9E12095043045d19F3E7a988D0C63dC2523".to_string(),
            token_type: TokenType::NATIVE,
            ip: ipnetwork::IpNetwork::V4("60.243.163.1".parse().unwrap()),
            magnification: None,
        });
        let response = withdraw(auth_user, State(state), payload).await.unwrap();
        println!("{:?}", response);
    }

    #[tokio::test]
    async fn test_erc20_withdraw_no_auth_load_test() {
        let state = setup().await;
        let executor_clone = state.executor.clone();
        tokio::spawn(async move {
            executor_clone.process_queue().await;
        });

        let start_time = tokio::time::Instant::now();
        let mut handles = vec![];

        for i in 0..100 {
            let state = state.clone();
            let handle = tokio::spawn(async move {
                let request_start = tokio::time::Instant::now();
                let auth_user = AuthUser {
                    is_github_authenticated: false,
                    user_id: "7313fc8b-491c-4275-b247-a7c489f88441".to_string(),
                };

                let ip = format!("60.103.163.{}", i + 1);
                let payload = Json(TokenTransferRequest {
                    to: "0xd53D4f100AaBA314bF033f99f86a312BfbdDF113".to_string(),
                    token_address: "0xb1baC9E12095043045d19F3E7a988D0C63dC2523".to_string(),
                    token_type: TokenType::ERC20,
                    ip: ipnetwork::IpNetwork::V4(ip.parse().unwrap()),
                    magnification: None,
                });

                let response = withdraw(auth_user, State(state), payload).await.unwrap();
                let request_duration = request_start.elapsed();
                println!(
                    "Request {} completed in {:?} with response: {:?}",
                    i + 1,
                    request_duration,
                    response
                );
                request_duration
            });
            handles.push(handle);
        }

        let durations: Vec<_> = futures::future::join_all(handles)
            .await
            .into_iter()
            .filter_map(|r| r.ok())
            .collect();

        let total_duration = start_time.elapsed();
        let avg_duration = durations.iter().sum::<tokio::time::Duration>() / durations.len() as u32;

        let mut file = std::fs::File::create("load_response.txt").expect("Failed to create file");
        use std::io::Write;
        writeln!(file, "\nLoad Test Summary:").expect("Failed to write to file");
        writeln!(file, "Total test duration: {:?}", total_duration)
            .expect("Failed to write to file");
        writeln!(file, "Average request duration: {:?}", avg_duration)
            .expect("Failed to write to file");
        writeln!(file, "Number of successful requests: {}", durations.len())
            .expect("Failed to write to file");
    }
}
