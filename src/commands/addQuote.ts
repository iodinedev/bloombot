import { SlashCommandBuilder } from 'discord.js'
import { modCommand } from '../helpers/commandPermissions'
import { database } from '../helpers/database'

export = {
  data: new SlashCommandBuilder()
    .setName('addquote')
    .setDescription('Adds a quote to the database.')
    .addStringOption(option =>
      option.setName('quote')
        .setDescription('The quote to add.')
        .setRequired(true))
    .addStringOption(option =>
      option.setName('author')
        .setDescription('The author of the quote.')
        .setRequired(false))
    .setDefaultMemberPermissions(modCommand())
    .setDMPermission(false),
  async execute (interaction) {
    const quote = interaction.options.getString('quote')
    const author = interaction.options.getString('author') || undefined

    await database.quoteBook.create({
      data: {
        quote,
        author
      }
    })

    return interaction.reply({ content: ':white_check_mark: Quote has been added.', ephemeral: true })
  }
}
