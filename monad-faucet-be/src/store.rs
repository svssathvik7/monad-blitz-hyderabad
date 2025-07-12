use chrono::{DateTime, Duration, Utc};
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

#[async_trait::async_trait]
pub trait Store {
    async fn get_user_by_id(&self, id: String) -> Result<User, sqlx::Error>;
    async fn create_user(&self, user: User) -> Result<User, sqlx::Error>;
    async fn get_user_by_github_id(&self, github_id: String) -> Result<User, sqlx::Error>;

    async fn create_token_transfer(
        &self,
        token_transfer: TokenTransfer,
    ) -> Result<TokenTransfer, sqlx::Error>;

    async fn create_token_entry(&self, token: Token) -> Result<Token, sqlx::Error>;
    async fn get_all_tokens(&self) -> Result<Vec<Token>, sqlx::Error>;
    async fn get_token_by_address(&self, address: String) -> Result<Token, sqlx::Error>;
    async fn get_token_from_symbol(&self, symbol: String) -> Result<Token, sqlx::Error>;
    async fn get_next_access(
        &self,
        field_name: FieldType,
        field_value: &str,
        token_address: &str,
    ) -> DateTime<Utc>;
}

#[derive(Clone, Debug)]
pub struct PgStore {
    db: Pool<Postgres>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub github_id: String,
    pub access_token: String,
    pub avatar_url: String,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "token_type", rename_all = "UPPERCASE")]
pub enum TokenType {
    ERC20,
    NATIVE,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTransfer {
    pub token_address: String,
    pub token_type: TokenType,
    pub tx_hash: String,
    pub from_address: String,
    pub to_address: String,
    pub amount: String,
    pub chain_id: i32,
    pub ip: IpNetwork,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub created_by: String,
    pub token_type: TokenType,
    pub address: String,
    pub logo_url: String,
    pub chain_id: i32,
    pub symbol: String,
    pub name: String,
    pub decimals: i32,
    pub withdraw_limit: String,
}

#[derive(Debug, PartialEq)]
pub enum FieldType {
    ip,
    to_address,
}
impl PgStore {
    pub fn new(db: Pool<Postgres>) -> Self {
        Self { db }
    }
}

#[async_trait::async_trait]
impl Store for PgStore {
    async fn get_user_by_id(&self, id: String) -> Result<User, sqlx::Error> {
        let user = sqlx::query!("SELECT * FROM users WHERE id = $1", id)
            .fetch_one(&self.db)
            .await?;
        Ok(User {
            id: user.id,
            username: user.username,
            github_id: user.github_id,
            access_token: user.access_token,
            avatar_url: user.avatar_url,
            email: user.email,
        })
    }

    async fn get_next_access(
        &self,
        field_name: FieldType, // "ip" or "to_address"
        field_value: &str,     // The actual value of ip or to_address
        token_address: &str,
    ) -> DateTime<Utc> {
        let query = if field_name == FieldType::ip {
            format!(
                "SELECT (updated_at::timestamptz + interval '1 day') as next_access
                FROM token_transfers 
                WHERE {:?} = $1::inet 
                  AND token_address = $2 
                ORDER BY updated_at DESC 
                LIMIT 1;",
                field_name
            )
        } else {
            format!(
                "SELECT (updated_at::timestamptz + interval '1 day') as next_access
                FROM token_transfers 
                WHERE {:?} = $1 
                  AND token_address = $2 
                ORDER BY updated_at DESC 
                LIMIT 1;",
                field_name
            )
        };

        match sqlx::query_scalar::<_, Option<DateTime<Utc>>>(&query)
            .bind(field_value)
            .bind(token_address)
            .fetch_optional(&self.db)
            .await
        {
            Ok(Some(next_access)) => next_access.unwrap_or(Utc::now() + Duration::days(1)),
            Ok(None) => Utc::now() - Duration::days(1),
            Err(e) => {
                eprintln!("Error while getting next access: {:?}", e);
                Utc::now() + Duration::days(1)
            }
        }
    }

    async fn create_user(&self, user: User) -> Result<User, sqlx::Error> {
        let record= sqlx::query!(
            "INSERT INTO users (id, username, github_id, access_token, avatar_url, email) VALUES ($1, $2, $3, $4, $5, $6) RETURNING *",
            user.id,
            user.username,
            user.github_id,
            user.access_token,
            user.avatar_url,
            user.email
        )
        .fetch_one(&self.db)
        .await?;
        Ok(User {
            id: record.id,
            username: record.username,
            github_id: record.github_id,
            access_token: record.access_token,
            avatar_url: record.avatar_url,
            email: record.email,
        })
    }

    async fn get_user_by_github_id(&self, github_id: String) -> Result<User, sqlx::Error> {
        let user = sqlx::query!("SELECT * FROM users WHERE github_id = $1", github_id)
            .fetch_one(&self.db)
            .await?;

        Ok(User {
            id: user.id,
            username: user.username,
            github_id: user.github_id,
            access_token: user.access_token,
            avatar_url: user.avatar_url,
            email: user.email,
        })
    }

