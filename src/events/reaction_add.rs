use crate::config::{self, CHANNELS, EMOTES};
use crate::database::DatabaseHandler;
use anyhow::Result;
use poise::serenity_prelude::Mentionable;
use poise::serenity_prelude::{ChannelId, Context, CreateEmbed, Reaction, ReactionType, UserId};

pub async fn reaction_add(
  ctx: &Context,
  database: &DatabaseHandler,
  add_reaction: &Reaction,
) -> Result<()> {
  let user = match add_reaction.user_id {
    Some(user) => user,
    None => return Ok(()),
  };

  check_report(ctx, &user, add_reaction).await?;
  add_star(ctx, database, add_reaction).await?;

  Ok(())
}

async fn check_report(ctx: &Context, user: &UserId, reaction: &Reaction) -> Result<()> {
  match reaction.emoji {
    ReactionType::Custom {
      animated: _,
      id,
      name: _,
    } => {
      if id == EMOTES.report {
        // Remove reaction from message
        reaction
          .delete(&ctx)
          .await
          .expect("Failed to remove reaction");

        let report_channel_id = ChannelId(CHANNELS.reportchannel);
        let message = reaction.message(&ctx).await?;
        let message_link = message.link().clone();
        let message_user = message.author;

        report_channel_id
          .send_message(ctx, |m| {
            m.embed(|e| {
              config::BloomBotEmbed::from(e)
                .title("Report")
                .author(|a| {
                  a.name(format!("{}", message_user.tag()))
                    .icon_url(message_user.face())
                })
                .description(message.content.clone())
                .field("Link", format!("[Go to message]({})", message_link), false)
                .footer(|f| {
                  f.text(format!(
                    "Reported in <#{}> by {}",
                    message.channel_id,
                    user.mention()
                  ))
                })
                .timestamp(message.timestamp)
            })
          })
          .await?;

        reaction
          .user(&ctx)
          .await?
          .dm(&ctx, |m| {
            m.embed(|e| {
              config::BloomBotEmbed::from(e)
                .title("Report")
                .description(format!("Your report has been sent to the moderation team."))
            })
          })
          .await?;
      }
    }
    _ => {}
  }

  Ok(())
}

async fn add_star(ctx: &Context, database: &DatabaseHandler, reaction: &Reaction) -> Result<()> {
  match &reaction.emoji {
    ReactionType::Unicode(emoji) => {
      if emoji == &EMOTES.star && reaction.channel_id != CHANNELS.starchannel {
        // Get count of star emoji on message
        let star_count = reaction
          .message(&ctx)
          .await?
          .reactions
          .iter()
          .find(|r| r.reaction_type == ReactionType::Unicode(EMOTES.star.to_string()))
          .map(|r| r.count)
          .unwrap_or(0);

        let mut transaction = database.start_transaction().await?;
        let star_message =
          DatabaseHandler::get_star_message_by_message_id(&mut transaction, &reaction.message_id)
            .await?;

        match star_message {
          Some(star_message) => {
            // Already exists, find the starboard channel
            let starboard_channel = star_message.starred_channel_id;

            // Get the starboard message
            let mut starboard_message = starboard_channel
              .message(&ctx, star_message.starred_message_id)
              .await?;

            let existing_embed = starboard_message.embeds.get(0).unwrap();
            let mut updated_embed: CreateEmbed = existing_embed.clone().into();

            updated_embed.footer(|f| f.text(format!("⭐ Times starred: {}", star_count)));

            match starboard_message
              .edit(ctx, |m| m.set_embed(updated_embed))
              .await
            {
              Ok(_) => (),
              Err(_) => {
                let _ = starboard_channel
                  .delete_message(&ctx, starboard_message.id)
                  .await;

                create_star_message(ctx, &mut transaction, reaction, star_count).await?;
                transaction.commit().await?;
              }
            }
          }
          None => {
            create_star_message(ctx, &mut transaction, reaction, star_count).await?;
            transaction.commit().await?;
          }
        }
      }
    }
    _ => {}
  }

  Ok(())
}

async fn create_star_message(
  ctx: &Context,
  transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
  reaction: &Reaction,
  star_count: u64,
) -> Result<()> {
  if star_count >= config::MIN_STARS {
    let starred_message = reaction.message(&ctx).await?;

    let mut embed = config::BloomBotEmbed::new()
      .author(|a| {
        a.name(starred_message.author.tag())
          .icon_url(starred_message.author.face())
      })
      .description(starred_message.content.clone())
      .field(
        "Link",
        format!(
          "**[Click to jump to message.]({})**",
          starred_message.link().clone()
        ),
        false,
      )
      .footer(|f| f.text(format!("⭐ Times starred: {}", star_count)))
      .to_owned();

    if let Some(attachment) = &starred_message.attachments.first() {
      embed = embed.image(attachment.url.clone()).to_owned();
    }

    let starboard_channel = ChannelId(CHANNELS.starchannel);

    let starboard_message = starboard_channel
      .send_message(ctx, |m| m.set_embed(embed))
      .await?;

    DatabaseHandler::insert_star_message(
      transaction,
      &reaction.message_id,
      &starboard_message.id,
      &reaction.channel_id,
    )
    .await?;
  }

  Ok(())
}
