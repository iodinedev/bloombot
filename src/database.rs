use crate::pagination::PageRow;
use anyhow::{Context, Result};
use chrono::Utc;
use futures::{stream::Stream, StreamExt, TryStreamExt};
use log::info;
use poise::serenity_prelude::{self as serenity, Mentionable};
use ulid::Ulid;

#[derive(Debug)]
struct Res {
  times_ago: Option<f64>,
  meditation_minutes: Option<i64>,
  meditation_count: Option<i64>,
}

#[derive(Debug)]
struct MeditationCountByDay {
  days_ago: Option<f64>,
}

pub struct DatabaseHandler {
  pool: sqlx::PgPool,
}

pub struct UserStats {
  pub all_minutes: i64,
  pub all_count: u64,
  pub timeframe_stats: TimeframeStats,
  pub streak: u64,
}

pub struct GuildStats {
  pub all_minutes: i64,
  pub all_count: u64,
  pub timeframe_stats: TimeframeStats,
}

#[derive(poise::ChoiceParameter)]
pub enum Timeframe {
  Yearly,
  Monthly,
  Weekly,
  Daily,
}

#[derive(Debug)]
pub struct TimeframeStats {
  pub sum: Option<i64>,
  pub count: Option<i64>,
}

pub struct MeditationData {
  pub id: String,
  pub user_id: serenity::UserId,
  pub meditation_minutes: i32,
  pub occurred_at: chrono::DateTime<Utc>,
}

impl PageRow for MeditationData {
  fn title(&self) -> String {
    format!("{} minutes", self.meditation_minutes)
  }

  fn body(&self) -> String {
    let now = chrono::Utc::now();

    if now - self.occurred_at < chrono::Duration::days(1) {
      return format!(
        "Date: {}\nID: `{}`",
        chrono_humanize::HumanTime::from(self.occurred_at),
        self.id
      );
    } else {
      return format!(
        "Date: `{}`\nID: `{}`",
        self.occurred_at.format("%Y-%m-%d %H:%M"),
        self.id
      );
    }
  }
}

pub struct QuoteData {
  pub quote: String,
  pub author: Option<String>,
}

impl PageRow for QuoteData {
  fn title(&self) -> String {
    self.quote.clone()
  }

  fn body(&self) -> String {
    self.author.clone().unwrap_or("Anonymous".to_string())
  }
}

pub struct SteamKeyData {
  pub steam_key: String,
  pub used: bool,
  pub reserved: Option<serenity::UserId>,
  pub guild_id: serenity::GuildId,
}

impl PageRow for SteamKeyData {
  fn title(&self) -> String {
    self.steam_key.clone()
  }

  fn body(&self) -> String {
    format!(
      "Used: {}\nReserved for: {}",
      match self.used {
        true => "Yes",
        false => "No",
      },
      match self.reserved {
        Some(reserved) => reserved.mention().to_string(),
        None => "Nobody".to_string(),
      },
    )
  }
}

pub struct CourseData {
  pub course_name: String,
  pub participant_role: serenity::RoleId,
  pub graduate_role: serenity::RoleId,
}

impl PageRow for CourseData {
  fn title(&self) -> String {
    self.course_name.clone()
  }

  fn body(&self) -> String {
    format!(
      "Participants: {}\nGraduates: {}",
      self.participant_role.mention(),
      self.graduate_role.mention()
    )
  }
}

#[derive(Debug)]
pub struct Term {
  pub id: String,
  pub term_name: String,
  pub meaning: String,
  pub usage: Option<String>,
  pub links: Option<Vec<String>>,
  pub category: Option<String>,
}

impl PageRow for Term {
  fn title(&self) -> String {
    self.term_name.clone()
  }

  fn body(&self) -> String {
    self.meaning.clone()
  }
}

#[derive(Debug, sqlx::FromRow)]
pub struct TermSearchResult {
  pub term_name: String,
  pub meaning: String,
  pub distance_score: Option<f64>,
}

pub struct StarMessage {
  pub record_id: String,
  pub starred_message_id: serenity::MessageId,
  pub board_message_id: serenity::MessageId,
  pub starred_channel_id: serenity::ChannelId,
}

impl DatabaseHandler {
  pub async fn new() -> Result<Self> {
    let database_url =
      std::env::var("DATABASE_URL").with_context(|| "Missing DATABASE_URL environment variable")?;
    let pool = sqlx::PgPool::connect(&database_url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    info!("Successfully applied migrations.");

    Ok(Self { pool })
  }

  pub async fn get_connection(&self) -> Result<sqlx::pool::PoolConnection<sqlx::Postgres>> {
    Ok(self.pool.acquire().await?)
  }

  pub async fn get_connection_with_retry(
    &self,
    max_retries: usize,
  ) -> Result<sqlx::pool::PoolConnection<sqlx::Postgres>> {
    let mut attempts = 0;

    loop {
      match self.get_connection().await {
        Ok(connection) => return Ok(connection),
        Err(e) => {
          if attempts >= max_retries {
            return Err(e);
          }

          // Check if the error is a sqlx::Error
          if let Some(sqlx_error) = e.downcast_ref::<sqlx::Error>() {
            // Now we can handle the sqlx::Error specifically
            if let sqlx::Error::Io(io_error) = sqlx_error {
              if io_error.kind() == std::io::ErrorKind::ConnectionReset {
                attempts += 1;
                // Wait for a moment before retrying
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                continue;
              }
            }
          }

          // If it's a different kind of error, we might want to return it immediately
          return Err(e);
        }
      }
    }
  }

  pub async fn start_transaction(&self) -> Result<sqlx::Transaction<'_, sqlx::Postgres>> {
    Ok(self.pool.begin().await?)
  }

