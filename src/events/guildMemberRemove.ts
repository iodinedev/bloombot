import { EmbedBuilder } from 'discord.js'
import { config } from '../config'

export = async (client, member) => {
  if (member.user.bot) return

  const embed = new EmbedBuilder()
    .setTitle('Member Left')
    .setColor(config.embedColor)
    .setAuthor({
      name: `${member.user.username}#${member.user.discriminator}`,
      url: member.user.displayAvatarURL()
    })
    .setDescription(`We wish you well on your future endeavors ${member.user.username}#${member.user.discriminator}. :pray:`)
  client.channels.cache.get(config.channels.welcome).send({ embeds: [embed] })
}
