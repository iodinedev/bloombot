import { addTerm } from '../helpers/terms'
import * as pickwinnerActions from '../helpers/pickwinner'
import { database } from '../helpers/database'

export = async (client, interaction) => {
  if (interaction.isCommand()) {
    const command = interaction.client.commands.get(interaction.commandName)

    // Logs user's command usage in database
    await database.commandUsage.create({
      data: {
        command: interaction.commandName,
        user: interaction.user.id,
        guild: interaction.guild.id
      }
    })

    if (!command) {
      console.error(`No command matching ${interaction.commandName} was found.`)
      return
    }

    try {
      await command.execute(interaction)
    } catch (error) {
      console.error(`Error executing ${interaction.commandName}`)
      console.error(error)

      // Check if the command has been responded to already
      if (interaction.replied) {
        await interaction.followUp({ content: 'A fatal error occured while executing the command.', ephemeral: true })
      } else {
        await interaction.reply({ content: 'A fatal error occured while executing the command.', ephemeral: true })
      }
    }
  } else if (interaction.isAutocomplete()) {
    const command = interaction.client.commands.get(interaction.commandName)

    if (!command) {
      console.error(`No command matching ${interaction.commandName} was found.`)
      return
    }

    try {
      await command.autocomplete(interaction)
    } catch (error) {
      console.error(error)
    }
  } else if (interaction.isButton()) {
    pickwinnerActions.acceptKey(interaction)
    pickwinnerActions.cancelKey(interaction)
  } else if (interaction.isModalSubmit()) {
    addTerm(interaction)
  }
}
