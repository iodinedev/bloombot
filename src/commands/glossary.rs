use crate::commands::BloomBotEmbed;
use crate::database::DatabaseHandler;
use crate::pagination::{PageRowRef, Pagination};
use crate::Context;
use anyhow::Result;
use log::info;
use pgvector;
use poise::serenity_prelude as serenity;

/// Glossary commands
///
/// Commands for interacting with the glossary.
///
/// Get `info` on a glossary entry, see a `list` of entries, or `search` for a relevant entry.
#[poise::command(
  slash_command,
  category = "Informational",
  subcommands("list", "info", "search"),
  subcommand_required,
  guild_only
)]
pub async fn glossary(_: Context<'_>) -> Result<()> {
  Ok(())
}

/// See a list of all glossary entries
///
/// Shows a list of all glossary entries.
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

  if current_page > 0 {
    current_page = current_page - 1
  }

  let entries = DatabaseHandler::get_all_glossary_terms(&mut transaction, &guild_id).await?;
  let entries: Vec<PageRowRef> = entries.iter().map(|entry| entry as _).collect();
  drop(transaction);
  let glossary = Pagination::new("Glossary", entries).await?;

  if glossary.get_page(current_page).is_none() {
    current_page = glossary.get_last_page_number();
  }

  let first_page = glossary.create_page_embed(current_page);

  ctx
    .send(|f| {
      f.components(|b| {
        if glossary.get_page_count() > 1 {
          b.create_action_row(|b| {
            b.create_button(|b| b.custom_id(&prev_button_id).label("Previous"))
              .create_button(|b| b.custom_id(&next_button_id).label("Next"))
          });
        }

        b
      });

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
      current_page = glossary.update_page_number(current_page, 1);
    } else if press.data.custom_id == prev_button_id {
      current_page = glossary.update_page_number(current_page, -1);
    } else {
      // This is an unrelated button interaction
      continue;
    }

    // Update the message with the new page contents
    press
      .create_interaction_response(ctx, |b| {
        b.kind(serenity::InteractionResponseType::UpdateMessage)
          .interaction_response_data(|f| f.set_embed(glossary.create_page_embed(current_page)))
      })
      .await?;
  }

  Ok(())
}

/// See information about a glossary entry
///
/// Shows information about a glossary entry.
#[poise::command(slash_command)]
pub async fn info(
  ctx: Context<'_>,
  #[description = "The term to show information about"] term: String,
) -> Result<()> {
  let guild_id = ctx.guild_id().unwrap();

  let mut transaction = ctx.data().db.start_transaction_with_retry(5).await?;

  let term_info = DatabaseHandler::get_term(&mut transaction, &guild_id, &term.as_str()).await?;
  let mut embed = BloomBotEmbed::new();

  match term_info {
    Some(term_info) => {
      embed.title(term_info.term_name);
      embed.description(term_info.meaning);
      let usage = term_info.usage.unwrap_or(String::new());
      if !usage.is_empty() {
        embed.field("Example of Usage:", usage, false);
      }
      let links = term_info.links.unwrap_or(Vec::new());
      if !links.is_empty() {
        embed.field(
          "Related Resources:",
          {
            let mut field = String::new();
            let mut count = 1;

            for link in links {
              field.push_str(&format!("{}. {}\n", count, link));
              count += 1;
            }

            field
          },
          false,
        );
      }
      let category = term_info.category.unwrap_or(String::new());
      if !category.is_empty() {
        embed.footer(|f| f.text(format!("Categories: {}", category)));
      }
    }
    None => {
      let possible_terms =
        DatabaseHandler::get_possible_terms(&mut transaction, &guild_id, &term.as_str(), 0.7)
          .await?;

      if possible_terms.len() == 1 {
        let possible_term = possible_terms.first().unwrap();

        embed.title(&possible_term.term_name);
        embed.description(&possible_term.meaning);
        let usage = possible_term.usage.clone().unwrap_or(String::new());
        if !usage.is_empty() {
          embed.field("Example of Usage:", usage, false);
        }
        let links = possible_term.links.clone().unwrap_or(Vec::new());
        if !links.is_empty() {
          embed.field(
            "Related Resources:",
            {
              let mut field = String::new();
              let mut count = 1;

              for link in links {
                field.push_str(&format!("{}. {}\n", count, link));
                count += 1;
              }

              field
            },
            false,
          );
        }
        let category = possible_term.category.clone().unwrap_or(String::new());
        if !category.is_empty() {
          embed.footer(|f| {
            f.text(format!(
              "Categories: {}\n\nYour search was `{}`, and it was corrected to `{}`.",
              category, term, possible_term.term_name
            ))
          });
        } else {
          embed.footer(|f| {
            f.text(format!(
              "Your search was `{}`, and it was corrected to `{}`.",
              term, possible_term.term_name
            ))
          });
        }
      } else if possible_terms.is_empty() {
        embed.title("Term not found");
        embed.description(format!(
          "The term `{}` was not found in the glossary.",
          term
        ));
      } else {
        embed.title("Term not found");
        embed.description(format!(
          "The term `{}` was not found in the glossary.",
          term
        ));

        embed.field(
          "Did you mean one of these?",
          {
            let mut field = String::new();

            for possible_term in possible_terms.iter().take(3) {
              field.push_str(&format!("`{}`\n", possible_term.term_name));
            }

            field
          },
          false,
        );

        embed
          .footer(|f| f.text("Try using `/search` to take advantage of a more powerful search."));
      }
    }
  }

  ctx
    .send(|f| {
      f.embeds = vec![embed];

      f
    })
    .await?;

  Ok(())
}

