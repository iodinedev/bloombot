import { SlashCommandBuilder } from "discord.js";
import { userCompleted } from "../helpers/courseCompleted";

export = {
	data: new SlashCommandBuilder()
		.setName('coursecomplete')
		.setDescription('Checks if you completed our Thinkific course and gives you a matching role.')
    .addStringOption(option =>
      option.setName('email')
        .setDescription('The email you used to sign up for the course.')
        .setRequired(true))
		.setDMPermission(false),
	async execute(interaction) {
		await interaction.reply({ content: 'This command is not implemented yet.' });
	},
};