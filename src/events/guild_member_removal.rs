use crate::config::{self, CHANNELS};
use anyhow::Result;
use poise::serenity_prelude::{self as serenity, Context, CreateMessage, User};

pub async fn guild_member_removal(ctx: &Context, user: &User) -> Result<()> {
  let welcome_channel = serenity::ChannelId::new(CHANNELS.welcome);

  welcome_channel
    .send_message(
      &ctx,
      CreateMessage::new().embed(
        config::BloomBotEmbed::new()
          .title("Member Left")
          .description(format!(
            "We wish you well on your future endeavors, {} :pray:",
            user.name
          )),
      ),
    )
    .await?;

  Ok(())
}
