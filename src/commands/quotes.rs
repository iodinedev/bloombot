use crate::commands::{commit_and_say, MessageType};
use crate::database::DatabaseHandler;
use crate::pagination::{PageRowRef, Pagination};
use crate::Context;
use anyhow::Result;
use poise::serenity_prelude as serenity;

/// Manage the list of quotes.
///
/// These quotes are both used for the `/quote` command and for motivation messages when a user runs `/add`
#[poise::command(
  slash_command,
  required_permissions = "MANAGE_ROLES",
  subcommands("list", "add"),
  subcommand_required
)]
pub async fn quotes(_: Context<'_>) -> Result<()> {
  Ok(())
}

/// Adds a quote to the database.
#[poise::command(slash_command)]
pub async fn add(
  ctx: Context<'_>,
  #[description = "The quote to add"] quote: String,
  #[description = "The author of the quote [defaults to 'Anonymous']"] author: Option<String>,
) -> Result<()> {
  let data = ctx.data();

  // We unwrap here, because we know that the command is guild-only.
  let guild_id = ctx.guild_id().unwrap();

  let mut transaction = data.db.start_transaction_with_retry(5).await?;
  DatabaseHandler::add_quote(
    &mut transaction,
    &guild_id,
    quote.as_str(),
    author.as_deref(),
  )
  .await?;

  commit_and_say(
    ctx,
    transaction,
    MessageType::TextOnly(format!(":white_check_mark: Quote has been added.")),
    true,
  )
  .await?;

  Ok(())
}

/// Lists all quotes in the database.
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

  let quotes = DatabaseHandler::get_all_quotes(&mut transaction, &guild_id).await?;
  let quotes: Vec<PageRowRef> = quotes.iter().map(|quote| quote as PageRowRef).collect();
  drop(transaction);
  let pagination = Pagination::new("Quotes", quotes).await?;

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
