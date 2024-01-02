use crate::commands::{commit_and_say, MessageType};
use crate::database::DatabaseHandler;
use crate::pagination::{PageRowRef, Pagination};
use crate::{Context, Data as AppData, Error as AppError};
use anyhow::Result;
use poise::serenity_prelude as serenity;
use poise::Modal;

#[derive(Debug, Modal)]
#[name = "Add a new quote"]
struct AddQuoteModal {
  #[name = "Quote text"]
  #[placeholder = "Input quote text here"]
  #[paragraph]
  #[max_length = 300]
  quote: String,
  #[name = "Author's name"]
  #[placeholder = "Defaults to \"Anonymous\""]
  author: Option<String>,
}

#[derive(Debug, Modal)]
#[name = "Edit a quote"]
struct EditQuoteModal {
  #[name = "Quote text"]
  #[paragraph]
  #[max_length = 300]
  quote: String,
  #[name = "Author's name"]
  author: Option<String>,
}

/// Commands for managing quotes
/// 
/// Commands to list, add, edit, or remove quotes.
///
/// These quotes are used both for the `/quote` command and for motivational messages when a user runs `/add`.
/// 
/// Requires `Manage Roles` permissions.
#[poise::command(
  slash_command,
  required_permissions = "MANAGE_ROLES",
  default_member_permissions = "MANAGE_ROLES",
  category = "Moderator Commands",
  subcommands("list", "add", "edit", "remove"),
  subcommand_required,
  //hide_in_help,
  guild_only
)]
pub async fn quotes(_: poise::Context<'_, AppData, AppError>) -> Result<()> {
  Ok(())
}

/// Add a quote to the database
/// 
/// Adds a quote to the database.
#[poise::command(slash_command)]
pub async fn add(ctx: poise::ApplicationContext<'_, AppData, AppError>) -> Result<()> {
  use poise::Modal as _;

  let quote_data = AddQuoteModal::execute(ctx).await?;

  match quote_data {
    Some(quote_data) => {
      let mut transaction = ctx.data().db.start_transaction_with_retry(5).await?;

      // We unwrap here, because we know that the command is guild-only.
      let guild_id = ctx.guild_id().unwrap();

      DatabaseHandler::add_quote(
        &mut transaction,
        &guild_id,
        quote_data.quote.as_str(),
        quote_data.author.as_deref(),
      )
      .await?;

      commit_and_say(
        poise::Context::Application(ctx),
        transaction,
        MessageType::TextOnly(format!(":white_check_mark: Quote has been added.")),
        true,
      )
      .await?;
    }
    None => {
      ctx
        .send(|f| f.content(":x: No data was provided.").ephemeral(true))
        .await?;
      return Ok(());
    }
  }

  Ok(())
}

/// Edit an existing quote
/// 
/// Edits an existing quote using a modal.
#[poise::command(slash_command)]
pub async fn edit(
  ctx: poise::ApplicationContext<'_, AppData, AppError>,
  #[description = "ID of the quote to edit"] quote_id: String,
) -> Result<()> {
  let mut transaction = ctx.data().db.start_transaction_with_retry(5).await?;

  // We unwrap here, because we know that the command is guild-only.
  let guild_id = ctx.guild_id().unwrap();

  let existing_quote =
    DatabaseHandler::get_quote(&mut transaction, &guild_id, quote_id.as_str()).await?;

  if existing_quote.is_none() {
    ctx.send(|f| f.content(":x: Invalid quote ID.").ephemeral(true))
    .await?;
    return Ok(());
  }

  let existing_quote = existing_quote.unwrap();

  let defaults = EditQuoteModal {
    quote: existing_quote.quote,
    author: existing_quote.author,
  };

  let quote_data = EditQuoteModal::execute_with_defaults(ctx, defaults).await?;

  match quote_data {
    Some(quote_data) => {
      let mut transaction = ctx.data().db.start_transaction_with_retry(5).await?;

      DatabaseHandler::edit_quote(
        &mut transaction,
        &existing_quote.id,
        quote_data.quote.as_str(),
        quote_data.author.as_deref(),
      )
      .await?;

      commit_and_say(
        poise::Context::Application(ctx),
        transaction,
        MessageType::TextOnly(format!(":white_check_mark: Quote has been edited.")),
        true,
      )
      .await?;
    }
    None => {
      ctx
        .send(|f| f.content(":x: No data was provided.").ephemeral(true))
        .await?;
      return Ok(());
    }
  }

  Ok(())
}

/// Remove a quote from the database
/// 
/// Removes a quote from the database.
#[poise::command(slash_command)]
pub async fn remove(
  ctx: Context<'_>,
  #[description = "The quote ID to remove"] id: String,
) -> Result<()> {
  let data = ctx.data();

  // We unwrap here, because we know that the command is guild-only.
  let guild_id = ctx.guild_id().unwrap();

  let mut transaction = data.db.start_transaction_with_retry(5).await?;
  if !DatabaseHandler::quote_exists(&mut transaction, &guild_id, id.as_str()).await? {
    ctx
      .send(|f| f.content(":x: Quote does not exist.").ephemeral(true))
      .await?;
    return Ok(());
  }

  DatabaseHandler::remove_quote(&mut transaction, &guild_id, id.as_str()).await?;

  commit_and_say(
    ctx,
    transaction,
    MessageType::TextOnly(format!(":white_check_mark: Quote has been removed.")),
    true,
  )
  .await?;

  Ok(())
}

/// List all quotes in the database
/// 
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

  if current_page > 0 { current_page = current_page - 1 }

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
