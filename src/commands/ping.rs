use crate::Context;
use anyhow::Result;

/// Replies with the bot's latency
#[poise::command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<()> {
  let response = ctx.send(|f| f.content("Getting latency...")).await?;

  let latency = ctx.ping().await;

  response
    .edit(ctx, |f| {
      f.content(format!(
        ":ping_pong: Pong! Latency is {}ms.",
        latency.as_millis()
      ))
    })
    .await?;

  Ok(())
}
