use crate::commands::{commit_and_say, MessageType};
use crate::config::{BloomBotEmbed, CHANNELS};
use crate::database::DatabaseHandler;
use crate::Context;
use anyhow::Result;
use poise::serenity_prelude as serenity;

/// Removes one of your meditation entries
#[poise::command(
  slash_command,
  rename = "remove",
  guild_only
)]
pub async fn remove_entry(
  ctx: Context<'_>,
  #[description = "The ID of the entry to remove"] id: String,
) -> Result<()> {
  let data = ctx.data();
  let guild_id = ctx.guild_id().unwrap();

  let mut transaction = data.db.start_transaction_with_retry(5).await?;

  let entry =
    match DatabaseHandler::get_meditation_entry(&mut transaction, &guild_id, id.as_str()).await? {
      Some(entry) => entry,
      None => {
        ctx.say(":x: No entry found with that ID.").await?;
        return Ok(());
      }
    };

  if entry.user_id != ctx.author().id {
    ctx.say(":x: You can only remove your own entries.").await?;
    return Ok(());
  }

  DatabaseHandler::delete_meditation_entry(&mut transaction, id.as_str()).await?;

  commit_and_say(
    ctx,
    transaction,
    MessageType::TextOnly(format!(":white_check_mark: Entry has been removed.")),
    true,
  )
  .await?;

  let log_embed = BloomBotEmbed::new()
    .title("Meditation Entry Removed")
    .description(format!(
      "**User**: {}\n**ID**: {}\n**Date**: {}\n**Time**: {} minutes",
      ctx.author(),
      entry.id,
      entry.occurred_at.format("%B %d, %Y"),
      entry.meditation_minutes
    ))
    .footer(|f| {
      f.icon_url(ctx.author().avatar_url().unwrap_or_default())
        .text(format!("Removed by {}", ctx.author()))
    })
    .to_owned();

  let log_channel = serenity::ChannelId(CHANNELS.bloomlogs);

  log_channel
    .send_message(ctx, |f| f.set_embed(log_embed))
    .await?;

  Ok(())
}
