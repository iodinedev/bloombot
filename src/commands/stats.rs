use crate::charts;
use crate::config::{BloomBotEmbed, ROLES};
use crate::database::{DatabaseHandler, TrackingProfile};
use crate::database::Timeframe;
use crate::Context;
use anyhow::Result;
use poise::serenity_prelude as serenity;

#[derive(poise::ChoiceParameter)]
pub enum StatsType {
  #[name = "Minutes"]
  MeditationMinutes,
  #[name = "Count"]
  MeditationCount,
}

#[derive(poise::ChoiceParameter)]
pub enum Privacy {
  #[name = "private"]
  Private,
  #[name = "public"]
  Public,
}

/// Show stats for the server or a user
/// 
/// Shows the stats for the whole server or a specified user.
/// 
/// Defaults to daily minutes for the server or yourself. Optionally specify the user, type (minutes or session count), and/or timeframe (daily, weekly, monthly, or yearly).
#[poise::command(
  slash_command,
  category = "Meditation Tracking",
  subcommands("user", "server"),
  subcommand_required,
  guild_only
)]
pub async fn stats(_: Context<'_>) -> Result<()> {
  Ok(())
}

/// Show the stats for a specified user
/// 
/// Shows the stats for a specified user.
/// 
/// Defaults to daily minutes for yourself. Optionally specify the user, type (minutes or session count), and/or timeframe (daily, weekly, monthly, or yearly).
#[poise::command(slash_command)]
pub async fn user(
  ctx: Context<'_>,
  #[description = "The user to get the stats of (Defaults to you)"] user: Option<serenity::User>,
  #[description = "The type of stats to get (Defaults to minutes)"]
  #[rename = "type"]
  stats_type: Option<StatsType>,
  #[description = "The timeframe to get the stats for (Defaults to daily)"] timeframe: Option<
    Timeframe,
  >,
  #[description = "Set visibility of response (Defaults to public)"] privacy: Option<Privacy>,
) -> Result<()> {
  let data = ctx.data();
  let mut transaction = data.db.start_transaction_with_retry(5).await?;

  let guild_id = ctx.guild_id().unwrap();

  let user = user.unwrap_or_else(|| ctx.author().clone());
  let user_nick_or_name = match user.nick_in(&ctx, guild_id).await {
    Some(nick) => nick,
    None => user.name.clone()
  };

  let tracking_profile = match DatabaseHandler::get_tracking_profile(&mut transaction, &guild_id, &user.id).await? {
    Some(tracking_profile) => tracking_profile,
    None => TrackingProfile { ..Default::default() }
  };

  let privacy = match privacy {
    Some(privacy) => match privacy {
      Privacy::Private => true,
      Privacy::Public => false
    },
    None => tracking_profile.stats_private
  };

  if privacy {
    ctx.defer_ephemeral().await?;
  } else {
    ctx.defer().await?;
  }

  if ctx.author().id != user.id
    && tracking_profile.stats_private
    && !ctx.author().has_role(&ctx, guild_id, ROLES.staff).await? {
    ctx
      .send(|f| {
        f.content(format!(
          "Sorry, {}'s stats are set to private.",
          user_nick_or_name
        ))
        .ephemeral(true)
        .allowed_mentions(|f| f.empty_parse())
      })
      .await?;

      return Ok(());
  }

  let stats_type = stats_type.unwrap_or(StatsType::MeditationMinutes);
  let timeframe = timeframe.unwrap_or(Timeframe::Daily);

  let timeframe_header = match timeframe {
    Timeframe::Yearly => "Years",
    Timeframe::Monthly => "Months",
    Timeframe::Weekly => "Weeks",
    Timeframe::Daily => "Days",
  };

  let stats =
    DatabaseHandler::get_user_stats(&mut transaction, &guild_id, &user.id, &timeframe).await?;

  let mut embed = BloomBotEmbed::new();
  let embed = embed
    .title(format!("Stats for {}", user_nick_or_name))
    .author(|f| {
      f.name(format!("{}'s Stats", user_nick_or_name))
        .icon_url(user.face())
    });

  match stats_type {
    StatsType::MeditationMinutes => {
      embed
        .field(
          "All-Time Meditation Minutes",
          format!("```{}```", stats.all_minutes),
          true,
        )
        .field(
          format!("Minutes The Past 12 {}", timeframe_header),
          format!("```{}```", stats.timeframe_stats.sum.unwrap_or(0)),
          true,
        );
    }
    StatsType::MeditationCount => {
      embed
        .field(
          "All-Time Session Count",
          format!("```{}```", stats.all_count),
          true,
        )
        .field(
          format!("Sessions The Past 12 {}", timeframe_header),
          format!("```{}```", stats.timeframe_stats.count.unwrap_or(0)),
          true,
        );
    }
  }

  let chart_stats =
    DatabaseHandler::get_user_chart_stats(&mut transaction, &guild_id, &user.id, &timeframe)
      .await?;
  let chart_drawer = charts::ChartDrawer::new()?;
  let chart = chart_drawer
    .draw(&chart_stats, &timeframe, &stats_type)
    .await?;
  let file_path = chart.get_file_path();

  embed.image(chart.get_attachment_url());

  //Hide footer if streaks disabled
  if tracking_profile.streaks_active {
    embed.footer(|f| f.text(format!("Current streak: {}", stats.streak)));
  }

  ctx
    .send(|f| {
      f.attachment(serenity::AttachmentType::Path(&file_path));
      f.embeds = vec![embed.to_owned()];

      f
    })
    .await?;

  Ok(())
}

