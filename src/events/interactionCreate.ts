import * as pickwinnerActions from "../helpers/pickwinner";

export = async (client, interaction) => {
	if (interaction.isCommand()) {
		const command = interaction.client.commands.get(interaction.commandName);

		if (!command) {
			console.error(`No command matching ${interaction.commandName} was found.`);
			return;
		}
	
		try {
			await command.execute(interaction);
		} catch (error) {
			console.error(`Error executing ${interaction.commandName}`);
			console.error(error);

			await interaction.reply({ content: 'A fatal error occured while executing the command.', ephemeral: true });
		}
	} else if (interaction.isAutocomplete()) {
		const command = interaction.client.commands.get(interaction.commandName);

		if (!command) {
			console.error(`No command matching ${interaction.commandName} was found.`);
			return;
		}

		try {
			await command.autocomplete(interaction);
		} catch (error) {
			console.error(error);
		}
	} else if (interaction.isButton()) {
		pickwinnerActions.acceptKey(interaction);
		pickwinnerActions.cancelKey(interaction);
	}
}