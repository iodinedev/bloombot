use crate::Context;
use anyhow::Result;
use rand::Rng;
use std::sync::Arc;

/// I will choose either ☕ or ⚰️. (My version of Russian Roulette)
#[poise::command(slash_command)]
pub async fn coffee(ctx: Context<'_>) -> Result<()> {
  let data = ctx.data();

  let rng = Arc::clone(&data.rng);
  let mut rng = rng.lock().await;

  let choice = rng.gen_range(0..2);

  match choice {
    0 => {
      ctx.say("☕").await?;
    }
    1 => {
      ctx.say("⚰️").await?;
    }
    _ => {
      ctx.say("Something went wrong.").await?;
    }
  }

  Ok(())
}
