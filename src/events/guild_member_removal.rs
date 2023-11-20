use crate::config::{self, CHANNELS};
use anyhow::Result;
use poise::serenity_prelude::{self as serenity, Context, User};

pub async fn guild_member_removal(ctx: &Context, user: &User) -> Result<()> {
  let welcome_channel = serenity::ChannelId(CHANNELS.welcome);

  welcome_channel
    .send_message(&ctx, |m| {
      m.embed(|e| {
        config::BloomBotEmbed::from(e)
          .title("Member Left")
          .description(format!(
            "We wish you well on your future endeavors, {} :pray:",
            user.name
          ))
      })
    })
    .await?;

  Ok(())
}
