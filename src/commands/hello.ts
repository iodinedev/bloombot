import { SlashCommandBuilder } from 'discord.js'

export = {
  data: new SlashCommandBuilder()
    .setName('hello')
    .setDescription('Says hello!')
    .setDMPermission(false),
  async execute (interaction) {
    await interaction.reply({ content: 'Hello, friend!' })
  }
}
