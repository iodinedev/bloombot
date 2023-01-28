import { SlashCommandBuilder, ButtonStyle, ButtonBuilder, ActionRowBuilder } from "discord.js";
import { makeSearchable } from "../helpers/glossary";
import { database } from "../helpers/database";
import { adminCommand } from "../helpers/commandPermissions";

export = {
	data: new SlashCommandBuilder()
		.setName('deleteterm')
		.setDescription('Deletes a term from the glossary.')
    .addStringOption(option =>
      option.setName('term')
        .setDescription('The term you want to update.')
        .setAutocomplete(true)
        .setRequired(true))
    .setDefaultMemberPermissions(adminCommand())
    .setDMPermission(false),
  async autocomplete(interaction) {
    const term: string = interaction.options.getString('term');
    const search: string = makeSearchable(term);

    const terms = await database.glossary.findMany({
      where: {
        search: {
          contains: search
        }
      }
    });

    const suggestions = terms.map(term => {
      return {
        name: term.term,
        value: term.search
      };
    })

    await interaction.respond(suggestions)
  },
  async execute(interaction) {
    const term: string = interaction.options.getString('term');
    const search: string = makeSearchable(term);

    // Ensure that the term exists
    const verifyTerm = await database.glossary.findUnique({
      where: {
        search: search
      }
    });

    if (!verifyTerm) {
      return interaction.reply({ content: 'That term does not exist!', ephemeral: true });
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

    interaction.reply({ content: 'Are you sure you want to delete this term?', ephemeral: true, components: [row] });

    const filter = i => i.user.id === interaction.user.id;
    const collector = interaction.channel.createMessageComponentCollector({ filter, time: 15000 });

    collector.on('collect', async i => {
      if (i.customId === 'yes') {
        await database.glossary.delete({
          where: {
            search: search
          }
        });

        interaction.editReply({ content: 'Term deleted!', ephemeral: true, components: [] });
      } else if (i.customId === 'no') {
        interaction.editReply({ content: 'Term not deleted.', ephemeral: true });
      }
    })

    collector.on('end', collected => {
      if (collected.size === 0) {
        interaction.editReply({ content: 'You did not respond in time. Term not deleted.', ephemeral: true });
      }
    })
  }
};