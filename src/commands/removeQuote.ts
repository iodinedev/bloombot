import { SlashCommandBuilder, ActionRowBuilder, ButtonBuilder, ButtonStyle } from "discord.js";
import { modCommand } from "../helpers/commandPermissions";
import { database } from "../helpers/database";

export = {
	data: new SlashCommandBuilder()
		.setName('removequote')
		.setDescription('Removes a quote from the database.')
		.addIntegerOption(option =>
			option.setName('id')
				.setDescription('The ID of the quote to remove. Use /listquotes to get a list of IDs.')
				.setRequired(true))
    .setDefaultMemberPermissions(modCommand())
    .setDMPermission(false),
	async execute(interaction) {
		const id: number = interaction.options.getInteger('id');

		const quoteExists = await database.quoteBook.findFirst({
			where: {
				id: id
			}
		});

		if (!quoteExists) {
			return interaction.reply({ content: ':x: Quote does not exist.', ephemeral: true });
		}

    const row = new ActionRowBuilder()
    .addComponents(
      new ButtonBuilder()
        .setCustomId('yes')
        .setLabel('Yes')
        .setStyle(ButtonStyle.Danger),
      new ButtonBuilder()
        .setCustomId('no')
        .setLabel('No')
        .setStyle(ButtonStyle.Primary)
    );

    interaction.reply({ content: `Are you sure you want to delete this quote?\n\`\`\`${quoteExists.quote}\`\`\``, ephemeral: true, components: [row] });

    const filter = i => i.user.id === interaction.user.id;
    const collector = interaction.channel.createMessageComponentCollector({ filter, time: 15000 });

    collector.on('collect', async i => {
      if (i.customId === 'yes') {
        try {
        await database.quoteBook.delete({
          where: {
            id: id
          }
        });
      } catch (error: any) {
        if (error.code === 'P2025') {
          return interaction.editReply({ content: ':x: Quote does not exist. If this is unexpected, please contact a developer.', ephemeral: true });
        }

        throw error;
      }

        interaction.editReply({ content: 'Quote deleted!', ephemeral: true, components: [] });
      } else if (i.customId === 'no') {
        interaction.editReply({ content: 'Quote not deleted.', ephemeral: true });
      }
    })

    collector.on('end', collected => {
      if (collected.size === 0) {
        interaction.editReply({ content: 'You did not respond in time. Quote not deleted.', ephemeral: true });
      }
    })
	},
};