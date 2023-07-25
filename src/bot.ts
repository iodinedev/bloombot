import * as dotenv from 'dotenv'
dotenv.config()

// Require the necessary discord.js classes
import Discord from 'discord.js'
import * as fs from 'fs'
import path from 'path'
import * as deploy from './helpers/deploy'
import { database } from './helpers/database'
import { backup } from './helpers/backup'
import { rollbar } from './helpers/rollbar'

let startup_sucessful = false;

console.log("Starting up...");

(async () => {
  try {
    // Backup the database
    setInterval(() => {
      backup(client)
    }, 1000 * 60 * 60 * 24)
  } catch (err: any) {
    rollbar.error(err)
    startup_sucessful = false
  }

  // Try to connect to the database
  try {
    await database.$connect()
  } catch (err: any) {
    rollbar.error(err)
    startup_sucessful = false
  }

  // Prints the database name
  try {
    const current_database: any = await database.$queryRaw`select current_database()`
    console.log(`Current database: ${current_database[0].current_database}`)
  } catch (err: any) {
    rollbar.error(err)
    startup_sucessful = false
  }

  const client = new Discord.Client({
    intents: [Discord.GatewayIntentBits.Guilds, Discord.GatewayIntentBits.GuildMembers, Discord.GatewayIntentBits.GuildMessages, Discord.GatewayIntentBits.MessageContent, Discord.GatewayIntentBits.GuildMessageReactions],
    partials: [Discord.Partials.Message, Discord.Partials.Channel, Discord.Partials.Reaction]
  })

  fs.readdirSync(path.join(__dirname, 'events')).filter(file => file.endsWith('.js')).forEach(file => {
    const event = require(path.join(__dirname, `events/${file}`))
    const eventName = file.split('.')[0]
    client.on(eventName, event.bind(null, client))
  })

  // Deploys slash commands and adds them to the client.commands collection
  deploy.deployAppCommands(client, startup_sucessful)

  client.login(process.env.DISCORD_TOKEN)
})()