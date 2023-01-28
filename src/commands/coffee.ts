import { SlashCommandBuilder } from "discord.js";

export = {
	data: new SlashCommandBuilder()
		.setName('coffee')
		.setDescription('I will choose either ☕ or ⚰️. (My version of Russian Roulette)'),
	async execute(interaction) {
		await interaction.reply({ content: Math.random() < 0.5 ? ':coffee:' : ':coffin:' });
	},
};