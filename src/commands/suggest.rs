use crate::config::{BloomBotEmbed, CHANNELS};
use crate::Context;
use anyhow::Result;
use poise::serenity_prelude as serenity;

/// Add a suggestion to the suggestions channel
#[poise::command(slash_command, member_cooldown = 3600, guild_only)]
pub async fn suggest(
  ctx: Context<'_>,
  #[description = "The suggestion to add"] suggestion: String,
) -> Result<()> {
  let channel_id = serenity::ChannelId(CHANNELS.suggestion);

  let suggestion_message = channel_id
    .send_message(ctx, |f| {
      f.embed(|e| {
        BloomBotEmbed::from(e)
          .description(suggestion)
      })
    })
    .await?;

  suggestion_message.react(ctx, '✅').await?;
  suggestion_message.react(ctx, '❌').await?;

  // Log in staff channel
  let log_channel = serenity::ChannelId(CHANNELS.logs);

  let suggestion_log = log_channel
    .send_message(ctx, |f| {
      f.embed(|e| {
        BloomBotEmbed::from(e)
          .title("New Suggestion")
          .description(suggestion)
          .author(|f| f.name(ctx.author().tag()).icon_url(ctx.author().face()))
          .footer(|f| f.text(format!("User ID: {}", ctx.author().id)))
      })
    })
    .await?;

  // Start thread for suggestion
  channel_id
    .create_public_thread(ctx, suggestion_message.id, |f| {
      f.name("Suggestion Discussion")
        .auto_archive_duration(1440)
        .kind(serenity::ChannelType::PublicThread)
    })
    .await?;

  ctx
    .say(format!(
      "Your suggestion has been added to <#{}>.",
      channel_id
    ))
    .ephemeral(true)
    .await?;

  Ok(())
}
