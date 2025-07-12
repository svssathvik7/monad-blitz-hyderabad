use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::Duration,
};

use alloy::{hex::FromHex, primitives::FixedBytes, signers::local::PrivateKeySigner};
use chrono::{DateTime, Utc};
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

use crate::{
    config::Config,
    faucet::{self, DripResponse},
    handlers::response::ResponseStatus,
    store::{FieldType, PgStore, Store, TokenType},
};

#[derive(Debug, serde::Deserialize, Clone)]
pub struct TokenTransferRequest {
    pub token_address: String,
    pub to: String,
    pub token_type: TokenType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub magnification: Option<u8>,
    pub ip: IpNetwork,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ErrorResponse {
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_access: Option<DateTime<Utc>>,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct ExecutorResponse {
    pub status: ResponseStatus,
    pub error: Option<ErrorResponse>,
    pub data: Option<DripResponse>,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct TokenDeployRequest {
    pub name: String,
    pub symbol: String,
    pub total_supply: u128,
    pub decimals: u8,
    pub deployer_address: String,
    pub file_name: String,
    pub file_data: Vec<u8>,
    pub ip: IpNetwork,
}

#[derive(Debug, Clone)]
pub struct Executor {
    pub withdraw_queue:
        Arc<Mutex<VecDeque<(TokenTransferRequest, oneshot::Sender<ExecutorResponse>)>>>,
    pub deploy_queue: Arc<Mutex<VecDeque<(TokenDeployRequest, oneshot::Sender<ExecutorResponse>)>>>,
    config: Config,
    store: PgStore,
}

impl Executor {
    pub fn new(store: PgStore) -> Self {
        Self {
            withdraw_queue: Arc::new(Mutex::new(VecDeque::new())),
            deploy_queue: Arc::new(Mutex::new(VecDeque::new())),
            config: Config::from_env(),
            store,
        }
    }

    pub async fn process_queue(&self) {
        let withdraw_queue = self.withdraw_queue.clone();
        let deploy_queue = self.deploy_queue.clone();
        let config1 = self.config.clone();
        let config2 = self.config.clone();
        let store1 = self.store.clone();
        let store2 = self.store.clone();
        let withdraw_task = tokio::spawn(async move {
            let executor = Executor {
                withdraw_queue,
                deploy_queue: Arc::new(Mutex::new(VecDeque::new())),
                config: config1,
                store: store1,
            };
            executor.process_withdraw_queue().await;
        });

        let deploy_task = tokio::spawn(async move {
            let executor = Executor {
                withdraw_queue: Arc::new(Mutex::new(VecDeque::new())),
                deploy_queue,
                config: config2,
                store: store2,
            };
            executor.process_deploy_queue().await;
        });

        let _ = tokio::try_join!(withdraw_task, deploy_task);
    }

    pub async fn process_withdraw_queue(&self) {
        loop {
            let task = {
                let mut queue = self.withdraw_queue.lock().expect("Failed to lock queue");
                queue.pop_front()
            };

            if let Some((request, responder)) = task {
                let response = self.process_transfer(request).await;

                responder
                    .send(response)
                    .expect("Failed to send response: receiver dropped");
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    async fn check_eligibility(
        &self,
        request: &TokenTransferRequest,
    ) -> Result<(), ExecutorResponse> {
        // Check if IP has claimed within 24 hours
        let next_access_by_ip = self
            .store
            .get_next_access(
                crate::store::FieldType::ip,
                &request.ip.clone().to_string(),
                &request.token_address.clone(),
            )
            .await;
        if next_access_by_ip > Utc::now() {
            return Err(ExecutorResponse {
                status: ResponseStatus::Error,
                error: Some(ErrorResponse {
                    message: format!(
                        "You have already claimed this in the last 24hrs. You can claim again after"
                    ),
                    next_access: Some(next_access_by_ip),
                }),
                data: None,
            });
        }

        // Check if wallet has claimed within 24 hours
        let next_access_by_wallet = self
            .store
            .get_next_access(
                FieldType::to_address,
                &request.to.clone().to_string(),
                &request.token_address.clone(),
            )
            .await;
        if next_access_by_wallet > Utc::now() {
            return Err(ExecutorResponse {
                status: ResponseStatus::Error,
                error: Some(ErrorResponse {
                    message: format!(
                        "You have already claimed this in the last 24hrs. You can claim again after"
                    ),
                    next_access: Some(next_access_by_wallet),
                }),
                data: None,
            });
        }

        Ok(())
    }

    async fn validate_and_get_withdraw_limit(
        &self,
        request: &TokenTransferRequest,
    ) -> Result<u128, ExecutorResponse> {
        let token = self
            .store
            .get_token_by_address(request.token_address.clone())
            .await
            .map_err(|_| ExecutorResponse {
                status: ResponseStatus::Error,
                error: Some(ErrorResponse {
                    message: "Token not found".to_string(),
                    next_access: None,
                }),
                data: None,
            })?;

        let mut withdraw_limit = token.withdraw_limit.parse::<u128>().unwrap_or_default();
        withdraw_limit = withdraw_limit * request.magnification.unwrap_or(1 as u8) as u128;

        if withdraw_limit == 0 {
            return Err(ExecutorResponse {
                status: ResponseStatus::Error,
                error: Some(ErrorResponse {
                    message: "Token withdraw limit is 0".to_string(),
                    next_access: None,
                }),
                data: None,
            });
        }

        Ok(withdraw_limit)
    }

    async fn execute_transfer(
        &self,
        request: &TokenTransferRequest,
        withdraw_limit: u128,
    ) -> ExecutorResponse {
        let faucet = faucet::Faucet::new(
            &self.config.private_key,
            &self.config.rpc_url,
            self.store.clone(),
        );

        let result = match request.token_type {
            TokenType::ERC20 => {
                faucet
                    .send_erc_20(
                        &request.token_address,
                        &request.to,
                        withdraw_limit,
                        request.ip,
                        request.magnification.unwrap_or(1),
                    )
                    .await
            }
            TokenType::NATIVE => {
                faucet
                    .send_native_token(
                        &request.to,
                        withdraw_limit,
                        request.ip,
                        request.magnification.unwrap_or(1),
                    )
                    .await
            }
        };

        match result {
            Ok(data) => ExecutorResponse {
                status: ResponseStatus::Success,
                error: None,
                data: Some(data),
            },
            Err(e) => ExecutorResponse {
                status: ResponseStatus::Error,
                error: Some(ErrorResponse {
                    message: e.to_string(),
                    next_access: None,
                }),
                data: None,
            },
        }
    }

    pub async fn process_transfer(&self, request: TokenTransferRequest) -> ExecutorResponse {
        // Check eligibility
        if let Err(response) = self.check_eligibility(&request).await {
            return response;
        }

        // Validate token and get withdraw limit
        let withdraw_limit = match self.validate_and_get_withdraw_limit(&request).await {
            Ok(limit) => limit,
            Err(response) => return response,
        };

        // Execute the transfer
        self.execute_transfer(&request, withdraw_limit).await
    }

    pub async fn process_deploy_queue(&self) {
        loop {
            let task = {
                let mut queue = self.deploy_queue.lock().expect("Failed to lock queue");
                queue.pop_front()
            };

            if let Some((request, responder)) = task {
                let response = self.process_deploy(request).await;

                responder
                    .send(response)
                    .expect("Failed to send response: receiver dropped");
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

    async fn process_deploy(&self, request: TokenDeployRequest) -> ExecutorResponse {
        if request.file_data.is_empty()
            || request.file_name.is_empty()
            || request.name.is_empty()
            || request.symbol.is_empty()
            || request.deployer_address.is_empty()
            || request.total_supply == 0
            || request.decimals == 0
        {
            return ExecutorResponse {
                status: ResponseStatus::Error,
                error: Some(ErrorResponse {
                    message: "Invalid request".to_string(),
                    next_access: None,
                }),
                data: None,
            };
        }
        let token = self
            .store
            .get_token_from_symbol(request.symbol.clone())
            .await;

        if token.is_ok() {
            return ExecutorResponse {
                status: ResponseStatus::Error,
                error: Some(ErrorResponse {
                    message: "Token with same symbol already exists".to_string(),
                    next_access: None,
                }),
                data: None,
            };
        }

        let imgix_url = self
            .upload_to_imgix(request.file_name.clone(), request.file_data.clone())
            .await;
        if imgix_url.is_err() {
            return ExecutorResponse {
                status: ResponseStatus::Error,
                error: Some(ErrorResponse {
                    message: imgix_url.err().unwrap_or_default(),
                    next_access: None,
                }),
                data: None,
            };
        }

        let faucet = faucet::Faucet::new(
            &self.config.deploy_key,
            &self.config.rpc_url,
            self.store.clone(),
        );
        match faucet
            .deploy_erc_20(
                request.name,
                request.symbol,
                request.total_supply,
                request.decimals,
                imgix_url.unwrap_or_default(),
                request.deployer_address.clone(),
            )
            .await
        {
            Ok(result) => {
                //send 20% to deployer
                let _ = faucet
                    .send_erc_20(
                        &result,
                        &request.deployer_address,
                        request.total_supply * 20 / 100 * 10u128.pow(request.decimals as u32),
                        request.ip,
                        1,
                    )
                    .await;

                //send 80% to faucet
                let withdraw_privkey = FixedBytes::from_hex(self.config.private_key.clone())
                    .expect("Invalid private key");
                let signer =
                    PrivateKeySigner::from_bytes(&withdraw_privkey).expect("Invalid private key");
                let withdraw_address = signer.address().to_string();
                let faucet_drip =
                    request.total_supply * 80 / 100 * 10u128.pow(request.decimals as u32);
                let _ = faucet
                    .send_erc_20(&result, &withdraw_address, faucet_drip, request.ip, 1)
                    .await;
                // on successful deployment, send the contract address in tx_hash
                // this tx_hash is sent as {contract_address: 0x123} in deploy_erc20.rs response
                ExecutorResponse {
                    status: ResponseStatus::Success,
                    error: None,
                    data: Some(DripResponse {
                        tx_hash: result,
                        amount: faucet_drip.to_string(),
                        magnification: 1,
                    }),
                }
            }
            Err(e) => ExecutorResponse {
                status: ResponseStatus::Error,
                error: Some(ErrorResponse {
                    message: e.to_string(),
                    next_access: None,
                }),
                data: None,
            },
        }
    }

    async fn upload_to_imgix(
        &self,
        file_name: String,
        file_data: Vec<u8>,
    ) -> Result<String, String> {
        let url = format!(
            "https://api.imgix.com/api/v1/sources/66d6dfc6847423eb9bbc7d49/upload/monad-faucet/{}",
            file_name
        );
        let client = reqwest::Client::new();
        let response = client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.config.imgix_key))
            .body(file_data.to_vec())
            .send()
            .await;

        match response {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(format!(
                        "https://garden-finance.imgix.net/monad-faucet/{}",
                        file_name
                    ))
                } else {
                    Err(response.text().await.unwrap())
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }
}
