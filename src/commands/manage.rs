use crate::commands::{commit_and_say, MessageType};
use crate::config::{BloomBotEmbed, CHANNELS};
use crate::database::DatabaseHandler;
use crate::pagination::{PageRowRef, Pagination};
use crate::Context;
use anyhow::Result;
use chrono::{Datelike, Timelike};
use poise::serenity_prelude::{self as serenity, Mentionable};

/// Commands for managing meditation entries
/// 
/// Commands to create, list, update, or delete meditation entries for a user, or completely reset a user's data.
/// 
/// Requires `Administrator` permissions.
#[poise::command(
  slash_command,
  subcommands("create", "list", "update", "delete", "reset"),
  subcommand_required,
  required_permissions = "BAN_MEMBERS",
  default_member_permissions = "BAN_MEMBERS",
  hide_in_help,
  guild_only
)]
pub async fn manage(_: Context<'_>) -> Result<()> {
  Ok(())
}

/// Create a new meditation entry for a user. Note that all times are in UTC.
/// 
/// Creates a new meditation entry for the user. Note that all times are in UTC.
#[poise::command(slash_command)]
pub async fn create(
  ctx: Context<'_>,
  #[description = "The user to create the entry for"] user: serenity::User,
  #[description = "The number of minutes for the entry"]
  #[min = 0]
  minutes: i32,
  #[description = "The year of the entry"] year: u32,
  #[description = "The month of the entry"]
  #[min = 1]
  #[max = 12]
  month: u32,
  #[description = "The day of the entry"]
  #[min = 1]
  #[max = 31]
  day: u32,
  #[description = "The hour of the entry (defaults to 0)"]
  #[min = 0]
  #[max = 23]
  hour: Option<u32>,
  #[description = "The minute of the entry (defaults to 0)"]
  #[min = 0]
  #[max = 59]
  minute: Option<u32>,
) -> Result<()> {
  let date = match chrono::NaiveDate::from_ymd_opt(year as i32, month, day) {
    Some(date) => date,
    None => {
      ctx
        .send(|f| {
          f.embed(|e| {
            e.title("Error")
              .description(format!("Invalid date provided: {}-{}-{}", year, month, day))
              .color(serenity::Color::RED)
          })
          .ephemeral(true)
        })
        .await?;
      return Ok(());
    }
  };

  let time = match chrono::NaiveTime::from_hms_opt(hour.unwrap_or(0), minute.unwrap_or(0), 0) {
    Some(time) => time,
    None => {
      ctx
        .send(|f| {
          f.embed(|e| {
            e.title("Error")
              .description(format!(
                "Invalid time provided: {}:{}",
                hour.unwrap_or(0),
                minute.unwrap_or(0)
              ))
              .color(serenity::Color::RED)
          })
          .ephemeral(true)
        })
        .await?;
      return Ok(());
    }
  };

  let datetime = chrono::NaiveDateTime::new(date, time).and_utc();

  let data = ctx.data();
  let guild_id = ctx.guild_id().unwrap();

  let mut transaction = data.db.start_transaction_with_retry(5).await?;

  DatabaseHandler::create_meditation_entry(
    &mut transaction,
    &guild_id,
    &user.id,
    minutes,
    datetime,
  )
  .await?;

  let success_embed = BloomBotEmbed::new()
    .title("Meditation Entry Created")
    .description(format!(
      "**User**: <@{}>\n**Date**: {}\n**Time**: {} minute(s)",
      user.id,
      datetime.format("%B %d, %Y"),
      minutes
    ))
    .to_owned();

  commit_and_say(
    ctx,
    transaction,
    MessageType::EmbedOnly(success_embed),
    true,
  )
  .await?;

  let log_embed = BloomBotEmbed::new()
    .title("Meditation Entry Created")
    .description(format!(
      "**User**: <@{}>\n**Date**: {}\n**Time**: {} minute(s)",
      user.id,
      datetime.format("%B %d, %Y"),
      minutes
    ))
    .footer(|f| {
      f.icon_url(ctx.author().avatar_url().unwrap_or_default())
        .text(format!("Created by {}", ctx.author()))
    })
    .to_owned();

  let log_channel = serenity::ChannelId(CHANNELS.bloomlogs);

  log_channel
    .send_message(ctx, |f| f.set_embed(log_embed))
    .await?;

  Ok(())
}