    async fn create_token_transfer(
        &self,
        token_transfer: TokenTransfer,
    ) -> Result<TokenTransfer, sqlx::Error> {
        let record = sqlx::query!(
            r#"INSERT INTO token_transfers (token_address, token_type, tx_hash, from_address, to_address, amount, chain_id, ip) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8) 
            RETURNING token_address, token_type as "token_type!: TokenType", tx_hash, from_address, to_address, amount, chain_id, ip"#,
            token_transfer.token_address,
            token_transfer.token_type as _,
            token_transfer.tx_hash,
            token_transfer.from_address,
            token_transfer.to_address,
            token_transfer.amount,
            token_transfer.chain_id,
            token_transfer.ip
        )
        .fetch_one(&self.db)
        .await?;

        Ok(TokenTransfer {
            token_address: record.token_address,
            token_type: record.token_type,
            tx_hash: record.tx_hash,
            from_address: record.from_address,
            to_address: record.to_address,
            amount: record.amount,
            chain_id: record.chain_id,
            ip: record
                .ip
                .unwrap_or(IpNetwork::V4("0.0.0.0".parse().unwrap())),
        })
    }

    async fn create_token_entry(&self, token: Token) -> Result<Token, sqlx::Error> {
        let record = sqlx::query!(
            r#"INSERT INTO tokens (created_by, token_type, address, logo_url, chain_id, symbol, name, decimals, withdraw_limit) 
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) 
            RETURNING created_by, token_type as "token_type!: TokenType", address, logo_url, chain_id, symbol, name, decimals, withdraw_limit"#,
            token.created_by,
            token.token_type as _,
            token.address,
            token.logo_url,
            token.chain_id,
            token.symbol,
            token.name,
            token.decimals,
            token.withdraw_limit
        )
        .fetch_one(&self.db)
        .await?;

        Ok(Token {
            created_by: record.created_by,
            token_type: record.token_type,
            address: record.address,
            logo_url: record.logo_url,
            chain_id: record.chain_id,
            symbol: record.symbol,
            name: record.name,
            decimals: record.decimals,
            withdraw_limit: record.withdraw_limit,
        })
    }

    async fn get_token_by_address(&self, address: String) -> Result<Token, sqlx::Error> {
        let record = sqlx::query!(
            r#"SELECT created_by, token_type as "token_type!: TokenType", address, logo_url, chain_id, symbol, name, decimals, withdraw_limit 
            FROM tokens WHERE address = $1"#,
            address
        )
        .fetch_one(&self.db)
        .await?;

        Ok(Token {
            created_by: record.created_by,
            token_type: record.token_type,
            address: record.address,
            logo_url: record.logo_url,
            chain_id: record.chain_id,
            symbol: record.symbol,
            name: record.name,
            decimals: record.decimals,
            withdraw_limit: record.withdraw_limit,
        })
    }

    async fn get_token_from_symbol(&self, symbol: String) -> Result<Token, sqlx::Error> {
        let record = sqlx::query!(
            r#"SELECT created_by, token_type as "token_type!: TokenType", address, logo_url, chain_id, symbol, name, decimals, withdraw_limit 
            FROM tokens WHERE symbol = $1"#,
            symbol
        )
        .fetch_one(&self.db)
        .await?;

        Ok(Token {
            created_by: record.created_by,
            token_type: record.token_type,
            address: record.address,
            logo_url: record.logo_url,
            chain_id: record.chain_id,
            symbol: record.symbol,
            name: record.name,
            decimals: record.decimals,
            withdraw_limit: record.withdraw_limit,
        })
    }

    async fn get_all_tokens(&self) -> Result<Vec<Token>, sqlx::Error> {
        let records = sqlx::query!(
            r#"SELECT created_by, token_type as "token_type!: TokenType", address, logo_url, chain_id, symbol, name, decimals, withdraw_limit 
            FROM tokens"#
        )
        .fetch_all(&self.db)
        .await?;

        Ok(records
            .into_iter()
            .map(|record| Token {
                created_by: record.created_by,
                token_type: record.token_type,
                address: record.address,
                logo_url: record.logo_url,
                chain_id: record.chain_id,
                symbol: record.symbol,
                name: record.name,
                decimals: record.decimals,
                withdraw_limit: record.withdraw_limit,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    async fn setup_test_db() -> Pool<Postgres> {
        dotenv::dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        db::init_db(&database_url)
            .await
            .expect("Failed to connect to DB")
    }

    #[tokio::test]
    async fn test_create_and_get_user() {
        let pool = setup_test_db().await;
        let store = PgStore::new(pool);

        let test_user = User {
            id: "test_id".to_string(),
            username: "test_user".to_string(),
            avatar_url: "https://example.com/avatar.png".to_string(),
            github_id: "test_github_id".to_string(),
            access_token: "test_access_token".to_string(),
            email: Some("test@example.com".to_string()),
        };

        // Test create_users
        if let Err(e) = store.create_user(test_user.clone()).await {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.code().map(|code| code == "23505").unwrap_or(false) {
                    println!("User already exists, continuing...");
                } else {
                    panic!("Unexpected database error: {:?}", e);
                }
            } else {
                panic!("Unexpected error: {:?}", e);
            }
        }

        // Test get_user_by_id
        let fetched_user = store.get_user_by_id(test_user.id.clone()).await.unwrap();
        assert_eq!(fetched_user.username, test_user.username);
        assert_eq!(fetched_user.avatar_url, test_user.avatar_url);

        // Cleanup
        sqlx::query!("DELETE FROM users WHERE id = $1", fetched_user.id)
            .execute(&store.db)
            .await
            .unwrap();
    }
}
