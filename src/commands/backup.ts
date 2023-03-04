import { SlashCommandBuilder } from 'discord.js'
import { adminCommand } from '../helpers/commandPermissions'
import { backup } from '../helpers/backup'

export = {
  data: new SlashCommandBuilder()
    .setName('backup')
    .setDescription('Runs a backup of the database.')
    .setDefaultMemberPermissions(adminCommand())
    .setDMPermission(false),
  async execute (interaction) {
    await interaction.deferReply()

    await backup(interaction.client)

    await interaction.editReply('Backup complete.')
  }
}
