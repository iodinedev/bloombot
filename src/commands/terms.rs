use crate::commands::{commit_and_say, MessageType};
use crate::database::DatabaseHandler;
use crate::{Context, Data as AppData, Error as AppError};
use anyhow::Result;
use pgvector;
use poise::serenity_prelude as serenity;
use poise::Modal;

#[derive(Debug, Modal)]
#[name = "Add a new term"]
struct AddTermModal {
  // #[name = "The term to add"]
  // #[placeholder = "For acronyms, use the full name here"]
  // term: String,
  #[name = "The definition of the term"]
  #[placeholder = "Include the acronym at the beginning of your definition"]
  #[paragraph]
  #[max_length = 1000]
  definition: String,
  #[name = "An example sentence showing the term in use"]
  example: Option<String>,
  #[name = "The category of the term"]
  category: Option<String>,
  #[name = "Links to further reading, comma separated"]
  links: Option<String>,
  #[name = "Term aliases, comma separated"]
  aliases: Option<String>,
}

#[derive(Debug, Modal)]
#[name = "Edit this term"]
struct UpdateTermModal {
  #[name = "The definition of the term"]
  #[paragraph]
  #[max_length = 1000]
  definition: String,
  #[name = "An example sentence showing the term in use"]
  example: Option<String>,
  #[name = "The category of the term"]
  category: Option<String>,
  #[name = "Links to further reading, comma separated"]
  links: Option<String>,
  #[name = "Term aliases, comma separated"]
  aliases: Option<String>,
}

pub async fn term_not_found(
  ctx: Context<'_>,
  transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
  guild_id: serenity::GuildId,
  term_name: String,
) -> Result<()> {
  let possible_terms =
    DatabaseHandler::get_possible_terms(transaction, &guild_id, term_name.as_str(), 0.8).await?;

  if possible_terms.len() == 1 {
    let possible_term = possible_terms.first().unwrap();

    ctx
      .send(|f| {
        f.content(format!(
          ":x: Term does not exist. Did you mean `{}`?",
          possible_term.term_name
        ))
        .ephemeral(true)
      })
      .await?;
  } else if possible_terms.len() > 1 {
    ctx
      .send(|f| {
        f.content(format!(
          ":x: Term does not exist. Did you mean one of these?\n{}",
          possible_terms
            .iter()
            .map(|term| format!("`{}`", term.term_name))
            .collect::<Vec<String>>()
            .join("\n")
        ))
        .ephemeral(true)
      })
      .await?;
  } else {
    ctx
      .send(|f| f.content(":x: Term does not exist.").ephemeral(true))
      .await?;
  }

  Ok(())
}

/// Commands for managing glossary entries
///
/// Commands to add, remove, or edit glossary entries.
///
/// Requires `Manage Roles` permissions.
#[poise::command(
  slash_command,
  required_permissions = "MANAGE_ROLES",
  default_member_permissions = "MANAGE_ROLES",
  category = "Moderator Commands",
  subcommands("add", "remove", "edit"),
  subcommand_required,
  //hide_in_help,
  guild_only
)]
pub async fn terms(_: poise::Context<'_, AppData, AppError>) -> Result<()> {
  Ok(())
}

