import { ChatInputCommandInteraction, SlashCommandBuilder } from 'discord.js'
import { adminCommand } from '../helpers/commandPermissions'
import { database } from '../helpers/database'

export = {
  data: new SlashCommandBuilder()
    .setName('addkey')
    .setDescription('Adds a Playne key to the database.')
    .addStringOption(option =>
      option.setName('key')
        .setDescription('The key to add.')
        .setRequired(true))
    .setDefaultMemberPermissions(adminCommand())
    .setDMPermission(false),
  async execute (interaction: ChatInputCommandInteraction) {
    const key: string = interaction.options.getString('key') ?? ''

    const keyExists = await database.steamKeys.findFirst({
      where: {
        key
      }
    })

    if (keyExists != null) {
      return interaction.reply({ content: ':x: Key already exists.', ephemeral: true })
    }

    await database.steamKeys.create({
      data: {
        key,
        used: false
      }
    })

    return interaction.reply({ content: ':white_check_mark: Key has been added.', ephemeral: true })
  }
}
