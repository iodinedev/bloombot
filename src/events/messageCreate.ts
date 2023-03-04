import path from 'path'
import * as fs from 'fs'

export = async (client, message) => {
  if (message.author.bot) return

  const args = message.content.split(/\s+/g)
  let prefix = '.'

  if (
    args[0] === `<@!${client.user.id}>` ||
    message.content.startsWith(`<@!${client.user.id}>`)
  ) {
    prefix = `<@!${client.user.id}>`
    if (args[0] === prefix) {
      args.shift()
      args[0] = prefix + args[0] // Dirty fix
    }
  }

  const command =
    message.content.startsWith(prefix) && args.shift().slice(prefix.length).split(' ')[0].toLowerCase()

  if (command) {
    const commandfile = fs.readdirSync(path.join(__dirname, '../commands/')).filter(file => file.endsWith('.js')).find(file => file === `${command}.js`)

    if (commandfile) {
      message.channel.sendTyping()

      return await message.channel.send('Message commands have been sunsetted. Please use slash commands instead.')
    }
  }
}
