use poise::serenity_prelude as serenity;
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
  let conn = &ctx.data().database;
  let user_id = ctx.author().id.as_u64();
  let guild_id = ctx.guild_id().unwrap();

  conn.add_user_meditation_time(user_id, guild_id.as_u64(), minutes).await?;

  let response = format!("You added {minutes} minutes to your meditation time");

  ctx.say(response).await?;

  Ok(())
}

/// Get meditation statistics for a user or the server
#[poise::command(
  slash_command,
  subcommands("user", "server"),
)]
pub async fn stats(_: Context<'_>) -> Result<(), Error> {
  // This will never be called because of the `subcommand_required` attribute
  Ok(())
}

/// Get the stats of you or a specified user in this server
#[poise::command(slash_command)]
pub async fn user(
  ctx: Context<'_>,
  #[description = "User to get stats for"]
  user: Option<serenity::Member>,
) -> Result<(), Error> {
  let user: serenity::Member = match user {
    Some(user) => user,
    None => match ctx.author_member().await {
      Some(member) => member.into_owned(),
      None => {
        ctx.say("You are not a member of this server").await?;
        return Ok(());
      }
    }
  };

  let conn = &ctx.data().database;
  let total = conn.get_user_meditation_time(user.user.id.as_u64(), ctx.guild_id().unwrap().as_u64()).await?;

  let response = format!("{user} has meditated for {total} minutes", user = user.user.name, total = total);

  ctx.send(|f| f
    .embed(|f| f
      .title("User Meditation Time")
      .description(response)
      .color(serenity::Color::DARK_GREEN)
    )).await?;

  Ok(())
}

/// Get the stats of the server
#[poise::command(slash_command)]
pub async fn server(ctx: Context<'_>) -> Result<(), Error> {
  let conn = &ctx.data().database;
  let total = conn.get_server_meditation_time(ctx.guild_id().unwrap().as_u64()).await?;

  let response = format!("The server has meditated for {total} minutes", total = total);

  ctx.send(|f| f
    .embed(|f| f
      .title("Server Meditation Time")
      .description(response)
      .color(serenity::Color::DARK_GREEN)
    )).await?;

  Ok(())
}