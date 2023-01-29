import { SlashCommandBuilder } from "discord.js";
import { database } from "../helpers/database";
import { updateRoles } from "../helpers/streaks";
import { config } from "../config";
import { getStreak } from "../helpers/streaks";

export = {
	data: new SlashCommandBuilder()
		.setName('add')
		.setDescription('Adds minutes to your meditation time.')
    .addIntegerOption(option =>
      option.setName('minutes')
        .setDescription('The number of minutes you want to add.')
        .setRequired(true))
    .setDMPermission(false),
	async execute(interaction) {
		const minutes: number = interaction.options.getInteger('minutes');

    const user = interaction.user.id;
    const guild = interaction.guild.id;

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
    });
    
    const motivation = config.motivational_messages[Math.floor(Math.random() * config.motivational_messages.length)];
    const update = await updateRoles(interaction.client, interaction.guild, interaction.user);

    
    await interaction.reply({ content: `Added **${minutes} minutes** to your meditation time! Your total meditation time is ${total._sum.session_time} minutes :tada:\n*${motivation}*` });

    if (update.new_streak.length > 0) {
      return interaction.followUp({
        content: `:tada: Congrats to <@${interaction.user.id}>, your hard work is paying off! Your current streak is ${await getStreak(interaction.client, interaction.guild, interaction.user)}, giving you the <@&${update.new_streak}> role!`,
        allowedMentions: {
          roles: [],
          users: [interaction.user.id]
        }
      });
    } else if (update.new_level.length > 0) {
      return interaction.followUp({
        content: `:tada: Congrats to <@${interaction.user.id}>, your hard work is paying off! Your total meditation minutes have given you the <@&${update.new_level}> role!`,
        allowedMentions: {
          roles: [],
          users: [interaction.user.id]
        },
      });
    }
	},
};