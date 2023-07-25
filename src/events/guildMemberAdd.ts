import { EmbedBuilder } from 'discord.js'
import { config } from '../config'
import { rollbar } from '../helpers/rollbar'

export = async (
  client,
  member
) => {
  if (member.user.bot) return

  const embed = new EmbedBuilder()
    .setTitle('<:MeditationMind:694481715174965268> **Welcome to the Meditation Mind community!** <:MeditationMind:694481715174965268>')
    .setColor(config.embedColor)
    .setDescription(`Here are a few ideas to help get yourself settled in:
> • Read our guidelines and learn about us in <#1030424719138246667>
> • Introduce yourself to the community in <#428836907942936587>
> • If you're new to meditation/mindfulness, check out <#788697102070972427>
> • Say hello and enjoy casual chat with other members in <#501464482996944909>
    
*Please note that the server uses Discord's Rules Screening feature.* You will need to agree to the rules to gain access to the server. If you don't see the pop-up, look for the notification bar at the bottom of your screen.
    
Once you have access, be sure to visit #Channels & Roles to grab some roles and get access to any channels that interest you.
    
Thanks for joining us. We hope you enjoy your stay!`)

  try {
    return await member.send({ embeds: [embed] });
  } catch (err: any) {
    if (err.code === 50007) {
      // User has DMs disabled
      return;
    }

    rollbar.error(err);
  }
}
