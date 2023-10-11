use anyhow::{Context as ErrorContext, Error, Result};
use commands::{
  add::add, add_key::add_key, coffee::coffee, complete::complete, courses::course, erase::erase,
  glossary::glossary, hello::hello, list_keys::list_keys, manage::manage, pick_winner::pick_winner,
  ping::ping, quote::quote, quotes::quotes, recent::recent, remove_entry::remove_entry,
  remove_key::remove_key, remove_quote::remove_quote, stats::stats, streak::streak,
  suggest::suggest, terms::terms, use_key::use_key,
};
use config::CHANNELS;
use dotenv::dotenv;
use log::{error, info};
use poise::serenity_prelude::{self as serenity, channel};
use poise::Event;
use pretty_env_logger;
use rand::rngs::SmallRng;
use rand::SeedableRng;
use std::sync::Arc;
use tokio::sync::Mutex;

mod charts;
mod commands;
mod config;
mod database;
mod embeddings;
mod events;
mod pagination;

pub struct Data {
  pub db: database::DatabaseHandler,
  pub rng: Arc<Mutex<SmallRng>>,
  pub embeddings: Arc<embeddings::OpenAIHandler>,
}
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[tokio::main]
async fn main() -> Result<()> {
  dotenv().ok();

  pretty_env_logger::init();

  let token =
    std::env::var("DISCORD_TOKEN").with_context(|| "Missing DISCORD_TOKEN environment variable")?;
  let test_guild = std::env::var("TEST_GUILD_ID");

  let framework = poise::Framework::builder()
    .options(poise::FrameworkOptions {
      commands: vec![
        add_key(),
        add(),
        coffee(),
        complete(),
        course(),
        erase(),
        glossary(),
        hello(),
        list_keys(),
        manage(),
        pick_winner(),
        ping(),
        quote(),
        quotes(),
        recent(),
        remove_entry(),
        remove_key(),
        remove_quote(),
        stats(),
        streak(),
        suggest(),
        terms(),
        use_key(),
      ],
      event_handler: |_ctx, event, _framework, _data| {
        Box::pin(event_handler(_ctx, event, _framework, _data))
      },
      on_error: |error| {
        Box::pin(async move {
          error_handler(error).await;
        })
      },
      ..Default::default()
    })
    .token(token)
    .intents(serenity::GatewayIntents::non_privileged())
    .setup(|ctx, _ready, framework| {
      Box::pin(async move {
        if let Ok(test_guild) = test_guild {
          info!("Registering commands in test guild {}", test_guild);

          let guild_id = serenity::GuildId(test_guild.parse::<u64>()?);
          poise::builtins::register_in_guild(ctx, &framework.options().commands, guild_id).await?;
        } else {
          poise::builtins::register_globally(ctx, &framework.options().commands).await?;
        }
        Ok(Data {
          db: database::DatabaseHandler::new().await?,
          rng: Arc::new(Mutex::new(SmallRng::from_entropy())),
          embeddings: Arc::new(embeddings::OpenAIHandler::new()?),
        })
      })
    });

  Ok(framework.run().await?)
}

async fn error_handler(error: poise::FrameworkError<'_, Data, Error>) {
  match error {
    poise::FrameworkError::Command { ctx, error } => {
      match ctx.say("An error occurred while running the command").await {
        Ok(_) => {}
        Err(e) => {
          error!("While handling error, could not send message: {}", e);
        }
      };

      let command = ctx.command();
      let channel_id = ctx.channel_id();
      let channel = match channel_id.to_channel(ctx).await {
        Ok(channel) => Some(channel),
        Err(_) => {
          error!("While handling error, could not get channel {}", channel_id);
          None
        }
      };

      // Whether it's a guild or DM channel
      let source = match &channel {
        Some(channel) => match channel {
          channel::Channel::Guild(_) => {
            let guild_name = match ctx.guild() {
              Some(guild) => guild.name,
              None => "unknown".to_string(),
            };
            format!("{} ({})", guild_name, channel.id())
          }
          channel::Channel::Private(_) => "DM".to_string(),
          channel::Channel::Category(_) => "category".to_string(),
          _ => "unknown".to_string(),
        },
        None => "unknown".to_string(),
      };
      let user = ctx.author();

      error!(
        "\x1B[1m/{}\x1B[0m failed with error: {:?}",
        command.name, error
      );
      error!("\tSource: {}", source);

      if let Some(channel) = channel {
        error!("\tChannel: {}", channel.id());
      }

      error!("\tUser: {}#{}", user.name, user.discriminator);
    }
    error => {
      if let Err(e) = poise::builtins::on_error(error).await {
        println!("Error while handling error: {}", e)
      }
    }
  }
}

async fn event_handler(
  ctx: &serenity::Context,
  event: &Event<'_>,
  _framework: poise::FrameworkContext<'_, Data, Error>,
  data: &Data,
) -> Result<(), Error> {
  let database = &data.db;

  match event {
    Event::GuildMemberAddition { new_member } => {
      events::guild_member_addition(ctx, new_member).await?;
    }
    Event::GuildMemberRemoval { user, .. } => {
      events::guild_member_removal(ctx, user).await?;
    }
    Event::GuildMemberUpdate {
      old_if_available,
      new,
    } => {
      events::guild_member_update(ctx, old_if_available, new).await?;
    }
    Event::MessageDelete {
      channel_id: _,
      deleted_message_id,
      guild_id: _,
    } => {
      events::message_delete(database, deleted_message_id).await?;
    }
    Event::ReactionAdd { add_reaction } => {
      events::reaction_add(ctx, database, add_reaction).await?;
    }
    Event::ReactionRemove { removed_reaction } => {
      events::reaction_remove(ctx, database, removed_reaction).await?;
    }
    Event::Ready { .. } => {
      let channel = serenity::ChannelId(CHANNELS.welcome);
      channel
        .send_message(&ctx, |m| {
          m.embed(|e| {
            config::BloomBotEmbed::from(e)
              .title("Bot Online")
              .description("The bot is now online and ready to receive commands.")
          })
        })
        .await?;
    }
    _ => {}
  }
  Ok(())
}
