import Discord from 'discord.js'
import fs from 'fs'
import path from 'path'
import { rollbar } from './rollbar'

const commands: any[] = []
const commandFiles = fs.readdirSync(path.join(__dirname, '../commands/')).filter(file => file.endsWith('.js'))

export const deployAppCommands = async (client) => {
  const rest = new Discord.REST({ version: '10' }).setToken(process.env.DISCORD_TOKEN!)
  client.commands = new Discord.Collection()

  for (const file of commandFiles) {
    const filePath = path.join(__dirname, '../commands/', file)
    const command = require(filePath)

    if ('data' in command && 'execute' in command) {
      commands.push(command.data.toJSON())
      client.commands.set(command.data.name, command)
    } else {
      console.log(`[WARNING] The command at ${filePath} is missing a required "data" or "execute" property.`)
    }
  }

  try {
    if (process.env.PRODUCTION! === 'true') {
      await rest.put(
        Discord.Routes.applicationCommands(process.env.CLIENT_ID!),
        { body: commands }
      )
    } else {
      console.log(`[WARNING] The bot is not in production mode. The commands will only be deployed to the guild with ID ${process.env.TEST_GUILD_ID}.`)

      try {
        await rest.put(
          Discord.Routes.applicationGuildCommands(process.env.CLIENT_ID!, process.env.TEST_GUILD_ID!),
          { body: commands }
        )
      } catch (error: any) {
        // APPLICATION_COMMAND_PERMISSIONS_UPDATE
        if (error.code === 50001) {
          return console.log(`[WARNING] The bot does not have permission to manage slash commands in the guild with ID ${process.env.TEST_GUILD_ID}.`)
        }

        // APPLICATION_COMMANDS_DUPLICATE_NAME
        if (error.code === 50041) {
          return console.log(`[WARNING] The bot has a duplicate command name in the guild with ID ${process.env.TEST_GUILD_ID}.`)
        }

        rollbar.error(error)
      }
    }
  } catch (error: any) {
    rollbar.error(error)
  }
}
