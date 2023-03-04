// Get the afk Table stored in the SQLite database
import { config } from '../config'
import { EmbedBuilder } from 'discord.js'

export async function checkReport (client, user, reaction): Promise<void> {
  const message = reaction.message
  const channel = client.channels.cache.get(message.channel.id)

  try {
    if (reaction._emoji && reaction._emoji.id === config.emotes.report) {
      await reaction.users.remove(user.id)

      const reportMessage = new EmbedBuilder()
        .setColor(config.embedColor)
        .setAuthor({
          name: `${message.author.username}#${message.author.discriminator}`,
          url: message.author.displayAvatarURL()
        })
        .setDescription(message.content)
        .setFields([
          {
            name: 'Link',
            value: `[Go to message](${message.url})`,
            inline: true
          }
        ])
        .setFooter({
          text: `Reported in #${channel.name} by ${user.username}#${user.discriminator}`
        })
        .setTimestamp(message.createdAt)

      client.channels.cache
        .get(config.channels.reportchannel)
        .send({ content: '<@&788760128010059786>', embeds: [reportMessage] })
        .then(() => {
          user.send(':white_check_mark: Reported to staff.')
        })
    }
  } catch (error: any) {
    const err: string = error.message

    user.send(
      ':x: Error when reporting to staff. Please take a screenshot of the message and DM a staff member.'
    )

    const errorMessage = new EmbedBuilder()
      .setColor(config.embedColor)
      .setTitle('Fatal Error')
      .setDescription(
        `Fatal error has been found when trying to report a message. Error: \`${err}\`.`
      )
      .setFooter({ text: `Action in #${channel.name}` })
      .setTimestamp(message.createdAt)

    message.guild.channels.cache
      .get(config.channels.logs)
      .send({ embeds: [errorMessage] })
  }
}
