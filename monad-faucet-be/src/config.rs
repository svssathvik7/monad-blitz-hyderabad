use dotenv::var;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub db_url: String,
    pub port: u16,
    pub host: String,
    pub captcha_secret_key: String,
    pub github_client_id: String,
    pub github_client_secret: String,
    pub github_redirect_uri: String,
    pub jwt_secret: String,
    pub private_key: String,
    pub rpc_url: String,
    pub deploy_key: String,
    pub imgix_key: String,
    pub orderbook_url: String,
    pub discord_webhook: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        let db_url = var("DATABASE_URL").expect("DATABASE_URL must be set");
        let port = var("PORT").unwrap_or("6969".to_string()).parse().unwrap();
        let host = var("HOST").unwrap_or("0.0.0.0".to_string());
        let captcha_secret_key = var("CAPTCHA_SECRET_KEY").expect("CAPTCHA_SECRET_KEY must be set");
        let github_client_id = var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID must be set");
        let github_client_secret =
            var("GITHUB_CLIENT_SECRET").expect("GITHUB_CLIENT_SECRET must be set");
        let github_redirect_uri =
            var("GITHUB_REDIRECT_URI").expect("GITHUB_REDIRECT_URI must be set");
        let jwt_secret = var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY must be set");
        let private_key = var("PRIVATE_KEY").expect("PRIVATE_KEY must be set");
        let rpc_url = var("RPC_URL").expect("RPC_URL must be set");
        let deploy_key = var("DEPLOY_KEY").expect("DEPLOY_KEY must be set");
        let imgix_key = var("IMGIX_KEY").expect("IMGIX_KEY must be set");
        let orderbook_url = var("ORDERBOOK_URL").expect("ORDERBOOK_URL must be set");
        let discord_webhook = var("DISCORD_WEBHOOK").ok();

        Self {
            db_url,
            port,
            host,
            captcha_secret_key,
            github_client_id,
            github_client_secret,
            github_redirect_uri,
            jwt_secret,
            private_key,
            rpc_url,
            deploy_key,
            imgix_key,
            orderbook_url,
            discord_webhook,
        }
    }
}
