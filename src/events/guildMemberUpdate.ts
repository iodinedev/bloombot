import { config } from '../config'
import { EmbedBuilder } from 'discord.js'
import { rollbar } from '../helpers/rollbar'

export = async (client, oldMember, newMember) => {
  if (
    newMember.roles.cache.some((role) => role.id === config.roles.patreon) &&
    !oldMember.roles.cache.some((role) => role.id === config.roles.patreon)
  ) {
    const patreonMsg = new EmbedBuilder()
      .setColor(config.embedColor)
      .setTitle('🎉 New Patron 🎉')
      .setDescription(
        `Please welcome **<@${newMember.id}>** as a new Patron.\n\nThank you for your generosity, it help keeps this server running!`
      )

    return await client.channels.cache
      .get(config.channels.patreon)
      .send({ embeds: [patreonMsg] })
  }

  if (oldMember.pending && !newMember.pending) {
    try {
      // Add roles and send welcome message to the welcome channel
      newMember.guild.channels.cache
        .get(config.channels.welcome)
        .send(
          `:tada: **A new member has arrived!** :tada:\nPlease welcome <@${newMember.id}> to the Meditation Mind Discord <@&828291690917265418>!\nWe're glad you've joined us. <:aww:578864572979478558>`
        )
        .then((message) => {
          message.react(config.emotes.wave)
        })
    } catch (err: any) {
      rollbar.error(err)
    }
  }
}
