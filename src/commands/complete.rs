use crate::commands::{commit_and_say, course_not_found, MessageType};
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
  guild_only
)]
pub async fn complete(
  ctx: Context<'_>,
  #[description = "The course you have completed"] course_name: String,
) -> Result<()> {
  ctx.defer_ephemeral().await?;

  let data = ctx.data();

  let guild_id = ctx.guild_id().unwrap();

  let mut transaction = data.db.start_transaction_with_retry(5).await?;
  let course =
    DatabaseHandler::get_course(&mut transaction, &guild_id, course_name.as_str()).await?;

  if course.is_none() {
    course_not_found(ctx, &mut transaction, guild_id, course_name).await?;
    return Ok(());
  }

  let course = course.unwrap();

  if !ctx
    .author()
    .has_role(ctx, guild_id, course.participant_role)
    .await?
  {
    ctx.say(format!(":x: You are not in the course: {}.", course_name)).await?;
    return Ok(());
  }

  let guild = ctx.guild().unwrap();
  let mut member = guild.member(ctx, ctx.author().id).await?;
  member.add_role(ctx, course.graduate_role).await?;
  member.remove_role(ctx, course.participant_role).await?;

  commit_and_say(
    ctx,
    transaction,
    MessageType::TextOnly(format!(
      ":tada: Congrats! You are now a graduate of the course: {}!",
      course_name
    )),
    true,
  )
  .await?;

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
