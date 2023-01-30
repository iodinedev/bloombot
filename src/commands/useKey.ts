import { SlashCommandBuilder } from "discord.js";
import { database } from "../helpers/database";
import { adminCommand } from "../helpers/commandPermissions";

export = {
	data: new SlashCommandBuilder()
		.setName('usekey')
		.setDescription('Use your key to redeem a game.')
		.setDefaultMemberPermissions(adminCommand())
    .setDMPermission(false),
	async execute(interaction) {
		const key = await database.steamKeys.findFirst({
			where: {
				used: false,
			}
		});

		if (!key) {
			return interaction.reply({ content: ':x: No keys available.', ephemeral: true });
		}

		await database.steamKeys.update({
			where: {
				key: key.key
			},
			data: {
				used: true,
			}
		});

		return interaction.reply({ content: `:white_check_mark: Key has successfully been selected and marked as used.\n\n\`\`\`${key.key}\`\`\`\n\n**This message is ephemeral and the key will be lost if you do not copy and paste it somewhere private.**`, ephemeral: true });
	},
};