import * as dotenv from 'dotenv'
dotenv.config()

// Require the necessary discord.js classes
import Discord from 'discord.js'
import * as fs from 'fs'
import path from 'path'
import * as deploy from './helpers/deploy'


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
deploy.deployAppCommands(client)

client.login(process.env.DISCORD_TOKEN)