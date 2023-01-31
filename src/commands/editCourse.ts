import { PermissionsBitField, SlashCommandBuilder } from "discord.js";
import { makeSearchable } from "../helpers/glossary";
import { database } from "../helpers/database";
import { adminCommand } from "../helpers/commandPermissions";

export = {
	data: new SlashCommandBuilder()
		.setName('editcourse')
		.setDescription('Updates an existing course.')
    .addStringOption(option =>
      option.setName('course')
        .setDescription('The course you want to update.')
        .setAutocomplete(true)
        .setRequired(true))
    .addStringOption(option =>
      option.setName('name')
        .setDescription('The new name of the course.')
        .setRequired(false))
    .addRoleOption(option =>
      option.setName('participant_role')
        .setDescription('The new participant role of the course.')
        .setRequired(false))
    .addRoleOption(option =>
      option.setName('graduate_role')
        .setDescription('The new graduate role of the course.')
        .setRequired(false))
    .setDefaultMemberPermissions(adminCommand())
    .setDMPermission(false),
  async autocomplete(interaction) {
    const course: string = interaction.options.getString('course');
    const search: string = makeSearchable(course);

    const courses = await database.courses.findMany({
      where: {
        name: {
          contains: search
        }
      }
    });

    const suggestions = courses.map(course => {
      return {
        name: course.name,
        value: course.name
      };
    })

    await interaction.respond(suggestions)
  },
  async execute(interaction) {
    const oldcourse: string = interaction.options.getString('course');
    const oldsearch: string = makeSearchable(oldcourse);

    // Ensure that the course exists
    const course = await database.courses.findUnique({
      where: {
        search: oldsearch
      }
    });

    if (!course) {
      return interaction.reply({ content: 'That course does not exist!', ephemeral: true });
    }

    const newcourse: string = interaction.options.getString('name') || oldcourse;
    const newsearch: string = makeSearchable(newcourse) || oldsearch;
    const participantRole: string = interaction.options.getRole('participant_role').id || course.participant_role;
    const graduateRole: string = interaction.options.getRole('graduate_role').id || course.graduate_role;

    if (oldsearch !== newsearch) {
      // Ensure that the new course name does not already exist
      const newcourse = await database.courses.findUnique({
        where: {
          search: newsearch
        }
      });

      if (newcourse) {
        return interaction.reply({ content: 'That course name already exists!', ephemeral: true });
      }
    }

    // Warn the database that nothing was updated if the user did not provide any new information
    if (oldsearch === newsearch && participantRole === course.participant_role && graduateRole === course.graduate_role) {
      return interaction.reply({ content: 'You did not provide any new information!', ephemeral: true });
    }

    // Verifies that the roles exist
    await interaction.guild.roles.fetch();
    const participantRoleExists = interaction.guild.roles.cache.find(role => role.id === participantRole);
    const graduateRoleExists = interaction.guild.roles.cache.find(role => role.id === graduateRole);

    if (!participantRoleExists) return interaction.reply({ content: ':x: Participant role does not exist.', ephemeral: true });
    if (!graduateRoleExists) return interaction.reply({ content: ':x: Graduate role does not exist.', ephemeral: true });

    if (participantRoleExists.managed) return interaction.reply({ content: ':x: Participant role is a bot role.', ephemeral: true });
    if (graduateRoleExists.managed) return interaction.reply({ content: ':x: Graduate role is a bot role.', ephemeral: true });

    // Ensures that the roles are nidot priveleged
    if (new PermissionsBitField(participantRoleExists.permissions.bitfield).has('Administrator')) return interaction.reply({ content: `:x: Participant role has admin permission.`, ephemeral: true, allowedMentions: { roles: [] } });
    if (new PermissionsBitField(graduateRoleExists.permissions.bitfield).has('Administrator')) return interaction.reply({ content: `:x: Graduate role has admin permissions.`, ephemeral: true, allowedMentions: { roles: [] } });

    // Ensures that the roles are not the same
    if (participantRole === graduateRole) return interaction.reply({ content: ':x: Participant and graduate roles cannot be the same.', ephemeral: true });

    if (!participantRoleExists) return interaction.reply({ content: ':x: Participant role does not exist.', ephemeral: true });
    if (!graduateRoleExists) return interaction.reply({ content: ':x: Graduate role does not exist.', ephemeral: true });
    
    // Update the course
    await database.courses.update({
      where: {
        search: oldsearch
      },
      data: {
        name: newcourse,
        search: newsearch,
        participant_role: participantRole,
        graduate_role: graduateRole
      }
    });

    return interaction.reply({ content: 'Course updated!', ephemeral: true });
  }
};