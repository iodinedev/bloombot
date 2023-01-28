import { SlashCommandBuilder } from "discord.js";
import { database } from "../helpers/database";

export = {
	data: new SlashCommandBuilder()
		.setName('usekey')
		.setDescription('Use your key to redeem a game.')
    .setDMPermission(true),
	async execute(interaction) {
		const key = await database.steamKeys.findFirst({
			where: {
				reservation: interaction.user.id
			}
		});

		if (!key) {
			return interaction.reply({ content: ':x: You do not have a key. You can win one by participating in the meditation challenges!', ephemeral: true });
		}

		if (interaction.inGuild()) {
			return interaction.reply({ content: ':x: This command can only be used in DMs.', ephemeral: true });
		}

		await database.steamKeys.update({
			where: {
				key: key.key
			},
			data: {
				used: true,
			}
		});

		return interaction.reply({ content: `:white_check_mark: Your key is: \`${key.key}\`` });
	},
};