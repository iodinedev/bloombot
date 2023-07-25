import { config } from 'dotenv'
config()

import { GatewayIntentBits, Partials, Client } from 'discord.js'
import { readdirSync  } from 'fs'
import { join } from 'path'
import { deployAppCommands } from './helpers/deploy'
import { rollbar } from './helpers/rollbar';
import { probeServer } from './helpers/probeServer'

process.on('unhandledRejection', (reason, promise) => {
  rollbar.error(reason, promise);
  probeServer.notLive()
})

const client = new Client({
  intents: [GatewayIntentBits.Guilds, GatewayIntentBits.GuildMembers, GatewayIntentBits.GuildMessages, GatewayIntentBits.MessageContent, GatewayIntentBits.GuildMessageReactions],
  partials: [Partials.Message, Partials.Channel, Partials.Reaction]
})

probeServer.init(client).start()

readdirSync(join(__dirname, 'events')).filter(file => file.endsWith('.js')).forEach(file => {
  const event = require(join(__dirname, `events/${file}`))
  const eventName = file.split('.')[0]
  client.on(eventName, event.bind(null, client))
})

// Deploys slash commands and adds them to the client.commands collection
deployAppCommands(client)

client.login(process.env.DISCORD_TOKEN)