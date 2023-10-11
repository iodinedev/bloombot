use crate::commands::{commit_and_say, course_not_found, MessageType};
use crate::database::DatabaseHandler;
use crate::Context;
use anyhow::Result;

/// Mark that you have completed a course.
#[poise::command(slash_command, guild_only, hide_in_help)]
pub async fn complete(
  ctx: Context<'_>,
  #[description = "The course you have completed"] course_name: String,
) -> Result<()> {
  let data = ctx.data();

  let guild_id = ctx.guild_id().unwrap();

  let mut transaction = data.db.start_transaction().await?;
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
    course_not_found(ctx, &mut transaction, guild_id, course_name).await?;
    return Ok(());
  }

  commit_and_say(
    ctx,
    transaction,
    MessageType::TextOnly(format!(
      ":white_check_mark: Course {} has been marked as completed.",
      course_name
    )),
    true,
  )
  .await?;

  Ok(())
}
