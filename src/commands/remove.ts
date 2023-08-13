import { SlashCommandBuilder, ActionRowBuilder, ButtonBuilder, ButtonStyle } from 'discord.js'
import { database } from '../helpers/database'
import { updateRoles } from '../helpers/streaks'

export = {
  data: new SlashCommandBuilder()
    .setName('remove')
    .setDescription('Removes a meditation session by ID.')
    .addIntegerOption(option =>
      option.setName('id')
        .setDescription('The ID of the meditation session you want to remove. Use `/recent` to find them.')
        .setRequired(true))
    .setDMPermission(false),
  async execute (interaction) {
    const meditation_id: number = interaction.options.getInteger('id')

    const meditation = await database.meditations.findUnique({
      where: {
        id: meditation_id
      }
    })

    if ((meditation == null) || meditation.session_user !== interaction.user.id || meditation.session_guild !== interaction.guild.id) {
      return interaction.reply({ content: ':x: Could not find that meditation session.', ephemeral: true })
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
      )

    const date = new Date(meditation.occurred_at)
    const month = date.getUTCMonth() + 1
    const day = date.getUTCDate()
    const year = date.getUTCFullYear()

    interaction.reply({ content: `Are you sure you want to delete your ${meditation.session_time}m session on ${day}/${month}/${year}?`, ephemeral: true, components: [row] })

    const filter = i => i.user.id === interaction.user.id
    const collector = interaction.channel.createMessageComponentCollector({ filter, time: 15000 })

    collector.on('collect', async i => {
      if (i.customId === 'yes') {
        collector.resetTimer()

        await database.meditations.delete({
          where: {
            id: meditation_id
          }
        })

        await updateRoles(interaction.client, interaction.guild, interaction.user)

        interaction.editReply({ content: 'Session deleted!', ephemeral: true, components: [] })
      } else if (i.customId === 'no') {
        collector.resetTimer()

        interaction.editReply({ content: 'Session not deleted.', ephemeral: true, components: [] })
      }
    })

    collector.on('end', collected => {
      if (collected.size === 0) {
        interaction.editReply({ content: 'You did not respond in time. Session not deleted.', ephemeral: true, components: [] })
      }
    })
  }
}
