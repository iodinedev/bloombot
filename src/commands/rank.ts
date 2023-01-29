import Discord, { SlashCommandBuilder } from "discord.js";
import { database } from "../helpers/database";
import { config } from "../config";
import { getStreak } from "../helpers/streaks";

export = {
	data: new SlashCommandBuilder()
		.setName('rank')
		.setDescription('Gets your rank or the rank of a specified user.')
    .addUserOption(option =>
      option.setName('user')
      .setDescription('The user to get the rank of.'))
    .setDMPermission(false),
	async execute(interaction) {
    const user = interaction.options.getUser('user') || interaction.user;

		const recent_meditations = await database.meditations.findMany({
      where: {
        session_user: user.id,
        session_guild: interaction.guild.id
      },
      orderBy: [
        {
          id: 'desc'
        }
      ],
      take: 3
    });

    if (!recent_meditations) {
      return interaction.reply({
        content: ":x: Looks like you don't have any meditation times! Use `.add` to add some time."
      });
    }
  
    const meditation_aggregation = await database.meditations.aggregate({
      where: {
        session_user:  user.id,
        session_guild: interaction.guild.id,
      },
      _count: {
        id: true
      },
      _sum: {
        session_time: true
      }
    });

    const user_count = meditation_aggregation._count.id || 0;
    const user_time = meditation_aggregation._sum.session_time || 0;
    const streak = await getStreak(interaction.client, interaction.guild, user);

    var meditations: string[] = [];

    recent_meditations.forEach((meditation) => {
      var date = new Date(meditation.occurred_at);
      var month = date.getUTCMonth() + 1;
      var day = date.getUTCDate();
      var year = date.getUTCFullYear();

      meditations.push(
        `**${meditation.session_time}m** on ${day}/${month}/${year}\nID: \`${meditation.id}\``
      );
    });

    const fields = [
      {
        name: 'Meditation Minutes',
        value: `${user_time}`,
        inline: false,
      },
      {
        name: 'Meditation Count',
        value: `${user_count}`,
        inline: false,
      },
      {
        name: 'Recent Meditations',
        value:
          meditations.length === 0 ? 'None' : `${meditations.join('\n')}`,
        inline: false,
      },
      {
        name: 'Current Streak',
        value: `${streak} days`,
        inline: false,
      },
    ];

    let rankEmbed = new Discord.EmbedBuilder()
      .setColor(config.embedColor)
      .setTitle('Meditation Stats')
      .setThumbnail(user.avatarURL())
      .addFields(...fields);

    return interaction.reply({
      embeds: [rankEmbed],
    });
	},
};