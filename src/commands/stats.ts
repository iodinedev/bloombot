import Discord, { SlashCommandBuilder } from "discord.js";
import { database } from "../helpers/database";
import { config } from "../config";
import { getStreak } from "../helpers/streaks";
import { Chart } from "chart.js";

const get_data = async (timeframe) => {
  if (timeframe === 'daily') {
    // Sums meditation times that have the same "days_ago" value
    const data = await database.$queryRaw`
    WITH "daily_data" AS (
      SELECT date_part('day', NOW() - DATE_TRUNC('day', "occurred_at")) AS "days_ago", "session_time"
      FROM "Meditations"
    ) SELECT "days_ago", SUM("session_time") AS "total_time", COUNT(*) AS "count"
    FROM "daily_data"
    GROUP BY "days_ago"
    ORDER BY "days_ago" ASC
    LIMIT 12;
    `;

    return data;
  }

  if (timeframe === 'weekly') {
    // Sums meditation times that have the same "weeks_ago" value, remembering that PostgreSQL's date_part function returns the week number of the year, not the week number of the month
    const data = await database.$queryRaw`
    WITH "weekly_data" AS (
    SELECT floor(extract(epoch from NOW() - "occurred_at")/(60*60*24*7)) AS "weeks_ago", "session_time"
    FROM "Meditations"
) SELECT "weeks_ago", SUM("session_time") AS "total_time", COUNT(*) AS "count"
FROM "weekly_data"
GROUP BY "weeks_ago"
ORDER BY "weeks_ago" ASC
LIMIT 12;
    `;

    return data;
  }

  if (timeframe === 'monthly') {
    // Sums meditation times that have the same "months_ago" value
    const data = await database.$queryRaw`
    WITH "monthly_data" AS (
      SELECT date_part('month', NOW() - DATE_TRUNC('month', "occurred_at")) AS "months_ago", "session_time"
      FROM "Meditations"
    ) SELECT "months_ago", SUM("session_time") AS "total_time", COUNT(*) AS "count"
    FROM "monthly_data"
    GROUP BY "months_ago"
    ORDER BY "months_ago" ASC
    LIMIT 12;
    `;

    return data;
  }

  if (timeframe === 'yearly') {
    // Sums meditation times that have the same "years_ago" value
    const data = await database.$queryRaw`
    WITH "yearly_data" AS (
      SELECT date_part('year', NOW() - DATE_TRUNC('year', "occurred_at")) AS "years_ago", "session_time"
      FROM "Meditations"
    ) SELECT "years_ago", SUM("session_time") AS "total_time", COUNT(*) AS "count"
    FROM "yearly_data"
    GROUP BY "years_ago"
    ORDER BY "years_ago" ASC
    LIMIT 12;
    `;

    return data;
  }
}

export = {
	data: new SlashCommandBuilder()
		.setName('stats')
		.setDescription('Gets the stats of the current guild.')
    .addStringOption(option =>
      option.setName('type')
      .setDescription('The type of stats to get.')
      .addChoices(
        { name: 'Meditation Minutes', value: 'meditation_minutes' },
        { name: 'Meditation Count', value: 'meditation_count' },
      )
      .setRequired(true))
    .addStringOption(option =>
      option.setName('timeframe')
      .setDescription('The timeframe to get the stats for.')
      .addChoices(
        { name: 'Yearly', value: 'yearly' },
        { name: 'Monthly', value: 'monthly' },
        { name: 'Weekly', value: 'weekly' },
        { name: 'Daily', value: 'daily' },
      )
      .setRequired(true)),
	async execute(interaction) {
    const user = interaction.options.getUser('user') || interaction.user;
    const type = interaction.options.getString('type');
    const timeframe = interaction.options.getString('timeframe');

    // Gets the data for the chart
    const raw_data: any = await get_data(timeframe);

    console.log(raw_data)

    // Makes the chart. Gets the last 12 days, weeks, months, or years of data. dd/mm/yyyy
    const data: {date, value}[] = [];

    if (timeframe === 'daily') {
      for (let i = 0; i < 12; i++) {
        const date = new Date();
        date.setDate(date.getDate() - i);
        const day = date.getDate();
        const month = date.getMonth() + 1;
        const year = date.getFullYear();

        data.push({
          date: `${day}/${month}/${year}`,
          value: type === "meditation_minutes" ? raw_data.total_time : raw_data.count
        });
      }
    } else if (timeframe === 'weekly') {
      for (let i = 0; i < 12; i++) {
        const date = new Date();
        date.setDate(date.getDate() - (i * 7));
        const day = date.getDate();
        const month = date.getMonth() + 1;
        const year = date.getFullYear();

        data.push({
          date: `${day}/${month}/${year}`,
          value: type === "meditation_minutes" ? raw_data.total_time : raw_data.count
        });
      }
    } else if (timeframe === 'monthly') {
      for (let i = 0; i < 12; i++) {
        const date = new Date();
        date.setMonth(date.getMonth() - i);
        const month = date.getMonth() + 1;
        const year = date.getFullYear();

        data.push({
          date: `${month}/${year}`,
          value: type === "meditation_minutes" ? raw_data.total_time : raw_data.count
        });
      }
    } else if (timeframe === 'yearly') {
      for (let i = 0; i < 12; i++) {
        const date = new Date();
        date.setFullYear(date.getFullYear() - i);
        const year = date.getFullYear();

        data.push({
          date: `${year}`,
          value: type === "meditation_minutes" ? raw_data.total_time : raw_data.count
        });
      }
    }
	},
};