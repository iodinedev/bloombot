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
				.setRequired(true)),
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
			await interaction.reply({ content: `The course does not exist: **${course}**.`, ephemeral: true });
			return;
		}

		// Ensure that the user is in the course
		const member = await interaction.guild.members.fetch(interaction.user.id);
		if (!member.roles.cache.has(courseEntry.participant_role)) {
			await interaction.reply({ content: `You are not in the course: **${course}**.`, ephemeral: true });
			return;
		}

		// Ensure that the user does not already have the role
		if (member.roles.cache.has(courseEntry.graduate_role)) {
			await interaction.reply({ content: `You have already completed the course: **${course}**.`, ephemeral: true });
			return;
		}

		// Add the role
		try {
			await member.roles.add(courseEntry.graduate_role);
			return interaction.reply({ content: `:tada: Congrats! I marked you as having completed the course: **${course}**.`, ephemeral: true });
		} catch (error: any) {
			if (error.code === 50013) {
				await interaction.reply({ content: `I don't have permission to give you the role for **${course}**.`, ephemeral: true });
				return;
			}

			throw error;
		}
	},
};