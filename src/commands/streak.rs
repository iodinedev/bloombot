use crate::database::DatabaseHandler;
use crate::Context;
use anyhow::Result;
use poise::serenity_prelude as serenity;

#[derive(poise::ChoiceParameter)]
pub enum Privacy {
  #[name = "private"]
  Private,
  #[name = "public"]
  Public,
}

/// See your current meditation streak
/// 
/// Shows your current meditation streak.
/// 
/// Can also be used to check another member's streak.
#[poise::command(slash_command, category = "Meditation Tracking", guild_only)]
pub async fn streak(
  ctx: Context<'_>,
  #[description = "The user to check the streak of"] user: Option<serenity::User>,
  #[description = "Set visibility of response (Default is public)"] privacy: Option<Privacy>,
) -> Result<()> {
  let data = ctx.data();

  let privacy = match privacy {
    Some(privacy) => match privacy {
      Privacy::Private => true,
      Privacy::Public => false
    },
    None => false
  };

  // We unwrap here, because we know that the command is guild-only.
  let guild_id = ctx.guild_id().unwrap();
  let user_id = match &user {
    Some(user) => user.id,
    None => ctx.author().id,
  };

  let mut transaction = data.db.start_transaction_with_retry(5).await?;
  let streak = DatabaseHandler::get_streak(&mut transaction, &guild_id, &user_id).await?;

  if user.is_some() {
    let user = user.unwrap();
    ctx
      .send(|f| {
        f.content(format!(
          "{}'s current meditation streak is {} days.",
          user.name, streak
        ))
        .ephemeral(privacy)
        .allowed_mentions(|f| f.empty_parse())
      })
      .await?;
  } else {
    ctx
      .send(|f| {
        f.content(format!(
          "Your current meditation streak is {} days.",
          streak
        ))
        .ephemeral(privacy)
      })
      .await?;
  }

  Ok(())
}
