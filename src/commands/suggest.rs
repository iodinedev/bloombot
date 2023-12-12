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
  // Log suggestion in staff channel
  let log_embed = BloomBotEmbed::new()
    .title("New Suggestion")
    .description(&suggestion)
    .author(|f| f.name(ctx.author().tag()).icon_url(ctx.author().face()))
    .to_owned();

  let log_channel = serenity::ChannelId(CHANNELS.logs);

  log_channel
    .send_message(ctx, |f| f.set_embed(log_embed))
    .await?;

  // Post suggestion and reactions
  let channel_id = serenity::ChannelId(CHANNELS.suggestion);

  let suggestion_message = channel_id
    .send_message(ctx, |f| {
      f.embed(|e| {
        BloomBotEmbed::from(e)
          .description(&suggestion)
      })
    })
    .await?;

  suggestion_message.react(ctx, '✅').await?;
  suggestion_message.react(ctx, '❌').await?;

  // Start thread for suggestion
  channel_id
    .create_public_thread(ctx, suggestion_message.id, |f| {
      f.name(format!(
          "Discussion: {}...",
          suggestion.chars().take(80).collect::<String>()
        ))
        .auto_archive_duration(1440)
        .kind(serenity::ChannelType::PublicThread)
    })
    .await?;

  ctx
    .send(|f| {
      f.content(format!(
        "Your suggestion has been added to <#{}>.",
        channel_id
      ))
      .ephemeral(true)
    })
    .await?;

  Ok(())
}
