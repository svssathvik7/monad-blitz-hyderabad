use sqlx::{prelude::FromRow, types::chrono::Utc, PgPool};

pub struct PgStore{
    pub pool: PgPool
}

#[derive(FromRow)]
pub struct Transfer{
    pub address: String,
    pub cid: String
}

impl PgStore{
    pub async fn new(db_url: &str) -> Self{
        let pool = PgPool::connect(db_url).await.unwrap();
        sqlx::query("CREATE TABLE IF NOT EXISTS transfers (address TEXT, cid TEXT, created_at TIMEZONETZ, updated_at TIMEZONETZ").execute(&pool).await.unwrap();
        Self{
            pool
        }
    }

    pub async fn insert(&self, data: Transfer) -> (){
        sqlx::query("INSERT INTO transfers (address, cid, created_at, updated_at) VALUES($1, $2, $3, $4)").bind(data.address.clone()).bind(data.cid.clone()).bind(Utc::now()).bind(Utc::now()).execute(&self.pool).await.unwrap();

        ()
    }

    pub async fn get_transfers(&self, address: String) -> Vec<Transfer>{
        let transfers = sqlx::query_as("SELECT address, cid FROM transfers WHERE address = $1").bind(address).fetch_all(&self.pool).await.unwrap();

        transfers
    }
}