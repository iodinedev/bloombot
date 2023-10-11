use crate::database::DatabaseHandler;
use crate::Context;
use anyhow::Result;
use poise::serenity_prelude as serenity;

/// Shows your current meditation streak
#[poise::command(slash_command, guild_only)]
pub async fn streak(
  ctx: Context<'_>,
  #[description = "The user to check the streak of"] user: Option<serenity::User>,
) -> Result<()> {
  let data = ctx.data();

  // We unwrap here, because we know that the command is guild-only.
  let guild_id = ctx.guild_id().unwrap();
  let user_id = match &user {
    Some(user) => user.id,
    None => ctx.author().id,
  };

  let mut transaction = data.db.start_transaction().await?;
  let streak = DatabaseHandler::get_streak(&mut transaction, &guild_id, &user_id).await?;

  if user.is_some() {
    let user = user.unwrap();
    ctx
      .send(|f| {
        f.content(format!(
          "{}'s current meditation streak is {} days.",
          user.name, streak
        ))
        .allowed_mentions(|f| f.empty_parse())
      })
      .await?;
  } else {
    ctx
      .say(format!(
        "Your current meditation streak is {} days.",
        streak
      ))
      .await?;
  }

  Ok(())
}
