use crate::{
    config::Config, constants::faucet, db, executor::Executor, handlers::middleware::AuthUser,
    store::PgStore, AppState,
};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};

use tracing::error;

pub async fn magnify_faucet_drip(
    orderbook_url: String,
    user: AuthUser,
    wallet_address: String,
) -> u8 {
    let is_github_authenticated = user.is_github_authenticated;

    let is_garden_user = match response.status {
        Status::Ok => response.result > 0,
        Status::Error => false,
    };

    match (is_github_authenticated) {
        (true) => faucet::MAGNIFICATION_GITHUB_AUTH, 
        (false) => faucet::MAGNIFICATION_NO_AUTH,
    }
}

pub async fn setup() -> AppState {
    dotenv().ok();
    let config = Config::from_env();

    let db_pool = db::init_db(&config.db_url)
        .await
        .expect("Failed to connect to DB");

    let store = PgStore::new(db_pool);
    let executor = Executor::new(store.clone());
    let state = AppState {
        store,
        config: config.clone(),
        executor,
    };
    state
}

#[cfg(test)]
mod tests {
    use httpmock::{Method::GET, MockServer};

    use crate::{
        constants::faucet,
        handlers::middleware::AuthUser,
        utils::{magnify_faucet_drip, OrderResponse, Status},
    };

    #[tokio::test]
    async fn test_magnify_faucet_drip() {
        let orderbook_server = MockServer::start();

        let wallet = "0x123";

        let combinations = vec![
            (
                true,
                true,
                faucet::MAGNIFICATION_GITHUB_AUTH + faucet::MAGNIFICATION_GARDEN_USER,
            ),
            (true, false, faucet::MAGNIFICATION_GITHUB_AUTH),
            (false, true, faucet::MAGNIFICATION_GARDEN_USER),
            (false, false, faucet::MAGNIFICATION_NO_AUTH),
        ];

        for (github_auth, garden_user, expected) in combinations {
            let mut response_mock = orderbook_server.mock(|when, then| {
                when.method(GET).path(format!("/user/{}/count", wallet));
                then.status(200).json_body_obj(&OrderResponse {
                    status: Status::Ok,
                    result: if garden_user { 1 } else { 0 },
                });
            });

            let user = AuthUser {
                is_github_authenticated: github_auth,
                user_id: "1".to_string(),
            };

            let maginification =
                magnify_faucet_drip(orderbook_server.url(""), user, wallet.to_string()).await;

            assert_eq!(expected, maginification);

            response_mock.assert_hits(1);
            response_mock.delete();
        }
    }
}
