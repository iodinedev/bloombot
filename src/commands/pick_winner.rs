use crate::config::{BloomBotEmbed, CHANNELS, ROLES};
use crate::database::DatabaseHandler;
use crate::Context;
use anyhow::Result;
use chrono::Datelike;
use futures::StreamExt;
use poise::serenity_prelude as serenity;

#[derive(Debug, Clone, Copy, poise::ChoiceParameter)]
pub enum Months {
  January,
  February,
  March,
  April,
  May,
  June,
  July,
  August,
  September,
  October,
  November,
  December,
}

async fn finalize_winner(
  reserved_key: String,
  ctx: Context<'_>,
  winner: serenity::Member,
  selected_date: chrono::DateTime<chrono::Utc>,
) -> Result<()> {
  let now = chrono::Utc::now();
  let announcement_embed = BloomBotEmbed::new()
    .title(":tada: This month's meditation challenger in the spotlight is... :tada:")
    .thumbnail(winner.user.avatar_url().unwrap_or_default())
    .field(
      "**Monthly hall-of-fame member**",
      format!(
        "**{}** is our server member of the month, with a meditation time of **{}** minutes!\nYou're doing great, keep at it!",
        winner.user, 0
      ),
      false,
    )
    .footer(|f| {
      f.text(format!(
        "Meditation challenge for {} | Selected on {}",
        selected_date.format("%B %Y"),
        now.format("%B %d, %Y")
      ))
    }).to_owned();

  let dm_embed = BloomBotEmbed::new()
    .title(":tada: You've won a key! :tada:")
    .thumbnail(winner.user.avatar_url().unwrap_or_default())
    .field(
      "**Congratulations!**",
      "**Congratulations on winning the giveaway!** 🥳\n\nYou\'ve won a key for Playne: The Meditation Game on Steam!\n\n**Would you like to redeem your key? Press \'Redeem\' below! Otherwise, click \'Cancel\' to keep it for someone else :\\)**",
      false,
    )
    .footer(|f| {
      f.text(format!(
        "From {} | If you have any problems, please contact a moderator and we will be happy to help!",
        ctx.guild().unwrap().name
      ))
    }).to_owned();

  let announement_channel = serenity::ChannelId(CHANNELS.announcement);
  let dm_channel = winner.user.create_dm_channel(ctx).await?;

  announement_channel
    .send_message(ctx, |f| f.set_embed(announcement_embed))
    .await?;

  let ctx_id = ctx.id();
  let redeem_id = format!("{}redeem", ctx_id);
  let cancel_id = format!("{}cancel", ctx_id);

  let mut dm_message = match dm_channel
    .send_message(ctx, |f| {
      f.set_embed(dm_embed).components(|c| {
        c.create_action_row(|a| {
          a.create_button(|b| {
            b.custom_id("redeem")
              .label("Redeem")
              .style(serenity::ButtonStyle::Success)
          })
          .create_button(|b| {
            b.custom_id("cancel")
              .label("Cancel")
              .style(serenity::ButtonStyle::Danger)
          })
        })
      })
    })
    .await
  {
    Ok(message) => message,
    Err(_) => {
      ctx
        .send(|f| f.content(":x: Could not send DM to member. Please run `/usekey` and copy a key manually if they want one.\n\n**No key has been used.**"))
        .await?;
      return Ok(());
    }
  };

  // Loop through incoming interactions with the buttons
  while let Some(press) = serenity::CollectComponentInteraction::new(ctx)
    // We defined our button IDs to start with `ctx_id`. If they don't, some other command's
    // button was pressed
    .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
    // Timeout when no navigation button has been pressed for 24 hours
    .timeout(std::time::Duration::from_secs(3600 * 24))
    .await
  {
    // Depending on which button was pressed, confirm or cancel
    if press.data.custom_id == redeem_id {
      let mut conn = ctx.data().db.get_connection_with_retry(5).await?;
      DatabaseHandler::mark_key_used(&mut conn, &reserved_key).await?;
      let hyperlink = format!(
        "[Redeem your key](https://store.steampowered.com/account/registerkey?key={})",
        reserved_key
      );

      dm_message
        .edit(ctx, |f| {
          f.set_components(serenity::CreateComponents(Vec::new()))
        })
        .await?;

      ctx
        .say(format!(
          "Awesome! Here is your key.\n```{}```\n{}",
          reserved_key, hyperlink
        ))
        .await?;
      return Ok(());
    } else if press.data.custom_id == cancel_id {
      dm_message
        .edit(ctx, |f| {
          f.set_components(serenity::CreateComponents(Vec::new()))
        })
        .await?;

      ctx
        .say("Alright, we'll keep it for someone else. Congrats again!")
        .await?;
      return Ok(());
    } else {
      // This is an unrelated button interaction
      continue;
    }
  }

  let timeout_embed = BloomBotEmbed::new()
    .title("**Congratulations!**")
    .description("**Congratulations on winning the giveaway!** 🥳\n\nYou\'ve won a key for Playne: The Meditation Game on Steam!\n\n**Would you like to redeem your key? Please DM our staff and we'll get one for you, right away!**")
    .footer(|f| {
      f.text(format!(
        "From {}",
        ctx.guild().unwrap().name
      ))
    }).to_owned();

  dm_message
    .edit(ctx, |f| {
      f.set_embed(timeout_embed)
        .set_components(serenity::CreateComponents(Vec::new()))
    })
    .await?;

  Ok(())
}

