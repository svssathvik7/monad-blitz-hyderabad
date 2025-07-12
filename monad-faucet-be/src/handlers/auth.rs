use std::time::Duration;

use crate::{executor::ErrorResponse, store::Store};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Error, Json,
};
use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

use crate::{handlers::response::Response, store::User, AppState};

use super::response::res_err;

#[derive(serde::Deserialize)]
pub struct AuthGithubQuery {
    pub code: String,
}

#[derive(serde::Serialize)]
pub struct AuthGithubResponse {
    pub token: String,
}

pub async fn auth(
    State(state): State<AppState>,
    Query(query): Query<AuthGithubQuery>,
) -> Result<Json<Response<AuthGithubResponse>>, (StatusCode, Json<Response<ErrorResponse>>)> {
    let code = query.code;
    if code.is_empty() {
        return Err((StatusCode::BAD_REQUEST, res_err("No code provided")));
    }

    let access_token = get_access_token_from_code(
        &state.config.github_client_id,
        &state.config.github_client_secret,
        &code,
        &state.config.github_redirect_uri,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            res_err(e.to_string().as_str()),
        )
    })?;
    let user = get_user_info(&access_token).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            res_err(e.to_string().as_str()),
        )
    })?;
    if let Err(e) = state.store.create_user(user.clone()).await {
        if let sqlx::Error::Database(db_err) = &e {
            if db_err.code().map(|code| code == "23505").unwrap_or(false) {
                let existing_user = state
                    .store
                    .get_user_by_github_id(user.github_id.clone())
                    .await
                    .map_err(|e| {
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            res_err(e.to_string().as_str()),
                        )
                    })?;
                // Use existing user details
                let token =
                    generate_jwt(&existing_user.id, &state.config.jwt_secret).map_err(|e| {
                        error!("Error generating jwt {}", e);
                        (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            res_err("Failed to authenticate user!"),
                        )
                    })?;
                return Ok(Response::ok(AuthGithubResponse { token }));
            }
        } else {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                res_err(e.to_string().as_str()),
            ));
        }
    }

    let token = generate_jwt(&user.id, &state.config.jwt_secret).map_err(|e| {
        error!("Error generating jwt {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            res_err("Failed to authenticate user!"),
        )
    })?;

    Ok(Response::ok(AuthGithubResponse { token }))
}

async fn get_access_token_from_code(
    client_id: &str,
    client_secret: &str,
    code: &str,
    redirect_uri: &str,
) -> Result<String, Error> {
    let client = Client::new();
    println!("Code: {}", code);
    let query_params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("code", code),
        ("redirect_uri", redirect_uri),
    ];
    let response = client
        .post("https://github.com/login/oauth/access_token")
        .query(&query_params)
        .header("Accept", "application/json")
        .send()
        .await
        .expect("Failed to send request");

    if response.status().is_success() {
        let response_body = response.text().await.expect("Failed to get response body");
        let response_json: serde_json::Value =
            serde_json::from_str(&response_body).expect("Failed to parse response body");
        println!("Response: {:?}", response_json);
        let access_token = match response_json["access_token"].as_str() {
            Some(token) => token.to_string(),
            None => {
                return Err(Error::new("Failed to get access token"));
            }
        };
        Ok(access_token)
    } else {
        Err(Error::new("Failed to get access token"))
    }
}

async fn get_user_info(access_token: &str) -> Result<User, Error> {
    let client = Client::new();
    let response = client
        .get("https://api.github.com/user")
        .header(header::ACCEPT, "application/json")
        .header(header::AUTHORIZATION, format!("Bearer {}", access_token))
        .header(header::USER_AGENT, "axum-github-auth")
        .header(header::CONTENT_TYPE, "application/json")
        .send()
        .await
        .expect("Failed to send request");

    if response.status().is_success() {
        let response_body = response.text().await.expect("Failed to get response body");
        let response_json: serde_json::Value =
            serde_json::from_str(&response_body).expect("Failed to parse response body");

        let user = User {
            id: Uuid::new_v4().to_string(),
            username: response_json["login"].as_str().unwrap().to_string(),
            avatar_url: response_json["avatar_url"].as_str().unwrap().to_string(),
            email: Some(
                response_json["email"]
                    .as_str()
                    .map_or_else(|| "".to_string(), |s| s.to_string()),
            ),
            github_id: response_json["id"].as_i64().unwrap_or_default().to_string(),
            access_token: access_token.to_string(),
        };
        Ok(user)
    } else {
        Err(Error::new(
            format!(
                "Failed to get user info: {}",
                response.text().await.unwrap()
            )
            .as_str(),
        ))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // User ID (or other unique identifier)
    pub exp: usize,  // Expiration timestamp
}

fn generate_jwt(user_id: &str, secret_key: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now() + Duration::from_secs(24 * 60 * 60); // 1 day

    let claims = Claims {
        sub: user_id.to_string(),
        exp: expiration.timestamp() as usize,
    };

    // Encode the JWT token
    let header = Header::new(Algorithm::HS256);
    let token = encode(
        &header,
        &claims,
        &EncodingKey::from_secret(secret_key.as_ref()),
    )?;

    Ok(token)
}

pub fn validate_and_decode_jwt(
    token: &str,
    secret_key: &str,
) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret_key.as_ref()),
        &Validation::default(),
    )
    .map_err(|e| e)?;
    if token.claims.exp < Utc::now().timestamp() as usize {
        return Err(jsonwebtoken::errors::Error::from(
            jsonwebtoken::errors::ErrorKind::ExpiredSignature,
        ));
    }
    Ok(token.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_user_info() {
        // give a valid access token
        let user = get_user_info("").await.unwrap();
        println!("{:?}", user);
        // give a valid username
        assert_eq!(user.username, "");
    }

    #[tokio::test]
    async fn test_generate_jwt() {
        let id = "12345678900987654321";
        let secret = "qV2zScNYyR6bB6";
        let token = generate_jwt(id, secret).unwrap();
        println!("{}", token);
        let claims = validate_and_decode_jwt(&token, secret).unwrap();
        assert_eq!(claims.sub, id);
    }
}
