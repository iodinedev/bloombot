use crate::commands::{commit_and_say, MessageType};
use crate::config::{StreakRoles, TimeSumRoles, BloomBotEmbed, CHANNELS};
use crate::database::DatabaseHandler;
use crate::Context;
use chrono::Duration;
use anyhow::Result;
use log::error;
use poise::serenity_prelude::{self as serenity, Mentionable};

#[derive(poise::ChoiceParameter)]
pub enum MinusOffsetChoices {
  #[name = "UTC-12 (BIT)"]
  UTCMinus12,
  #[name = "UTC-11 (NUT, SST)"]
  UTCMinus11,
  #[name = "UTC-10 (CKT, HAST, HST, TAHT)"]
  UTCMinus10,
  #[name = "UTC-9:30 (MART, MIT)"]
  UTCMinus9_30,
  #[name = "UTC-9 (AKST, GAMT, GIT, HADT)"]
  UTCMinus9,
  #[name = "UTC-8 (AKDT, CIST, PST)"]
  UTCMinus8,
  #[name = "UTC-7 (MST, PDT)"]
  UTCMinus7,
  #[name = "UTC-6 (CST, EAST, GALT, MDT)"]
  UTCMinus6,
  #[name = "UTC-5 (ACT, CDT, COT, CST, EASST, ECT, EST, PET)"]
  UTCMinus5,
  #[name = "UTC-4:30 (VET)"]
  UTCMinus4_30,
  #[name = "UTC-4 (AMT, AST, BOT, CDT, CLT, COST, ECT, EDT, FKT, GYT, PYT)"]
  UTCMinus4,
  #[name = "UTC-3:30 (NST, NT)"]
  UTCMinus3_30,
  #[name = "UTC-3 (ADT, AMST, ART, BRT, CLST, FKST, GFT, PMST, PYST, ROTT, SRT, UYT)"]
  UTCMinus3,
  #[name = "UTC-2:30 (NDT)"]
  UTCMinus2_30,
  #[name = "UTC-2 (BRST, FNT, GST, PMDT, UYST)"]
  UTCMinus2,
  #[name = "UTC-1 (AZOST, CVT, EGT)"]
  UTCMinus1,
}

#[derive(poise::ChoiceParameter)]
pub enum PlusOffsetChoices {
  #[name = "UTC+1 (BST, CET, IST, WAT, WEST)"]
  UTCPlus1,
  #[name = "UTC+2 (CAT, CEST, EET, IST, SAST, WAST)"]
  UTCPlus2,
  #[name = "UTC+3 (AST, EAT, EEST, FET, IDT, IOT, MSK, USZ1)"]
  UTCPlus3,
  #[name = "UTC+3:30 (IRST)"]
  UTCPlus3_30,
  #[name = "UTC+4 (AMT, AZT, GET, GST, MUT, RET, SAMT, SCT, VOLT)"]
  UTCPlus4,
  #[name = "UTC+4:30 (AFT, IRDT)"]
  UTCPlus4_30,
  #[name = "UTC+5 (HMT, MAWT, MVT, ORAT, PKT, TFT, TJT, TMT, UZT, YEKT)"]
  UTCPlus5,
  #[name = "UTC+5:30 (IST, SLST)"]
  UTCPlus5_30,
  #[name = "UTC+5:45 (NPT)"]
  UTCPlus5_45,
  #[name = "UTC+6 (BDT, BIOT, BST, BTT, KGT, OMST, VOST)"]
  UTCPlus6,
  #[name = "UTC+6:30 (CCT, MMT, MST)"]
  UTCPlus6_30,
  #[name = "UTC+7 (CXT, DAVT, HOVT, ICT, KRAT, THA, WIT)"]
  UTCPlus7,
  #[name = "UTC+8 (ACT, AWST, BDT, CHOT, CIT, CST, CT, HKT, IRKT, MST, MYT, PST, SGT, SST, ULAT, WST)"]
  UTCPlus8,
  #[name = "UTC+8:45 (CWST)"]
  UTCPlus8_45,
  #[name = "UTC+9 (AWDT, EIT, JST, KST, TLT, YAKT)"]
  UTCPlus9,
  #[name = "UTC+9:30 (ACST, CST)"]
  UTCPlus9_30,
  #[name = "UTC+10 (AEST, ChST, CHUT, DDUT, EST, PGT, VLAT)"]
  UTCPlus10,
  #[name = "UTC+10:30 (ACDT, CST, LHST)"]
  UTCPlus10_30,
  #[name = "UTC+11 (AEDT, BST, KOST, LHST, MIST, NCT, PONT, SAKT, SBT, SRET, VUT, NFT)"]
  UTCPlus11,
  #[name = "UTC+12 (FJT, GILT, MAGT, MHT, NZST, PETT, TVT, WAKT)"]
  UTCPlus12,
  #[name = "UTC+12:45 (CHAST)"]
  UTCPlus12_45,
  #[name = "UTC+13 (NZDT, PHOT, TKT, TOT)"]
  UTCPlus13,
  #[name = "UTC+13:45 (CHADT)"]
  UTCPlus13_45,
  #[name = "UTC+14 (LINT)"]
  UTCPlus14,
}

