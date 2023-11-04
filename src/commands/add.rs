use crate::commands::{commit_and_say, MessageType};
use crate::config::{StreakRoles, TimeSumRoles};
use crate::database::DatabaseHandler;
use crate::Context;
use anyhow::Result;
use log::error;
use poise::serenity_prelude::{self as serenity, Mentionable};

/// Adds minutes to your meditation time.
#[poise::command(slash_command, guild_only)]
pub async fn add(
  ctx: Context<'_>,
  #[description = "Number of minutes to add"]
  #[min = 1]
  minutes: i32,
) -> Result<()> {
  let data = ctx.data();

  // We unwrap here, because we know that the command is guild-only.
  let guild_id = ctx.guild_id().unwrap();
  let user_id = ctx.author().id;

  let mut transaction = data.db.start_transaction_with_retry(5).await?;
  DatabaseHandler::add_minutes(&mut transaction, &guild_id, &user_id, minutes).await?;

  let user_sum =
    DatabaseHandler::get_user_meditation_sum(&mut transaction, &guild_id, &user_id).await?;
  let user_streak = DatabaseHandler::get_streak(&mut transaction, &guild_id, &user_id).await?;
  let random_quote = DatabaseHandler::get_random_quote(&mut transaction, &guild_id).await?;

  let response = match random_quote {
    Some(quote) => {
      // Strip non-alphanumeric characters from the quote
      let quote = quote
        .quote
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || c.is_ascii_punctuation())
        .map(|c| {
          if c.is_ascii_punctuation() {
            format!("\\{c}")
          } else {
            c.to_string()
          }
        })
        .collect::<String>();

      format!("Added **{minutes} minutes** to your meditation time! Your total meditation time is now {user_sum} minutes :tada:\n*{quote}*")
    }
    None => {
      format!("Added **{minutes} minutes** to your meditation time! Your total meditation time is now {user_sum} minutes :tada:")
    }
  };

  if minutes > 300 {
    let ctx_id = ctx.id();

    let confirm_id = format!("{}confirm", ctx_id);
    let cancel_id = format!("{}cancel", ctx_id);

    let check = ctx
      .send(|f| {
        f.content(format!(
          "Are you sure you want to add **{}** minutes to your meditation time?",
          minutes
        ))
        .components(|c| {
          c.create_action_row(|a| {
            a.create_button(|b| {
              b.custom_id(confirm_id.clone())
                .label("Yes")
                .style(serenity::ButtonStyle::Success)
            })
            .create_button(|b| {
              b.custom_id(cancel_id.clone())
                .label("No")
                .style(serenity::ButtonStyle::Danger)
            })
          })
        })
      })
      .await?;

    // Loop through incoming interactions with the navigation buttons
    while let Some(press) = serenity::CollectComponentInteraction::new(ctx)
      // We defined our button IDs to start with `ctx_id`. If they don't, some other command's
      // button was pressed
      .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
      // Timeout when no navigation button has been pressed in one minute
      .timeout(std::time::Duration::from_secs(60))
      .await
    {
      // Depending on which button was pressed, go to next or previous page
      if press.data.custom_id != confirm_id && press.data.custom_id != cancel_id {
        // This is an unrelated button interaction
        continue;
      }

      let confirm = press.data.custom_id == confirm_id;

      // Update the message to reflect the action
      match press
        .create_interaction_response(ctx, |b| {
          b.kind(serenity::InteractionResponseType::UpdateMessage)
            .interaction_response_data(|f| {
              if confirm {
                f.content(response)
                  .set_components(serenity::CreateComponents(Vec::new()))
              } else {
                f.content("Cancelled.")
                  .set_components(serenity::CreateComponents(Vec::new()))
              }
            })
        })
        .await
      {
        Ok(_) => {
          if confirm {
            match DatabaseHandler::commit_transaction(transaction).await {
              Ok(_) => {}
              Err(e) => {
                check.edit(ctx, |f| f
                  .content(":bangbang: A fatal error occured while trying to save your changes. Nothing has been saved.")).await?;
                return Err(anyhow::anyhow!("Could not send message: {}", e));
              }
            }
          }
        }
        Err(e) => {
          check
            .edit(ctx, |f| {
              f.content(":x: An error occured. Nothing has been saved.")
            })
            .await?;
          return Err(anyhow::anyhow!("Could not send message: {}", e));
        }
      }

      return Ok(());
    }
  }

  let guild_count =
    DatabaseHandler::get_guild_meditation_count(&mut transaction, &guild_id).await?;
  let guild_sum = DatabaseHandler::get_guild_meditation_sum(&mut transaction, &guild_id).await?;

  commit_and_say(ctx, transaction, MessageType::TextOnly(response), false).await?;

  if guild_count % 10 == 0 {
    let time_in_hours = guild_sum / 60;

    ctx.say(format!("Awesome sauce! This server has collectively generated {} hours of realmbreaking meditation!", time_in_hours)).await?;
  }

  let guild = ctx.guild().unwrap();
  let mut member = guild.member(ctx, user_id).await?;

  let current_time_roles = TimeSumRoles::get_users_current_roles(&guild, &member);
  let current_streak_roles = StreakRoles::get_users_current_roles(&guild, &member);

  let updated_time_role = TimeSumRoles::from_sum(user_sum);
  let updated_streak_role = StreakRoles::from_streak(user_streak);

  if let Some(updated_time_role) = updated_time_role {
    if !current_time_roles.contains(&updated_time_role.to_role_id()) {
      for role in current_time_roles {
        match member.remove_role(ctx, role).await {
          Ok(_) => {}
          Err(err) => {
            error!("Error removing role: {}", err);
            ctx.send(|f| f
              .content(":x: An error occured while updating your time roles. Your entry has been saved, but your roles have not been updated. Please contact a moderator.")
              .allowed_mentions(|f| f.empty_parse())).await?;

            return Ok(());
          }
        }
      }

      match member.add_role(ctx, updated_time_role.to_role_id()).await {
        Ok(_) => {}
        Err(err) => {
          error!("Error adding role: {}", err);
          ctx.send(|f| f
            .content(":x: An error occured while updating your time roles. Your entry has been saved, but your roles have not been updated. Please contact a moderator.")
            .allowed_mentions(|f| f.empty_parse())).await?;

          return Ok(());
        }
      }

      ctx.send(|f| f
        .content(format!(":tada: Congrats to {}, your hard work is paying off! Your total meditation minutes have given you the <@&{}> role!", member.mention(), updated_time_role.to_role_id()))
        .allowed_mentions(|f| f.empty_parse())).await?;
    }
  }

  if let Some(updated_streak_role) = updated_streak_role {
    if !current_streak_roles.contains(&updated_streak_role.to_role_id()) {
      for role in current_streak_roles {
        match member.remove_role(ctx, role).await {
          Ok(_) => {}
          Err(err) => {
            error!("Error removing role: {}", err);

            ctx.send(|f| f
              .content(":x: An error occured while updating your streak roles. Your entry has been saved, but your roles have not been updated. Please contact a moderator.")
              .allowed_mentions(|f| f.empty_parse())).await?;

            return Ok(());
          }
        }
      }

      match member.add_role(ctx, updated_streak_role.to_role_id()).await {
        Ok(_) => {}
        Err(err) => {
          error!("Error adding role: {}", err);

          ctx.send(|f| f
            .content(":x: An error occured while updating your streak roles. Your entry has been saved, but your roles have not been updated. Please contact a moderator.")
            .allowed_mentions(|f| f.empty_parse())).await?;

          return Ok(());
        }
      }

      ctx.send(|f| f
        .content(format!(":tada: Congrats to {}, your hard work is paying off! Your current streak is {}, giving you the <@&{}> role!", member.mention(), user_streak, updated_streak_role.to_role_id()))
        .allowed_mentions(|f| f.empty_parse())).await?;
    }
  }

  Ok(())
}
