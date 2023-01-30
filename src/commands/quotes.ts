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

    if (quote.length === 0) return interaction.reply({ content: ':x: No quotes found.', ephemeral: true });

    const quoteText = quote[random].quote;
    const quoteAuthor = quote[random].author;

    const quoteEmbed = new EmbedBuilder()
      .setColor(config.embedColor)
      .setDescription(`*${quoteText}*\n- ${quoteAuthor}`);

		await interaction.reply({ embeds: [quoteEmbed] });
	},
};