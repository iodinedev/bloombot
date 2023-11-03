use crate::database::DatabaseHandler;
use crate::Context;
use anyhow::Result;

/// Selects an unused Steam key from the database, returning it and marking it as used.
#[poise::command(
  slash_command,
  required_permissions = "ADMINISTRATOR",
  rename = "usekey"
)]
pub async fn use_key(ctx: Context<'_>) -> Result<()> {
  ctx.defer_ephemeral().await?;

  let data = ctx.data();

  let guild_id = ctx.guild_id().unwrap();

  let mut transaction = data.db.start_transaction_with_retry(5).await?;

  if !DatabaseHandler::unused_key_exists(&mut transaction, &guild_id).await? {
    ctx
      .send(|f| f.content(":x: No unused keys found.").ephemeral(true))
      .await?;
    return Ok(());
  }

  let key = DatabaseHandler::get_key_and_mark_used(&mut transaction, &guild_id).await?;
  let key = key.unwrap();

  ctx
    .send(|f| {
      f.content(format!(":white_check_mark: {}", key))
        .ephemeral(true)
    })
    .await?;

  Ok(())
}
