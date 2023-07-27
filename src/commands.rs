use crate::{Context, Error};

/// Show this help menu
#[poise::command(slash_command)]
pub async fn help(
  ctx: Context<'_>,
  #[description = "Specific command to show help about"]
  #[autocomplete = "poise::builtins::autocomplete_command"]
  command: Option<String>,
) -> Result<(), Error> {
  poise::builtins::help(
    ctx,
    command.as_deref(),
    poise::builtins::HelpConfiguration {
      extra_text_at_bottom: "This is an example bot made to showcase features of my custom Discord bot framework",
      ..Default::default()
    },
  )
  .await?;
  Ok(())
}

/// Add minutes to your meditation time
#[poise::command(slash_command)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Minutes to add to your meditation time"]
    #[min = 1]
    minutes: u32,
) -> Result<(), Error> {
  let response = format!("You added {minutes} minutes to your meditation time");

  ctx.say(response).await?;

  Ok(())
}