/// List all meditation entries for a user
/// 
/// Lists all meditation entries for a user.
#[poise::command(slash_command)]
pub async fn list(
  ctx: Context<'_>,
  #[description = "The user to list the entries for"] user: serenity::User,
  #[description = "The page to show"] page: Option<usize>,
) -> Result<()> {
  let data = ctx.data();
  let guild_id = ctx.guild_id().unwrap();

  let mut transaction = data.db.start_transaction_with_retry(5).await?;

  // Define some unique identifiers for the navigation buttons
  let ctx_id = ctx.id();
  let prev_button_id = format!("{}prev", ctx_id);
  let next_button_id = format!("{}next", ctx_id);

  let mut current_page = page.unwrap_or(0);

  if current_page > 0 { current_page = current_page - 1 }

  let entries =
    DatabaseHandler::get_user_meditation_entries(&mut transaction, &guild_id, &user.id).await?;
  drop(transaction);
  let entries: Vec<PageRowRef> = entries.iter().map(|entry| entry as _).collect();
  let pagination = Pagination::new("Meditation Entries", entries).await?;

  if pagination.get_page(current_page).is_none() {
    current_page = pagination.get_last_page_number();
  }

  let first_page = pagination.create_page_embed(current_page);

  ctx
    .send(|f| {
      f.components(|b| {
        if pagination.get_page_count() > 1 {
          b.create_action_row(|b| {
            b.create_button(|b| b.custom_id(&prev_button_id).label("Previous"))
              .create_button(|b| b.custom_id(&next_button_id).label("Next"))
          });
        }

        b
      })
      .ephemeral(true);

      f.embeds = vec![first_page];

      f
    })
    .await?;

  // Loop through incoming interactions with the navigation buttons
  while let Some(press) = serenity::CollectComponentInteraction::new(ctx)
    // We defined our button IDs to start with `ctx_id`. If they don't, some other command's
    // button was pressed
    .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
    // Timeout when no navigation button has been pressed for 24 hours
    .timeout(std::time::Duration::from_secs(3600 * 24))
    .await
  {
    // Depending on which button was pressed, go to next or previous page
    if press.data.custom_id == next_button_id {
      current_page = pagination.update_page_number(current_page, 1);
    } else if press.data.custom_id == prev_button_id {
      current_page = pagination.update_page_number(current_page, -1);
    } else {
      // This is an unrelated button interaction
      continue;
    }

    // Update the message with the new page contents
    press
      .create_interaction_response(ctx, |b| {
        b.kind(serenity::InteractionResponseType::UpdateMessage)
          .interaction_response_data(|f| f.set_embed(pagination.create_page_embed(current_page)))
      })
      .await?;
  }

  Ok(())
}

