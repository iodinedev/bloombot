use crate::commands::{commit_and_say, course_not_found, MessageType};
use crate::database::DatabaseHandler;
use crate::pagination::{PageRowRef, Pagination};
use crate::Context;
use anyhow::Result;
use poise::serenity_prelude as serenity;

/// Commands for managing courses
/// 
/// Commands to add, edit, list, or remove courses.
/// 
/// Requires `Administrator` permissions.
#[poise::command(
  slash_command,
  required_permissions = "ADMINISTRATOR",
  subcommands("add", "remove", "edit", "list"),
  subcommand_required,
  hide_in_help,
  guild_only
)]
pub async fn course(_: Context<'_>) -> Result<()> {
  Ok(())
}

/// Add a course and its associated graduate role to the database
/// 
/// Adds a course and its associated graduate role to the database.
#[poise::command(slash_command)]
pub async fn add(
  ctx: Context<'_>,
  #[description = "Name of the course"] course_name: String,
  #[description = "The role participants of the course are assumed to have"]
  participant_role: serenity::Role,
  #[description = "Role to be given to graduates"] graduate_role: serenity::Role,
) -> Result<()> {
  ctx.defer_ephemeral().await?;

  let data = ctx.data();

  // We unwrap here, because we know that the command is guild-only.
  let guild_id = ctx.guild_id().unwrap();

  let mut transaction = data.db.start_transaction_with_retry(5).await?;
  if DatabaseHandler::course_exists(&mut transaction, &guild_id, course_name.as_str()).await? {
    ctx.say(":x: Course already exists.").await?;
    return Ok(());
  }

  // Verify that the roles are in the guild
  if !participant_role.guild_id.eq(&guild_id) {
    ctx
      .say(":x: The participant role must be in the same guild as the command.")
      .await?;
    return Ok(());
  }
  if !graduate_role.guild_id.eq(&guild_id) {
    ctx
      .say(":x: The graduate role must be in the same guild as the command.")
      .await?;
    return Ok(());
  }

  // Verify that the roles are not managed by an integration
  if participant_role.managed {
    ctx
      .say(":x: The participant role must not be a bot role.")
      .await?;
    return Ok(());
  }
  if graduate_role.managed {
    ctx
      .say(":x: The graduate role must not be a bot role.")
      .await?;
    return Ok(());
  }

  // Verify that the roles are not privileged
  if participant_role.permissions.administrator() {
    ctx
      .say(":x: The participant role must not be an administrator role.")
      .await?;
    return Ok(());
  }
  if graduate_role.permissions.administrator() {
    ctx
      .say(":x: The graduate role must not be an administrator role.")
      .await?;
    return Ok(());
  }

  if participant_role == graduate_role {
    ctx
      .say(":x: The participant role and the graduate role must not be the same.")
      .await?;
    return Ok(());
  }

  DatabaseHandler::add_course(
    &mut transaction,
    &guild_id,
    course_name.as_str(),
    &participant_role,
    &graduate_role,
  )
  .await?;

  commit_and_say(
    ctx,
    transaction,
    MessageType::TextOnly(":white_check_mark: Course has been added.".to_string()),
    true,
  )
  .await?;

  Ok(())
}

/// Update the roles for an existing course
/// 
/// Updates the roles for an existing course.
#[poise::command(slash_command)]
pub async fn edit(
  ctx: Context<'_>,
  #[description = "Name of the course"] course_name: String,
  #[description = "Update the role that participants of the course are assumed to have"]
  participant_role: Option<serenity::Role>,
  #[description = "Update the role that graduates of the course are given"]
  graduate_role: Option<serenity::Role>,
) -> Result<()> {
  if participant_role.is_none() && graduate_role.is_none() {
    ctx
      .send(|f| f.content(":x: No changes were provided.").ephemeral(true))
      .await?;
    return Ok(());
  }

  let data = ctx.data();

  // We unwrap here, because we know that the command is guild-only.
  let guild_id = ctx.guild_id().unwrap();

  let mut transaction = data.db.start_transaction_with_retry(5).await?;
  let course =
    DatabaseHandler::get_course(&mut transaction, &guild_id, course_name.as_str()).await?;

  if course.is_none() {
    course_not_found(ctx, &mut transaction, guild_id, course_name).await?;
    return Ok(());
  }

  Ok(())
}

/// List all courses
/// 
/// Lists all courses in the database.
#[poise::command(slash_command)]
pub async fn list(
  ctx: Context<'_>,
  #[description = "The page to show"] page: Option<usize>,
) -> Result<()> {
  let data = ctx.data();

  // We unwrap here, because we know that the command is guild-only.
  let guild_id = ctx.guild_id().unwrap();

  let mut transaction = data.db.start_transaction_with_retry(5).await?;

  // Define some unique identifiers for the navigation buttons
  let ctx_id = ctx.id();
  let prev_button_id = format!("{}prev", ctx_id);
  let next_button_id = format!("{}next", ctx_id);

  let mut current_page = page.unwrap_or(0);

  let courses = DatabaseHandler::get_all_courses(&mut transaction, &guild_id).await?;
  let courses: Vec<PageRowRef> = courses.iter().map(|course| course as _).collect();
  drop(transaction);
  let pagination = Pagination::new("Courses", courses).await?;

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

/// Remove a course from the database
/// 
/// Removes a course from the database.
#[poise::command(slash_command)]
pub async fn remove(
  ctx: Context<'_>,
  #[description = "Name of the course"] course_name: String,
) -> Result<()> {
  ctx.defer_ephemeral().await?;

  let data = ctx.data();

  // We unwrap here, because we know that the command is guild-only.
  let guild_id = ctx.guild_id().unwrap();

  let mut transaction = data.db.start_transaction_with_retry(5).await?;
  if !DatabaseHandler::course_exists(&mut transaction, &guild_id, course_name.as_str()).await? {
    ctx.say(":x: Course does not exist.").await?;
    return Ok(());
  }

  DatabaseHandler::remove_course(&mut transaction, &guild_id, course_name.as_str()).await?;

  commit_and_say(
    ctx,
    transaction,
    MessageType::TextOnly(":white_check_mark: Course has been removed.".to_string()),
    true,
  )
  .await?;

  Ok(())
}
