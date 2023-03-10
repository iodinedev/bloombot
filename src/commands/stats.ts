import Discord, { SlashCommandBuilder, AttachmentBuilder } from 'discord.js'
import { database } from '../helpers/database'
import { config } from '../config'
import { getStreak } from '../helpers/streaks'
import { channelGuard } from '../helpers/guards'
import { get_data, make_chart, get_all_time } from '../helpers/stats'

export = {
  data: new SlashCommandBuilder()
    .setName('stats')
    .setDescription('Gets your stats or the stats of a specified user.')
    .addSubcommand(subcommand =>
      subcommand.setName('user')
        .setDescription('Gets your stats, or the stats of a specified user.')
        .addUserOption(option =>
          option.setName('user')
            .setDescription('The user to get the stats of.'))
        .addStringOption(option =>
          option.setName('type')
            .setDescription('The type of stats to get.')
            .addChoices(
              { name: 'Meditation Minutes', value: 'meditation_minutes' },
              { name: 'Meditation Count', value: 'meditation_count' }
            )
            .setRequired(false))
        .addStringOption(option =>
          option.setName('timeframe')
            .setDescription('The timeframe to get the stats for.')
            .addChoices(
              { name: 'Yearly', value: 'yearly' },
              { name: 'Monthly', value: 'monthly' },
              { name: 'Weekly', value: 'weekly' },
              { name: 'Daily', value: 'daily' }
            )
            .setRequired(false)))
    .addSubcommand(subcommand =>
      subcommand.setName('server')
        .setDescription('Gets the stats of the current guild.')
        .addStringOption(option =>
          option.setName('type')
            .setDescription('The type of stats to get.')
            .addChoices(
              { name: 'Meditation Minutes', value: 'meditation_minutes' },
              { name: 'Meditation Count', value: 'meditation_count' }
            )
            .setRequired(false))
        .addStringOption(option =>
          option.setName('timeframe')
            .setDescription('The timeframe to get the stats for.')
            .addChoices(
              { name: 'Yearly', value: 'yearly' },
              { name: 'Monthly', value: 'monthly' },
              { name: 'Weekly', value: 'weekly' },
              { name: 'Daily', value: 'daily' }
            )
            .setRequired(false)))
    .setDMPermission(false),
  async execute (interaction) {
    if (!(await channelGuard)(interaction, [config.channels.meditation, config.channels.commands], interaction.channelId)) return

    const subcommand = interaction.options.getSubcommand()

    if (subcommand === 'user') {
      const user = interaction.options.getUser('user') || interaction.user
      const type = interaction.options.getString('type') || 'meditation_minutes'
      const timeframe = interaction.options.getString('timeframe') || 'daily'

      const member = interaction.guild.members.cache.get(user.id)

      const recent_meditations = await database.meditations.findMany({
        where: {
          session_user: user.id,
          session_guild: interaction.guild.id
        },
        orderBy: [
          {
            occurred_at: 'desc'
          }
        ],
        take: 3
      })

      const meditation_aggregation = await database.meditations.aggregate({
        where: {
          session_user: user.id,
          session_guild: interaction.guild.id
        },
        _count: {
          id: true
        },
        _sum: {
          session_time: true
        }
      })

      const user_count = meditation_aggregation._count.id || 0
      const user_time = meditation_aggregation._sum.session_time || 0
      const streak = await getStreak(interaction.client, interaction.guild, user)

      const meditations: string[] = []

      recent_meditations.forEach((meditation) => {
        const date = new Date(meditation.occurred_at)
        const month = date.getUTCMonth() + 1
        const day = date.getUTCDate()
        const year = date.getUTCFullYear()

        meditations.push(
          `**${meditation.session_time}m** on ${day}/${month}/${year}\nID: \`${meditation.id}\``
        )
      })

      // Gets the data for the chart
      const raw_data: any = await get_data(timeframe, interaction.guild.id, user.id)

      const embedColor = member.displayHexColor === '#000000' ? '#ffffff' : member.displayHexColor
      const canvas = make_chart(raw_data, timeframe, type, embedColor)

      const timeframeWords = {
        daily: 'Days',
        weekly: 'Weeks',
        monthly: 'Months',
        yearly: 'Years'
      }

      const fields = [
        {
          name: 'All-Time Meditation Minutes',
          value: `\`\`\`${user_time.toLocaleString()}\`\`\``,
          inline: false
        },
        {
          name: 'All-Time Session Count',
          value: `\`\`\`${user_count.toLocaleString()}\`\`\``,
          inline: false
        },
        { name: `Minutes The Past 12 ${timeframeWords[timeframe]}`, value: `\`\`\`${raw_data.reduce((a, b) => a + Number(b.total_time), 0).toLocaleString()}\`\`\``, inline: true },
        { name: `Sessions The Past 12 ${timeframeWords[timeframe]}`, value: `\`\`\`${raw_data.reduce((a, b) => a + Number(b.count), 0).toLocaleString()}\`\`\``, inline: true },
        {
          name: 'Current Streak',
          value: `\`\`\`${streak} days\`\`\``,
          inline: false
        }
      ]

      const attachment = new AttachmentBuilder(canvas, { name: 'chart.png' })

      const statsEmbed = new Discord.EmbedBuilder()
        .setColor(config.embedColor)
        .setAuthor({ name: `${user.username}'s Stats`, iconURL: user.avatarURL() })
        .addFields(...fields)
        .setImage('attachment://chart.png')

      return interaction.reply({
        embeds: [statsEmbed],
        files: [attachment]
      })
    } else if (subcommand === 'server') {
      const type = interaction.options.getString('type') || 'meditation_count'
      const timeframe = interaction.options.getString('timeframe') || 'daily'

      // Gets the data for the chart
      const raw_data: any = await get_data(timeframe, interaction.guild.id)

      console.log(timeframe)
      console.log(raw_data)

      const canvas = make_chart(raw_data, timeframe, type)

      const all_time = await get_all_time(interaction)
      const guild_name = interaction.guild.name

      const timeframeWords = {
        daily: 'Days',
        weekly: 'Weeks',
        monthly: 'Months',
        yearly: 'Years'
      }

      const allSessionTime = all_time._sum.session_time ? all_time._sum.session_time : 0
      const allSessionCount = all_time._count.id ? all_time._count.id : 0

      const attachment = new AttachmentBuilder(canvas, { name: 'chart.png' })
      const embed = new Discord.EmbedBuilder()
        .setTitle('Stats')
        .setColor(config.embedColor)
        .setDescription(`Here are the stats for **${guild_name}**.`)
        .addFields(
          { name: 'All-Time Meditation Minutes', value: `\`\`\`${allSessionTime.toLocaleString()}\`\`\`` },
          { name: 'All-Time Session Count', value: `\`\`\`${allSessionCount.toLocaleString()}\`\`\`` },
          { name: `Minutes The Past 12 ${timeframeWords[timeframe]}`, value: `\`\`\`${raw_data.reduce((a, b) => a + Number(b.total_time), 0).toLocaleString()}\`\`\``, inline: true },
          { name: `Sessions The Past 12 ${timeframeWords[timeframe]}`, value: `\`\`\`${raw_data.reduce((a, b) => a + Number(b.count), 0).toLocaleString()}\`\`\``, inline: true }
        )
        .setImage('attachment://chart.png')
      return interaction.reply({ embeds: [embed], files: [attachment] })
    }

    return interaction.reply({ content: 'Please specify a subcommand.', ephemeral: true })
  }
}
