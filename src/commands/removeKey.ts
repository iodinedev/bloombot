import { SlashCommandBuilder, ActionRowBuilder, ButtonBuilder, ButtonStyle } from "discord.js";
import { adminCommand } from "../helpers/commandPermissions";
import { database } from "../helpers/database";

export = {
	data: new SlashCommandBuilder()
		.setName('removekey')
		.setDescription('Removes a Playne key from the database.')
		.addStringOption(option =>
			option.setName('key')
				.setDescription('The key to remove.')
				.setRequired(true)
				.setAutocomplete(true))
    .setDefaultMemberPermissions(adminCommand())
    .setDMPermission(false),
	async autocomplete(interaction) {
		const key: string = interaction.options.getString('key');
		
		const possibleKeys = await database.steamKeys.findMany({
			where: {
				key: {
					contains: key
				}
			}
		});

		const suggestions = possibleKeys.map(key => {
			return {
				name: key.key,
				value: key.key
			};
		});

		await interaction.respond(suggestions)
	},
	async execute(interaction) {
		const key: string = interaction.options.getString('key');

		const keyExists = await database.steamKeys.findFirst({
			where: {
				key: key
			}
		});

		if (!keyExists) {
			return interaction.reply({ content: ':x: Key does not exist.', ephemeral: true });
		}

		if (!keyExists.used) {
			const row = new ActionRowBuilder()
      .addComponents(
        new ButtonBuilder()
          .setCustomId('yes')
          .setLabel('Yes')
          .setStyle(ButtonStyle.Danger),
        new ButtonBuilder()
          .setCustomId('no')
          .setLabel('No')
          .setStyle(ButtonStyle.Primary)
      );

			interaction.reply({ content: 'Are you sure you want to delete this key? It hasn\'t been used yet.', ephemeral: true, components: [row] });

			const filter = i => i.user.id === interaction.user.id;
			const collector = interaction.channel.createMessageComponentCollector({ filter, time: 15000 });

			collector.on('collect', async i => {
				if (i.customId === 'yes') {
					await database.steamKeys.delete({
						where: {
							key: key
						}
					});

					interaction.editReply({ content: 'Key deleted!', ephemeral: true, components: [] });
				} else if (i.customId === 'no') {
					interaction.editReply({ content: 'Key not deleted.', ephemeral: true });
				}
			})

			collector.on('end', collected => {
				if (collected.size === 0) {
					interaction.editReply({ content: 'You did not respond in time. Key not deleted.', ephemeral: true });
				}
			})
		} else {
			await database.steamKeys.delete({
				where: {
					key: key
				}
			});

			interaction.reply({ content: 'Key deleted!', ephemeral: true, components: [] });
		}
	},
};