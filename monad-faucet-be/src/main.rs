use crate::common::setup_tracing_with_webhook;
use crate::config::Config;
use crate::handlers::{
    auth::auth, deploy_erc20::deploy_erc20, health::health_check, test_auth::test_auth,
    tokens::tokens, turnstile_captcha::verify_turnstile_captcha, user::user, withdraw::withdraw,
};
use axum::{routing::get, routing::post, Router};
use executor::Executor;
use reqwest::Method;
use store::PgStore;
use tokio::net::TcpListener;
use tower_http::cors::{AllowHeaders, CorsLayer};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
use utils::setup;

mod common;
mod config;
mod constants;
mod db;
mod executor;
mod faucet;
mod handlers;
mod store;
mod utils;

#[derive(Clone)]
pub struct AppState {
    pub store: PgStore,
    pub config: Config,
    pub executor: Executor,
}

const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

#[tokio::main]
async fn main() {
    let state = setup().await;
    match state.config.discord_webhook.clone() {
        Some(webhook) => {
            setup_tracing_with_webhook(&webhook, "Monad faucet", tracing::Level::WARN, None)
                .expect("Failed to setup tracing with webhook");
        }
        None => {
            tracing_subscriber::fmt().pretty().init();
        }
    }

    let executor_clone = state.executor.clone();

    tokio::spawn(async move {
        executor_clone.process_queue().await;
    });

    let cors = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_origin(vec![
            "https://faucet.wtf".parse().unwrap(),
            "http://localhost:5173".parse().unwrap(),
        ])
        .allow_headers(AllowHeaders::any());

    let app = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/verify-turnstile-captcha", post(verify_turnstile_captcha))
        .route("/auth", get(auth))
        .route("/user", get(user))
        .route("/test_auth", get(test_auth))
        .route("/withdraw", post(withdraw))
        .route("/deploy/erc20", post(deploy_erc20))
        .route("/tokens", get(tokens))
        .layer(axum::Extension(state.clone()))
        .layer(cors)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .with_state(state.clone());

    let addr = format!("{}:{}", state.clone().config.host, state.config.port).to_string();
    let tcp_listener = TcpListener::bind(&addr).await.unwrap();
    println!("Listening on {}", &addr);
    axum::serve(tcp_listener, app).await.unwrap();
}
