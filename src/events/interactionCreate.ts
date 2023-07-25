import { addTerm } from '../helpers/terms'
import * as pickwinnerActions from '../helpers/pickwinner'
import { database } from '../helpers/database'
import { rollbar } from '../helpers/rollbar'

export = async (client, interaction) => {
  try {
    if (interaction.isCommand()) {
      const command = interaction.client.commands.get(interaction.commandName)

      // Sets the guild ID if the command was executed in a guild, otherwise sets it to "null"
      const guild = interaction.guild ? interaction.guild.id : 'null'

      try {
        // Logs user's command usage in database
        await database.commandUsage.create({
          data: {
            command: interaction.commandName,
            user: interaction.user.id,
            guild: guild
          }
        })
      } catch {}

      if (!command) {
        rollbar.error(`No command matching ${interaction.commandName} was found.`)
        return
      }

      try {
        await command.execute(interaction)
      } catch (error: any) {
        rollbar.error(error)

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
        rollbar.error(`No command matching ${interaction.commandName} was found.`)
        return
      }

      try {
        await command.autocomplete(interaction)
      } catch (error: any) {
        rollbar.error(error)

        // Check if the command has been responded to already
        if (interaction.replied) {
          await interaction.followUp({ content: 'A fatal error occured while executing the command.', ephemeral: true })
        } else {
          await interaction.reply({ content: 'A fatal error occured while executing the command.', ephemeral: true })
        }
      }
    } else if (interaction.isButton()) {
      pickwinnerActions.acceptKey(interaction)
      pickwinnerActions.cancelKey(interaction)
    } else if (interaction.isModalSubmit()) {
      addTerm(interaction)
    }
  } catch (error: any) {
    rollbar.error(error)
  }
}
