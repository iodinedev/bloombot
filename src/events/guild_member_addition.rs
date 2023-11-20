/* use crate::config;
use anyhow::Result;
use poise::serenity_prelude::{Context, Member};

pub async fn guild_member_addition(ctx: &Context, new_member: &Member) -> Result<()> {
  new_member.user.direct_message(&ctx, |m| m.embed(|e|
    config::BloomBotEmbed::from(e)
    .title("Welcome to the Meditation Mind community!")
    .description("Here are a few ideas to help get yourself settled in:\n> • Read our guidelines and learn about us in <#1030424719138246667>\n> • Introduce yourself to the community in <#428836907942936587>\n> • If you're new to meditation/mindfulness, check out <#788697102070972427>\n> • Say hello and enjoy casual chat with other members in <#501464482996944909>\n\n*Please note that the server uses Discord's Rules Screening feature.* You will need to agree to the rules to gain access to the server. If you don't see the pop-up, look for the notification bar at the bottom of your screen.\n\nOnce you have access, be sure to visit #Channels & Roles to grab some roles and get access to any channels that interest you.\n\nThanks for joining us. We hope you enjoy your stay!"))).await?;

  Ok(())
} */
