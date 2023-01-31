import { PermissionsBitField, SlashCommandBuilder } from "discord.js";
import { adminCommand } from "../helpers/commandPermissions";
import { database } from "../helpers/database";
import { makeSearchable } from "../helpers/strings";

export = {
	data: new SlashCommandBuilder()
		.setName('addcourse')
		.setDescription('Adds a course and subsequent graduate role to the database.')
		.addStringOption(option =>
			option.setName('course_name')
				.setDescription('The course name to add.')
				.setRequired(true))
    .addRoleOption(option =>
      option.setName('participant_role')
        .setDescription('The role participants of the course have.')
        .setRequired(true))
    .addRoleOption(option =>
      option.setName('graduate_role')
        .setDescription('The role graduates of the course have.')
        .setRequired(true))
    .setDefaultMemberPermissions(adminCommand())
    .setDMPermission(false),
	async execute(interaction) {
		const courseName: string = interaction.options.getString('course_name');
    const participantRole = interaction.options.getRole('participant_role');
    const graduateRole = interaction.options.getRole('graduate_role');

		const courseExists = await database.courses.findFirst({
      where: {
        name: courseName
      }
    });

    if (courseExists) return interaction.reply({ content: ':x: Course already exists.', ephemeral: true });


    // Verifies that the roles exist and are not bot roles
    await interaction.guild.roles.fetch();
    const participantRoleExists = interaction.guild.roles.cache.find(role => role.id === participantRole.id);
    const graduateRoleExists = interaction.guild.roles.cache.find(role => role.id === graduateRole.id);

    if (!participantRoleExists) return interaction.reply({ content: ':x: Participant role does not exist.', ephemeral: true });
    if (!graduateRoleExists) return interaction.reply({ content: ':x: Graduate role does not exist.', ephemeral: true });

    if (participantRole.managed) return interaction.reply({ content: ':x: Participant role is a bot role.', ephemeral: true });
    if (graduateRole.managed) return interaction.reply({ content: ':x: Graduate role is a bot role.', ephemeral: true });

    // Ensures that the roles are not priveleged
    if (new PermissionsBitField(participantRole.permissions.bitfield).has('Administrator')) return interaction.reply({ content: `:x: Participant role has admin permission.`, ephemeral: true, allowedMentions: { roles: [] } });
    if (new PermissionsBitField(graduateRole.permissions.bitfield).has('Administrator')) return interaction.reply({ content: `:x: Graduate role has admin permissions.`, ephemeral: true, allowedMentions: { roles: [] } });

    // Ensures that the roles are not the same
    if (participantRole.id === graduateRole.id) return interaction.reply({ content: ':x: Participant and graduate roles cannot be the same.', ephemeral: true });

    await database.courses.create({
      data: {
        name: courseName,
        search: makeSearchable(courseName),
        participant_role: participantRole.id,
        graduate_role: graduateRole.id,
        guild: interaction.guild.id
      }
    });
		
		return interaction.reply({ content: ':white_check_mark: Course has been added.', ephemeral: true });
	},
};