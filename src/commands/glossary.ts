import { database } from '../helpers/database'
import Discord from 'discord.js'
import { clean, makeSearchable } from '../helpers/strings'
import { config } from '../config'

export = {
  data: new Discord.SlashCommandBuilder()
    .setName('glossary')
    .setDescription('Shows a list of all glossary terms, or info about a specific term.')
    .addSubcommand(subcommand =>
      subcommand.setName('list')
        .setDescription('Shows a list of all glossary terms.')
        .addIntegerOption(option =>
          option.setName('page')
            .setDescription('The page you want to see.')
            .setRequired(false)))
    .addSubcommand(subcommand =>
      subcommand.setName('info')
        .setDescription('Shows info about a specific term.')
        .addStringOption(option =>
          option.setName('term')
            .setDescription('The term you want info about.')
            .setRequired(true)))
    .addSubcommand(subcommand =>
      subcommand.setName('search')
        .setDescription('Searches for a term.')
        .addStringOption(option =>
          option.setName('term')
            .setDescription('The term you want info about.')
            .setRequired(true)))
    .setDMPermission(false),
  async autocomplete (interaction) {
    const term: string = interaction.options.getString('term')
    const search: string = makeSearchable(term)

    const terms = await database.glossary.findMany({
      where: {
        search: {
          contains: search
        }
      }
    })

    const suggestions = terms.map(term => {
      return {
        name: term.term,
        value: term.search
      }
    })

    await interaction.respond(suggestions)
  },
  async execute (interaction) {
    const subcommand = interaction.options.getSubcommand()

    if (subcommand === 'list') {
      let page = 0

      if (interaction.options.getInteger('page') !== null) {
        page = interaction.options.getInteger('page') - 1
      }

      if (page < 0) return interaction.reply({ content: 'That\'s not a valid page!', ephemeral: true })

      // Max 10 fields, uses buttons to paginate. If one of the terms is too long, it will be omitted.
      const terms = await database.glossary.findMany()
      const embeds: any[] = []
      let embed = new Discord.EmbedBuilder()
        .setTitle('Glossary')
        .setColor(config.embedColor)
        .setDescription('Here\'s a list of all my glossary terms:')

      if (terms.length === 0) {
        embed.setDescription('There are no terms yet!')
        return interaction.reply({ embeds: [embed], ephemeral: true })
      }

      if (page > Math.ceil(terms.length / 10)) return interaction.reply({ content: `That's not a valid page! Last page is \`${Math.ceil(terms.length / 10)}\`.`, ephemeral: true })

      for (let i = 0; i < terms.length; i++) {
        if (terms[i].term.length > 1024 || terms[i].definition.length > 1024) continue

        const fields = embed.toJSON().fields
        if ((fields != null) && fields.length === 10) {
          embeds.push(embed)
          embed = new Discord.EmbedBuilder()
            .setTitle('Glossary')
            .setColor(config.embedColor)
            .setDescription('Here\'s a list of all my glossary terms:')
        }

        embed.addFields({ name: terms[i].term, value: terms[i].definition })
        embed.setFooter({ text: `Page ${embeds.length + 1} of ${Math.ceil(terms.length / 10)}` })
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
        const msg = await interaction.reply({ embeds: [embeds[page]], components: [row], fetchReply: true })

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
          await i.update({ embeds: [embeds[page]], components: [row] })
        })

        collector.on('end', async () => {
          (<any>row.components[0]).setDisabled(true);
          (<any>row.components[1]).setDisabled(true)
        })
      } else {
        await interaction.reply({ embeds: [embeds[page]] })
      }
    } else if (subcommand === 'info') {
      const term = interaction.options.getString('term')

      if (term) {
        const search = makeSearchable(term)
        const termData = await database.glossary.findFirst({
          where: {
            search
          }
        })

        if (termData == null) return interaction.reply({ content: 'That\'s not a valid term!', ephemeral: true })

        if (termData.term.length > 1024 || termData.definition.length > 1024) return interaction.reply({ content: 'That term is too long! Tell the manager to shorten it.', ephemeral: true })

        const fields: any[] = []
        if (termData.usage) fields.push({ name: 'Usage', value: termData.usage })
        if (termData.category) fields.push({ name: 'Category', value: termData.category })
        if (termData.links.length > 0) fields.push({ name: 'Links', value: termData.links.join('\n') })

        const embed = new Discord.EmbedBuilder()
          .setTitle(`Term: ${termData.term}`)
          .setColor(config.embedColor)
          .setDescription(termData.definition)
          .addFields(fields)
        return interaction.reply({ embeds: [embed] })
      } else {
        return interaction.reply({ content: 'You need to specify a term!', ephemeral: true })
      }
    } else if (subcommand === 'search') {
      const term = interaction.options.getString('term')

      if (term) {
        const search = makeSearchable(term)
        const terms = await database.glossary.findMany({
          where: {
            search: {
              contains: search
            }
          }
        })

        let page = 0

        // Max 10 fields, uses buttons to paginate
        const embeds: any[] = []
        let embed = new Discord.EmbedBuilder()
          .setTitle('Search Results')
          .setColor(config.embedColor)
          .setDescription(`Here\'s a list of all the terms matching the search \`${clean(term)}\`:`)

        if (terms.length === 0) {
          embed.setDescription('No terms match your search.')
          return interaction.reply({ embeds: [embed], ephemeral: true })
        }

        if (page > Math.ceil(terms.length / 10)) return interaction.reply({ content: `That's not a valid page! Last page is \`${Math.ceil(terms.length / 10)}\`.`, ephemeral: true })

        for (let i = 0; i < terms.length; i++) {
          const fields = embed.toJSON().fields
          if ((fields != null) && fields.length === 10) {
            embeds.push(embed)
            embed = new Discord.EmbedBuilder()
              .setTitle('Search Results')
              .setColor(config.embedColor)
              .setDescription(`Here\'s a list of all the terms matching the search \`${clean(term)}\`:`)
          }

          embed.addFields({ name: terms[i].term, value: terms[i].definition })
          embed.setFooter({ text: `Search results page ${embeds.length + 1} of ${Math.ceil(terms.length / 10)}` })
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
          const msg = await interaction.reply({ embeds: [embeds[page]], components: [row], fetchReply: true })

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
            await i.update({ embeds: [embeds[page]], components: [row] })
          })

          collector.on('end', async () => {
            (<any>row.components[0]).setDisabled(true);
            (<any>row.components[1]).setDisabled(true)
          })
        } else {
          await interaction.reply({ embeds: [embeds[page]] })
        }
      }
    }

    return interaction.reply({ content: 'You need to specify a subcommand!', ephemeral: true })
  }
}
