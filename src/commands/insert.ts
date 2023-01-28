import { SlashCommandBuilder } from "discord.js";
import { modCommand } from "../helpers/commandPermissions";
import { database } from "../helpers/database";

export = {
	data: new SlashCommandBuilder()
		.setName('insert')
		.setDescription('Inserts a meditation session into the database.')
    .addUserOption(
      option => option.setName('user')
      .setDescription('The user to insert the meditation session for.')
      .setRequired(true))
    .addIntegerOption(option =>
      option.setName('date')
        .setDescription('The date of the meditation session. Use /timestamp to generate a UNIX timestamp.')
        .setRequired(true))
    .addIntegerOption(option =>
      option.setName('minutes')
        .setDescription('The number of minutes you want to add. Leave blank if you want to add 0 minutes to mend streaks.')
        .setRequired(false))
    .setDefaultMemberPermissions(modCommand())
    .setDMPermission(false),
	async execute(interaction) {
    const user = interaction.options.getUser('user');
    var timestamp: Date;

    try {
      timestamp = new Date(interaction.options.getInteger('date'));
    } catch (error) {
      return interaction.reply({ content: 'Invalid date. Use /timestamp to generate a UNIX timestamp.', ephemeral: true });
    }
    const minutes: number = interaction.options.getInteger('minutes') || 0;

		await database.meditations.create({
      data: {
        session_user: user.id,
        session_time: minutes,
        session_guild: interaction.guild.id,
        occurred_at: timestamp
      }
    })

    const human_date = `${timestamp.getMonth() + 1}/${timestamp.getDate()}/${timestamp.getFullYear()} ${timestamp.getHours()}:${timestamp.getMinutes()}`;

    await interaction.reply({ content: `Added ${minutes} minutes to ${user.username}'s meditation time at ${human_date}!` });
	},
};