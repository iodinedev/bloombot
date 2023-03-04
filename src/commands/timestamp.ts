import { SlashCommandBuilder } from 'discord.js'
import { modCommand } from '../helpers/commandPermissions'

export = {
  data: new SlashCommandBuilder()
    .setName('timestamp')
    .setDescription('Generates a UNIX timestamp. Hour, minute, and second individually default to 0.')
    .addIntegerOption(option =>
      option.setName('year')
        .setDescription('The year of the timestamp.')
        .setRequired(true))
    .addIntegerOption(option =>
      option.setName('month')
        .setDescription('The month of the timestamp.')
        .setRequired(true))
    .addIntegerOption(option =>
      option.setName('day')
        .setDescription('The day of the timestamp.')
        .setRequired(true))
    .addIntegerOption(option =>
      option.setName('hour')
        .setDescription('The hour of the timestamp.')
        .setRequired(false))
    .addIntegerOption(option =>
      option.setName('minute')
        .setDescription('The minute of the timestamp.')
        .setRequired(false))
    .addIntegerOption(option =>
      option.setName('second')
        .setDescription('The second of the timestamp.')
        .setRequired(false))
    .setDefaultMemberPermissions(modCommand())
    .setDMPermission(false),
  async execute (interaction) {
    const year = interaction.options.getInteger('year')
    const month = interaction.options.getInteger('month') - 1
    const day = interaction.options.getInteger('day')
    const hour = interaction.options.getInteger('hour') || 0
    const minute = interaction.options.getInteger('minute') || 0
    const second = interaction.options.getInteger('second') || 0

    const date = new Date(year, month, day, hour, minute, second)
    const timestamp = date.getTime()

    return interaction.reply({ content: `${timestamp}`, ephemeral: true })
  }
}
