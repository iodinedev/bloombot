import { SlashCommandBuilder, ActionRowBuilder, ButtonBuilder, ButtonStyle } from "discord.js";
import { adminCommand } from "../helpers/commandPermissions";
import { database } from "../helpers/database";
import { makeSearchable } from "../helpers/glossary";

export = {
	data: new SlashCommandBuilder()
		.setName('removecourse')
    .setDescription('Removes a course from the database.')
    .addStringOption(option =>
      option.setName('course')
        .setDescription('The course you want to remove.')
        .setAutocomplete(true)
        .setRequired(true))
    .setDefaultMemberPermissions(adminCommand())
    .setDMPermission(false),
  async autocomplete(interaction) {
    const course: string = interaction.options.getString('course');
    const search: string = makeSearchable(course);

    const terms = await database.courses.findMany({
      where: {
        name: {
          contains: search
        }
      }
    });

    const suggestions = terms.map(term => {
      return {
        name: term.name,
        value: term.name
      };
    })

    await interaction.respond(suggestions)
  },
  async execute(interaction) {
    const course: string = interaction.options.getString('course');
    const search: string = makeSearchable(course);

    // Ensure that the course exists
    const verifyCourse = await database.courses.findUnique({
      where: {
        search: search
      }
    });

    if (!verifyCourse) {
      await interaction.reply('That course does not exist.');
      return;
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

    await interaction.reply({
      content: `Are you sure you want to remove **${course}**? This won't remove any roles users have that have already been assigned.`,
      components: [row],
      ephemeral: true
    });

    const filter = i => i.user.id === interaction.user.id;
    const collector = interaction.channel.createMessageComponentCollector({ filter, time: 15000 });

    collector.on('collect', async i => {
      if (i.customId === 'yes') {
        await database.courses.delete({
          where: {
            search: search
          }
        });

        interaction.editReply({ content: 'Course deleted!', ephemeral: true, components: [] });
      } else if (i.customId === 'no') {
        interaction.editReply({ content: 'Course not deleted.', ephemeral: true });
      }
    })

    collector.on('end', collected => {
      if (collected.size === 0) {
        interaction.editReply({ content: 'You did not respond in time. Course not deleted.', ephemeral: true });
      }
    })
  }
}