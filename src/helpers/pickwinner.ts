import { ChannelType, EmbedBuilder, ActionRowBuilder, ButtonBuilder, ButtonStyle, hideLinkEmbed, hyperlink } from 'discord.js';
import { config } from '../config';
import { database } from './database';
import { adminCommand } from './commandPermissions';

export const acceptKey = async (interaction: any) => {
  if (interaction.customId !== 'redeemKey') return;

  if (interaction.user.bot) return interaction.followUp({ content: ':x: Bots cannot redeem keys.', ephemeral: true });
  if (interaction.channel.type !== ChannelType.DM) return interaction.followUp({ content: ':x: You must use this in a DM.', ephemeral: true });

  await interaction.update({ components: [] });

  const reservedKey = await database.steamKeys.findFirst({
    where: {
      reserved: interaction.user.id
    }
  });

  if (!reservedKey) {
    // Delete the components from the message
    return interaction.followUp({ content: ':x: You do not have a key reserved.', ephemeral: true });
  }

  await database.steamKeys.update({
    where: {
      key: reservedKey.key
    },
    data: {
      used: true,
      reserved: null
    }
  });

  // Send to moderation channel
  try {
    const moderationChannel = await interaction.client.channels.fetch(config.channels.moderator);
    const moderationEmbed = new EmbedBuilder()
      .setTitle('Key Redeemed')
      .setColor(config.embedColor)
      .setThumbnail(interaction.user.avatarURL())
      .setDescription(`**Key redeemed by ${interaction.user.tag}**`)
      
    await moderationChannel.send({ embeds: [moderationEmbed] });
  } catch (error) {
    console.error(error);
  }

  const link = hyperlink("Redeem your key", `https://store.steampowered.com/account/registerkey?key=${reservedKey.key}`);
  const hiddenLink = hideLinkEmbed(link);

  return interaction.followUp({ content: `Awesome! Here is your key.\n\`\`\`${reservedKey.key}\`\`\`\n${hiddenLink}` });
}

export const cancelKey = async (interaction: any) => {
  if (interaction.customId !== 'cancelKey') return;

  await interaction.update({ components: [] });

  const reservedKey = await database.steamKeys.findFirst({
    where: {
      reserved: interaction.user.id
    }
  });

  if (!reservedKey) return;

  try {
    await database.steamKeys.update({
      where: {
        key: reservedKey.key
      },
      data: {
        reserved: null
      }
    });
  } catch (error) {
    console.error(error);
    return;
  }

  // Send to moderation channel
  try {
    const moderationChannel = await interaction.client.channels.fetch(config.channels.moderator);
    const moderationEmbed = new EmbedBuilder()
      .setTitle('Key Cancelled')
      .setColor(config.embedColor)
      .setThumbnail(interaction.user.avatarURL())
      .setDescription(`**Key cancelled by <@${interaction.user.id}>** Key has been returned to the pool.`)
      .setFields([
        {
          name: 'User',
          value: `<@${interaction.user.id}>`
        }
      ]);

    await moderationChannel.send({ embeds: [moderationEmbed] });
  } catch (error) {
    console.error(error);
  }

  return interaction.followUp({ content: ':white_check_mark: Key cancelled.', ephemeral: true });
}
