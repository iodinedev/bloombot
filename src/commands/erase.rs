use crate::config::{BloomBotEmbed, CHANNELS};
use crate::Context;
use anyhow::Result;
use poise::serenity_prelude as serenity;

/// Delete a message and notify the user
/// 
/// Deletes a message and notifies the user via DM with an optional reason.
/// 
/// Requires `Manage Messages` permissions.
#[poise::command(
  slash_command,
  required_permissions = "MANAGE_MESSAGES",
  default_member_permissions = "MANAGE_MESSAGES",
  category = "Moderator Commands",
  //hide_in_help,
  guild_only
)]
pub async fn erase(
  ctx: Context<'_>,
  #[description = "The message to delete"] message: serenity::Message,
  #[max_length = 1000]
  #[description = "The reason for deleting the message"]
  reason: Option<String>,
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
      f.content(":white_check_mark: Message deleted. Sending the reason in DMs...".to_string())
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

  if !message.content.is_empty() {
    // If longer than 1024 - 6 characters for the embed, truncate to 1024 - 3 for "..."
    let content = match message.content.len() > 1018 {
      true => format!(
        "{}...",
        message.content.chars().take(1015).collect::<String>()
      ),
      false => message.content.clone(),
    };

    dm_embed.field("Message Content", format!("```{}```", content), false);

    dm_embed.footer(|f| f.text(
      "If you have any questions or concerns regarding this action, please contact a moderator. Replies sent to Bloom are not viewable by staff."
    ));
  }

  match message
    .author
    .direct_message(ctx, |f| f.set_embed(dm_embed.to_owned()))
    .await
  {
    Ok(_) => {
      mod_confirmation
        .edit(ctx, |f| {
          f.content(":white_check_mark: Message deleted. Sent the reason in DMs.")
            .ephemeral(true)
        })
        .await?;
    }
    Err(_) => {
      let notification_thread = channel_id
        .create_private_thread(ctx, |create_thread| create_thread
          .name(format!(
            "Private Notification: Message Deleted"
          )))
        .await?;

      notification_thread
        .edit_thread(ctx, |edit_thread| edit_thread
          .invitable(false)
          .locked(true)
        )
        .await?;

      dm_embed.footer(|f| f.text(
        "If you have any questions or concerns regarding this action, please contact staff via ModMail."
      ));
    
      let thread_initial_message = format!(
        "Private notification for <@{}>:",
        message.author.id
      );
    
      notification_thread.send_message(ctx, |create_message| {
        create_message
          .content(thread_initial_message)
          .set_embed(dm_embed.to_owned())
          .allowed_mentions(|create_allowed_mentions| {
            create_allowed_mentions
              .users([message.author.id])
          })
      })
      .await?;

      mod_confirmation
        .edit(ctx, |f| {
          f.content(format!(
            ":white_check_mark: Message deleted. Could not send the reason in DMs. Private thread created: <#{}>",
            notification_thread.id
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

  if !message.content.is_empty() {
    // If longer than 1024 - 6 characters for the embed, truncate to 1024 - 3 for "..."
    let content = match message.content.len() > 1018 {
      true => format!(
        "{}...",
        message.content.chars().take(1015).collect::<String>()
      ),
      false => message.content.clone(),
    };

    log_embed.field(
      "Message Content",
      format!("```{}```", content),
      false,
    );
  }

  log_embed.footer(|f| {
    f.icon_url(ctx.author().avatar_url().unwrap_or_default())
      .text(format!("Deleted by {} ({})", ctx.author().name, ctx.author().id))
  });

  let log_channel = serenity::ChannelId(CHANNELS.logs);

  log_channel
    .send_message(ctx, |f| f.set_embed(log_embed))
    .await?;

  Ok(())
}
