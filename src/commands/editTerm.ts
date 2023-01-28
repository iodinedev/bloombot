import { SlashCommandBuilder } from "discord.js";
import { makeSearchable } from "../helpers/glossary";
import { database } from "../helpers/database";
import { adminCommand } from "../helpers/commandPermissions";

export = {
	data: new SlashCommandBuilder()
		.setName('updateterm')
		.setDescription('Updates an existing term in the glossary.')
    .addStringOption(option =>
      option.setName('term')
        .setDescription('The term you want to update.')
        .setAutocomplete(true)
        .setRequired(true))
    .addStringOption(option =>
      option.setName('name')
        .setDescription('The new name of the term.')
        .setRequired(false))
    .addStringOption(option =>
      option.setName('definition')
        .setDescription('The new definition of the term.')
        .setRequired(false))
    .addStringOption(option =>
      option.setName('usage')
        .setDescription('The new example sentence showing how to use the term.')
        .setRequired(false))
    .addStringOption(option =>
      option.setName('category')
        .setDescription('The new category the term belongs to.')
        .setRequired(false))
    .addStringOption(option =>
      option.setName('links')
        .setDescription('The new links to resources that explain the term. (Must be separated by commas.)')
        .setRequired(false))
    .setDefaultMemberPermissions(adminCommand())
    .setDMPermission(false),
  async autocomplete(interaction) {
    const term: string = interaction.options.getString('term');
    const search: string = makeSearchable(term);

    const terms = await database.glossary.findMany({
      where: {
        search: {
          contains: search
        }
      }
    });

    const suggestions = terms.map(term => {
      return {
        name: term.term,
        value: term.search
      };
    })

    await interaction.respond(suggestions)
  },
  async execute(interaction) {
    const oldterm: string = interaction.options.getString('term');
    const oldsearch: string = makeSearchable(oldterm);

    // Ensure that the term exists
    const term = await database.glossary.findUnique({
      where: {
        search: oldsearch
      }
    });

    if (!term) {
      return interaction.reply({ content: 'That term does not exist!', ephemeral: true });
    }

    const newterm: string = interaction.options.getString('name') || oldterm;
    const newsearch: string = makeSearchable(newterm) || oldsearch;
    const definition: string = interaction.options.getString('definition') || term.definition;
    const usage: string = interaction.options.getString('usage') || term.usage;
    const category: string = interaction.options.getString('category') || term.category;
    const links: string = interaction.options.getString('links') || term.links.join(', ');

    if (oldsearch !== newsearch) {
      // Ensure that the new term does not already exist
      const newterm = await database.glossary.findUnique({
        where: {
          search: newsearch
        }
      });

      if (newterm) {
        return interaction.reply({ content: 'That term already exists!', ephemeral: true });
      }
    }

    // Warn the database that nothing was updated if the user did not provide any new information
    if (oldsearch === newsearch && definition === term.definition && usage === term.usage && category === term.category && links === term.links.join(', ')) {
      return interaction.reply({ content: 'You did not provide any new information!', ephemeral: true });
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
      await database.glossary.update({
        where: {
          search: oldsearch
        },
        data: {
          term: newterm,
          definition: definition,
          usage: usage,
          category: category,
          links: linksArray

        }
      });
    } else {
      // Add the term to the database
      await database.glossary.update({
        where: {
          search: oldsearch
        },
        data: {
          term: newterm,
          definition: definition,
          usage: usage,
          category: category,
          links: []
        }
      });
    }

    return interaction.reply({ content: 'Term updated!', ephemeral: true });
  }
};