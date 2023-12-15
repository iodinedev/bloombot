use crate::config::{BloomBotEmbed, CHANNELS};
use crate::database::DatabaseHandler;
use crate::Context;
use anyhow::Result;
use poise::serenity_prelude as serenity;

/// Indicate that you have completed a course
/// 
/// Indicates that you have completed a course.
/// 
/// Marks the specified course as complete, removing the participant role and awarding the graduate role for that course.
#[poise::command(
  slash_command,
  rename = "coursecomplete",
  hide_in_help,
  dm_only
)]
pub async fn complete(
  ctx: Context<'_>,
  #[description = "The course you have completed"] course_name: String,
) -> Result<()> {
  let data = ctx.data();

  let mut transaction = data.db.start_transaction_with_retry(5).await?;
  
  let course = match DatabaseHandler::get_course_in_dm(&mut transaction, course_name.as_str()).await? {
    Some(course) => course,
    None => {
      ctx.say(format!(":x: Course not found. Please contact server staff for assistance.")).await?;
      return Ok(());
    }
  };
  
  let guild_id = course.guild_id;

  let guild = match ctx.cache().guild(guild_id) {
    Some(guild) => guild,
    None => {
      ctx.say(format!(":x: Can't retrieve server information. Please contact server staff for assistance.")).await?;
      return Ok(());
    }
  };

  let mut member = match guild.member(ctx, ctx.author().id).await {
    Ok(member) => member,
    Err(_) => {
      ctx.say(format!(":x: You don't appear to be a member of the server. If I'm mistaken, please contact server staff for assistance.")).await?;
      return Ok(());
    }
  };

  if !member.user.has_role(ctx, guild_id, course.participant_role).await? {
    ctx.say(format!(":x: You are not in the course: **{}**.", course_name)).await?;
    return Ok(());
  }

  if member.user.has_role(ctx, guild_id, course.graduate_role).await? {
    ctx.say(format!(":x: You have already claimed the graduate role for course: **{}**.", course_name)).await?;
    return Ok(());
  }

  member.add_role(ctx, course.graduate_role).await?;
  member.remove_role(ctx, course.participant_role).await?;

  ctx.say(format!(":tada: Congrats! You are now a graduate of the course: **{}**!", course_name)).await?;

  // Log completion in staff logs
  let log_embed = BloomBotEmbed::new()
    .title("New Course Graduate")
    .description(format!(
      "**User**: <@{}>\n**Course**: {}",
      member.user.id,
      course_name
    ))
    .to_owned();

  let log_channel = serenity::ChannelId(CHANNELS.logs);

  log_channel
    .send_message(ctx, |f| f.set_embed(log_embed))
    .await?;

  Ok(())
}
