use crate::Context;
use anyhow::Result;

/// Say hello to Bloom!
/// 
/// Says hello to Bloom.
/// 
/// Don't worry - Bloom is friendly :)
#[poise::command(slash_command)]
pub async fn hello(ctx: Context<'_>) -> Result<()> {
  ctx.say("Hello, friend!").await?;

  Ok(())
}
