#![warn(clippy::str_to_string)]

mod commands;

use poise::serenity_prelude as serenity;
use std::{collections::HashMap, env::var, sync::Mutex};
use log::info;
use dotenv::dotenv;

// Custom user data passed to all command functions
pub struct Data {
  votes: Mutex<HashMap<String, u32>>,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
  match error {
    poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
    poise::FrameworkError::Command { error, ctx } => {
      info!("Error in command `{}`: {:?}", ctx.command().name, error,);
    }
    error => {
      if let Err(e) = poise::builtins::on_error(error).await {
        info!("Error while handling error: {}", e)
      }
    }
  }
}

#[tokio::main]
async fn main() {
  dotenv().ok();

  let commands = vec![commands::help(), commands::vote(), commands::getvotes()];

  pretty_env_logger::init();

  let options = poise::FrameworkOptions {
    commands,
    prefix_options: poise::PrefixFrameworkOptions {
      ..Default::default()
    },
    on_error: |error| Box::pin(on_error(error)),
    /// This code is run before every command
    pre_command: |ctx| {
      Box::pin(async move {
        info!("Executing command {}...", ctx.command().qualified_name);
      })
    },
    /// Every command invocation must pass this check to continue execution
    command_check: Some(|ctx| {
      Box::pin(async move {
        if ctx.author().id == 123456789 {
          return Ok(false);
        }
        Ok(true)
      })
    }),
    /// Enforce command checks even for owners (enforced by default)
    /// Set to true to bypass checks, which is useful for testing
    skip_checks_for_owners: false,
    event_handler: |_ctx, event, _framework, _data| {
      Box::pin(async move {
        info!("Got an event in event handler: {:?}", event.name());
        Ok(())
      })
    },
    ..Default::default()
  };

  poise::Framework::builder()
    .token(
      var("DISCORD_TOKEN")
        .expect("Missing `DISCORD_TOKEN` env var, see README for more information."),
    )
    .setup(move |ctx, _ready, framework| {
      Box::pin(async move {
        info!("Logged in as {}", _ready.user.name);
        match var("DISCORD_TEST_GUILD") {
          Ok(guild) => {
            info!("Registering commands in test guild {}...", guild);
            
            let guild_id = serenity::GuildId(guild.parse::<u64>().unwrap());

            poise::builtins::register_in_guild(ctx, &framework.options().commands, guild_id).await?
          },
          Err(_) => poise::builtins::register_globally(ctx, &framework.options().commands).await?
        }

        Ok(Data {
          votes: Mutex::new(HashMap::new()),
        })
      })
    })
    .options(options)
    .intents(
      serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT,
    )
    .run()
    .await
    .unwrap();
}