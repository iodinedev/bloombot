import { SlashCommandBuilder, EmbedBuilder, ActionRowBuilder, ButtonBuilder, ButtonStyle } from 'discord.js'
import { adminCommand } from '../helpers/commandPermissions'
import { database } from '../helpers/database'
import { updateRoles } from '../helpers/streaks'

export = {
  data: new SlashCommandBuilder()
    .setName('manage')
    .setDescription('Options for managing users\' meditation entries.')
    .addSubcommand(subcommand =>
      subcommand.setName('insert')
        .setDescription('Insert a new meditation entry.')
        .addUserOption(
          option => option.setName('user')
            .setDescription('The user to insert the meditation session for.')
            .setRequired(true))
        .addIntegerOption(option =>
          option.setName('date')
            .setDescription('The date of the meditation session. Use /timestamp to generate a UNIX timestamp.')
            .setRequired(true))
        .addIntegerOption(option =>
          option.setName('minutes')
            .setDescription('The number of minutes you want to add. Leave blank if you want to add 0 minutes to mend streaks.')
            .setMinValue(0)
            .setRequired(false)))
    .addSubcommand(subcommand =>
      subcommand.setName('list')
        .setDescription('List all meditation entries for a user.')
        .addUserOption(
          option => option.setName('user')
            .setDescription('The user to list the meditation sessions for.')
            .setRequired(true))
        .addIntegerOption(option =>
          option.setName('page')
            .setDescription('The page of the list to view.')
            .setRequired(false)))
    .addSubcommand(subcommand =>
      subcommand.setName('update')
        .setDescription('Update a meditation entry.')
        .addIntegerOption(option =>
          option.setName('id')
            .setDescription('The ID of the meditation entry to update.')
            .setRequired(true))
        .addIntegerOption(option =>
          option.setName('date')
            .setDescription('The date of the meditation session. Use /timestamp to generate a UNIX timestamp.')
            .setRequired(false))
        .addIntegerOption(option =>
          option.setName('minutes')
            .setDescription('The number of minutes you want to set the meditation session to.')
            .setMinValue(1)
            .setRequired(false)))
    .addSubcommand(subcommand =>
      subcommand.setName('delete')
        .setDescription('Delete a meditation entry.')
        .addIntegerOption(option =>
          option.setName('id')
            .setDescription('The ID of the meditation entry to delete.')
            .setRequired(true)))
    .addSubcommand(subcommand =>
      subcommand.setName('reset')
        .setDescription('Reset a user\'s meditation stats.')
        .addUserOption(
          option => option.setName('user')
            .setDescription('The user to reset the stats for.')
            .setRequired(true)))
    .setDefaultMemberPermissions(adminCommand())
    .setDMPermission(false),
  async execute (interaction) {
    const subcommand = interaction.options.getSubcommand()

    if (subcommand === 'insert') {
      const user = interaction.options.getUser('user')
      let timestamp: Date

      try {
        timestamp = new Date(interaction.options.getInteger('date'))
      } catch (error) {
        return interaction.reply({ content: 'Invalid date. Use /timestamp to generate a UNIX timestamp.', ephemeral: true })
      }
      const minutes: number = interaction.options.getInteger('minutes') || 0

      await database.meditations.create({
        data: {
          id: interaction.id,
          session_user: user.id,
          session_time: minutes,
          session_guild: interaction.guild.id,
          occurred_at: timestamp
        }
      })

      const human_date = `${timestamp.getMonth() + 1}/${timestamp.getDate()}/${timestamp.getFullYear()} ${timestamp.getHours()}:${timestamp.getMinutes()}`

      await interaction.reply({ content: `Added ${minutes} minutes to ${user.username}'s meditation time at ${human_date}!`, ephemeral: true })
    } else if (subcommand === 'list') {
      const user = interaction.options.getUser('user')

      let page = 0

      if (interaction.options.getInteger('page') !== null) {
        page = interaction.options.getInteger('page') - 1
      }

      if (page < 0) return interaction.reply({ content: 'That\'s not a valid page!', ephemeral: true })

      const sessions = await database.meditations.findMany({
        where: {
          session_user: user.id,
          session_guild: interaction.guild.id
        },
        orderBy: [
          {
            occurred_at: 'desc'
          }
        ]
      })

      const embeds: any[] = []
      let embed = new EmbedBuilder()
        .setTitle('Entries')
        .setDescription('Here\'s a list of your meditation sessions:')

      if (sessions.length === 0) {
        embed.setDescription('There are no entries yet!')
        return interaction.reply({ embeds: [embed], ephemeral: true })
      }

      if (page > Math.ceil(sessions.length / 10)) return interaction.reply({ content: `That's not a valid page! Last page is \`${Math.ceil(sessions.length / 10)}\`.`, ephemeral: true })

      const today = new Date()

      for (let i = 0; i < sessions.length; i++) {
        const fields = embed.toJSON().fields
        if ((fields != null) && fields.length === 10) {
          embeds.push(embed)
          embed = new EmbedBuilder()
            .setTitle('Entries')
            .setDescription('Here\'s a list of your meditation sessions:')
        }

        // Show time if the date is today, otherwise show the date
        const date = new Date(sessions[i].occurred_at)
        let dateTime = `\`${sessions[i].id}\` - ${date.toLocaleDateString('en-US', { weekday: 'long', year: 'numeric', month: 'long', day: 'numeric' })}`

        if (date.getDate() === today.getDate() && date.getMonth() === today.getMonth() && date.getFullYear() === today.getFullYear()) {
          dateTime = `ID: \`${sessions[i].id}\` - ${date.toLocaleTimeString('en-US', { hour: 'numeric', minute: 'numeric', hour12: true })} Today`
        }

        embed.addFields({ name: dateTime, value: `\`\`\`${sessions[i].session_time} minutes\`\`\`` })
        embed.setFooter({ text: `Page ${embeds.length + 1} of ${Math.ceil(sessions.length / 10)}` })
      }

      embeds.push(embed)

      const row = new ActionRowBuilder()
        .addComponents(
          new ButtonBuilder()
            .setCustomId('previous')
            .setLabel('Previous')
            .setStyle(ButtonStyle.Primary)
            .setDisabled(true),
          new ButtonBuilder()
            .setCustomId('next')
            .setLabel('Next')
            .setStyle(ButtonStyle.Primary)
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
    } else if (subcommand === 'update') {
      const id = interaction.options.getInteger('id')

      const session = await database.meditations.findUnique({
        where: {
          id
        }
      })

      if (session == null) return interaction.reply({ content: 'That\'s not a valid ID!', ephemeral: true })

      let occurred_at

      // In theory we could optimize this by only doing the try/catch if the date is not null, but I really don't care what you think.
      // So, if you're reading this, you're probably a developer. If you're a developer, you're probably a good developer. If you're a good developer, you're probably a good person.
      // If you're a good person, you probably care about the efficiency of your code. If you care about the efficiency of your code, you probably don't use try/catch blocks for control flow.
      // If you don't use try/catch blocks for control flow, you probably don't use JavaScript.
      // - GitHub Copilot
      try {
        if (interaction.options.getInteger('date') === null) {
          occurred_at = new Date(session.occurred_at)
        } else {
          occurred_at = new Date(interaction.options.getInteger('date'))
        }
      } catch (e) {
        if (e instanceof TypeError) {
          return interaction.reply({ content: 'That\'s not a valid date!', ephemeral: true })
        }

        throw e
      }

      const minutes = interaction.options.getInteger('minutes') || session.session_time

      await database.meditations.update({
        where: {
          id
        },
        data: {
          session_time: minutes,
          occurred_at
        }
      })

      await interaction.reply({ content: `Updated session \`${id}\`!`, ephemeral: true })
    } else if (subcommand === 'delete') {
      const id = interaction.options.getInteger('id')

      const session = await database.meditations.findUnique({
        where: {
          id
        }
      })

      if (session == null) return interaction.reply({ content: 'That\'s not a valid ID!', ephemeral: true })

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

      const date = new Date(session.occurred_at)
      const month = date.getUTCMonth() + 1
      const day = date.getUTCDate()
      const year = date.getUTCFullYear()

      interaction.reply({ content: `Are you sure you want to delete <@${session.session_user}>'s ${session.session_time}m session on ${day}/${month}/${year}?`, ephemeral: true, components: [row] })

      const filter = i => i.user.id === interaction.user.id
      const collector = interaction.channel.createMessageComponentCollector({ filter, time: 15000 })

      collector.on('collect', async i => {
        if (i.customId === 'yes') {
          collector.resetTimer()

          await database.meditations.delete({
            where: {
              id
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
    } else if (subcommand === 'reset') {
      const user = interaction.options.getUser('user')

      const data = await database.meditations.aggregate({
        where: {
          session_user: user.id,
          session_guild: interaction.guild.id
        },
        _sum: {
          session_time: true
        },
        _count: true
      })

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

      interaction.reply({ content: `Are you sure you want to delete this user's stats? They have ${data._sum.session_time} minutes and ${data._count} sessions.`, components: [row], ephemeral: true })

      const filter = i => i.user.id === interaction.user.id
      const collector = interaction.channel.createMessageComponentCollector({ filter, time: 15000 })

      collector.on('collect', async i => {
        if (i.customId === 'yes') {
          await database.meditations.deleteMany({
            where: {
              session_user: user.id,
              session_guild: interaction.guild.id
            }
          })

          interaction.editReply({ content: 'User cleared!', ephemeral: true, components: [] })
        } else if (i.customId === 'no') {
          interaction.editReply({ content: 'Nothing has been changed.', ephemeral: true })
        }
      })

      collector.on('end', collected => {
        if (collected.size === 0) {
          interaction.editReply({ content: 'You did not respond in time. Nothing has been changed.', ephemeral: true })
        }
      })
    } else {
      interaction.reply({ content: 'That\'s not a valid subcommand!', ephemeral: true })
    }
  }
}
