import { SlashCommandBuilder, ActionRowBuilder, ButtonBuilder, ButtonStyle } from "discord.js";
import { adminCommand } from "../helpers/commandPermissions";
import { database } from "../helpers/database";

export = {
	data: new SlashCommandBuilder()
		.setName('resetstats')
		.setDescription('Resets a user\'s meditation stats.')
    .addUserOption(
      option => option.setName('user')
      .setDescription('The user to reset the stats for.')
      .setRequired(true))
    .setDefaultMemberPermissions(adminCommand())
    .setDMPermission(false),
	async execute(interaction) {
		const user = interaction.options.getUser('user');

    const data = await database.meditations.aggregate({
      where: {
        session_user: user.id,
        session_guild: interaction.guild.id
      },
      _sum: {
        session_time: true
      },
      _count: true
    });

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

    interaction.reply({ content: `Are you sure you want to delete this user's stats? They have ${data._sum.session_time} minutes and ${data._count} sessions.`, components: [row], ephemeral: true });

    const filter = i => i.user.id === interaction.user.id;
    const collector = interaction.channel.createMessageComponentCollector({ filter, time: 15000 });

    collector.on('collect', async i => {
      if (i.customId === 'yes') {
        await database.meditations.deleteMany({
          where: {
            session_user: user.id,
            session_guild: interaction.guild.id
          }
        });

        interaction.editReply({ content: 'User cleared!', ephemeral: true, components: [] });
      } else if (i.customId === 'no') {
        interaction.editReply({ content: 'Nothing has been changed.', ephemeral: true });
      }
    })

    collector.on('end', collected => {
      if (collected.size === 0) {
        interaction.editReply({ content: 'You did not respond in time. Nothing has been changed.', ephemeral: true });
      }
    })

	},
};