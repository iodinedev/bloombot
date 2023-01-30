import Discord, { SlashCommandBuilder, AttachmentBuilder } from "discord.js";
import { database } from "../helpers/database";
import { config } from "../config";
import Chart from "chart.js/auto"
import { createCanvas } from "canvas";
import { getStreak } from "../helpers/streaks";

const get_data = async (timeframe, guild, user) => {  
  console.log(guild, user)
  if (timeframe === 'daily') {
    // Sums meditation times that have the same "times_ago" value
    const data = await database.$queryRaw`
    WITH "daily_data" AS (
      SELECT date_part('day', NOW() - DATE_TRUNC('day', "occurred_at")) AS "times_ago", "session_time"
      FROM "Meditations"
      WHERE "session_user" = ${user} AND "session_guild" = ${guild}
    ) SELECT "times_ago", SUM("session_time") AS "total_time", COUNT(*) AS "count"
    FROM "daily_data"
    GROUP BY "times_ago"
    ORDER BY "times_ago" ASC
    LIMIT 12;
    `;

    return data;
  }

  if (timeframe === 'weekly') {
    // Sums meditation times that have the same "times_ago" value, remembering that PostgreSQL's date_part function returns the week number of the year, not the week number of the month
    const data = await database.$queryRaw`
    WITH "weekly_data" AS (
    SELECT floor(extract(epoch from NOW() - "occurred_at")/(60*60*24*7)) AS "times_ago", "session_time"
    FROM "Meditations"
      WHERE "session_user" = ${user} AND "session_guild" = ${guild}
) SELECT "times_ago", SUM("session_time") AS "total_time", COUNT(*) AS "count"
FROM "weekly_data"
GROUP BY "times_ago"
ORDER BY "times_ago" ASC
LIMIT 12;
    `;

    return data;
  }

  if (timeframe === 'monthly') {
    // Sums meditation times that have the same "times_ago" value
    const data = await database.$queryRaw`
    WITH "monthly_data" AS (
      SELECT (EXTRACT(YEAR FROM NOW()) - EXTRACT(YEAR FROM "occurred_at"))*12 + (EXTRACT(MONTH FROM NOW()) - EXTRACT(MONTH FROM "occurred_at")) AS "times_ago", "session_time"
      FROM "Meditations"
      WHERE "session_user" = ${user} AND "session_guild" = ${guild}
    ) SELECT "times_ago", SUM("session_time") AS "total_time", COUNT(*) AS "count"
    FROM "monthly_data"
    GROUP BY "times_ago"
    ORDER BY "times_ago" ASC
    LIMIT 12;
    `;

    return data;
  }

  if (timeframe === 'yearly') {
    // Sums meditation times that have the same "times_ago" value
    const data = await database.$queryRaw`
    WITH "yearly_data" AS (
      SELECT extract(year from NOW()) - extract(year from "occurred_at") AS "times_ago", "session_time"
      FROM "Meditations"
      WHERE "session_user" = ${user} AND "session_guild" = ${guild}
    ) SELECT "times_ago", SUM("session_time") AS "total_time", COUNT(*) AS "count"
    FROM "yearly_data"
    GROUP BY "times_ago"
    ORDER BY "times_ago" ASC
    LIMIT 12;
    `;

    return data;
  }
}

