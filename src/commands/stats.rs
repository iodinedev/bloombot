use crate::charts;
use crate::config::BloomBotEmbed;
use crate::database::DatabaseHandler;
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

/// Gets the stats of the server or a specified user.
#[poise::command(slash_command, subcommands("user", "server"), subcommand_required)]
pub async fn stats(_: Context<'_>) -> Result<()> {
  Ok(())
}

/// Gets the stats of a specified user.
#[poise::command(slash_command)]
pub async fn user(
  ctx: Context<'_>,
  #[description = "The user to get the stats of. (Defaults to you)"] user: Option<serenity::User>,
  #[description = "The type of stats to get. (Defaults to minutes)"]
  #[rename = "type"]
  stats_type: Option<StatsType>,
  #[description = "The timeframe to get the stats for. (Defaults to daily)"] timeframe: Option<
    Timeframe,
  >,
) -> Result<()> {
  ctx.defer().await?;

  let data = ctx.data();
  let mut transaction = data.db.start_transaction_with_retry(5).await?;

  let guild_id = ctx.guild_id().unwrap();

  let user = user.unwrap_or_else(|| ctx.author().clone());
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
    .title(format!("Stats for {}", user.tag()))
    .author(|f| {
      f.name(format!("{}'s Stats", user.tag()))
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
          format!("```{}```", stats.timeframe_stats.sum.unwrap()),
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
          format!("```{}```", stats.timeframe_stats.count.unwrap()),
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

  embed.footer(|f| f.text(format!("Current streak: {}", stats.streak)));

  ctx
    .send(|f| {
      f.attachment(serenity::AttachmentType::Path(&file_path));
      f.embeds = vec![embed.to_owned()];

      f
    })
    .await?;

  Ok(())
}

/// Gets the stats of the current guild.
#[poise::command(slash_command)]
pub async fn server(
  ctx: Context<'_>,
  #[description = "The type of stats to get. (Defaults to minutes)"] stats_type: Option<StatsType>,
  #[description = "The timeframe to get the stats for. (Defaults to daily)"] timeframe: Option<
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
          format!("```{}```", stats.timeframe_stats.sum.unwrap()),
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
          format!("```{}```", stats.timeframe_stats.count.unwrap()),
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
