use sqlx::{Pool, Postgres};
use std::env;
use dotenv::dotenv;
use log::info;

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

  pub async fn get_user_meditation_time(&self, user_id: &u64, guild_id: &u64) -> sqlx::Result<u32> {
    let user_id = user_id.to_string();
    let guild_id = guild_id.to_string();

    let test = sqlx::query!(
      "SELECT (session_user_id, session_minutes) FROM meditations"
    ).fetch_all(&self.pool).await?;

    info!("test: {:?}", test);

    // Sum session_minutes for all sessions for the user
    let row = sqlx::query!(
      r#"
        SELECT SUM(session_minutes) AS total
        FROM meditations
        WHERE session_user_id = $1 AND session_guild_id = $2
      "#,
      user_id,
      guild_id
    )
      .fetch_one(&self.pool)
      .await?;

    Ok(row.total.unwrap_or(0) as u32)
  }

  pub async fn add_user_meditation_time(&self, user_id: &u64, guild_id: &u64, minutes: u32) -> sqlx::Result<()> {
    let user_id = user_id.to_string();
    let guild_id = guild_id.to_string();

    // Insert a new row into the meditations table
    sqlx::query!(
      r#"
        INSERT INTO meditations (session_user_id, session_minutes, session_guild_id)
        VALUES ($1, $2, $3)
      "#,
      user_id,
      minutes as i32,
      guild_id
    )
      .execute(&self.pool)
      .await?;

    Ok(())
  }

  pub async fn get_server_meditation_time(&self, guild_id: &u64) -> sqlx::Result<u32> {
    let guild_id = guild_id.to_string();

    // Sum session_minutes for all sessions for the user
    let row = sqlx::query!(
      r#"
        SELECT SUM(session_minutes) AS total
        FROM meditations
        WHERE session_guild_id = $1
      "#,
      guild_id
    )
      .fetch_one(&self.pool)
      .await?;

    Ok(row.total.unwrap_or(0) as u32)
  }
}
