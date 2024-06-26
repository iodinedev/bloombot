use crate::commands::BloomBotEmbed;
use crate::database::DatabaseHandler;
use crate::Context;
use anyhow::Result;
use poise::serenity_prelude::CreateEmbedFooter;

/// See information about a term
///
/// Shows information about a term.
#[poise::command(slash_command, category = "Informational", guild_only)]
pub async fn whatis(
  ctx: Context<'_>,
  #[description = "The term to show information about"] term: String,
) -> Result<()> {
  let guild_id = ctx.guild_id().unwrap();

  let mut transaction = ctx.data().db.start_transaction_with_retry(5).await?;

  let term_info = DatabaseHandler::get_term(&mut transaction, &guild_id, &term.as_str()).await?;
  let mut embed = BloomBotEmbed::new();

  match term_info {
    Some(term_info) => {
      embed = embed.title(term_info.term_name);
      match term_info.meaning.split_once('\n') {
        Some(one_liner) => {
          embed = embed.description(format!(
            "{}\n\n*Use </glossary info:1135659962308243479> for more information.*",
            one_liner.0.to_string()
          ));
        }
        None => {
          embed = embed.description(term_info.meaning);
        }
      };
    }
    None => {
      let possible_terms =
        DatabaseHandler::get_possible_terms(&mut transaction, &guild_id, &term.as_str(), 0.7)
          .await?;

      if possible_terms.len() == 1 {
        let possible_term = possible_terms.first().unwrap();

        embed = embed.title(&possible_term.term_name);
        match &possible_term.meaning.split_once('\n') {
          Some(one_liner) => {
            embed = embed.description(format!(
              "{}\n\n*Use </glossary info:1135659962308243479> for more information.*",
              one_liner.0.to_string()
            ));
          }
          None => {
            embed = embed.description(&possible_term.meaning);
          }
        };

        embed = embed.footer(CreateEmbedFooter::new(format!(
          "*You searched for '{}'. The closest term available was '{}'.",
          term, possible_term.term_name,
        )));
      } else if possible_terms.is_empty() {
        embed = embed
          .title("Term not found")
          .description(format!(
            "The term `{}` was not found in the glossary. If you believe it should be included, use </glossary suggest:1135659962308243479> to suggest it for addition.",
            term
          ));

        ctx
          .send({
            let mut f = poise::CreateReply::default();
            f.embeds = vec![embed];
            f.ephemeral(true)
          })
          .await?;

        return Ok(());
      } else {
        embed = embed.title("Term not found").description(format!(
          "The term `{}` was not found in the glossary.",
          term
        ));

        embed = embed.field(
          "Did you mean one of these?",
          {
            let mut field = String::new();

            for possible_term in possible_terms.iter().take(3) {
              field.push_str(&format!("`{}`\n", possible_term.term_name));
            }

            field.push_str(&format!("\n\n*Try using </glossary search:1135659962308243479> to take advantage of a more powerful search, or use </glossary suggest:1135659962308243479> to suggest the term for addition to the glossary.*"));

            field
          },
          false,
        );

        ctx
          .send({
            let mut f = poise::CreateReply::default();
            f.embeds = vec![embed];
            f.ephemeral(true)
          })
          .await?;

        return Ok(());
      }
    }
  }

  ctx
    .send({
      let mut f = poise::CreateReply::default();
      f.embeds = vec![embed];
      f
    })
    .await?;

  Ok(())
}
