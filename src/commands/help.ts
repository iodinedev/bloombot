import { SlashCommandBuilder, EmbedBuilder } from "discord.js";

export = {
  // Ephemeral to not clutter the chat
	data: new SlashCommandBuilder()
		.setName('help')
		.setDescription('Shows a list of all commands, or info about a specific command.')
    .addStringOption(option =>
      option.setName('command')
        .setDescription('The command you want info about.')
        .setRequired(false))
    .setDMPermission(false),
	async execute(interaction) {
    const command = interaction.options.getString('command');
    if (command) {
      const cmd = interaction.client.commands.get(command);
      if (!cmd) return interaction.reply({ content: 'That\'s not a valid command!', ephemeral: true });
      const embed = new EmbedBuilder()
        .setTitle(`Command: ${cmd.data.name}`)
        .setDescription(cmd.data.description)
        .addFields({ name: 'Usage', value: `\`${cmd.data.name}\`` });
      return interaction.reply({ embeds: [embed], ephemeral: true });
    } else {
      const embed = new EmbedBuilder()
        .setTitle('Commands')
        .setDescription('Here\'s a list of all my commands:')
        .addFields({ name: 'Commands', value: interaction.client.commands.map(cmd => `\`${cmd.data.name}\``).join(', ') });
      return interaction.reply({ embeds: [embed], ephemeral: true });
    }
	},
};