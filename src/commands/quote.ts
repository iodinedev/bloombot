import { EmbedBuilder, SlashCommandBuilder } from "discord.js";
import { database } from "../helpers/database";
import { config } from "../config";

export = {
	data: new SlashCommandBuilder()
		.setName('quote')
		.setDescription('Gets a motivational quote.')
		.setDMPermission(false),
	async execute(interaction) {
    // Get count of quote commands used in the last minute
    const quoteCount = await database.commandUsage.count({
      where: {
        command: 'quote',
        date: {
          gte: new Date(Date.now() - 60000)
        }
      }
    });

    // If quote count is greater than 3, return
    if (quoteCount > 3) return interaction.reply({ content: ':x: You can only use this command 3 times per minute.', ephemeral: true });

    const quote = await database.quoteBook.findMany();
    const random = Math.floor(Math.random() * quote.length);

    if (quote.length === 0) return interaction.reply({ content: ':x: No quotes found.', ephemeral: true });

    const quoteText = quote[random].quote;
    const quoteAuthor = quote[random].author;

    const quoteEmbed = new EmbedBuilder()
      .setColor(config.embedColor)
      .setDescription(`${quoteText}\n\n- ${quoteAuthor}`);

		await interaction.reply({ embeds: [quoteEmbed] });
	},
};