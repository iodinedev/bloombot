import { SlashCommandBuilder, AttachmentBuilder, EmbedBuilder } from "discord.js";
import { database } from "../helpers/database";
import Chart from "chart.js/auto"
import { createCanvas } from "canvas";
import { config } from "../config";
import { channelGuard } from "../helpers/guards";

const get_data = async (timeframe, guild) => {
  if (timeframe === 'daily') {
    // Sums meditation times that have the same "times_ago" value
    const data = await database.$queryRaw`
    WITH "daily_data" AS (
      SELECT date_part('day', NOW() - DATE_TRUNC('day', "occurred_at")) AS "times_ago", "session_time"
      FROM "Meditations"
      WHERE "session_guild" = ${guild}
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
    WHERE "session_guild" = ${guild}
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
      WHERE "session_guild" = ${guild}
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
      WHERE "session_guild" = ${guild}
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
		.setName('serverstats')
		.setDescription('Gets the stats of the current guild.')
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
    if (!(await channelGuard)(interaction, [config.channels.meditation, config.channels.commands], interaction.channelId)) return;
    
    const type = interaction.options.getString('type') || 'meditation_count';
    const timeframe = interaction.options.getString('timeframe') || 'daily';

    // Gets the data for the chart
    const raw_data: any = await get_data(timeframe, interaction.guild.id);
    const parsed_data = raw_data.reduce((acc, data) => {
      acc[data.times_ago] = data;
      return acc;
    }, {})

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
            backgroundColor: `#${config.embedColor.toString(16)}`,
            borderColor: `#${config.embedColor.toString(16)}`,
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

    const all_time = await get_all_time(interaction);
    const guild_name = interaction.guild.name;

    const timeframeWords = {
      'daily': 'Days',
      'weekly': 'Weeks',
      'monthly': 'Months',
      'yearly': 'Years'
    }

    const attachment = new AttachmentBuilder(canvas.toBuffer(), {name: "chart.png"});
    const embed = new EmbedBuilder()
      .setTitle("Stats")
      .setColor(config.embedColor)
      .setDescription(`Here are the stats for **${guild_name}**.`)
      .addFields(
        { name: "All-Time Meditation Minutes", value: `\`\`\`${all_time._sum.session_time}\`\`\`` },
        { name: "All-Time Session Count", value: `\`\`\`${all_time._count.id}\`\`\`` },
        { name: `Minutes The Past 12 ${timeframeWords[timeframe]}`, value: `\`\`\`${raw_data.reduce((a, b) => a + Number(b.total_time), 0)}\`\`\``, inline: true },
        { name: `Sessions The Past 12 ${timeframeWords[timeframe]}`, value: `\`\`\`${raw_data.reduce((a, b) => a + Number(b.count), 0)}\`\`\``, inline: true },
      )
      .setImage("attachment://chart.png");
    return interaction.reply({ embeds: [embed], files: [attachment] });
	},
};

const get_all_time = async (interaction) => {
  const data = await database.meditations.aggregate({
    where: {
      session_guild: interaction.guildId,
    },
    _sum: {
      session_time: true,
    },
    _count: {
      id: true,
    }
  });

  return data;
}