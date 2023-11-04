use crate::commands::{commit_and_say, MessageType};
use crate::database::DatabaseHandler;
use crate::Context;
use anyhow::Result;

/// Removes a quote from the database.
#[poise::command(
  slash_command,
  required_permissions = "MANAGE_ROLES",
  rename = "removequote",
  guild_only
)]
pub async fn remove_quote(
  ctx: Context<'_>,
  #[description = "The quote ID to remove"] id: String,
) -> Result<()> {
  let data = ctx.data();

  // We unwrap here, because we know that the command is guild-only.
  let guild_id = ctx.guild_id().unwrap();

  let mut transaction = data.db.start_transaction_with_retry(5).await?;
  if !DatabaseHandler::quote_exists(&mut transaction, &guild_id, id.as_str()).await? {
    ctx
      .send(|f| f.content(":x: Quote does not exist.").ephemeral(true))
      .await?;
    return Ok(());
  }

  DatabaseHandler::remove_quote(&mut transaction, &guild_id, id.as_str()).await?;

  commit_and_say(
    ctx,
    transaction,
    MessageType::TextOnly(format!(":white_check_mark: Quote has been removed.")),
    true,
  )
  .await?;

  Ok(())
}
