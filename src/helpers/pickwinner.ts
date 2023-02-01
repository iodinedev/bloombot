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
    const deleteButton = new ActionRowBuilder()
      .addComponents(
        new ButtonBuilder()
          .setCustomId('deleteKey')
          .setLabel('Delete')
          .setStyle(ButtonStyle.Danger)
      );

    const moderationChannel = await interaction.client.channels.fetch(config.channels.moderator);
    const moderationEmbed = new EmbedBuilder()
      .setTitle('Key Redeemed')
      .setColor(config.embedColor)
      .setThumbnail(interaction.user.avatarURL())
      .setDescription(`**Key redeemed by ${interaction.user.tag}** To delete the key from the database, click the button below. This won't delete the key from the user's account.`)
      
    await moderationChannel.send({ embeds: [moderationEmbed], components: [deleteButton] });
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

export const deleteKey = async (interaction: any) => {
  if (interaction.customId !== 'deleteKey') return;

  // Interaction user must be admin. adminCommand() returns the appropriate permissions bitfield.
  if (!interaction.member.permissions.has(adminCommand())) return interaction.followUp({ content: ':x: You do not have permission to use this command.', ephemeral: true });

  await interaction.update({ components: [] });

  try {
    const moderationChannel = await interaction.client.channels.fetch(config.channels.moderator);
    const moderationEmbed = new EmbedBuilder()
      .setTitle('Key Deleted')
      .setColor(config.embedColor)
      .setThumbnail(interaction.user.avatarURL())
      .setDescription(`**Key deleted by <@${interaction.user.id}>**`)
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

  return interaction.followUp({ content: ':white_check_mark: Key deleted.', ephemeral: true });
}