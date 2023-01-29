import { SlashCommandBuilder, EmbedBuilder, APIEmbedField, PermissionsBitField } from "discord.js";
import { modCommand, adminCommand } from "../helpers/commandPermissions";

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
      // Only get commands the user can use cmd.data.default_member_permissions
      const admin_commands = interaction.client.commands.filter(cmd => new PermissionsBitField(cmd.data.default_member_permissions).has(new PermissionsBitField(adminCommand())) && interaction.member.permissions.has(cmd.data.default_member_permissions));
      const mod_commands = interaction.client.commands.filter(cmd => new PermissionsBitField(cmd.data.default_member_permissions).has(new PermissionsBitField(modCommand())) && interaction.member.permissions.has(cmd.data.default_member_permissions));
      const commands = interaction.client.commands.filter(cmd => cmd.data.default_member_permissions === undefined);

      const fields: APIEmbedField[] = [];

      if (admin_commands.size > 0) {
        fields.push({ name: 'Admin Commands', value: admin_commands.map(cmd => `\`${cmd.data.name}\``).join(', ') });
      }

      if (mod_commands.size > 0) {
        fields.push({ name: 'Mod Commands', value: mod_commands.map(cmd => `\`${cmd.data.name}\``).join(', ') });
      }

      if (commands.size > 0) {
        fields.push({ name: 'Commands', value: commands.map(cmd => `\`${cmd.data.name}\``).join(', ') });
      }

      const embed = new EmbedBuilder()
        .setTitle('Commands')
        .setDescription('Here\'s a list of all my commands:')
        .addFields(fields);
      return interaction.reply({ embeds: [embed], ephemeral: true });
    }
	},
};