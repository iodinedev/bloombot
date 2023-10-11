use crate::config;
use anyhow::Result;
use poise::serenity_prelude::{self as serenity, Context, Member};

enum UpdateType {
  BecamePatron,
  StoppedPending,
}

impl UpdateType {
  fn get_type(old: &Member, new: &Member) -> Option<Self> {
    let patron_role = serenity::RoleId(config::ROLES.patreon);

    if !old.roles.contains(&patron_role) && new.roles.contains(&patron_role) {
      Some(Self::BecamePatron)
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
      UpdateType::BecamePatron => {
        let patron_channel = serenity::ChannelId(config::CHANNELS.patreon);

        patron_channel
          .send_message(&ctx, |m| {
            m.embed(|e| {
              crate::config::BloomBotEmbed::from(e)
                .title("New Patron")
                .description(format!(
                  "Please welcome {} as a new Patron.\n\nThank you for your generosity, it help keeps this server running!",
                  new.user.tag()
                ))
            })
          })
          .await?;
      }
      UpdateType::StoppedPending => {
        let welcome_channel = serenity::ChannelId(config::CHANNELS.welcome);

        welcome_channel
          .send_message(&ctx, |m| {
            m.embed(|e| {
              config::BloomBotEmbed::from(e)
                .title("New Member")
                .description(format!(
                  ":tada: **A new member has arrived!** :tada:\nPlease welcome {} to the Meditation Mind Discord <@&828291690917265418>!\nWe're glad you've joined us. <:aww:578864572979478558>",
                  new.user.tag()
                ))
            })
          })
          .await?;
      }
    }
  }

  Ok(())
}