/// Add a new term to the glossary
///
/// Adds a new term to the glossary.
#[poise::command(slash_command)]
pub async fn add(
  ctx: poise::ApplicationContext<'_, AppData, AppError>,
  #[description = "The term to add"] term_name: String,
) -> Result<()> {
  use poise::Modal as _;

  let term_data = AddTermModal::execute(ctx).await?;

  match term_data {
    Some(term_data) => {
      let mut transaction = ctx.data().db.start_transaction_with_retry(5).await?;

      // We unwrap here, because we know that the command is guild-only.
      let guild_id = ctx.guild_id().unwrap();

      let links = match term_data.links {
        Some(links) => links.split(",").map(|s| s.trim().to_string()).collect(),
        None => Vec::new(),
      };

      let aliases = match term_data.aliases {
        Some(aliases) => aliases.split(",").map(|s| s.trim().to_string()).collect(),
        None => Vec::new(),
      };

      let vector = pgvector::Vector::from(
        ctx
          .data()
          .embeddings
          .create_embedding(term_name.clone(), ctx.author().id)
          .await?,
      );

      DatabaseHandler::add_term(
        &mut transaction,
        term_name.as_str(),
        term_data.definition.as_str(),
        term_data.example.as_deref(),
        links.as_slice(),
        term_data.category.as_deref(),
        aliases.as_slice(),
        &guild_id,
        vector,
      )
      .await?;

      commit_and_say(
        poise::Context::Application(ctx),
        transaction,
        MessageType::TextOnly(format!(":white_check_mark: Term has been added.")),
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

/// Update an existing term in the glossary
///
/// Updates an existing term in the glossary.
#[poise::command(slash_command)]
pub async fn edit(
  ctx: poise::ApplicationContext<'_, AppData, AppError>,
  #[description = "The term to edit"] term_name: String,
) -> Result<()> {
  let mut transaction = ctx.data().db.start_transaction_with_retry(5).await?;

  // We unwrap here, because we know that the command is guild-only.
  let guild_id = ctx.guild_id().unwrap();

  let existing_term =
    DatabaseHandler::get_term(&mut transaction, &guild_id, term_name.as_str()).await?;

  if existing_term.is_none() {
    term_not_found(
      poise::Context::Application(ctx),
      &mut transaction,
      guild_id,
      term_name,
    )
    .await?;
    return Ok(());
  }

  let existing_term = existing_term.unwrap();
  let links = match existing_term.links {
    Some(links) => Some(links.join(", ")),
    None => None,
  };
  let aliases = match existing_term.aliases {
    Some(aliases) => Some(aliases.join(", ")),
    None => None,
  };

  let existing_definition = existing_term.meaning.clone();

  let defaults = UpdateTermModal {
    definition: existing_term.meaning,
    example: existing_term.usage,
    category: existing_term.category,
    links,
    aliases,
  };

  let term_data = UpdateTermModal::execute_with_defaults(ctx, defaults).await?;

  match term_data {
    Some(term_data) => {
      let mut transaction = ctx.data().db.start_transaction_with_retry(5).await?;

      let links = match term_data.links {
        Some(links) => links.split(",").map(|s| s.trim().to_string()).collect(),
        None => Vec::new(),
      };

      let vector = if term_data.definition != existing_definition {
        Some(pgvector::Vector::from(
          ctx
            .data()
            .embeddings
            .create_embedding(existing_term.term_name, ctx.author().id)
            .await?,
        ))
      } else {
        None
      };

      let aliases = match term_data.aliases {
        Some(aliases) => aliases.split(",").map(|s| s.trim().to_string()).collect(),
        None => Vec::new(),
      };

      DatabaseHandler::edit_term(
        &mut transaction,
        &existing_term.id,
        term_data.definition.as_str(),
        term_data.example.as_deref(),
        links.as_slice(),
        term_data.category.as_deref(),
        aliases.as_slice(),
        vector,
      )
      .await?;

      commit_and_say(
        poise::Context::Application(ctx),
        transaction,
        MessageType::TextOnly(format!(":white_check_mark: Term has been edited.")),
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

/// Remove a term from the glossary
///
/// Removes a term from the glossary.
#[poise::command(slash_command)]
pub async fn remove(
  ctx: Context<'_>,
  #[description = "The term to remove"] term: String,
) -> Result<()> {
  let data = ctx.data();

  // We unwrap here, because we know that the command is guild-only.
  let guild_id = ctx.guild_id().unwrap();

  let mut transaction = data.db.start_transaction_with_retry(5).await?;
  if !DatabaseHandler::term_exists(&mut transaction, &guild_id, term.as_str()).await? {
    ctx
      .send(|f| f.content(":x: Term does not exist.").ephemeral(true))
      .await?;
    return Ok(());
  }

  DatabaseHandler::remove_term(&mut transaction, term.as_str(), &guild_id).await?;

  commit_and_say(
    ctx,
    transaction,
    MessageType::TextOnly(format!(":white_check_mark: Term has been removed.")),
    true,
  )
  .await?;

  Ok(())
}
