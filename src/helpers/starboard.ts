import { EmbedBuilder } from 'discord.js'
import { config } from '../config'
import { database } from './database'

import type { Message } from 'discord.js'

export async function addStar (client, user, reaction) {
  if (
    reaction._emoji &&
      reaction._emoji.name === config.emotes.star &&
      reaction.message.channel.id != config.channels.starchannel
  ) {
    const stars = reaction.count
    const username = reaction.message.author.username
    const message = reaction.message.content
    const avatar = reaction.message.author.displayAvatarURL()
    const link = `https://discordapp.com/channels/${reaction.message.guild.id}/${reaction.message.channel.id}/${reaction.message.id}`

    const att = reaction.message.attachments

    const result = await database.stars.findUnique({ where: { messageID: reaction.message.id } })

    if (result === null) {
      if (reaction.count >= config.min_stars) {
        const starBoardMessage = new EmbedBuilder()
          .setColor(config.embedColor)
          .setAuthor({
            name: username,
            url: avatar
          })
          .setDescription(
            message + '\n\n**[Click to jump to message.](' + link + ')**'
          )
          .setFooter({ text: '⭐ Times starred: ' + stars })

        if (att.size > 0) {
          const att_arr = Array.from(att, ([name, value]) => value)

          starBoardMessage.setImage(att_arr[0].url)
        }

        const channel = await client.channels.cache.get(
          config.channels.starchannel
        )

        channel.send({ embeds: [starBoardMessage] }).then((sentmessage) => {
          const starObject = {
            id: sentmessage.id,
            messageID: reaction.message.id,
            embedID: sentmessage.id,
            messageChannelID: reaction.message.channel.id
          }

          database.stars.create({ data: starObject }).then(() => {

          })
        })
      }
    } else {
      const starchannel = await reaction.message.guild.channels.cache.find(
        (channel) => config.channels.starchannel === channel.id
      )

      starchannel.messages.fetch(result.embedID).then(async (starmessage) => {
        const starmessageEmbed = starmessage.embeds[0]
        const times = reaction.count

        const starboardMessage = EmbedBuilder.from(starmessage.embeds[0])
          .setFooter({ text: '⭐ Times starred: ' + times.toString() })
        return await starmessage.edit({ embeds: [starboardMessage] })
      })
    }
  }
}

export async function removeStar (client, user, reaction) {
  if (reaction._emoji && reaction._emoji.name === config.emotes.star) {
    const result = await database.stars.findUnique({ where: { messageID: reaction.message.id } })

    if (result !== null) {
      client.channels.cache
        .get(config.channels.starchannel)
        .messages.fetch(result.embedID)
        .then(async (starmessage: Message) => {
          const count = reaction.count;

          if (count >= config.min_stars) {
            const starmessageEmbed = EmbedBuilder.from(starmessage.embeds[0])
              .setFooter({
                text: '⭐ Times starred: ' + count.toString()
              })

            return await starmessage.edit({ embeds: [starmessageEmbed] });
          } else {
            database.stars.delete({ where: { messageID: reaction.message.id } }).then(async () => {
              return await starmessage.delete()
            })
          }
        })
    }
  }
}

export async function removeMessage (client, message) {
  let result = await database.stars.findUnique({ where: { messageID: message.id } })

  if (result !== null) {
    client.channels.cache
      .get(config.channels.starchannel)
      .messages.fetch(result.embedID)
      .then(async (starmessage) => {
        await database.stars.delete({ where: { messageID: message.id } }).then((_) => {
          return starmessage.delete()
        })
      })
  }

  result = await database.stars.findUnique({ where: { embedID: message.id } })

  if (result !== null) {
    await database.stars.delete({ where: { embedID: message.id } }).then(
      client.channels.cache
        .get(result.messageChannelID)
        .messages.fetch(result.messageID)
        .then((starmessage) => {
          starmessage.reactions.removeAll()
        })
    )
  }
}
