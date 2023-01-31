import { SlashCommandBuilder } from "discord.js";
import { database } from "../helpers/database";
import { makeSearchable } from "../helpers/glossary";

export = {
	data: new SlashCommandBuilder()
		.setName('coursecomplete')
		.setDescription('Mark that you have completed a course.')
    .addStringOption(option =>
      option.setName('course')
				.setDescription('The course you want to mark as complete.')
				.setAutocomplete(true)
				.setRequired(true)),
	async autocomplete(interaction) {
		const course: string = interaction.options.getString('course');
		const search: string = makeSearchable(course);

		const terms = await database.courses.findMany({
			where: {
				name: {
					contains: search
				}
			}
		});

		const suggestions = terms.map(term => {
			return {
				name: term.name,
				value: term.name
			};
		});

		await interaction.respond(suggestions);
	},
	async execute(interaction) {
		const course: string = interaction.options.getString('course');
		const search: string = makeSearchable(course);

		// Ensure that the course exists
		const courseEntry = await database.courses.findUnique({
			where: {
				search: search
			}
		});

		if (!courseEntry) {
			await interaction.reply(`The course **${course}** does not exist.`);
			return;
		}

		// Ensure that the user is in the course
		const member = await interaction.guild.members.fetch(interaction.user.id);
		if (!member.roles.cache.has(courseEntry.participant_role)) {
			await interaction.reply(`You are not in the course **${course}**.`);
			return;
		}

		// Ensure that the user does not already have the role
		if (member.roles.cache.has(courseEntry.graduate_role)) {
			await interaction.reply(`You have already completed the course **${course}**.`);
			return;
		}

		// Add the role
		await member.roles.add(courseEntry.graduate_role);
		return interaction.reply(`:tada: Congrats! I marked you as having completed the course **${course}**.`);
	},
};