/// Update a meditation entry for a user. Note that all times are in UTC.
/// 
/// Updates a meditation entry for a user. Note that all times are in UTC.
#[poise::command(slash_command)]
pub async fn update(
  ctx: Context<'_>,
  #[description = "The entry to update"] entry_id: String,
  #[description = "The number of minutes for the entry"]
  #[min = 0]
  minutes: Option<i32>,
  #[description = "The year of the entry"] year: Option<i32>,
  #[description = "The month of the entry"]
  #[min = 1]
  #[max = 12]
  month: Option<u32>,
  #[description = "The day of the entry"]
  #[min = 1]
  #[max = 31]
  day: Option<u32>,
  #[description = "The hour of the entry (defaults to 0)"]
  #[min = 0]
  #[max = 23]
  hour: Option<u32>,
  #[description = "The minute of the entry (defaults to 0)"]
  #[min = 0]
  #[max = 59]
  minute: Option<u32>,
) -> Result<()> {
  let existing_entry = {
    let data = ctx.data();
    let guild_id = ctx.guild_id().unwrap();

    let mut transaction = data.db.start_transaction_with_retry(5).await?;

    DatabaseHandler::get_meditation_entry(&mut transaction, &guild_id, &entry_id).await?
  };

  if minutes.is_none() && year.is_none() && month.is_none() && day.is_none() && hour.is_none() && minute.is_none() {
    ctx
      .send(|f| {
        f.embed(|e| {
          e.title("Error")
            .description("You must provide at least one option to update the entry.")
            .color(serenity::Color::RED)
        })
        .ephemeral(true)
      })
      .await?;
    return Ok(());
  }

  match existing_entry {
    Some(existing_entry) => {
      let minutes = minutes.unwrap_or(existing_entry.meditation_minutes);

      let existing_date = existing_entry.occurred_at;
      let year = year.unwrap_or(existing_date.year());
      let month = month.unwrap_or(existing_date.month());
      let day = day.unwrap_or(existing_date.day());
      let hour = hour.unwrap_or(existing_date.hour());
      let minute = minute.unwrap_or(existing_date.minute());

      let date = match chrono::NaiveDate::from_ymd_opt(year as i32, month, day) {
        Some(date) => date,
        None => {
          ctx
            .send(|f| {
              f.embed(|e| {
                e.title("Error")
                  .description(format!("Invalid date provided: {}-{}-{}", year, month, day))
                  .color(serenity::Color::RED)
              })
              .ephemeral(true)
            })
            .await?;
          return Ok(());
        }
      };

      let time = match chrono::NaiveTime::from_hms_opt(hour, minute, 0) {
        Some(time) => time,
        None => {
          ctx
            .send(|f| {
              f.embed(|e| {
                e.title("Error")
                  .description(format!(
                    "Invalid time provided: {}:{}",
                    hour,
                    minute
                  ))
                  .color(serenity::Color::RED)
              })
              .ephemeral(true)
            })
            .await?;
          return Ok(());
        }
      };

      let datetime = chrono::NaiveDateTime::new(date, time).and_utc();

      let data = ctx.data();

      let mut transaction = data.db.start_transaction_with_retry(5).await?;

      DatabaseHandler::update_meditation_entry(&mut transaction, &entry_id, minutes, datetime).await?;

      let success_embed = BloomBotEmbed::new()
        .title("Meditation Entry Updated")
        .description(format!(
          "**User**: <@{}>\n**ID**: {}\n\n**Before:** {} minute(s) on {}\n**After:** {} minute(s) on {}",
          existing_entry.user_id,
          entry_id,
          existing_entry.meditation_minutes,
          existing_date.format("%B %d, %Y at %l:%M %P"),
          minutes,
          datetime.format("%B %d, %Y at %l:%M %P")
        ))
        .to_owned();
      commit_and_say(
        ctx,
        transaction,
        MessageType::EmbedOnly(success_embed),
        true,
      )
      .await?;

      let log_embed = BloomBotEmbed::new()
        .title("Meditation Entry Updated")
        .description(format!(
          "**User**: <@{}>\n**ID**: {}\n\n__**Before**__\n**Date**: {}\n**Time**: {} minute(s)\n\n__**After**__\n**Date**: {}\n**Time**: {} minute(s)",
          existing_entry.user_id,
          entry_id,
          existing_date.format("%B %d, %Y at %l:%M %P"),
          existing_entry.meditation_minutes,
          datetime.format("%B %d, %Y at %l:%M %P"),
          minutes
        ))
        .footer(|f| {
          f.icon_url(ctx.author().avatar_url().unwrap_or_default())
            .text(format!("Updated by {}", ctx.author()))
        })
        .to_owned();

      let log_channel = serenity::ChannelId(CHANNELS.bloomlogs);

      log_channel
        .send_message(ctx, |f| f.set_embed(log_embed))
        .await?;

      Ok(())
    },
    None => {
      ctx
        .send(|f| {
          f.embed(|e| {
            e.title("Error")
              .description(format!("No meditation entry found with ID `{}`.", entry_id))
              .footer(|f| f.text("Use `/manage list` to see a user's entries."))
              .color(serenity::Color::RED)
          })
          .ephemeral(true)
        })
        .await?;

      Ok(())
    }
  }
}