/// Pick a winner for the monthly meditation challenge
/// 
/// Picks the winner for the monthly meditation challenge and allows them to claim an unused Playne key.
///
/// Finds a user who meets the following criteria:
/// - Has the `@meditation challengers` role
/// - Has tracked at least 30 minutes during the specified month
/// - Has at least 8 sessions during the specified month
/// If multiple users meet this criteria, one is chosen at random.
#[poise::command(
  slash_command,
  required_permissions = "BAN_MEMBERS",
  rename = "pickwinner",
  hide_in_help,
  guild_only
)]
pub async fn pick_winner(
  ctx: Context<'_>,
  #[description = "The year to pick a winner for (defaults to this year in UTC)"] year: Option<i32>,
  #[description = "The month to pick a winner for (defaults to this month in UTC)"] month: Option<
    Months,
  >,
) -> Result<()> {
  ctx.defer_ephemeral().await?;

  let data = ctx.data();

  let guild_id = ctx.guild_id().unwrap();

  let mut transaction = data.db.start_transaction_with_retry(5).await?;

  if !DatabaseHandler::unused_key_exists(&mut transaction, &guild_id).await? {
    ctx
      .send(|f| f.content(":x: No unused keys found.").ephemeral(true))
      .await?;
    return Ok(());
  }

  let year = year.unwrap_or_else(|| {
    let now = chrono::Utc::now();
    now.year()
  });

  let month = match month {
    Some(month) => match month {
      Months::January => 1,
      Months::February => 2,
      Months::March => 3,
      Months::April => 4,
      Months::May => 5,
      Months::June => 6,
      Months::July => 7,
      Months::August => 8,
      Months::September => 9,
      Months::October => 10,
      Months::November => 11,
      Months::December => 12,
    },
    None => {
      let now = chrono::Utc::now();
      now.month()
    }
  };

  let start_date = match chrono::NaiveDate::from_ymd_opt(year, month, 1) {
    Some(date) => date,
    None => {
      ctx
        .send(|f| f.content("Invalid date.").ephemeral(true))
        .await?;
      return Ok(());
    }
  };

  let end_date = match chrono::NaiveDate::from_ymd_opt(year, month + 1, 1) {
    Some(date) => date,
    None => {
      ctx
        .send(|f| f.content("Invalid date.").ephemeral(true))
        .await?;
      return Ok(());
    }
  };

  let time = chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap();

  let start_datetime = chrono::NaiveDateTime::new(start_date, time).and_utc();
  let end_datetime = chrono::NaiveDateTime::new(end_date, time).and_utc();

  let mut conn = data.db.get_connection_with_retry(5).await?;
  // Since the stream is async, we can't use the same connection for the transaction
  let mut database_winner_candidates =
    DatabaseHandler::get_winner_candidates(&mut conn, start_datetime, end_datetime, &guild_id);

  // The database already randomizes the order... we can use the first one that has the role
  let winner_role_id = serenity::RoleId(ROLES.meditation_challenger);

  let guild = ctx.guild().unwrap();

  while let Some(winner) = database_winner_candidates.next().await {
    let winner = match winner {
      Ok(winner) => winner,
      Err(_) => {
        continue;
      }
    };

    let member = match guild.member(ctx, winner).await {
      Ok(member) => member,
      Err(_) => {
        continue;
      }
    };

    if !member.roles.contains(&winner_role_id) {
      continue;
    }

    let reserved_key = match DatabaseHandler::reserve_key(
      &mut transaction,
      &guild_id,
      &ctx.author().id,
    )
    .await?
    {
      Some(key) => key,
      None => {
        ctx
          .send(|f| f.content(":x: No unused keys found. Please add one and run `/usekey` to give them one if they want one."))
          .await?;
        return Ok(());
      }
    };

    DatabaseHandler::commit_transaction(transaction).await?;

    finalize_winner(reserved_key, ctx, member, start_datetime).await?;
    return Ok(());
  }

  ctx
    .send(|f| f.content("No winner found.").ephemeral(true))
    .await?;

  Ok(())
}
