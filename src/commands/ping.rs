use crate::Context;
use anyhow::Result;

/// Check the bot's latency
///
/// Replies with the bot's latency.
#[poise::command(slash_command, category = "Utilities")]
pub async fn ping(ctx: Context<'_>) -> Result<()> {
  let response = ctx
    .send(poise::CreateReply::default().content("Getting latency..."))
    .await?;

  let latency = ctx.ping().await;

  response
    .edit(
      ctx,
      poise::CreateReply::default().content(format!(
        ":ping_pong: Pong! Latency is {}ms.",
        latency.as_millis()
      )),
    )
    .await?;

  Ok(())
}
