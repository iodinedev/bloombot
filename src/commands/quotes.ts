import { EmbedBuilder, SlashCommandBuilder } from "discord.js";
import { database } from "../helpers/database";
import { config } from "../config";

export = {
	data: new SlashCommandBuilder()
		.setName('quote')
		.setDescription('Gets a motivational quote.')
		.setDMPermission(false),
	async execute(interaction) {
    const quote = await database.quoteBook.findMany();
    const random = Math.floor(Math.random() * quote.length);

    if (quote.length === 0) return interaction.reply({ content: ':x: No quotes found. Here\'s a backup: "*And those who were seen dancing were thought to be insane by those who could not hear the music.*"', ephemeral: true });

    const quoteText = quote[random].quote;
    const quoteAuthor = quote[random].author;

    const quoteEmbed = new EmbedBuilder()
      .setTitle('Quote')
      .setColor(config.embedColor)
      .setDescription(`*${quoteText}*`)
      .setFooter({ text: `- ${quoteAuthor}` });

		await interaction.reply({ embeds: [quoteEmbed] });
	},
};