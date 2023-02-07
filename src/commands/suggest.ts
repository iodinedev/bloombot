import { SlashCommandBuilder, EmbedBuilder } from "discord.js";
import { config } from "../config";

export = {
	data: new SlashCommandBuilder()
		.setName('suggest')
		.setDescription('Adds a suggestion to the server suggestions channel.')
    .addStringOption(option =>
      option.setName('suggestion')
        .setDescription('The suggestion you want to add.')
        .setRequired(true))
		.setDMPermission(false),
	async execute(interaction) {
    const suggestion = interaction.options.getString('suggestion');
    const suggestionChannel = interaction.guild.channels.fetch(config.channels.suggestion);

    const suggestionEmbed = new EmbedBuilder()
      .setColor(config.embedColor)
      .setDescription(suggestion);

    // Log in staff channel
    try {
      const staffChannel = await interaction.guild.channels.fetch(config.channels.logs);
      const staffEmbed = new EmbedBuilder()
        .setTitle('New Suggestion')
        .setColor(config.embedColor)
        .setDescription(suggestion)
        .setAuthor({ name: interaction.user.tag, iconURL: interaction.user.avatarURL() });

      await staffChannel.send({ embeds: [staffEmbed] });
    } catch(error: any) {
      // Channel not found, missing permissions, or cannot send messages
      if (error.code === 10003 || error.code === 50001 || error.code === 50013) {
        return interaction.reply({ content: 'Could not create suggestion.', ephemeral: true });
      } else {
        throw error;
      }
    }

    const suggestionMessage = await suggestionChannel.send({ embeds: [suggestionEmbed] });

    await suggestionMessage.react('✅');
    await suggestionMessage.react('❌');

    // Create a thread for discussion
    await suggestionMessage.startThread({
      name: 'Discussion',
      autoArchiveDuration: 60,
      reason: 'Suggestion thread'
    });

    await interaction.reply({ content: 'Your suggestion has been sent!', ephemeral: true });
	},
};