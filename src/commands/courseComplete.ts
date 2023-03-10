import { EmbedBuilder, SlashCommandBuilder } from 'discord.js'
import { config } from '../config'
import { database } from '../helpers/database'
import { makeSearchable } from '../helpers/strings'

export = {
  data: new SlashCommandBuilder()
    .setName('coursecomplete')
    .setDescription('Mark that you have completed a course.')
    .addStringOption(option =>
      option.setName('course')
        .setDescription('The course you want to mark as complete.')
        .setRequired(true))
    .setDefaultMemberPermissions(0)
    .setDMPermission(true),
  async execute(interaction) {
    const course: string = interaction.options.getString('course')
    const search: string = makeSearchable(course)

    // Ensure that the course exists
    const courseEntry = await database.courses.findUnique({
      where: {
        search
      }
    })

    if (courseEntry == null) {
      await interaction.reply({ content: `The course does not exist: **${course}**.`, ephemeral: true })
      return
    }

    // Ensure that the user is in the course
    const guild = await interaction.client.guilds.fetch(courseEntry.guild)
    const member = await guild.members.fetch(interaction.user.id)
    if (!member.roles.cache.has(courseEntry.participant_role)) {
      await interaction.reply({ content: `You are not in the course: **${course}**.`, ephemeral: true })
      return
    }

    // Ensure that the user does not already have the role
    if (member.roles.cache.has(courseEntry.graduate_role)) {
      await interaction.reply({ content: `You have already completed the course: **${course}**.`, ephemeral: true })
      return
    }

    // Notify staff
    try {
      const staffChannel = await guild.channels.fetch(config.channels.logs)

      const logsEmbed = new EmbedBuilder()
        .setTitle('Course Marked Completed')
        .setColor(config.embedColor)
        .addFields(
          { name: 'Course', value: course, inline: true },
          { name: 'User', value: `<@${interaction.user.id}>`, inline: true }
        )
        .setTimestamp()

      await staffChannel.send({ embeds: [logsEmbed] })
    } catch {}

    // Add the role
    try {
      await member.roles.add(courseEntry.graduate_role)
      return interaction.reply({ content: `:tada: Congrats! I marked you as having completed the course: **${course}**.`, ephemeral: true })
    } catch (error: any) {
      if (error.code === 50013) {
        await interaction.reply({ content: `I don't have permission to give you the role for **${course}**.`, ephemeral: true })
        return
      }

      console.log(error)

      throw error
    }
  },
  config: {
    hidden: true
  }
}
