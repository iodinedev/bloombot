import { database } from '../helpers/database'
import Discord from 'discord.js'
import { modCommand } from '../helpers/commandPermissions'
import { clean } from '../helpers/strings'

export = {
  data: new Discord.SlashCommandBuilder()
    .setName('listquotes')
    .setDescription('Shows a list of all quotes.')
    .addIntegerOption(option =>
      option.setName('id')
        .setDescription('The ID of the quote you want to see.')
        .setRequired(false))
    .addIntegerOption(option =>
      option.setName('page')
        .setDescription('The page you want to see.')
        .setRequired(false))
    .setDefaultMemberPermissions(modCommand())
    .setDMPermission(false),
  async execute (interaction) {
    const id = interaction.options.getInteger('id')
    let page = 0

    if (interaction.options.getInteger('page') && interaction.options.getInteger('page') > 0) {
      page = interaction.options.getInteger('page') - 1
    }

    if (id) {
      const quote = await database.quoteBook.findUnique({
        where: {
          id
        }
      })

      if (quote == null) {
        return interaction.reply({ content: 'That quote doesn\'t exist!', ephemeral: true })
      }

      const embed = new Discord.EmbedBuilder()
        .setTitle(`Quote ID: ${quote.id}`)
        .setDescription(quote.quote)
        .addFields([
          { name: 'Author', value: quote.author }
        ])

      return interaction.reply({ embeds: [embed], ephemeral: true })
    } else {
      // Max 10 fields, uses buttons to paginate
      const quotes = await database.quoteBook.findMany()
      const embeds: any[] = []
      let embed = new Discord.EmbedBuilder()
        .setTitle('Quotes')
        .setDescription('Here\'s a list of all the quotes:')

      if (quotes.length === 0) {
        embed.setDescription('There are no quotes yet!')
        return interaction.reply({ embeds: [embed], ephemeral: true })
      }

      if (page > Math.ceil(quotes.length / 10)) return interaction.reply({ content: `That's not a valid page! Last page is \`${Math.ceil(quotes.length / 10)}\`.`, ephemeral: true })

      for (let i = 0; i < quotes.length; i++) {
        const fields = embed.toJSON().fields
        if ((fields != null) && fields.length === 10) {
          embeds.push(embed)
          embed = new Discord.EmbedBuilder()
            .setTitle('Quotes')
            .setDescription('Here\'s a list of all the quotes:')
        }

        const cut = quotes[i].quote.slice(0, 150)
        const value = quotes[i].quote.length > 150 ? `${clean(cut)}...` : clean(quotes[i].quote)

        embed.addFields({ name: `ID: \`${quotes[i].id}\``, value })
        embed.setFooter({ text: `Page ${embeds.length + 1} of ${Math.ceil(quotes.length / 10)}` })
      }

      embeds.push(embed)

      const row = new Discord.ActionRowBuilder()
        .addComponents(
          new Discord.ButtonBuilder()
            .setCustomId('previous')
            .setLabel('Previous')
            .setStyle(Discord.ButtonStyle.Primary)
            .setDisabled(true),
          new Discord.ButtonBuilder()
            .setCustomId('next')
            .setLabel('Next')
            .setStyle(Discord.ButtonStyle.Primary)
        )

      if (embeds.length > 1) {
        const msg = await interaction.reply({ embeds: [embeds[page]], components: [row], fetchReply: true, ephemeral: true })

        const filter = (i: any) => i.customId === 'previous' || i.customId === 'next'
        const collector = msg.createMessageComponentCollector({ filter, time: 60000 })

        collector.on('collect', async (i: any) => {
          if (i.customId === 'previous') {
            collector.resetTimer()

            page--
            if (page === 0) {
              (<any>row.components[0]).setDisabled(true)
            }
            (<any>row.components[1]).setDisabled(false)
          } else if (i.customId === 'next') {
            collector.resetTimer()

            page++
            if (page === embeds.length - 1) {
              (<any>row.components[1]).setDisabled(true)
            }
            (<any>row.components[0]).setDisabled(false)
          }
          await i.update({ embeds: [embeds[page]], components: [row], ephemeral: true })
        })

        collector.on('end', async () => {
          (<any>row.components[0]).setDisabled(true);
          (<any>row.components[1]).setDisabled(true)
        })
      } else {
        await interaction.reply({ embeds: [embeds[page]], ephemeral: true })
      }
    }
  }
}
