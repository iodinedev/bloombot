import { SlashCommandBuilder } from "discord.js";
import { database } from "../helpers/database";
import { updateRoles } from "../helpers/streaks";

export = {
	data: new SlashCommandBuilder()
		.setName('add')
		.setDescription('Adds minutes to your meditation time.')
    .addIntegerOption(option =>
      option.setName('minutes')
        .setDescription('The number of minutes you want to add.')
        .setRequired(true)),
	async execute(interaction) {
		const minutes: number = interaction.options.getInteger('minutes');
    const now = `${Date.now()}`;
    const user = interaction.user.id;
    const guild = interaction.guild.id;

    await updateRoles(interaction.client, interaction.guild, interaction.user);

    await database.meditations.create({
      data: {
        session_user: user,
        session_time: minutes,
        session_guild: guild
      }
    })

    await interaction.reply({ content: `Added ${minutes} minutes to your meditation time!` });
	},
};