export = {
	data: new SlashCommandBuilder()
		.setName('stats')
		.setDescription('Gets your stats or the stats of a specified user.')
    .addUserOption(option =>
      option.setName('user')
      .setDescription('The user to get the stats of.'))
    .addStringOption(option =>
      option.setName('type')
      .setDescription('The type of stats to get.')
      .addChoices(
        { name: 'Meditation Minutes', value: 'meditation_minutes' },
        { name: 'Meditation Count', value: 'meditation_count' },
      )
      .setRequired(false))
    .addStringOption(option =>
      option.setName('timeframe')
      .setDescription('The timeframe to get the stats for.')
      .addChoices(
        { name: 'Yearly', value: 'yearly' },
        { name: 'Monthly', value: 'monthly' },
        { name: 'Weekly', value: 'weekly' },
        { name: 'Daily', value: 'daily' },
      )
      .setRequired(false))
    .setDMPermission(false),
	async execute(interaction) {
    const user = interaction.options.getUser('user') || interaction.user;
    const type = interaction.options.getString('type') || 'meditation_minutes';
    const timeframe = interaction.options.getString('timeframe') || 'daily';

    const member = interaction.guild.members.cache.get(user.id);

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

    // Gets the data for the chart
    const raw_data: any = await get_data(timeframe, interaction.guild.id, user.id);
    const parsed_data = raw_data.reduce((acc, data) => {
      acc[data.times_ago] = data;
      return acc;
    }, {})

    console.log(raw_data)
    console.log(parsed_data)

    // Makes the chart. Gets the last 12 days, weeks, months, or years of data. dd/mm/yyyy
    const data: {date, value}[] = [];

    if (timeframe === 'daily') {
      for (let i = 0; i < 12; i++) {
        const date = new Date();
        date.setDate(date.getDate() - i);
        const day = date.getDate();
        const month = date.getMonth() + 1;

        var to_push = 0;

        if (parsed_data[i]) {
          to_push = type === "meditation_minutes" ? parsed_data[i].total_time : parsed_data[i].count;
        }

        data.push({
          date: `${day}/${month}`,
          value: to_push
        });
      }
    } else if (timeframe === 'weekly') {
      for (let i = 0; i < 12; i++) {
        // Date has an offset of -7 days to show the start of the week
        const date = new Date();
        date.setDate(date.getDate() - ((i * 7) + 7));
        const day = date.getDate();
        const month = date.getMonth() + 1;

        var to_push = 0;

        if (parsed_data[i]) {
          to_push = type === "meditation_minutes" ? parsed_data[i].total_time : parsed_data[i].count;
        }

        data.push({
          date: `${day}/${month}`,
          value: to_push
        });
      }
    } else if (timeframe === 'monthly') {
      for (let i = 0; i < 12; i++) {
        const date = new Date();
        date.setMonth(date.getMonth() - i);
        const month = date.getMonth() + 1;
        const year = date.getFullYear();

        var to_push = 0;

        if (parsed_data[i]) {
          to_push = type === "meditation_minutes" ? parsed_data[i].total_time : parsed_data[i].count;
        }

        data.push({
          date: `${month}/${year}`,
          value: to_push
        });
      }
    } else if (timeframe === 'yearly') {
      for (let i = 0; i < 12; i++) {
        const date = new Date();
        date.setFullYear(date.getFullYear() - i);
        const year = date.getFullYear();

        var to_push = 0;

        if (parsed_data[i]) {
          to_push = type === "meditation_minutes" ? parsed_data[i].total_time : parsed_data[i].count;
        }

        data.push({
          date: `${year}`,
          value: to_push
        });
      }
    }

    // Inverts the data so that it's in the right order
    data.reverse();

    const labels = data.map(d => d.date);
    const values = data.map(d => Number(d.value));
    const header = type === "meditation_minutes" ? "# of Minutes" : "# of Sessions";

    // Makes the chart
    const canvas = createCanvas(400, 250);
    const canvas_ctx: any = canvas.getContext("2d");

    Chart.defaults.color = '#ffffff';

    new Chart(canvas_ctx, {
      type: "bar",
      data: {
        labels: labels,
        datasets: [
          {
            label: header,
            data: values,
            backgroundColor: member.displayHexColor === "#000000" ? "#ffffff" : member.displayHexColor,
            borderColor: `#fff`,
            borderWidth: 1,
          },
        ],
      },
      options: {
        scales: {
          y: {
            beginAtZero: true,
          },
        }
      },
    });

    const timeframeWords = {
      'daily': 'Days',
      'weekly': 'Weeks',
      'monthly': 'Months',
      'yearly': 'Years'
    }

    const fields = [
      {
        name: 'All-Time Meditation Minutes',
        value: `\`\`\`${user_time}\`\`\``,
        inline: false,
      },
      {
        name: 'All-Time Session Count',
        value: `\`\`\`${user_count}\`\`\``,
        inline: false,
      },
      { name: `Minutes The Past 12 ${timeframeWords[timeframe]}`, value: `\`\`\`${raw_data.reduce((a, b) => a + Number(b.total_time), 0)}\`\`\``, inline: true },
      { name: `Sessions The Past 12 ${timeframeWords[timeframe]}`, value: `\`\`\`${raw_data.reduce((a, b) => a + Number(b.count), 0)}\`\`\``, inline: true },
      {
        name: 'Current Streak',
        value: `\`\`\`${streak} days\`\`\``,
        inline: false,
      },
    ];

    const attachment = new AttachmentBuilder(canvas.toBuffer(), {name: "chart.png"});

    let statsEmbed = new Discord.EmbedBuilder()
      .setColor(config.embedColor)
      .setAuthor({ name: `${user.username}'s Stats`, iconURL: user.avatarURL() })
      .addFields(...fields)
      .setImage('attachment://chart.png');

    return interaction.reply({
      embeds: [statsEmbed],
      files: [attachment],
    });
	},
};