use axum::{http::Method, routing::get, Router};
use tokio::net::TcpListener;
use tower_http::cors::{AllowHeaders, Any, CorsLayer};

use crate::handlers::health::check_status;

pub mod handlers;

#[tokio::main]
async fn main() {
    const HOST: &str = "0.0.0.0";
    const PORT: &str = "6969";

    let cors = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_origin(Any)
        .allow_headers(AllowHeaders::any());

    let app = Router::new()
        .route("/health", get(check_status))
        .layer(cors);

    let addr = format!("{}:{}", HOST, PORT);
    let tcp_listener = TcpListener::bind(&addr)
        .await
        .expect("Failed to bind address");

    println!("Listening on {}", &addr);
    axum::serve(tcp_listener, app)
        .await
        .expect("Failed to serve axum server");
}
