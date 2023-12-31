use crate::database::{DatabaseHandler, TrackingProfile};
use crate::{Context, config};
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
/// Shows your current meditation streak. Setting the visibility here will override your custom streak privacy settings.
/// 
/// Can also be used to check another member's streak, unless set to private.
#[poise::command(slash_command, category = "Meditation Tracking", guild_only)]
pub async fn streak(
  ctx: Context<'_>,
  #[description = "The user to check the streak of"] user: Option<serenity::User>,
  #[description = "Set visibility of response (Default is public)"] privacy: Option<Privacy>,
) -> Result<()> {
  let data = ctx.data();

  // We unwrap here, because we know that the command is guild-only.
  let guild_id = ctx.guild_id().unwrap();
  let user_id = match &user {
    Some(user) => user.id,
    None => ctx.author().id,
  };

  let mut transaction = data.db.start_transaction_with_retry(5).await?;
  let streak = DatabaseHandler::get_streak(&mut transaction, &guild_id, &user_id).await?;

  let tracking_profile = match DatabaseHandler::get_tracking_profile(&mut transaction, &guild_id, &user_id).await? {
    Some(tracking_profile) => tracking_profile,
    None => TrackingProfile { ..Default::default() }
  };

  let privacy = match privacy {
    Some(privacy) => match privacy {
      Privacy::Private => true,
      Privacy::Public => false
    },
    None => tracking_profile.streaks_private
  };

  if user.is_some() && (user_id != ctx.author().id) {
    let user = user.unwrap();
    let user_nick_or_name = match user.nick_in(&ctx, guild_id).await {
      Some(nick) => nick,
      None => user.name.clone()
    };

    if tracking_profile.streaks_private {
      //Show for staff even when private
      if ctx.author().has_role(&ctx, guild_id, config::ROLES.staff).await? {
        ctx
        .send(|f| {
          f.content(format!(
            "{}'s current **private** meditation streak is {} days.",
            user_nick_or_name, streak
          ))
          .ephemeral(true)
          .allowed_mentions(|f| f.empty_parse())
        })
        .await?;

        return Ok(());
      } else {
        ctx
        .send(|f| {
          f.content(format!(
            "Sorry, {}'s meditation streak is set to private.",
            user_nick_or_name
          ))
          .ephemeral(true)
          .allowed_mentions(|f| f.empty_parse())
        })
        .await?;

        return Ok(());
      }
    } else {
      ctx
      .send(|f| {
        f.content(format!(
          "{}'s current meditation streak is {} days.",
          user_nick_or_name, streak
        ))
        .ephemeral(privacy)
        .allowed_mentions(|f| f.empty_parse())
      })
      .await?;

      return Ok(());
    }
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

      return Ok(());
  }
}
