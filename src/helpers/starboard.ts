import { EmbedBuilder } from 'discord.js';
import  { config } from '../config';
import { database } from './database';

export class starboardActions {
  static async addStar(client, user, reaction) {
    if (
      reaction._emoji &&
      reaction._emoji.name === config.emotes.star &&
      reaction.message.channel.id != config.channels.starchannel
    ) {
      var stars = reaction.count;
      var username = reaction.message.author.username;
      var message = reaction.message.content;
      var avatar = reaction.message.author.displayAvatarURL();
      var link = `https://discordapp.com/channels/${reaction.message.guild.id}/${reaction.message.channel.id}/${reaction.message.id}`;

      var att = reaction.message.attachments;

      let result = await database.stars.findUnique({ where: { messageID: reaction.message.id} });

      if (result === null) {
        if (reaction.count >= config.min_stars) {
          let starBoardMessage = new EmbedBuilder()
            .setColor(config.embedColor)
            .setAuthor({
              name: username,
              url: avatar,
            })
            .setDescription(
              message + '\n\n**[Click to jump to message.](' + link + ')**'
            )
            .setFooter({ text: '⭐ Times starred: ' + stars });

          if (att.size > 0) {
            const att_arr = Array.from(att, ([name, value]) => value);

            starBoardMessage.setImage(att_arr[0].url);
          }

          let channel = await client.channels.cache.get(
            config.channels.starchannel
          );

          channel.send({ embeds: [starBoardMessage] }).then((sentmessage) => {
            let starObject = {
              messageID: reaction.message.id,
              embedID: sentmessage.id,
              messageChannelID: reaction.message.channel.id,
            };

            database.stars.create({data: starObject}).then(() => {
              return;
            });
          });
        }
      } else {
        const starchannel = await reaction.message.guild.channels.cache.find(
          (channel) => config.channels.starchannel === channel.id
        );

        starchannel.messages.fetch(result.embedID).then(async (starmessage) => {
          var starmessageEmbed = starmessage.embeds[0];
          var times = reaction.count;
          starmessageEmbed.setFooter('⭐ Times starred: ' + times.toString());
          return await starmessage.edit({ embeds: [starmessageEmbed] });
        });
      }
    }
  }

  static async removeStar(client, user, reaction) {
    if (reaction._emoji && reaction._emoji.name === config.emotes.star) {
      let result = await database.stars.findUnique({ where: { messageID: reaction.message.id }});

      if (result !== null) {
        client.channels.cache
          .get(config.channels.starchannel)
          .messages.fetch(result.embedID)
          .then((starmessage) => {
            if (reaction.count > 0) {
              var starmessageEmbed = starmessage.embeds[0];
              var times = starmessageEmbed.footer.text.substring(
                16,
                starmessageEmbed.footer.text.length
              );
              times = reaction.count;
              starmessageEmbed.setFooter(
                '⭐ Times starred: ' + times.toString()
              );
              return starmessage.edit(starmessageEmbed);
            } else {
              database.stars.delete({ where: { messageID: reaction.message.id }}).then(() => {
                return starmessage.delete();
              });
            }
          });
      }
    }
  }

  static async removeMessage(client, message) {
    let result = await database.stars.findUnique({ where: { messageID: message.id }});

    if (result !== null) {
      client.channels.cache
        .get(config.channels.starchannel)
        .messages.fetch(result.embedID)
        .then((starmessage) => {
          database.stars.delete({ where: { messageID: message.id} }).then((_) => {
            return starmessage.delete();
          });
        });
    }

    result = await database.stars.findUnique({ where: { embedID: message.id }});

    if (result !== null) {
      database.stars.delete({where: { embedID: message.id} }).then(
        client.channels.cache
          .get(result.messageChannelID)
          .messages.fetch(result.messageID)
          .then((starmessage) => {
            starmessage.reactions.removeAll();
          })
      );
    }
  }
}