/// Search glossary entries using keywords or phrases
///
/// Searches glossary entries using keywords or phrases, leveraging AI to find the closest matches.
#[poise::command(slash_command)]
pub async fn search(
  ctx: Context<'_>,
  #[description = "The term to search for"] search: String,
) -> Result<()> {
  ctx.defer().await?;

  let data = ctx.data();

  // We unwrap here, because we know that the command is guild-only.
  let guild_id = ctx.guild_id().unwrap();

  let start_time = std::time::Instant::now();
  let mut transaction = data.db.start_transaction_with_retry(5).await?;
  let vector = pgvector::Vector::from(
    data
      .embeddings
      .create_embedding(search.clone(), ctx.author().id)
      .await?,
  );
  let possible_terms =
    DatabaseHandler::search_terms_by_vector(&mut transaction, &guild_id, vector, 3).await?;
  let end_time = std::time::Instant::now();

  let mut embed = BloomBotEmbed::new();
  embed.title(format!("You searched for `{}`", search));

  if possible_terms.is_empty() {
    embed.description("No terms were found. Try browsing the glossary with `/glossary list`.");
  } else {
    for (index, possible_term) in possible_terms.iter().enumerate() {
      if possible_term.distance_score.unwrap_or(1.0) > 0.5 {
        continue;
      }
      let relevance_description = match possible_term.distance_score {
        Some(score) => {
          let similarity_score = ((1.0 - score) * 100.0) as i32;
          info!(
            "Term {} has a similarity score of {}",
            index + 1,
            similarity_score
          );
          match similarity_score {
            100..=i32::MAX => "Exact match",
            80..=99 => "Very similar",
            60..=79 => "Similar",
            40..=59 => "Somewhat similar",
            20..=39 => "Not very similar",
            0..=19 => "Not similar",
            _ => "Unknown",
          }
        }
        None => "Unknown",
      };

      let meaning = match possible_term.meaning.chars().count() > 157 {
        true => {
          let truncate = possible_term.meaning.chars().take(157).collect::<String>();
          let truncate_split = match truncate.rsplit_once(' ') {
            Some(pair) => pair.0.to_string(),
            None => truncate.to_string(),
          };
          let truncate_final = if truncate_split
            .chars()
            .last()
            .unwrap()
            .is_ascii_punctuation()
          {
            truncate_split
              .chars()
              .take(truncate_split.chars().count() - 1)
              .collect::<String>()
          } else {
            truncate_split
          };
          format!("{}...", truncate_final)
        }
        false => possible_term.meaning.clone(),
      };

      embed.field(
        format!("Term {}: `{}`", index + 1, &possible_term.term_name),
        format!(
          "```{}```\n> Estimated relevance: *{}*",
          meaning, relevance_description
        ),
        false,
      );
    }
  }

  embed.footer(|f| {
    f.text(format!(
      "Search took {}ms",
      (end_time - start_time).as_millis()
    ))
  });

  ctx
    .send(|f| {
      f.embeds = vec![embed];

      f
    })
    .await?;

  Ok(())
}