  pub async fn start_transaction_with_retry(
    &self,
    max_retries: usize,
  ) -> Result<sqlx::Transaction<'_, sqlx::Postgres>> {
    let mut attempts = 0;

    loop {
      match self.start_transaction().await {
        Ok(transaction) => return Ok(transaction),
        Err(e) => {
          if attempts >= max_retries {
            return Err(e);
          }

          // Check if the error is a sqlx::Error
          if let Some(sqlx_error) = e.downcast_ref::<sqlx::Error>() {
            // Now we can handle the sqlx::Error specifically
            if let sqlx::Error::Io(io_error) = sqlx_error {
              if io_error.kind() == std::io::ErrorKind::ConnectionReset {
                attempts += 1;
                // Wait for a moment before retrying
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                continue;
              }
            }
          }

          // If it's a different kind of error, we might want to return it immediately
          return Err(e);
        }
      }
    }
  }

  pub async fn commit_transaction(
    transaction: sqlx::Transaction<'_, sqlx::Postgres>,
  ) -> Result<()> {
    transaction.commit().await?;
    Ok(())
  }

  /// This function is not technically necessary, as the transaction will be rolled back when dropped.
  /// However, for readability, it is recommended to call this function when you want to rollback a transaction.
  pub async fn rollback_transaction(
    transaction: sqlx::Transaction<'_, sqlx::Postgres>,
  ) -> Result<()> {
    transaction.rollback().await?;
    Ok(())
  }

  pub async fn add_minutes(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
    user_id: &serenity::UserId,
    minutes: i32,
  ) -> Result<()> {
    sqlx::query!(
      r#"
        INSERT INTO meditation (record_id, user_id, meditation_minutes, guild_id) VALUES ($1, $2, $3, $4)
      "#,
      Ulid::new().to_string(),
      user_id.to_string(),
      minutes,
      guild_id.to_string(),
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
  }

  pub async fn create_meditation_entry(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
    user_id: &serenity::UserId,
    minutes: i32,
    occurred_at: chrono::DateTime<Utc>,
  ) -> Result<()> {
    sqlx::query!(
      r#"
        INSERT INTO meditation (record_id, user_id, meditation_minutes, guild_id, occurred_at) VALUES ($1, $2, $3, $4, $5)
      "#,
      Ulid::new().to_string(),
      user_id.to_string(),
      minutes,
      guild_id.to_string(),
      occurred_at,
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
  }

  pub async fn get_user_meditation_entries(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
    user_id: &serenity::UserId,
  ) -> Result<Vec<MeditationData>> {
    let rows = sqlx::query!(
      r#"
        SELECT record_id, user_id, meditation_minutes, occurred_at FROM meditation WHERE user_id = $1 AND guild_id = $2 ORDER BY occurred_at DESC
      "#,
      user_id.to_string(),
      guild_id.to_string(),
    )
    .fetch_all(&mut **transaction)
    .await?;

    let meditation_entries = rows
      .into_iter()
      .map(|row| MeditationData {
        id: row.record_id,
        user_id: serenity::UserId(row.user_id.parse::<u64>().unwrap()),
        meditation_minutes: row.meditation_minutes,
        occurred_at: row.occurred_at,
      })
      .collect();

    Ok(meditation_entries)
  }

  pub async fn get_meditation_entry(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
    meditation_id: &str,
  ) -> Result<Option<MeditationData>> {
    let row = sqlx::query!(
      r#"
        SELECT record_id, user_id, meditation_minutes, occurred_at FROM meditation WHERE record_id = $1 AND guild_id = $2
      "#,
      meditation_id,
      guild_id.to_string(),
    )
    .fetch_optional(&mut **transaction)
    .await?;

    let meditation_entry = match row {
      Some(row) => Some(MeditationData {
        id: row.record_id,
        user_id: serenity::UserId(row.user_id.parse::<u64>().unwrap()),
        meditation_minutes: row.meditation_minutes,
        occurred_at: row.occurred_at,
      }),
      None => None,
    };

    Ok(meditation_entry)
  }

  pub async fn update_meditation_entry(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    meditation_id: &str,
    minutes: i32,
    occurred_at: chrono::DateTime<Utc>,
  ) -> Result<()> {
    sqlx::query!(
      r#"
        UPDATE meditation SET meditation_minutes = $1, occurred_at = $2 WHERE record_id = $3
      "#,
      minutes,
      occurred_at,
      meditation_id,
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
  }

  pub async fn delete_meditation_entry(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    meditation_id: &str,
  ) -> Result<()> {
    sqlx::query!(
      r#"
        DELETE FROM meditation WHERE record_id = $1
      "#,
      meditation_id,
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
  }

  pub async fn reset_user_meditation_entries(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
    user_id: &serenity::UserId,
  ) -> Result<()> {
    sqlx::query!(
      r#"
        DELETE FROM meditation WHERE user_id = $1 AND guild_id = $2
      "#,
      user_id.to_string(),
      guild_id.to_string(),
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
  }

  pub fn get_winner_candidates<'a>(
    conn: &'a mut sqlx::pool::PoolConnection<sqlx::Postgres>,
    start_date: chrono::DateTime<Utc>,
    end_date: chrono::DateTime<Utc>,
    guild_id: &'a serenity::GuildId,
  ) -> impl Stream<Item = Result<serenity::UserId>> + 'a {
    // All entries that are greater than 0 minutes and within the start and end date
    // We only want a user ID to show up once, so we group by user ID and sum the meditation minutes
    let rows_stream = sqlx::query!(
      r#"
        SELECT user_id FROM meditation WHERE meditation_minutes > 0 AND occurred_at >= $1 AND occurred_at <= $2 AND guild_id = $3 GROUP BY user_id ORDER BY RANDOM()
      "#,
      start_date,
      end_date,
      guild_id.to_string(),
    ).fetch(&mut **conn);

    let user_id_stream = rows_stream.map(|row| {
      let row = row?;

      let user_id = serenity::UserId(row.user_id.parse::<u64>().unwrap());

      Ok(user_id)
    });

    user_id_stream
  }

  pub async fn get_user_meditation_sum(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
    user_id: &serenity::UserId,
  ) -> Result<i64> {
    let row = sqlx::query!(
      r#"
        SELECT SUM(meditation_minutes) AS user_total FROM meditation WHERE user_id = $1 AND guild_id = $2
      "#,
      user_id.to_string(),
      guild_id.to_string(),
    )
    .fetch_one(&mut **transaction)
    .await?;

    let user_total = row.user_total.unwrap();

    Ok(user_total)
  }

  pub async fn get_user_meditation_count(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
    user_id: &serenity::UserId,
  ) -> Result<u64> {
    let row = sqlx::query!(
      r#"
        SELECT SUM(meditation_minutes) AS user_total FROM meditation WHERE user_id = $1 AND guild_id = $2
      "#,
      user_id.to_string(),
      guild_id.to_string(),
    )
    .fetch_one(&mut **transaction)
    .await?;

    let user_total = row.user_total.unwrap();

    Ok(user_total.try_into().unwrap())
  }

  pub async fn get_guild_meditation_sum(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
  ) -> Result<i64> {
    let row = sqlx::query!(
      r#"
        SELECT SUM(meditation_minutes) AS guild_total FROM meditation WHERE guild_id = $1
      "#,
      guild_id.to_string(),
    )
    .fetch_one(&mut **transaction)
    .await?;

    let guild_total = row.guild_total.unwrap();

    Ok(guild_total)
  }

  pub async fn get_guild_meditation_count(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
  ) -> Result<u64> {
    let row = sqlx::query!(
      r#"
        SELECT COUNT(record_id) AS guild_total FROM meditation WHERE guild_id = $1
      "#,
      guild_id.to_string(),
    )
    .fetch_one(&mut **transaction)
    .await?;

    let guild_total = row.guild_total.unwrap();

    Ok(guild_total.try_into().unwrap())
  }

  pub async fn get_all_quotes(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
  ) -> Result<Vec<QuoteData>> {
    let rows = sqlx::query!(
      r#"
        SELECT quote, author FROM quote WHERE guild_id = $1
      "#,
      guild_id.to_string(),
    )
    .fetch_all(&mut **transaction)
    .await?;

    let quotes = rows
      .into_iter()
      .map(|row| QuoteData {
        quote: row.quote,
        author: row.author,
      })
      .collect();

    Ok(quotes)
  }

  pub async fn get_random_motivation(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
  ) -> Result<Option<String>> {
    let row = sqlx::query!(
      r#"
        SELECT quote FROM quote WHERE guild_id = $1 ORDER BY RANDOM() LIMIT 1
      "#,
      guild_id.to_string(),
    )
    .fetch_optional(&mut **transaction)
    .await?;

    Ok(row.map(|row| row.quote))
  }

  pub async fn get_streak(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
    user_id: &serenity::UserId,
  ) -> Result<u64> {
    let mut row = sqlx::query_as!(
      MeditationCountByDay,
      r#"
      WITH cte AS (
        SELECT date_part('day', NOW() - DATE_TRUNC('day', "occurred_at")) AS "days_ago"
        FROM meditation 
        WHERE user_id = $1 AND guild_id = $2
        AND "occurred_at"::date <= NOW()::date
      )
      SELECT "days_ago"
      FROM cte
      GROUP BY "days_ago"
      ORDER BY "days_ago" ASC;
      "#,
      user_id.to_string(),
      guild_id.to_string(),
    )
    .fetch(&mut **transaction);

    let mut last = 0;
    let mut streak = 0;

    if let Some(first) = row.try_next().await? {
      let days_ago = first.days_ago.unwrap() as i32;

      if days_ago > 1 {
        return Ok(0);
      }

      last = days_ago;
      streak = 1;
    }

    while let Some(row) = row.try_next().await? {
      let days_ago = row.days_ago.unwrap() as i32;

      if days_ago != last + 1 {
        break;
      }

      last = days_ago;
      streak += 1;
    }

    Ok(streak)
  }

  pub async fn course_exists(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
    course_name: &str,
  ) -> Result<bool> {
    let row = sqlx::query!(
      r#"
        SELECT EXISTS(SELECT 1 FROM course WHERE course_name = $1 AND guild_id = $2)
      "#,
      course_name,
      guild_id.to_string(),
    )
    .fetch_one(&mut **transaction)
    .await?;

    Ok(row.exists.unwrap())
  }

  pub async fn add_course(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
    course_name: &str,
    participant_role: &serenity::Role,
    graduate_role: &serenity::Role,
  ) -> Result<()> {
    sqlx::query!(
      r#"
        INSERT INTO course (record_id, course_name, participant_role, graduate_role, guild_id) VALUES ($1, $2, $3, $4, $5)
      "#,
      Ulid::new().to_string(),
      course_name,
      participant_role.id.to_string(),
      graduate_role.id.to_string(),
      guild_id.to_string(),
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
  }

  pub async fn steam_key_exists(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
    key: &str,
  ) -> Result<bool> {
    let row = sqlx::query!(
      r#"
        SELECT EXISTS(SELECT 1 FROM steamkey WHERE steam_key = $1 AND guild_id = $2)
      "#,
      key,
      guild_id.to_string(),
    )
    .fetch_one(&mut **transaction)
    .await?;

    Ok(row.exists.unwrap())
  }

  pub async fn add_steam_key(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
    key: &str,
  ) -> Result<()> {
    sqlx::query!(
      r#"
        INSERT INTO steamkey (record_id, steam_key, guild_id, used) VALUES ($1, $2, $3, $4)
      "#,
      Ulid::new().to_string(),
      key,
      guild_id.to_string(),
      false,
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
  }

  pub async fn get_all_steam_keys(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
  ) -> Result<Vec<SteamKeyData>> {
    let rows = sqlx::query!(
      r#"
        SELECT steam_key, reserved, used, guild_id FROM steamkey WHERE guild_id = $1
      "#,
      guild_id.to_string(),
    )
    .fetch_all(&mut **transaction)
    .await?;

    let steam_keys = rows
      .into_iter()
      .map(|row| SteamKeyData {
        steam_key: row.steam_key,
        reserved: match row.reserved {
          Some(reserved) => Some(serenity::UserId(reserved.parse::<u64>().unwrap())),
          None => None,
        },
        used: row.used,
        guild_id: serenity::GuildId(row.guild_id.parse::<u64>().unwrap()),
      })
      .collect();

    Ok(steam_keys)
  }

  pub async fn add_quote(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
    quote: &str,
    author: Option<&str>,
  ) -> Result<()> {
    sqlx::query!(
      r#"
        INSERT INTO quote (record_id, quote, author, guild_id) VALUES ($1, $2, $3, $4)
      "#,
      Ulid::new().to_string(),
      quote,
      author,
      guild_id.to_string(),
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
  }

  pub async fn add_term(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    term_name: &str,
    meaning: &str,
    usage: Option<&str>,
    links: &[String],
    category: Option<&str>,
    guild_id: &serenity::GuildId,
    vector: pgvector::Vector,
  ) -> Result<()> {
    sqlx::query(
      r#"
        INSERT INTO term (record_id, term_name, meaning, usage, links, category, guild_id, embedding) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
      "#)
      .bind(Ulid::new().to_string())
      .bind(term_name)
      .bind(meaning)
      .bind(usage)
      .bind(links)
      .bind(category)
      .bind(guild_id.to_string())
      .bind(vector)
      .execute(&mut **transaction)
      .await?;

    Ok(())
  }

  pub async fn search_terms_by_vector(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
    search_vector: pgvector::Vector,
    limit: usize,
  ) -> Result<Vec<TermSearchResult>> {
    // For some reason, pgvector wants a vector to look like a string [1,2,3] instead of an array.
    // I'm sorry for what you are about to see.
    // let pgvector_format = format!("{:?}", search_vector);

    let terms: Vec<TermSearchResult> = sqlx::query_as(
      r#"
        SELECT term_name, meaning, embedding <-> $1 AS distance_score
        FROM term
        WHERE guild_id = $2
        ORDER BY distance_score ASC
        LIMIT $3
      "#,
    )
    .bind(search_vector)
    .bind(guild_id.to_string())
    .bind(limit as i64)
    .fetch_all(&mut **transaction)
    .await?;

    Ok(terms)
  }

  pub async fn get_term(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
    term_name: &str,
  ) -> Result<Option<Term>> {
    let row = sqlx::query!(
      r#"
        SELECT record_id, term_name, meaning, usage, links, category
        FROM term
        WHERE LOWER(term_name) = LOWER($1) AND guild_id = $2
      "#,
      term_name,
      guild_id.to_string(),
    )
    .fetch_optional(&mut **transaction)
    .await?;

    let term = match row {
      Some(row) => Some(Term {
        id: row.record_id,
        term_name: row.term_name,
        meaning: row.meaning,
        usage: row.usage,
        links: row.links,
        category: row.category,
      }),
      None => None,
    };

    Ok(term)
  }

  pub async fn edit_term(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    original_id: &str,
    meaning: &str,
    usage: Option<&str>,
    links: &[String],
    category: Option<&str>,
    vector: Option<pgvector::Vector>,
  ) -> Result<()> {
    sqlx::query(
      r#"
        UPDATE term
        SET meaning = $1, usage = $2, links = $3, category = $4, embedding = COALESCE($5, embedding)
        WHERE record_id = $6
      "#,
    )
    .bind(meaning)
    .bind(usage)
    .bind(links)
    .bind(category)
    .bind(vector)
    .bind(original_id)
    .execute(&mut **transaction)
    .await?;

    Ok(())
  }

  pub async fn get_all_courses(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
  ) -> Result<Vec<CourseData>> {
    let rows = sqlx::query!(
      r#"
        SELECT course_name, participant_role, graduate_role
        FROM course
        WHERE guild_id = $1
        ORDER BY course_name ASC
      "#,
      guild_id.to_string(),
    )
    .fetch_all(&mut **transaction)
    .await?;

    let courses = rows
      .into_iter()
      .map(|row| CourseData {
        course_name: row.course_name,
        participant_role: serenity::RoleId(row.participant_role.parse::<u64>().unwrap()),
        graduate_role: serenity::RoleId(row.graduate_role.parse::<u64>().unwrap()),
      })
      .collect();

    Ok(courses)
  }

  pub async fn get_course(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
    course_name: &str,
  ) -> Result<Option<CourseData>> {
    let row = sqlx::query!(
      r#"
        SELECT course_name, participant_role, graduate_role
        FROM course
        WHERE LOWER(course_name) = LOWER($1) AND guild_id = $2
      "#,
      course_name,
      guild_id.to_string(),
    )
    .fetch_optional(&mut **transaction)
    .await?;

    let course_data = match row {
      Some(row) => Some(CourseData {
        course_name: row.course_name,
        participant_role: serenity::RoleId(row.participant_role.parse::<u64>()?),
        graduate_role: serenity::RoleId(row.graduate_role.parse::<u64>()?),
      }),
      None => None,
    };

    Ok(course_data)
  }

  pub async fn get_possible_course(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
    course_name: &str,
    similarity: f32,
  ) -> Result<Option<CourseData>> {
    let row = sqlx::query!(
      r#"
        SELECT course_name, participant_role, graduate_role, SIMILARITY(LOWER(course_name), LOWER($1)) AS similarity_score
        FROM course
        WHERE SIMILARITY(LOWER(course_name), LOWER($1)) > $2 AND guild_id = $3
        ORDER BY similarity_score DESC
        LIMIT 1
      "#,
      course_name,
      similarity,
      guild_id.to_string(),
    )
    .fetch_optional(&mut **transaction)
    .await?;

    let course_data = match row {
      Some(row) => Some(CourseData {
        course_name: row.course_name,
        participant_role: serenity::RoleId(row.participant_role.parse::<u64>()?),
        graduate_role: serenity::RoleId(row.graduate_role.parse::<u64>()?),
      }),
      None => None,
    };

    Ok(course_data)
  }

  pub async fn get_possible_terms(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
    term_name: &str,
    similarity: f32,
  ) -> Result<Vec<Term>> {
    let row = sqlx::query!(
      r#"
        SELECT record_id, term_name, meaning, usage, links, category, SIMILARITY(LOWER(term_name), LOWER($1)) AS similarity_score
        FROM term
        WHERE SIMILARITY(LOWER(term_name), LOWER($1)) > $2 AND guild_id = $3
        ORDER BY similarity_score DESC
        LIMIT 1
      "#,
      term_name,
      similarity,
      guild_id.to_string(),
    )
    .fetch_all(&mut **transaction)
    .await?;

    Ok(
      row
        .into_iter()
        .map(|row| Term {
          id: row.record_id,
          term_name: row.term_name,
          meaning: row.meaning,
          usage: row.usage,
          links: row.links,
          category: row.category,
        })
        .collect(),
    )
  }

  pub async fn get_term_count(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
  ) -> Result<u64> {
    let row = sqlx::query!(
      r#"
        SELECT COUNT(record_id) AS term_count FROM term WHERE guild_id = $1
      "#,
      guild_id.to_string(),
    )
    .fetch_one(&mut **transaction)
    .await?;

    let term_count = row.term_count.unwrap();

    Ok(term_count.try_into().unwrap())
  }

  pub async fn get_all_glossary_terms(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
  ) -> Result<Vec<Term>> {
    let rows = sqlx::query!(
      r#"
        SELECT record_id, term_name, meaning
        FROM term
        WHERE guild_id = $1
        ORDER BY term_name ASC
      "#,
      guild_id.to_string(),
    )
    .fetch_all(&mut **transaction)
    .await?;

    let glossary = rows
      .into_iter()
      .map(|row| Term {
        id: row.record_id,
        term_name: row.term_name,
        meaning: row.meaning,
        usage: None,
        links: None,
        category: None,
      })
      .collect();

    Ok(glossary)
  }

  pub async fn unused_key_exists(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
  ) -> Result<bool> {
    let row = sqlx::query!(
      r#"
        SELECT EXISTS(SELECT 1 FROM steamkey WHERE used = FALSE AND reserved IS NULL AND guild_id = $1)
      "#,
      guild_id.to_string(),
    )
    .fetch_one(&mut **transaction)
    .await?;

    Ok(row.exists.unwrap())
  }

  pub async fn reserve_key(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
    user_id: &serenity::UserId,
  ) -> Result<Option<String>> {
    let row = sqlx::query!(
      r#"
        UPDATE steamkey SET reserved = $1 WHERE steam_key = (SELECT steam_key FROM steamkey WHERE used = FALSE AND reserved IS NULL AND guild_id = $2 ORDER BY RANDOM() LIMIT 1) RETURNING steam_key
      "#,
      user_id.to_string(),
      guild_id.to_string(),
    )
    .fetch_optional(&mut **transaction)
    .await?;

    Ok(row.map(|row| row.steam_key))
  }

  pub async fn mark_key_used(
    connection: &mut sqlx::pool::PoolConnection<sqlx::Postgres>,
    key: &str,
  ) -> Result<()> {
    sqlx::query!(
      r#"
        UPDATE steamkey SET used = TRUE WHERE steam_key = $1
      "#,
      key,
    )
    .execute(&mut **connection)
    .await?;

    Ok(())
  }

  pub async fn get_key_and_mark_used(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
  ) -> Result<Option<String>> {
    let row = sqlx::query!(
      r#"
        UPDATE steamkey SET used = TRUE WHERE steam_key = (SELECT steam_key FROM steamkey WHERE used = FALSE AND reserved IS NULL AND guild_id = $1 ORDER BY RANDOM() LIMIT 1) RETURNING steam_key
      "#,
      guild_id.to_string(),
    )
    .fetch_optional(&mut **transaction)
    .await?;

    Ok(row.map(|row| row.steam_key))
  }

  pub async fn get_random_quote(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
  ) -> Result<Option<QuoteData>> {
    let row = sqlx::query!(
      r#"
        SELECT quote, author FROM quote WHERE guild_id = $1 ORDER BY RANDOM() LIMIT 1
      "#,
      guild_id.to_string(),
    )
    .fetch_optional(&mut **transaction)
    .await?;

    let quote = match row {
      Some(row) => Some(QuoteData {
        quote: row.quote,
        author: row.author,
      }),
      None => None,
    };

    Ok(quote)
  }

  pub async fn remove_course(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
    course_name: &str,
  ) -> Result<()> {
    sqlx::query!(
      r#"
        DELETE FROM course WHERE course_name = $1 AND guild_id = $2
      "#,
      course_name,
      guild_id.to_string(),
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
  }

  pub async fn remove_steam_key(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
    key: &str,
  ) -> Result<()> {
    sqlx::query!(
      r#"
        DELETE FROM steamkey WHERE steam_key = $1 AND guild_id = $2
      "#,
      key,
      guild_id.to_string(),
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
  }

  pub async fn remove_quote(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
    quote: &str,
  ) -> Result<()> {
    sqlx::query!(
      r#"
        DELETE FROM quote WHERE record_id = $1 AND guild_id = $2
      "#,
      quote,
      guild_id.to_string(),
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
  }

  pub async fn term_exists(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
    term_name: &str,
  ) -> Result<bool> {
    let row = sqlx::query!(
      r#"
        SELECT EXISTS(SELECT 1 FROM term WHERE term_name = $1 AND guild_id = $2)
      "#,
      term_name,
      guild_id.to_string(),
    )
    .fetch_one(&mut **transaction)
    .await?;

    Ok(row.exists.unwrap())
  }

  pub async fn remove_term(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    term_name: &str,
    guild_id: &serenity::GuildId,
  ) -> Result<()> {
    sqlx::query!(
      r#"
        DELETE FROM term WHERE term_name = $1 AND guild_id = $2
      "#,
      term_name,
      guild_id.to_string(),
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
  }

  pub async fn get_user_stats(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
    user_id: &serenity::UserId,
    timeframe: &Timeframe,
  ) -> Result<UserStats> {
    // Get total count, total sum, and count/sum for timeframe
    let end_time = chrono::Utc::now();
    let start_time = match timeframe {
      Timeframe::Daily => end_time - chrono::Duration::days(12),
      Timeframe::Weekly => end_time - chrono::Duration::weeks(12),
      Timeframe::Monthly => end_time - chrono::Duration::days(30 * 12),
      Timeframe::Yearly => end_time - chrono::Duration::days(365 * 12),
    };

    let total_data = sqlx::query!(
      r#"
        SELECT COUNT(record_id) AS total_count, SUM(meditation_minutes) AS total_sum
        FROM meditation
        WHERE guild_id = $1 AND user_id = $2
      "#,
      guild_id.to_string(),
      user_id.to_string(),
    )
    .fetch_one(&mut **transaction)
    .await?;

    let timeframe_data = sqlx::query_as!(
      TimeframeStats,
      r#"
        SELECT COUNT(record_id) AS count, SUM(meditation_minutes) AS sum
        FROM meditation
        WHERE guild_id = $1 AND user_id = $2 AND occurred_at >= $3 AND occurred_at <= $4
      "#,
      guild_id.to_string(),
      user_id.to_string(),
      start_time,
      end_time,
    )
    .fetch_one(&mut **transaction)
    .await?;

    let user_stats = UserStats {
      all_minutes: total_data.total_sum.unwrap_or(0),
      all_count: total_data.total_count.unwrap_or(0).try_into()?,
      timeframe_stats: timeframe_data,
      streak: DatabaseHandler::get_streak(transaction, guild_id, user_id).await?,
    };

    Ok(user_stats)
  }

  pub async fn get_guild_stats(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
    timeframe: &Timeframe,
  ) -> Result<GuildStats> {
    // Get total count, total sum, and count/sum for timeframe
    let end_time = chrono::Utc::now();
    let start_time = match timeframe {
      Timeframe::Daily => end_time - chrono::Duration::days(12),
      Timeframe::Weekly => end_time - chrono::Duration::weeks(12),
      Timeframe::Monthly => end_time - chrono::Duration::days(30 * 12),
      Timeframe::Yearly => end_time - chrono::Duration::days(365 * 12),
    };

    let total_data = sqlx::query!(
      r#"
        SELECT COUNT(record_id) AS total_count, SUM(meditation_minutes) AS total_sum
        FROM meditation
        WHERE guild_id = $1
      "#,
      guild_id.to_string(),
    )
    .fetch_one(&mut **transaction)
    .await?;

    let timeframe_data = sqlx::query_as!(
      TimeframeStats,
      r#"
        SELECT COUNT(record_id) AS count, SUM(meditation_minutes) AS sum
        FROM meditation
        WHERE guild_id = $1 AND occurred_at >= $2 AND occurred_at <= $3
      "#,
      guild_id.to_string(),
      start_time,
      end_time,
    )
    .fetch_one(&mut **transaction)
    .await?;

    let guild_stats = GuildStats {
      all_minutes: total_data.total_sum.unwrap_or(0),
      all_count: total_data.total_count.unwrap_or(0).try_into()?,
      timeframe_stats: timeframe_data,
    };

    Ok(guild_stats)
  }

  pub async fn quote_exists(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
    quote_id: &str,
  ) -> Result<bool> {
    let row = sqlx::query!(
      r#"
        SELECT EXISTS(SELECT 1 FROM quote WHERE record_id = $1 AND guild_id = $2)
      "#,
      quote_id,
      guild_id.to_string(),
    )
    .fetch_one(&mut **transaction)
    .await?;

    Ok(row.exists.unwrap())
  }

  pub async fn get_user_chart_stats(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
    user_id: &serenity::UserId,
    timeframe: &Timeframe,
  ) -> Result<Vec<TimeframeStats>> {
    // Get the last 12 days, weeks, months, or years
    let rows: Vec<Res> = match timeframe {
      Timeframe::Daily => {
        sqlx::query_as!(
          Res,
          r#"WITH "daily_data" AS (
            SELECT date_part('day', NOW() - DATE_TRUNC('day', "occurred_at")) AS times_ago, meditation_minutes
            FROM meditation
            WHERE guild_id = $1 AND user_id = $2
          ) SELECT "times_ago", SUM(meditation_minutes) AS meditation_minutes, COUNT(*) AS meditation_count
          FROM "daily_data"
          WHERE "times_ago" <= 12
          GROUP BY "times_ago";"#,
          guild_id.to_string(),
          user_id.to_string(),
        ).fetch_all(&mut **transaction).await?
      },
      Timeframe::Weekly => {
        sqlx::query_as!(
          Res,
          r#"WITH "weekly_data" AS (
            SELECT floor(extract(epoch from NOW() - "occurred_at")/(60*60*24*7))::float AS "times_ago", meditation_minutes
            FROM meditation
            WHERE "guild_id" = $1 AND "user_id" = $2
        ) SELECT "times_ago", SUM(meditation_minutes) AS meditation_minutes, COUNT(*) AS meditation_count
            FROM "weekly_data"
            WHERE "times_ago" <= 12
        GROUP BY "times_ago";"#,
          guild_id.to_string(),
          user_id.to_string(),
        ).fetch_all(&mut **transaction).await?
      },
      Timeframe::Monthly => {
        sqlx::query_as!(
          Res,
          r#"WITH "monthly_data" AS (
            SELECT floor(extract(epoch from NOW() - "occurred_at")/(60*60*24*30))::float AS "times_ago", meditation_minutes
            FROM meditation
            WHERE "guild_id" = $1 AND "user_id" = $2
        ) SELECT "times_ago", SUM(meditation_minutes) AS meditation_minutes, COUNT(*) AS meditation_count
            FROM "monthly_data"
            WHERE "times_ago" <= 12
        GROUP BY "times_ago";"#,
          guild_id.to_string(),
          user_id.to_string(),
        ).fetch_all(&mut **transaction).await?
      },
      Timeframe::Yearly => {
        sqlx::query_as!(
          Res,
          r#"WITH "yearly_data" AS (
            SELECT floor(extract(epoch from NOW() - "occurred_at")/(60*60*24*365))::float AS "times_ago", meditation_minutes
            FROM meditation
            WHERE "guild_id" = $1 AND "user_id" = $2
        ) SELECT "times_ago", SUM(meditation_minutes) AS meditation_minutes, COUNT(*) AS meditation_count
            FROM "yearly_data"
            WHERE "times_ago" <= 12
        GROUP BY "times_ago";"#,
          guild_id.to_string(),
          user_id.to_string(),
        ).fetch_all(&mut **transaction).await?
      },
    };

    let stats: Vec<TimeframeStats> = (0..12)
      .map(|i| {
        let row = rows.iter().find(|row| row.times_ago.unwrap() == i as f64);

        let meditation_minutes = match row {
          Some(row) => row.meditation_minutes.unwrap_or(0),
          None => 0,
        };

        let meditation_count = match row {
          Some(row) => row.meditation_count.unwrap_or(0),
          None => 0,
        };

        TimeframeStats {
          sum: Some(meditation_minutes),
          count: meditation_count.try_into().unwrap(),
        }
      })
      .rev()
      .collect();

    Ok(stats)
  }

  pub async fn get_guild_chart_stats(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    guild_id: &serenity::GuildId,
    timeframe: &Timeframe,
  ) -> Result<Vec<TimeframeStats>> {
    // Get the last 12 days, weeks, months, or years
    let rows: Vec<Res> = match timeframe {
      Timeframe::Daily => {
        sqlx::query_as!(
          Res,
          r#"WITH "daily_data" AS (
            SELECT date_part('day', NOW() - DATE_TRUNC('day', "occurred_at")) AS times_ago, meditation_minutes
            FROM meditation
            WHERE guild_id = $1
          ) SELECT "times_ago", SUM(meditation_minutes) AS meditation_minutes, COUNT(*) AS meditation_count
          FROM "daily_data"
          WHERE "times_ago" <= 12
          GROUP BY "times_ago";"#,
          guild_id.to_string(),
        ).fetch_all(&mut **transaction).await?
      },
      Timeframe::Weekly => {
        sqlx::query_as!(
          Res,
          r#"WITH "weekly_data" AS (
            SELECT floor(extract(epoch from NOW() - "occurred_at")/(60*60*24*7))::float AS "times_ago", meditation_minutes
            FROM meditation
            WHERE "guild_id" = $1
        ) SELECT "times_ago", SUM(meditation_minutes) AS meditation_minutes, COUNT(*) AS meditation_count
            FROM "weekly_data"
            WHERE "times_ago" <= 12
        GROUP BY "times_ago";"#,
          guild_id.to_string(),
        ).fetch_all(&mut **transaction).await?
      },
      Timeframe::Monthly => {
        sqlx::query_as!(
          Res,
          r#"WITH "monthly_data" AS (
            SELECT floor(extract(epoch from NOW() - "occurred_at")/(60*60*24*30))::float AS "times_ago", meditation_minutes
            FROM meditation
            WHERE "guild_id" = $1
        ) SELECT "times_ago", SUM(meditation_minutes) AS meditation_minutes, COUNT(*) AS meditation_count
            FROM "monthly_data"
            WHERE "times_ago" <= 12
        GROUP BY "times_ago";"#,
          guild_id.to_string(),
        ).fetch_all(&mut **transaction).await?
      },
      Timeframe::Yearly => {
        sqlx::query_as!(
          Res,
          r#"WITH "yearly_data" AS (
            SELECT floor(extract(epoch from NOW() - "occurred_at")/(60*60*24*365))::float AS "times_ago", meditation_minutes
            FROM meditation
            WHERE "guild_id" = $1
        ) SELECT "times_ago", SUM(meditation_minutes) AS meditation_minutes, COUNT(*) AS meditation_count
            FROM "yearly_data"
            WHERE "times_ago" <= 12
        GROUP BY "times_ago";"#,
          guild_id.to_string(),
        ).fetch_all(&mut **transaction).await?
      },
    };

    let stats: Vec<TimeframeStats> = (0..12)
      .map(|i| {
        let row = rows.iter().find(|row| row.times_ago.unwrap() == i as f64);

        let meditation_minutes = match row {
          Some(row) => row.meditation_minutes.unwrap_or(0),
          None => 0,
        };

        let meditation_count = match row {
          Some(row) => row.meditation_count.unwrap_or(0),
          None => 0,
        };

        TimeframeStats {
          sum: Some(meditation_minutes),
          count: meditation_count.try_into().unwrap(),
        }
      })
      .rev()
      .collect();

    Ok(stats)
  }

  pub async fn get_star_message_by_message_id(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    message_id: &serenity::MessageId,
  ) -> Result<Option<StarMessage>> {
    let row = sqlx::query!(
      r#"
        SELECT record_id, starred_message_id, board_message_id, starred_channel_id
        FROM "star"
        WHERE starred_message_id = $1
      "#,
      message_id.to_string(),
    )
    .fetch_optional(&mut **transaction)
    .await?;

    let star_message = match row {
      Some(row) => Some(StarMessage {
        record_id: row.record_id,
        starred_message_id: serenity::MessageId(row.starred_message_id.parse::<u64>().unwrap()),
        board_message_id: serenity::MessageId(row.board_message_id.parse::<u64>().unwrap()),
        starred_channel_id: serenity::ChannelId(row.starred_channel_id.parse::<u64>().unwrap()),
      }),
      None => None,
    };

    Ok(star_message)
  }

  pub async fn delete_star_message(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    record_id: &str,
  ) -> Result<()> {
    sqlx::query!(
      r#"
        DELETE FROM "star" WHERE record_id = $1
      "#,
      record_id,
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
  }

  pub async fn insert_star_message(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    starred_message_id: &serenity::MessageId,
    board_message_id: &serenity::MessageId,
    starred_channel_id: &serenity::ChannelId,
  ) -> Result<()> {
    sqlx::query!(
      r#"
        INSERT INTO "star" (record_id, starred_message_id, board_message_id, starred_channel_id) VALUES ($1, $2, $3, $4)
      "#,
      Ulid::new().to_string(),
      starred_message_id.to_string(),
      board_message_id.to_string(),
      starred_channel_id.to_string(),
    )
    .execute(&mut **transaction)
    .await?;

    Ok(())
  }
}
