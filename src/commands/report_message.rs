use crate::config::{BloomBotEmbed, CHANNELS, ROLES};
use crate::Context;
use anyhow::Result;
use poise::serenity_prelude as serenity;

/// Reports a message to server staff
/// 
/// To use, right-click the message that you want to report, then go to "Apps" > "Report Message".
#[poise::command(
  ephemeral,
	context_menu_command = "Report Message",
  category = "Context Menu Commands",
	hide_in_help,
  guild_only
)]
pub async fn report_message(
	ctx: Context<'_>,
	#[description = "Message to report"]
	message: serenity::Message,
) -> Result<()> {
  let reporting_user = ctx.author();
  let report_channel_id = serenity::ChannelId(CHANNELS.reportchannel);
  let message_link = message.link().clone();
  let message_user = message.author;
  let message_channel_name = message.channel_id.to_channel(&ctx).await.unwrap().guild().unwrap().name;

  let message_content = match message.content.is_empty() {
    true => {
      match message.attachments.first() {
        Some(attachment) => format!("**Attachment**\n{}", attachment.url.clone()),
        None => message.content.clone()
      }
    },
    false => message.content.clone()
  };

  report_channel_id
    .send_message(&ctx, |m| {
      m.content(format!("<@&{}> Message Reported", ROLES.staff))
        .embed(|e| {
          BloomBotEmbed::from(e)
            .author(|a| a.name(format!("{}", &message_user.name)).icon_url(message_user.face()))
            .description(message_content)
            .field("Link", format!("[Go to message]({})", message_link), false)
            .footer(|f| {
              f.text(format!(
                "Author ID: {}\nReported via context menu in #{} by {} ({})",
                &message_user.id,
                message_channel_name,
                reporting_user.name,
                reporting_user.id
              ))
            })
            .timestamp(message.timestamp)
        })
    })
    .await?;

  ctx
    .send(|f| f.content("Your report has been sent to the moderation team.").ephemeral(true))
    .await?;

	Ok(())
}