/// Show the stats for the server
/// 
/// Shows the stats for the whole server.
/// 
/// Defaults to daily minutes for yourself. Optionally specify the user, type (minutes or session count), and/or timeframe (daily, weekly, monthly, or yearly).
#[poise::command(slash_command)]
pub async fn server(
  ctx: Context<'_>,
  #[description = "The type of stats to get (Defaults to minutes)"] stats_type: Option<StatsType>,
  #[description = "The timeframe to get the stats for (Defaults to daily)"] timeframe: Option<
    Timeframe,
  >,
) -> Result<()> {
  ctx.defer().await?;

  let data = ctx.data();

  let guild_id = ctx.guild_id().unwrap();

  let stats_type = stats_type.unwrap_or(StatsType::MeditationMinutes);
  let timeframe = timeframe.unwrap_or(Timeframe::Daily);

  let timeframe_header = match timeframe {
    Timeframe::Yearly => "Years",
    Timeframe::Monthly => "Months",
    Timeframe::Weekly => "Weeks",
    Timeframe::Daily => "Days",
  };

  let mut transaction = data.db.start_transaction_with_retry(5).await?;

  let stats = DatabaseHandler::get_guild_stats(&mut transaction, &guild_id, &timeframe).await?;

  let mut embed = BloomBotEmbed::new();
  let embed = embed
    .title(format!("Stats for {}", ctx.guild().unwrap().name))
    .author(|f| {
      f.name(format!("{}'s Stats", ctx.guild().unwrap().name))
        .icon_url(ctx.guild().unwrap().icon_url().unwrap_or_default())
    });

  match stats_type {
    StatsType::MeditationMinutes => {
      embed
        .field(
          "All-Time Meditation Minutes",
          format!("```{}```", stats.all_minutes),
          true,
        )
        .field(
          format!("Minutes The Past 12 {}", timeframe_header),
          format!("```{}```", stats.timeframe_stats.sum.unwrap_or(0)),
          true,
        );
    }
    StatsType::MeditationCount => {
      embed
        .field(
          "All-Time Session Count",
          format!("```{}```", stats.all_count),
          true,
        )
        .field(
          format!("Sessions The Past 12 {}", timeframe_header),
          format!("```{}```", stats.timeframe_stats.count.unwrap_or(0)),
          true,
        );
    }
  }

  let chart_stats =
    DatabaseHandler::get_guild_chart_stats(&mut transaction, &guild_id, &timeframe).await?;
  let chart_drawer = charts::ChartDrawer::new()?;
  let chart = chart_drawer
    .draw(&chart_stats, &timeframe, &stats_type)
    .await?;
  let file_path = chart.get_file_path();

  embed.image(chart.get_attachment_url());

  ctx
    .send(|f| {
      f.attachment(serenity::AttachmentType::Path(&file_path));
      f.embeds = vec![embed.to_owned()];

      f
    })
    .await?;

  Ok(())
}