/// Delete a meditation entry for a user
/// 
/// Deletes a meditation entry for the user.
#[poise::command(slash_command)]
pub async fn delete(
  ctx: Context<'_>,
  #[description = "The entry to delete"] entry_id: String,
) -> Result<()> {
  let data = ctx.data();
  let guild_id = ctx.guild_id().unwrap();

  let mut transaction = data.db.start_transaction_with_retry(5).await?;

  let entry =
    match DatabaseHandler::get_meditation_entry(&mut transaction, &guild_id, &entry_id).await? {
      Some(entry) => entry,
      None => {
        ctx
          .send(|f| {
            f.embed(|e| {
              e.title("Error")
                .description(format!("No meditation entry found with ID `{}`.", entry_id))
                .footer(|f| f.text("Use `/manage list` to see a user's entries."))
                .color(serenity::Color::RED)
            })
            .ephemeral(true)
          })
          .await?;
        return Ok(());
      }
    };

  DatabaseHandler::delete_meditation_entry(&mut transaction, &entry_id).await?;

  let success_embed = BloomBotEmbed::new()
    .title("Meditation Entry Deleted")
    .description(format!(
      "**User**: <@{}>\n**ID**: {}\n**Date**: {}\n**Time**: {} minute(s)",
      entry.user_id,
      entry.id,
      entry.occurred_at.format("%B %d, %Y"),
      entry.meditation_minutes
    ))
    .to_owned();
  
  commit_and_say(
    ctx,
    transaction,
    MessageType::EmbedOnly(success_embed),
    true,
  )
  .await?;

  let log_embed = BloomBotEmbed::new()
    .title("Meditation Entry Deleted")
    .description(format!(
      "**User**: <@{}>\n**ID**: {}\n**Date**: {}\n**Time**: {} minute(s)",
      entry.user_id,
      entry.id,
      entry.occurred_at.format("%B %d, %Y"),
      entry.meditation_minutes
    ))
    .footer(|f| {
      f.icon_url(ctx.author().avatar_url().unwrap_or_default())
        .text(format!("Deleted by {}", ctx.author()))
    })
    .to_owned();

  let log_channel = serenity::ChannelId(CHANNELS.bloomlogs);

  log_channel
    .send_message(ctx, |f| f.set_embed(log_embed))
    .await?;

  Ok(())
}

/// Reset all meditation entries for a user
/// 
/// Resets all meditation entries for a user.
#[poise::command(slash_command)]
pub async fn reset(
  ctx: Context<'_>,
  #[description = "The user to reset the entries for"] user: serenity::User,
) -> Result<()> {
  let data = ctx.data();
  let guild_id = ctx.guild_id().unwrap();

  let mut transaction = data.db.start_transaction_with_retry(5).await?;

  DatabaseHandler::reset_user_meditation_entries(&mut transaction, &guild_id, &user.id).await?;

  let ctx_id = ctx.id();

  let confirm_id = format!("{}confirm", ctx_id);
  let cancel_id = format!("{}cancel", ctx_id);

  ctx
    .send(|f| {
      f.content(format!(
        "Are you sure you want to reset all meditation entries for {}?",
        user.mention()
      ))
      .ephemeral(true)
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

    let confirmed = press.data.custom_id == confirm_id;

    // Update the message with the new page contents
    if confirmed {
      match press
        .create_interaction_response(ctx, |b| {
          b.kind(serenity::InteractionResponseType::UpdateMessage)
            .interaction_response_data(|f| {
              f.content("Confirmed.")
                .set_components(serenity::CreateComponents(Vec::new()))
            })
        })
        .await
      {
        Ok(_) => {
          DatabaseHandler::commit_transaction(transaction).await?;

          let log_embed = BloomBotEmbed::new()
            .title("Meditation Entries Reset")
            .description(format!(
              "**User**: <@{}>",
              user.id
            ))
            .footer(|f| {
              f.icon_url(ctx.author().avatar_url().unwrap_or_default())
                .text(format!("Reset by {}", ctx.author()))
            })
            .to_owned();
        
          let log_channel = serenity::ChannelId(CHANNELS.bloomlogs);
        
          log_channel
            .send_message(ctx, |f| f.set_embed(log_embed))
            .await?;
          
          return Ok(());
        }
        Err(e) => {
          DatabaseHandler::rollback_transaction(transaction).await?;
          return Err(anyhow::anyhow!(
            "Failed to tell user that the meditation entries were reset: {}",
            e
          ));
        }
      }
    } else {
      press
        .create_interaction_response(ctx, |b| {
          b.kind(serenity::InteractionResponseType::UpdateMessage)
            .interaction_response_data(|f| {
              f.content("Cancelled.")
                .set_components(serenity::CreateComponents(Vec::new()))
            })
        })
        .await?;
    }
  }

  // This happens when the user didn't press any button for 60 seconds
  Ok(())
}
