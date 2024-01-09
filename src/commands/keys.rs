use crate::commands::{commit_and_say, MessageType};
use crate::pagination::{PageRowRef, Pagination};
use crate::database::DatabaseHandler;
use crate::Context;
use anyhow::Result;
use poise::serenity_prelude as serenity;

/// Commands for managing Playne keys
/// 
/// Commands to list, add, remove, or use Playne keys.
/// 
/// Requires `Administrator` permissions.
#[poise::command(
  slash_command,
  required_permissions = "ADMINISTRATOR",
  default_member_permissions = "ADMINISTRATOR",
  category = "Admin Commands",
  subcommands("list_keys", "add_key", "remove_key", "use_key", "recipients"),
  //hide_in_help,
  guild_only
)]
pub async fn keys(_: Context<'_>) -> Result<()> {
  Ok(())
}

/// List all Playne keys in the database
/// 
/// Lists all Playne keys in the database.
#[poise::command(
  slash_command,
  rename = "list",
)]
pub async fn list_keys(
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

  let keys = DatabaseHandler::get_all_steam_keys(&mut transaction, &guild_id).await?;
  let keys: Vec<PageRowRef> = keys.iter().map(|key| key as PageRowRef).collect();
  drop(transaction);
  let pagination = Pagination::new("Playne Keys", keys).await?;

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

/// Add a Playne key to the database
/// 
/// Adds a Playne key to the database.
#[poise::command(
  slash_command,
  rename = "add",
)]
pub async fn add_key(
  ctx: Context<'_>,
  #[description = "The Playne key to add"] key: String,
) -> Result<()> {
  let data = ctx.data();

  // We unwrap here, because we know that the command is guild-only.
  let guild_id = ctx.guild_id().unwrap();

  let mut transaction = data.db.start_transaction_with_retry(5).await?;
  if DatabaseHandler::steam_key_exists(&mut transaction, &guild_id, key.as_str()).await? {
    ctx
      .send(|f| f.content(":x: Key already exists.").ephemeral(true))
      .await?;
    return Ok(());
  }

  DatabaseHandler::add_steam_key(&mut transaction, &guild_id, key.as_str()).await?;

  commit_and_say(
    ctx,
    transaction,
    MessageType::TextOnly(":white_check_mark: Key has been added.".to_string()),
    true,
  )
  .await?;

  Ok(())
}

/// Remove a Playne key from the database
/// 
/// Removes a Playne key from the database.
#[poise::command(
  slash_command,
  rename = "remove",
)]
pub async fn remove_key(
  ctx: Context<'_>,
  #[description = "The Playne key to remove"] key: String,
) -> Result<()> {
  let data = ctx.data();

  // We unwrap here, because we know that the command is guild-only.
  let guild_id = ctx.guild_id().unwrap();

  let mut transaction = data.db.start_transaction_with_retry(5).await?;
  if !DatabaseHandler::steam_key_exists(&mut transaction, &guild_id, key.as_str()).await? {
    ctx
      .send(|f| f.content(":x: Key does not exist.").ephemeral(true))
      .await?;
    return Ok(());
  }

  DatabaseHandler::remove_steam_key(&mut transaction, &guild_id, key.as_str()).await?;

  commit_and_say(
    ctx,
    transaction,
    MessageType::TextOnly(format!(":white_check_mark: Key has been removed.")),
    true,
  )
  .await?;

  Ok(())
}

/// Retrieve a Playne key
/// 
/// Selects an unused Playne key from the database, returning it and marking it as used.
#[poise::command(
  slash_command,
  rename = "use",
)]
pub async fn use_key(ctx: Context<'_>) -> Result<()> {
  ctx.defer_ephemeral().await?;

  let data = ctx.data();

  let guild_id = ctx.guild_id().unwrap();

  let mut transaction = data.db.start_transaction_with_retry(5).await?;

  if !DatabaseHandler::unused_key_exists(&mut transaction, &guild_id).await? {
    ctx
      .send(|f| f.content(":x: No unused keys found.").ephemeral(true))
      .await?;
    return Ok(());
  }

  let key = DatabaseHandler::get_key_and_mark_used(&mut transaction, &guild_id).await?;
  let key = key.unwrap();

  ctx
    .send(|f| {
      f.content(format!(":white_check_mark: Key retrieved and marked used: `{}`", key))
        .ephemeral(true)
    })
    .await?;

  Ok(())
}

/// Commands for managing Playne key recipients
/// 
/// Commands to list, add, update, or remove Playne key recipients.
#[poise::command(
  slash_command,
  subcommands("list_recipients", "add_recipient", "update_recipient", "remove_recipient")
)]
pub async fn recipients(_: Context<'_>) -> Result<()> {
  Ok(())
}

