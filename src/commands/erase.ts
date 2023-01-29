import { EmbedBuilder, SlashCommandBuilder } from "discord.js";
import { modCommand } from "../helpers/commandPermissions";
import { config } from "../config";

export = {
	data: new SlashCommandBuilder()
		.setName('erase')
		.setDescription('Deletes a message.')
    .addStringOption(option =>
      option.setName('message')
        .setDescription('The message ID of the message you want to delete.')
        .setRequired(true))
    .addStringOption(option =>
      option.setName('reason')
        .setDescription('The reason for deleting the message.')
        .setRequired(false))
    .setDefaultMemberPermissions(modCommand())
    .setDMPermission(false),
	async execute(interaction) {
		const messageID: string = interaction.options.getString('message');
    const reason: string = interaction.options.getString('reason') || "No reason provided.";

    var message;

    try {
      message = await interaction.channel.messages.fetch(messageID);
    } catch (error: any) {
      if (error.code === 10008) {
        return interaction.reply({ content: 'That message does not exist!', ephemeral: true });
      } else if (error.code === 50035) {
        return interaction.reply({ content: 'Please provide a message ID.', ephemeral: true });
      } else {
        return interaction.reply({ content: 'An unknown error occurred.', ephemeral: true });
      }
    }

    if (!message) {
      return interaction.reply({ content: 'That message does not exist!', ephemeral: true });
    }

    try {
      await message.delete();
    } catch (error) {
      console.error(error);
      return interaction.reply({ content: 'Message could not be deleted.', ephemeral: true });
    }

    // This is to escape the ` character.
    const sanitized = message.content.replace(/`/g, '\\`');
    const images = message.attachments.map(attachment => attachment.url);

    const embed = new EmbedBuilder()
      .setTitle('A message you sent was deleted.')
      .setColor(config.embedColor)
      .setDescription(`**Reason:** ${reason}`)
      .addFields({ name: 'Message Content', value: `\`\`\`${sanitized}\`\`\`` });

    const imageEmbeds = images.map(image => {
      return new EmbedBuilder()
        .setTitle('An image you sent was deleted.')
        .setColor(config.embedColor)
        .setImage(image);
    });

    try {
      await message.author.send({ embeds: [embed, ...imageEmbeds] });
      await interaction.reply({ content: 'Message deleted and user notified.', ephemeral: true });
    } catch (error) {
      await interaction.reply({ content: 'Message deleted but user could not be notified.', ephemeral: true });
    } finally {
      // Logs the message
      const logEmbed = new EmbedBuilder()
        .setTitle('Message Deleted')
        .setColor(config.embedColor)
        .setDescription(`**Message ID:** \`${messageID}\`\n**Reason:** \`${reason}\`\n**Channel:** <#${message.channel.id}>\n**Author:** ${message.author.tag} (\`${message.author.id}\`)`)
        .addFields({ name: 'Message Content', value: `\`\`\`${sanitized}\`\`\`` })
        .setTimestamp(new Date())
        .setFooter({ text: `Deleted by ${interaction.user.tag}`, iconURL: interaction.user.avatarURL() });

      const imageLogEmbeds = images.map(image => {
        return new EmbedBuilder()
          .setTitle('Image Deleted')
          .setColor(config.embedColor)
          .setImage(image)
          .setTimestamp(new Date())
          .setFooter({ text: `Deleted by ${interaction.user.tag}`, iconURL: interaction.user.avatarURL() });
      });

      const logChannel = interaction.guild.channels.cache.get(config.channels.logs);

      try {
        return logChannel.send({ embeds: [logEmbed, ...imageLogEmbeds] });
      } catch {
        return interaction.followUp({ content: 'Message not logged.', ephemeral: true });
      }
    }
	},
};