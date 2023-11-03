use crate::commands::{commit_and_say, MessageType};
use crate::database::DatabaseHandler;
use crate::Context;
use anyhow::Result;

/// Adds a Playne key to the database.
#[poise::command(
  slash_command,
  required_permissions = "ADMINISTRATOR",
  rename = "addkey",
  guild_only
)]
pub async fn add_key(
  ctx: Context<'_>,
  #[description = "The Playne key to add"] key: String,
) -> Result<()> {
  let data = ctx.data();

  // We unwrap here, because we know that the command is guild-only.
  let guild_id = ctx.guild_id().unwrap();

  let mut transaction = data.db.start_transaction_with_retry(5).await?;
  if DatabaseHandler::steam_key_exists(&mut transaction, &guild_id, key.as_str()).await? {
    ctx
      .send(|f| f.content(":x: Key already exists.").ephemeral(true))
      .await?;
    return Ok(());
  }

  DatabaseHandler::add_steam_key(&mut transaction, &guild_id, key.as_str()).await?;

  commit_and_say(
    ctx,
    transaction,
    MessageType::TextOnly(":white_check_mark: Key has been added.".to_string()),
    true,
  )
  .await?;

  Ok(())
}