/// Add minutes to your meditation time, with optional UTC offset
/// 
/// Adds a specified number of minutes to your meditation time. You can add minutes each time you meditate or add the combined minutes for multiple sessions.
/// 
/// You may wish to add large amounts of time on occasion, e.g., after a silent retreat. Time tracking is based on the honor system and members are welcome to track any legitimate time spent practicing.
/// 
/// Vanity roles are purely cosmetic, so there is nothing to be gained from cheating. Furthermore, exceedingly large false entries will skew the server stats, which is unfair to other members. Please be considerate.
#[poise::command(slash_command, guild_only)]
pub async fn add(
  ctx: Context<'_>,
  #[description = "Number of minutes to add"]
  #[min = 1]
  minutes: i32,
  #[description = "Specify a UTC offset for a Western Hemisphere time zone"]
  #[rename = "western_hemisphere_offset"]
  minus_offset: Option<MinusOffsetChoices>,
  #[description = "Specify a UTC offset for an Eastern Hemisphere time zone"]
  #[rename = "eastern_hemisphere_offset"]
  plus_offset: Option<PlusOffsetChoices>,
) -> Result<()> {
  let data = ctx.data();

  // We unwrap here, because we know that the command is guild-only.
  let guild_id = ctx.guild_id().unwrap();
  let user_id = ctx.author().id;

  let mut transaction = data.db.start_transaction_with_retry(5).await?;

  let minus_offset = match minus_offset {
    Some(minus_offset) => match minus_offset {
      MinusOffsetChoices::UTCMinus12 => -720,
      MinusOffsetChoices::UTCMinus11 => -660,
      MinusOffsetChoices::UTCMinus10 => -600,
      MinusOffsetChoices::UTCMinus9_30 => -570,
      MinusOffsetChoices::UTCMinus9 => -540,
      MinusOffsetChoices::UTCMinus8 => -480,
      MinusOffsetChoices::UTCMinus7 => -420,
      MinusOffsetChoices::UTCMinus6 => -360,
      MinusOffsetChoices::UTCMinus5 => -300,
      MinusOffsetChoices::UTCMinus4_30 => -270,
      MinusOffsetChoices::UTCMinus4 => -240,
      MinusOffsetChoices::UTCMinus3_30 => -210,
      MinusOffsetChoices::UTCMinus3 => -180,
      MinusOffsetChoices::UTCMinus2_30 => -150,
      MinusOffsetChoices::UTCMinus2 => -120,
      MinusOffsetChoices::UTCMinus1 => -60,
    },
    None => 0
  };

  let plus_offset = match plus_offset {
    Some(plus_offset) => match plus_offset {
      PlusOffsetChoices::UTCPlus1 => 60,
      PlusOffsetChoices::UTCPlus2 => 120,
      PlusOffsetChoices::UTCPlus3 => 180,
      PlusOffsetChoices::UTCPlus3_30 => 210,
      PlusOffsetChoices::UTCPlus4 => 240,
      PlusOffsetChoices::UTCPlus4_30 => 270,
      PlusOffsetChoices::UTCPlus5 => 300,
      PlusOffsetChoices::UTCPlus5_30 => 330,
      PlusOffsetChoices::UTCPlus5_45 => 345,
      PlusOffsetChoices::UTCPlus6 => 360,
      PlusOffsetChoices::UTCPlus6_30 => 390,
      PlusOffsetChoices::UTCPlus7 => 420,
      PlusOffsetChoices::UTCPlus8 => 480,
      PlusOffsetChoices::UTCPlus8_45 => 525,
      PlusOffsetChoices::UTCPlus9 => 540,
      PlusOffsetChoices::UTCPlus9_30 => 570,
      PlusOffsetChoices::UTCPlus10 => 600,
      PlusOffsetChoices::UTCPlus10_30 => 630,
      PlusOffsetChoices::UTCPlus11 => 660,
      PlusOffsetChoices::UTCPlus12 => 720,
      PlusOffsetChoices::UTCPlus12_45 => 765,
      PlusOffsetChoices::UTCPlus13 => 780,
      PlusOffsetChoices::UTCPlus13_45 => 825,
      PlusOffsetChoices::UTCPlus14 => 840,
    },
    None => 0
  };

  if minus_offset != 0 && plus_offset != 0 {
    ctx.send(|f| f.content(format!("Cannot specify multiple time zones. Please try again with only one offset.")).ephemeral(true)).await?;
    return Ok(());
  } else if minus_offset != 0 {
    let adjusted_datetime = chrono::Utc::now() + Duration::minutes(minus_offset);
    DatabaseHandler::create_meditation_entry(&mut transaction, &guild_id, &user_id, minutes, adjusted_datetime).await?;
  } else if plus_offset != 0 {
    let adjusted_datetime = chrono::Utc::now() + Duration::minutes(plus_offset);
    DatabaseHandler::create_meditation_entry(&mut transaction, &guild_id, &user_id, minutes, adjusted_datetime).await?;
  } else {
    DatabaseHandler::add_minutes(&mut transaction, &guild_id, &user_id, minutes).await?;
  }

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
        //.filter(|c| c.is_alphanumeric() || c.is_whitespace() || c.is_ascii_punctuation() || matches!(c, '’' | '‘' | '“' | '”' | '—' | '…' | 'ā'))
        .filter(|c| !matches!(c, '*'))
        .map(|c| {
          if c.is_ascii_punctuation() {
            if matches!(c, '_' | '~') {
              c.to_string()
            } else {
              format!("\\{c}")
            }
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

      if confirm {
        // Log large add in Bloom logs channel
        let log_embed = BloomBotEmbed::new()
          .title("Large Meditation Entry Added")
          .description(format!(
            "**User**: {}\n**Time**: {} minutes",
          ctx.author(),
            minutes
          ))
          .footer(|f| {
            f.icon_url(ctx.author().avatar_url().unwrap_or_default())
              .text(format!("Added by {}", ctx.author()))
          })
          .to_owned();

        let log_channel = serenity::ChannelId(CHANNELS.bloomlogs);

        log_channel
          .send_message(ctx, |f| f.set_embed(log_embed))
          .await?;
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
