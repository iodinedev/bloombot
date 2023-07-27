use sqlx::{Pool, Postgres};
use std::env;
use dotenv::dotenv;

pub struct Database {
  pool: Pool<Postgres>,
}

impl Database {
  pub async fn new() -> sqlx::Result<Self> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")
      .expect("DATABASE_URL is not set in .env file");
    let pool = Pool::connect(&database_url).await?;
    
    Ok(Self { pool })
  }

  pub async fn get_conn(&self) -> sqlx::Result<sqlx::pool::PoolConnection<Postgres>> {
    self.pool.acquire().await
  }
}
