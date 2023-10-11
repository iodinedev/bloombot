use crate::config::{BloomBotEmbed, CHANNELS};
use crate::Context;
use anyhow::Result;
use poise::serenity_prelude as serenity;

/// Deletes a message.
#[poise::command(slash_command, required_permissions = "MANAGE_MESSAGES", hide_in_help)]
pub async fn erase(
  ctx: Context<'_>,
  #[description = "The message to delete"] message: serenity::Message,
  #[description = "The reason for deleting the message"] reason: Option<String>,
) -> Result<()> {
  ctx.defer_ephemeral().await?;

  let channel_id = message.channel_id;
  let message_id = message.id;

  ctx
    .http()
    .delete_message(channel_id.into(), message_id.into())
    .await?;

  let mod_confirmation = ctx
    .send(|f| {
      f.content(format!(
        ":white_check_mark: Message deleted. Sending the reason in DMs..."
      ))
      .ephemeral(true)
    })
    .await?;

  let reason = reason.unwrap_or("No reason provided.".to_string());

  let mut dm_embed = BloomBotEmbed::new();

  dm_embed
    .title("A message you sent has been deleted.")
    .description(format!("**Reason**: {}", reason));

  if let Some(attachment) = message.attachments.first() {
    dm_embed.field("Attachment", attachment.url.clone(), false);
  }

  if message.content != "" {
    dm_embed.field(
      "Message Content",
      format!("```{}```", message.content),
      false,
    );
  }

  match message
    .author
    .direct_message(ctx, |f| f.set_embed(dm_embed.to_owned()))
    .await
  {
    Ok(_) => {
      mod_confirmation
        .edit(ctx, |f| {
          f.content(format!(
            ":white_check_mark: Message deleted. Sent the reason in DMs."
          ))
          .ephemeral(true)
        })
        .await?;
    }
    Err(_) => {
      mod_confirmation
        .edit(ctx, |f| {
          f.content(format!(
            ":white_check_mark: Message deleted. Could not send the reason in DMs."
          ))
          .ephemeral(true)
        })
        .await?;
    }
  };

  let mut log_embed = BloomBotEmbed::new();

  log_embed.title("Message Deleted").description(format!(
    "**Channel**: <#{}>\n**Author**: {}\n**Reason**: {}",
    message.channel_id, message.author, reason
  ));

  if let Some(attachment) = message.attachments.first() {
    log_embed.field("Attachment", attachment.url.clone(), false);
  }

  if message.content != "" {
    log_embed.field(
      "Message Content",
      format!("```{}```", message.content),
      false,
    );
  }

  log_embed.footer(|f| {
    f.icon_url(ctx.author().avatar_url().unwrap_or_default())
      .text(format!("Deleted by {}", ctx.author()))
  });

  let log_channel = serenity::ChannelId(CHANNELS.logs);

  log_channel
    .send_message(ctx, |f| f.set_embed(log_embed))
    .await?;

  Ok(())
}
