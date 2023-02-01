import { ButtonBuilder, ActionRowBuilder, ButtonStyle, EmbedBuilder, SlashCommandBuilder } from "discord.js";
import { adminCommand } from "../helpers/commandPermissions";
import { database } from "../helpers/database";
import { config } from "../config";

const getWinner = (roleCandidates: any[], databaseCandidates: any[]): any | null => {
	const database_ids = databaseCandidates.map(candidate => candidate.session_user);
	const role_ids = roleCandidates.map(candidate => candidate.id);

	const intersection = database_ids.filter(id => role_ids.includes(id));

	if (intersection.length === 0) {
		return null;
	}

	const winner = roleCandidates.find(member => member.id === intersection[Math.floor(Math.random() * intersection.length)]);

	return winner;
};

export = {
	data: new SlashCommandBuilder()
		.setName('pickwinner')
		.setDescription('Picks the winner and gives them an unused key.')
		.addIntegerOption(option =>
			option.setName('month')
				.setDescription('The month to pick the winner for.')
				.addChoices(
					{
						name: 'January',
						value: 1
					},
					{
						name: 'February',
						value: 2
					},
					{
						name: 'March',
						value: 3
					},
					{
						name: 'April',
						value: 4
					},
					{
						name: 'May',
						value: 5
					},
					{
						name: 'June',
						value: 6
					},
					{
						name: 'July',
						value: 7
					},
					{
						name: 'August',
						value: 8
					},
					{
						name: 'September',
						value: 9
					},
					{
						name: 'October',
						value: 10
					},
					{
						name: 'November',
						value: 11
					},
					{
						name: 'December',
						value: 12
					}
				)
				.setRequired(true))
    .setDefaultMemberPermissions(adminCommand())
    .setDMPermission(false),
	async execute(interaction) {
		const month = interaction.options.getInteger('month') - 1;
		const start = new Date(new Date((new Date()).setMonth(month)).setDate(1)).setHours(0, 0, 0, 0);
		const end = new Date(new Date((new Date()).setMonth(month + 1)).setDate(1)).setHours(0, 0, 0, 0);

		await interaction.deferReply({ ephemeral: true });

		const keyExists = await database.steamKeys.findFirst({
			where: {
				used: false,
				reserved: null
			}
		});

		if (!keyExists) {
			return interaction.editReply({ content: ':x: No keys available.', ephemeral: true });
		}

		await interaction.guild.members.fetch();

		const candidates = await database.meditations.findMany({
			where: {
				session_guild: interaction.guild.id,
				occurred_at: {
					gte: new Date(start),
					lt: new Date(end)
				},
				session_time: {
					gt: 0
				}
			}
		});

		const members = interaction.guild.members.cache.filter(member => member.roles.cache.has(config.roles.meditation_challenger));
	
		const member = getWinner(members, candidates);

		if (!member) {
			return interaction.editReply({ content: ':x: No members available.', ephemeral: true });
		}
		
		try {
			// Using the database to do this aggregation is faster, even though it requires a second query.
			const monthly_total = await database.meditations.aggregate({
				where: {
					session_user: member.id,
					session_guild: interaction.guild.id,
					occurred_at: {
						gte: new Date(start),
						lt: new Date(end)
					},
					session_time: {
						gt: 0
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
		} catch (error: any) {
			if (error.code === 50013) {
				return interaction.editReply({ content: ':x: I do not have permission to send messages in the announcement channel. Nothing has been sent to the user.', ephemeral: true });
			} else if (error.code === 50001) {
				return interaction.editReply({ content: ':x: I do not have permission to view the announcement channel. Nothing has been sent to the user.', ephemeral: true });
			}

			throw error;
		}

		const row = new ActionRowBuilder()
      .addComponents(
        new ButtonBuilder()
          .setCustomId('redeemKey')
					.setLabel('Redeem')
					.setStyle(ButtonStyle.Primary),
        new ButtonBuilder()
          .setCustomId('cancelKey')
					.setLabel('Cancel')
					.setStyle(ButtonStyle.Danger)
      );

		const dmEmbed = new EmbedBuilder()
			.setTitle(":tada: You've won a key! :tada:")
			.setColor(config.embedColor)
			.setThumbnail(member.user.avatarURL())
			.setFields([
				{
					name: `**Congratulations!**`,
					value: `**Congratulations on winning the giveaway!** 🥳\n\nYou've won a key for Playne: The Meditation Game on Steam!\n\n**Would you like to redeem your key? Press 'Redeem' below! Otherwise, click 'Cancel' to keep it for someone else :\\)**`
				}
			])
			.setFooter({ text: `From ${interaction.guild.name}. If you have any problems, please contact a moderator and we will be happy to help!`, iconURL: interaction.guild.iconURL() })

		try {
			await member.send({ embeds: [dmEmbed], components: [row] });
		} catch {
			return interaction.editReply({ content: `:x: Could not send DM to member. Please run \`/usekey\` and copy a key manually if they want one.\n\n**No key has been used.**`, ephemeral: true });
		}

		await database.steamKeys.update({
			where: {
				key: keyExists.key
			},
			data: {
				reserved: member.id
			}
		});

		await interaction.editReply({ content: `:white_check_mark: Successfully picked winner and sent them a key.`, ephemeral: true });
	},
};