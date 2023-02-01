import { CategoryChannel, EmbedBuilder, SlashCommandBuilder } from "discord.js";
import { adminCommand } from "../helpers/commandPermissions";
import { database } from "../helpers/database";
import { config } from "../config";

export = {
	data: new SlashCommandBuilder()
		.setName('pickwinner')
		.setDescription('Picks the winner and gives them an unused key.')
    .setDefaultMemberPermissions(adminCommand())
    .setDMPermission(false),
	async execute(interaction) {
		const keyExists = await database.steamKeys.findFirst({
			where: {
				used: false
			}
		});

		if (!keyExists) {
			return interaction.reply({ content: ':x: No keys available.', ephemeral: true });
		}

		await interaction.guild.members.fetch();

		const members = interaction.guild.members.cache.filter(member => member.roles.cache.has(config.roles.meditation_challenger));
		const member = members.random();

		if (!member) {
			return interaction.reply({ content: ':x: No members available.', ephemeral: true });
		}

		await database.steamKeys.update({
			where: {
				key: keyExists.key
			},
			data: {
				used: true
			}
		});

		const dmEmbed = new EmbedBuilder()
			.setTitle(":tada: You've won a key! :tada:")
			.setThumbnail(member.user.avatarURL())
			.setFields([
				{
					name: `**Congratulations!**`,
					value: `You've won a key for Playne: The Meditation Game on Steam!\n\n**Your key is:** \`${keyExists.key}\`\n\nThanks for participating in the meditation challenges!`
				}
			])
			.setFooter({ text: `From ${interaction.guild.name}` })

		try {
			await member.send({ embeds: [dmEmbed] });
		} catch {
			return interaction.reply({ content: `:x: Could not send DM to member. The key chosen is \`${keyExists.key}\`. Please copy it and provide it to the user elsewhere.\n\n**This message is ephemeral and the key will be lost if you do not copy it immediately.**`, ephemeral: true });
		}

		const monthly_total = await database.meditations.aggregate({
			where: {
				session_user: member.id,
				session_guild: interaction.guild.id,
				occurred_at: {
					gte: new Date(new Date().setDate(1))
				}
			},
			_sum: {
				session_time: true
			}
		});

		const date = new Date();
		const day = date.getDate();
		const month = date.getMonth() + 1;
		const year = date.getFullYear();

		const time = monthly_total._sum.session_time ? monthly_total._sum.session_time : 0;

		const announcement_embed = new EmbedBuilder()
			.setTitle(":tada: This month's meditation challenger in the spotlight is... :tada:")
			.setThumbnail(member.user.avatarURL())
			.setFields([
				{
					name: `**Monthly hall-of-fame member**`,
					value: `**${member.user}** is our server member of the month, with a meditation time of **${time}** minutes!\nYou're doing great, keep at it!`
				}
			])
			.setFooter({ text: `Meditation challenge for ${month}/${year} | Selected on ${day}/${month}/${year}` })

		const announcement_channel = await interaction.guild.channels.fetch(config.channels.announcement);

		await announcement_channel.send({ embeds: [announcement_embed] });		

		await interaction.reply({ content: `:white_check_mark: Successfully picked winner and sent them a key.`, ephemeral: true });
	},
};