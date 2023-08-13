import { SlashCommandBuilder, ActionRowBuilder, ButtonBuilder, ButtonStyle } from 'discord.js'
import type { ChatInputCommandInteraction, InteractionCollector, ButtonInteraction, GuildMember, CollectorFilter } from 'discord.js'
import { database } from '../helpers/database'
import { updateRoles, getStreak } from '../helpers/streaks'
import { config } from '../config'
import { channelGuard } from '../helpers/guards'
import { alphanumeric } from '../helpers/strings'

export = {
  data: new SlashCommandBuilder()
    .setName('add')
    .setDescription('Adds minutes to your meditation time.')
    .addIntegerOption(option =>
      option.setName('minutes')
        .setDescription('The number of minutes you want to add.')
        .setMinValue(1)
        .setRequired(true))
    .setDMPermission(false),
  async execute (interaction: ChatInputCommandInteraction) {
    if (!(await channelGuard(interaction, [config.channels.meditation, config.channels.commands], interaction.channelId))) return

    const minutes: number = interaction.options.getInteger('minutes') ?? 0
    const random_id = Math.random().toString(36).substring(2, 15) + Math.random().toString(36).substring(2, 15);

    // Check with user if they want to add the minutes if they are over 300
    if (minutes > 300) {
      const row = new ActionRowBuilder<ButtonBuilder>()
        .addComponents(
          new ButtonBuilder()
            .setCustomId(`yes-${random_id}`)
            .setLabel('Yes')
            .setStyle(ButtonStyle.Danger),
          new ButtonBuilder()
            .setCustomId(`no-${random_id}`)
            .setLabel('No')
            .setStyle(ButtonStyle.Primary)
        )

      await interaction.reply({ content: `Are you sure you want to add ${minutes} minutes?`, components: [row] })

      const filter: CollectorFilter<any> = (i: GuildMember): boolean => i.user.id === interaction.user.id
      const collector = interaction.channel?.createMessageComponentCollector({ filter, time: 15000 }) as InteractionCollector<ButtonInteraction>

      collector?.on('collect', async i => {
        if (i.customId === `yes-${random_id}`) {
          collector.resetTimer()

          await addMinutes(interaction, minutes, true)
        } else if (i.customId === `no-${random_id}`) {
          collector.resetTimer()

          await interaction.editReply({ content: 'Time not added.', components: [] })
        }
      })

      collector?.on('end', async (collected) => {
        if (collected.size === 0) {
          await interaction.editReply({ content: 'You did not respond in time. Time not added.', components: [] })
        }
      })
    } else {
      await addMinutes(interaction, minutes, false)
    }
  }
}

async function addMinutes (interaction, minutes: number, replied): Promise<void> {
  const user = interaction.user.id
  const guild = interaction.guild.id

  await database.meditations.create({
    data: {
      session_user: user,
      session_time: minutes,
      session_guild: guild
    }
  })

  const total = await database.meditations.aggregate({
    where: {
      session_user: user,
      session_guild: guild
    },
    _sum: {
      session_time: true
    }
  })

  const guildTotal = await database.meditations.aggregate({
    where: {
      session_guild: guild
    },
    _sum: {
      session_time: true
    },
    _count: {
      session_time: true
    }
  })

  const motivationMessages = (await database.quoteBook.findMany()).map((quote) => alphanumeric(quote.quote))

  const motivation = motivationMessages.length > 0 ? `\n*${motivationMessages[Math.floor(Math.random() * motivationMessages.length)]}*` : ''
  const update = await updateRoles(interaction.client, interaction.guild, interaction.user)

  if (replied === false) {
    await interaction.reply({ content: `Added **${minutes} minutes** to your meditation time! Your total meditation time is ${(total._sum.session_time ?? 0).toLocaleString()} minutes :tada:${motivation}` })
  } else {
    await interaction.editReply({ content: `Added **${minutes} minutes** to your meditation time! Your total meditation time is ${(total._sum.session_time ?? 0).toLocaleString()} minutes :tada:${motivation}`, components: [] })
  }

  if (guildTotal._count.session_time % 10 === 0 && guildTotal._count.session_time > 0) {
    const timeInHours = Math.round((guildTotal._sum.session_time ?? 0) / 60)

    await interaction.channel
      .send(
        `Awesome sauce! This server has collectively generated ${timeInHours.toLocaleString()} hours of realmbreaking meditation!`
      )
  }

  if (update.new_streak.length > 0) {
    return interaction.followUp({
      content: `:tada: Congrats to <@${interaction.user.id}>, your hard work is paying off! Your current streak is ${await getStreak(interaction.client, interaction.guild, interaction.user)}, giving you the <@&${update.new_streak}> role!`,
      allowedMentions: {
        roles: [],
        users: [interaction.user.id]
      }
    })
  } else if (update.new_level.length > 0) {
    return interaction.followUp({
      content: `:tada: Congrats to <@${interaction.user.id}>, your hard work is paying off! Your total meditation minutes have given you the <@&${update.new_level}> role!`,
      allowedMentions: {
        roles: [],
        users: [interaction.user.id]
      }
    })
  }
}