/// List all Playne key recipients in the database
/// 
/// Lists all Playne key recipients in the database.
#[poise::command(
  slash_command,
  rename = "list",
)]
pub async fn list_recipients(
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

  let recipients = DatabaseHandler::get_steamkey_recipients(&mut transaction, &guild_id).await?;
  let recipients: Vec<PageRowRef> = recipients.iter().map(|recipient| recipient as PageRowRef).collect();
  drop(transaction);
  let pagination = Pagination::new("Playne Key Recipients", recipients).await?;

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

/// Add a Playne key recipient to the database
/// 
/// Adds a Playne key recipient to the database.
#[poise::command(
  slash_command,
  rename = "add",
)]
pub async fn add_recipient(
  ctx: Context<'_>,
  #[description = "Playne key recipient to add"] recipient: serenity::User,
  #[description = "Received key as challenge prize"] challenge_prize: Option<bool>,
  #[description = "Received key as donator perk"] donator_perk: Option<bool>,
  #[description = "Total number of Playne keys received"] total_keys: i16,
) -> Result<()> {
  let data = ctx.data();

  // We unwrap here, because we know that the command is guild-only.
  let guild_id = ctx.guild_id().unwrap();

  let mut transaction = data.db.start_transaction_with_retry(5).await?;

  if DatabaseHandler::get_steamkey_recipient(&mut transaction, &guild_id, &recipient.id).await?.is_some() {
    ctx
      .send(|f| f.content(":x: Recipient already in database. Use `/keys recipients update` to update.").ephemeral(true))
      .await?;
    return Ok(());
  }

  DatabaseHandler::add_steamkey_recipient(&mut transaction, &guild_id, &recipient.id, challenge_prize, donator_perk, total_keys).await?;

  commit_and_say(
    ctx,
    transaction,
    MessageType::TextOnly(":white_check_mark: Recipient has been added.".to_string()),
    true,
  )
  .await?;

  Ok(())
}

/// Update a Playne key recipient in the database
/// 
/// Updates a Playne key recipient in the database.
#[poise::command(
  slash_command,
  rename = "update",
)]
pub async fn update_recipient(
  ctx: Context<'_>,
  #[description = "Playne key recipient to update"] recipient: serenity::User,
  #[description = "Received key as challenge prize"] challenge_prize: Option<bool>,
  #[description = "Received key as donator perk"] donator_perk: Option<bool>,
  #[description = "Total number of Playne keys received"] total_keys: Option<i16>,
) -> Result<()> {
  if challenge_prize.is_none() && donator_perk.is_none() && total_keys.is_none() {
    ctx
      .send(|f| f.content(":x: No new data supplied. Update aborted.").ephemeral(true))
      .await?;
    return Ok(());
  }

  let data = ctx.data();

  // We unwrap here, because we know that the command is guild-only.
  let guild_id = ctx.guild_id().unwrap();

  let mut transaction = data.db.start_transaction_with_retry(5).await?;
  let steamkey_recipient = DatabaseHandler::get_steamkey_recipient(&mut transaction, &guild_id, &recipient.id).await?;

  match steamkey_recipient {
    Some(steamkey_recipient) => {
      let challenge_prize = match challenge_prize {
        Some(_) => challenge_prize,
        None => steamkey_recipient.challenge_prize
      };
      let donator_perk = match donator_perk {
        Some(_) => donator_perk,
        None => steamkey_recipient.donator_perk
      };
      let total_keys = match total_keys {
        Some(value) => value,
        None => steamkey_recipient.total_keys
      };
      DatabaseHandler::update_steamkey_recipient(&mut transaction, &guild_id, &recipient.id, challenge_prize, donator_perk, total_keys).await?;
    },
    None => {
      ctx
      .send(|f| f.content(":x: Recipient not in database. Use `/keys recipients add` to add a recipient.").ephemeral(true))
      .await?;
      return Ok(());
    }
  }

  commit_and_say(
    ctx,
    transaction,
    MessageType::TextOnly(":white_check_mark: Recipient has been updated.".to_string()),
    true,
  )
  .await?;

  Ok(())
}

/// Remove a Playne key recipient from the database
/// 
/// Removes a Playne key recipient from the database.
#[poise::command(
  slash_command,
  rename = "remove",
)]
pub async fn remove_recipient(
  ctx: Context<'_>,
  #[description = "Playne key recipient to remove"] recipient: serenity::User,
) -> Result<()> {
  let data = ctx.data();

  // We unwrap here, because we know that the command is guild-only.
  let guild_id = ctx.guild_id().unwrap();

  let mut transaction = data.db.start_transaction_with_retry(5).await?;
  if DatabaseHandler::get_steamkey_recipient(&mut transaction, &guild_id, &recipient.id).await?.is_none() {
    ctx
      .send(|f| f.content(":x: Recipient not in database.").ephemeral(true))
      .await?;
    return Ok(());
  }

  DatabaseHandler::remove_steamkey_recipient(&mut transaction, &guild_id, &recipient.id).await?;

  commit_and_say(
    ctx,
    transaction,
    MessageType::TextOnly(format!(":white_check_mark: Recipient has been removed.")),
    true,
  )
  .await?;

  Ok(())
}
