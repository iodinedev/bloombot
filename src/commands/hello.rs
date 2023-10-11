use crate::Context;
use anyhow::Result;

/// Says hello!
#[poise::command(slash_command)]
pub async fn hello(ctx: Context<'_>) -> Result<()> {
  ctx.say("Hello, friend!").await?;

  Ok(())
}
