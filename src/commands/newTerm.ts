import { SlashCommandBuilder } from "discord.js";
import { makeSearchable } from "../helpers/strings";
import { database } from "../helpers/database";
import { modCommand } from "../helpers/commandPermissions";

export = {
	data: new SlashCommandBuilder()
		.setName('newterm')
		.setDescription('Adds a new term to the glossary.')
    .addStringOption(option =>
      option.setName('term')
        .setDescription('The term you want to add.')
        .setRequired(true))
    .addStringOption(option =>
      option.setName('definition')
        .setDescription('The definition of the term.')
        .setRequired(true))
    .addStringOption(option =>
      option.setName('usage')
        .setDescription('An example sentence showing how to use the term.')
        .setRequired(true))
    .addStringOption(option =>
      option.setName('category')
        .setDescription('The category the term belongs to.')
        .setRequired(true))
    .addStringOption(option =>
      option.setName('links')
        .setDescription('Links to resources that explain the term. (Must be separated by commas.)')
        .setRequired(false))
    .setDefaultMemberPermissions(modCommand())
    .setDMPermission(false),
	async execute(interaction) {
    const term: string = interaction.options.getString('term');
    const definition: string = interaction.options.getString('definition');
    const usage: string = interaction.options.getString('usage');
    const category: string = interaction.options.getString('category');
    const links: string = interaction.options.getString('links');
    const search: string = makeSearchable(term);

    // Ensure that the term does not already exist
    const verifyTerm = await database.glossary.findUnique({
      where: {
        search: search
      }
    });

    if (verifyTerm) {
      return interaction.reply({ content: 'That term already exists!', ephemeral: true });
    }

    // Ensure that links are separated by commas, unless there are no links or only one link
    if (links) {
      const linksArray = links.split(',');
      
      // Ensure that all links are valid URLs
      for (let i = 0; i < linksArray.length; i++) {
        linksArray[i] = linksArray[i].trim();
        try {
          new URL(linksArray[i]);
        } catch (error) {
          return interaction.reply({ content: 'One or more of your links is not a valid URL!', ephemeral: true });
        }
      }

      // Add the term to the database
      await database.glossary.create({
        data: {
          term: term,
          search: search,
          definition: definition,
          usage: usage,
          category: category,
          links: linksArray,
        }
      });

      return interaction.reply({ content: 'Term added!', ephemeral: true });
    }

    // Add the term to the database
    await database.glossary.create({
      data: {
        term: term,
        search: search,
        definition: definition,
        usage: usage,
        category: category,
        links: [],
      }
    });

    return interaction.reply({ content: 'Term added!', ephemeral: true });
  },
};