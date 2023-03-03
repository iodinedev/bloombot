import { database } from "./database";
import { makeSearchable } from "./strings";

export const addTerm = async (interaction) => {
  // Check customId to ensure that this is the correct modal
  if (interaction.customId !== 'termModal') {
    return;
  }
  
  const term: string = interaction.fields.getTextInputValue('termInput');
  const definition: string = interaction.fields.getTextInputValue('definitionInput');
  const usage: string = interaction.fields.getTextInputValue('usageInput') || '';
  const category: string = interaction.fields.getTextInputValue('categoryInput') || '';
  const links: string = interaction.fields.getTextInputValue('linksInput');
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
}