import { SlashCommandBuilder } from 'discord.js'
import { getStreak } from '../helpers/streaks'
import { config } from '../config'
import { channelGuard } from '../helpers/guards'

export = {
  data: new SlashCommandBuilder()
    .setName('streak')
    .setDescription('Gets your streak or the streak of a specified user.')
    .addUserOption(
      option => option.setName('user')
        .setDescription('The user to get the streak of.')
        .setRequired(false))
    .setDMPermission(false),
  async execute (interaction) {
    if (!(await channelGuard)(interaction, [config.channels.meditation, config.channels.commands], interaction.channelId)) return

    const user = interaction.options.getUser('user') || interaction.user
    const streak = await getStreak(interaction.client, interaction.guild, user)

    if (user === interaction.user) {
		  return interaction.reply({ content: `Your streak is ${streak} days.` })
    } else {
      return interaction.reply({ content: `${user.username}'s streak is ${streak} days.` })
    }
  }
}
