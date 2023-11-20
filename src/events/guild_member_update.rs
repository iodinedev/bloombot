use crate::config;
use anyhow::Result;
use poise::serenity_prelude::{self as serenity, Context, Member};

enum UpdateType {
  BecameDonator,
  StoppedPending,
}

impl UpdateType {
  fn get_type(old: &Member, new: &Member) -> Option<Self> {
    let patreon_role = serenity::RoleId(config::ROLES.patreon);
    let kofi_role = serenity::RoleId(config::ROLES.kofi);

    if (!old.roles.contains(&patreon_role) && !old.roles.contains(&kofi_role)) && (new.roles.contains(&patreon_role) || new.roles.contains(&kofi_role)) {
      Some(Self::BecameDonator)
    } else if old.pending && !new.pending {
      Some(Self::StoppedPending)
    } else {
      None
    }
  }
}

pub async fn guild_member_update(
  ctx: &Context,
  old_if_available: &Option<Member>,
  new: &Member,
) -> Result<()> {
  let old = match old_if_available {
    Some(old) => old,
    None => return Ok(()),
  };

  if let Some(update_type) = UpdateType::get_type(old, new) {
    match update_type {
      UpdateType::BecameDonator => {
        let donator_channel = serenity::ChannelId(config::CHANNELS.donators);

        donator_channel
          .send_message(&ctx, |m| {
            m.embed(|e| {
              crate::config::BloomBotEmbed::from(e)
                .title(":tada: New Donator :tada:")
                .description(format!(
                  "Please welcome <@{}> as a new donator.\n\nThank you for your generosity! It help keeps this community alive <:loveit:579017125809881089>",
                  new.user.id
                ))
            })
          })
          .await?;
      }
      UpdateType::StoppedPending => {
        let welcome_channel = serenity::ChannelId(config::CHANNELS.welcome);

        welcome_channel
          .send_message(&ctx, |m| {
            m.content(format!("Please give <@{}> a warm welcome, <@&{}>!", new.user.id, config::ROLES.welcome_team))
              .embed(|e| {
                config::BloomBotEmbed::from(e)
                  .title(":tada: A new member has arrived! :tada:")
                  .description(format!(
                    "Welcome to the Meditation Mind community, <@{}>!\n\nCheck out <id:customize> to grab some roles and customize your community experience.\n\nWe're glad you've joined us! <:aww:578864572979478558>",
                    new.user.id
                  ))
                  .thumbnail("https://meditationmind.org/wp-content/uploads/2020/04/Webp.net-resizeimage-1.png")
            })
          })
          .await?;
      }
    }
  }

  Ok(())
}
