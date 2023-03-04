import { database } from './database'
import { config } from '../config'
import { createCanvas } from 'canvas'
import Chart from 'chart.js/auto'

export const get_data = async (timeframe, guild, user = null) => {
  let get_user = false

  if (user !== null) {
    get_user = true
  }

  if (timeframe === 'daily') {
    // Sums meditation times that have the same "times_ago" value
    const data = await database.$queryRaw`
    WITH "daily_data" AS (
      SELECT date_part('day', NOW() - DATE_TRUNC('day', "occurred_at")) AS "times_ago", "session_time"
      FROM "Meditations"
      WHERE "session_guild" = ${guild}
      AND (${get_user} IS TRUE AND "session_user" = ${user})
        OR (${get_user} IS FALSE AND TRUE)
    ) SELECT "times_ago", SUM("session_time") AS "total_time", COUNT(*) AS "count"
    FROM "daily_data"
    WHERE "times_ago" <= 12
    GROUP BY "times_ago"
    ORDER BY "times_ago" ASC;
    `

    return data
  }

  if (timeframe === 'weekly') {
    // Sums meditation times that have the same "times_ago" value, remembering that PostgreSQL's date_part function returns the week number of the year, not the week number of the month
    const data = await database.$queryRaw`
    WITH "weekly_data" AS (
    SELECT floor(extract(epoch from NOW() - "occurred_at")/(60*60*24*7)) AS "times_ago", "session_time"
    FROM "Meditations"
    WHERE "session_guild" = ${guild}
    AND (${get_user} IS TRUE AND "session_user" = ${user})
      OR (${get_user} IS FALSE AND TRUE)
) SELECT "times_ago", SUM("session_time") AS "total_time", COUNT(*) AS "count"
FROM "weekly_data"
    WHERE "times_ago" <= 12
GROUP BY "times_ago"
ORDER BY "times_ago" ASC;
    `

    return data
  }

  if (timeframe === 'monthly') {
    // Sums meditation times that have the same "times_ago" value
    const data = await database.$queryRaw`
    WITH "monthly_data" AS (
      SELECT (EXTRACT(YEAR FROM NOW()) - EXTRACT(YEAR FROM "occurred_at"))*12 + (EXTRACT(MONTH FROM NOW()) - EXTRACT(MONTH FROM "occurred_at")) AS "times_ago", "session_time"
      FROM "Meditations"
      WHERE "session_guild" = ${guild}
      AND (${get_user} IS TRUE AND "session_user" = ${user})
        OR (${get_user} IS FALSE AND TRUE)
    ) SELECT "times_ago", SUM("session_time") AS "total_time", COUNT(*) AS "count"
    FROM "monthly_data"
    WHERE "times_ago" <= 12
    GROUP BY "times_ago"
    ORDER BY "times_ago" ASC;
    `

    return data
  }

  if (timeframe === 'yearly') {
    // Sums meditation times that have the same "times_ago" value
    const data = await database.$queryRaw`
    WITH "yearly_data" AS (
      SELECT extract(year from NOW()) - extract(year from "occurred_at") AS "times_ago", "session_time"
      FROM "Meditations"
      WHERE "session_guild" = ${guild}
      AND (${get_user} IS TRUE AND "session_user" = ${user})
        OR (${get_user} IS FALSE AND TRUE)
    ) SELECT "times_ago", SUM("session_time") AS "total_time", COUNT(*) AS "count"
    FROM "yearly_data"
    WHERE "times_ago" <= 12
    GROUP BY "times_ago"
    ORDER BY "times_ago" ASC;
    `

    return data
  }
}

export const make_chart = (raw_data, timeframe, type, color = `#${config.embedColor.toString(16)}`): Buffer => {
  const parsed_data = raw_data.reduce((acc, data) => {
    acc[data.times_ago] = data
    return acc
  }, {})

  // Makes the chart. Gets the last 12 days, weeks, months, or years of data. dd/mm/yyyy
  const data: Array<{ date, value }> = []

  if (timeframe === 'daily') {
    for (let i = 0; i < 12; i++) {
      const date = new Date()
      date.setDate(date.getDate() - i)
      const day = date.getDate()
      const month = date.getMonth() + 1

      let to_push = 0

      if (parsed_data[i]) {
        to_push = type === 'meditation_minutes' ? parsed_data[i].total_time : parsed_data[i].count
      }

      data.push({
        date: `${day}/${month}`,
        value: to_push
      })
    }
  } else if (timeframe === 'weekly') {
    for (let i = 0; i < 12; i++) {
      // Date has an offset of -7 days to show the start of the week
      const date = new Date()
      date.setDate(date.getDate() - ((i * 7) + 7))
      const day = date.getDate()
      const month = date.getMonth() + 1

      let to_push = 0

      if (parsed_data[i]) {
        to_push = type === 'meditation_minutes' ? parsed_data[i].total_time : parsed_data[i].count
      }

      data.push({
        date: `${day}/${month}`,
        value: to_push
      })
    }
  } else if (timeframe === 'monthly') {
    for (let i = 0; i < 12; i++) {
      const date = new Date()
      date.setMonth(date.getMonth() - i)
      const month = date.getMonth() + 1
      const year = date.getFullYear()

      let to_push = 0

      if (parsed_data[i]) {
        to_push = type === 'meditation_minutes' ? parsed_data[i].total_time : parsed_data[i].count
      }

      data.push({
        date: `${month}/${year}`,
        value: to_push
      })
    }
  } else if (timeframe === 'yearly') {
    for (let i = 0; i < 12; i++) {
      const date = new Date()
      date.setFullYear(date.getFullYear() - i)
      const year = date.getFullYear()

      let to_push = 0

      if (parsed_data[i]) {
        to_push = type === 'meditation_minutes' ? parsed_data[i].total_time : parsed_data[i].count
      }

      data.push({
        date: `${year}`,
        value: to_push
      })
    }
  }

  // Inverts the data so that it's in the right order
  data.reverse()

  const labels = data.map(d => d.date)
  const values = data.map(d => Number(d.value))
  const header = type === 'meditation_minutes' ? '# of Minutes' : '# of Sessions'

  // Makes the chart
  const canvas = createCanvas(400, 250)
  const canvas_ctx: any = canvas.getContext('2d')

  Chart.defaults.color = '#ffffff'

  new Chart(canvas_ctx, {
    type: 'bar',
    data: {
      labels,
      datasets: [
        {
          label: header,
          data: values,
          backgroundColor: color,
          borderColor: '#fff',
          borderWidth: 1
        }
      ]
    },
    options: {
      scales: {
        y: {
          beginAtZero: true
        }
      }
    }
  })

  return canvas.toBuffer()
}

export const get_all_time = async (interaction) => {
  const data = await database.meditations.aggregate({
    where: {
      session_guild: interaction.guildId
    },
    _sum: {
      session_time: true
    },
    _count: {
      id: true
    }
  })

  return data
}
