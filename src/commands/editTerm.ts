import { SlashCommandBuilder, ModalBuilder, TextInputBuilder, ActionRowBuilder, TextInputStyle, type ModalActionRowComponentBuilder } from 'discord.js'
import { modCommand } from '../helpers/commandPermissions'
import { makeSearchable } from '../helpers/strings'
import { database } from '../helpers/database'

export = {
  data: new SlashCommandBuilder()
    .setName('updateterm')
    .setDescription('Updates an existing term in the glossary using a modal.')
    .addStringOption(option =>
      option.setName('term')
        .setDescription('The term you want to update.')
        .setAutocomplete(true)
        .setRequired(true))
    // .addStringOption(option =>
    //   option.setName('term')
    //     .setDescription('The term you want to update.')
    //     .setAutocomplete(true)
    //     .setRequired(true))
    // .addStringOption(option =>
    //   option.setName('name')
    //     .setDescription('The new name of the term.')
    //     .setRequired(false))
    // .addStringOption(option =>
    //   option.setName('definition')
    //     .setDescription('The new definition of the term.')
    //     .setRequired(false))
    // .addStringOption(option =>
    //   option.setName('usage')
    //     .setDescription('The new example sentence showing how to use the term.')
    //     .setRequired(false))
    // .addStringOption(option =>
    //   option.setName('category')
    //     .setDescription('The new category the term belongs to.')
    //     .setRequired(false))
    // .addStringOption(option =>
    //   option.setName('links')
    //     .setDescription('The new links to resources that explain the term. (Must be separated by commas.)')
    //     .setRequired(false))
    .setDefaultMemberPermissions(modCommand())
    .setDMPermission(false),
  async autocomplete (interaction) {
    const term = interaction.options.getString('term')
    const search = makeSearchable(term)

    console.log(search)

    const terms = await database.glossary.findMany({
      where: {
        search: {
          contains: search
        }
      }
    })

    const choices = terms.map(term => {
      return {
        name: term.term,
        value: term.term
      }
    })

    await interaction.respond(choices)
  },
  async execute (interaction) {
    const term = interaction.options.getString('term')
    const search = makeSearchable(term)

    const termData = await database.glossary.findUnique({
      where: {
        search
      }
    })

    if (termData == null) {
      return interaction.reply({ content: `The term **${term}** does not exist.`, ephemeral: true })
    }

    const modal = new ModalBuilder()
      .setCustomId('updateTermModal')
      .setTitle(`Edit '${term}'`)

    const termInput = new TextInputBuilder()
      .setCustomId('termInput')
      .setLabel('Term')
      .setPlaceholder('The name of the term.')
      .setValue(termData.term)
      .setStyle(TextInputStyle.Short)

    const definitionInput = new TextInputBuilder()
      .setCustomId('definitionInput')
      .setLabel('Definition')
      .setPlaceholder('The definition of the term.')
      .setMaxLength(1000)
      .setValue(termData.definition)
      .setStyle(TextInputStyle.Paragraph)

    const usageInput = new TextInputBuilder()
      .setCustomId('usageInput')
      .setLabel('Usage')
      .setPlaceholder('An example sentence showing how to use the term.')
      .setRequired(false)
      .setValue(termData.usage)
      .setStyle(TextInputStyle.Short)

    const categoryInput = new TextInputBuilder()
      .setCustomId('categoryInput')
      .setLabel('Category')
      .setPlaceholder('The category the term belongs to.')
      .setRequired(false)
      .setValue(termData.category)
      .setStyle(TextInputStyle.Short)

    const linksInput = new TextInputBuilder()
      .setCustomId('linksInput')
      .setLabel('Links')
      .setPlaceholder('Links to resources that explain the term. (Must be separated by commas.)')
      .setRequired(false)
      .setValue(termData.links.join(', '))
      .setStyle(TextInputStyle.Short)

    const firstRow = new ActionRowBuilder<ModalActionRowComponentBuilder>().addComponents(termInput)
    const secondRow = new ActionRowBuilder<ModalActionRowComponentBuilder>().addComponents(definitionInput)
    const thirdRow = new ActionRowBuilder<ModalActionRowComponentBuilder>().addComponents(usageInput)
    const fourthRow = new ActionRowBuilder<ModalActionRowComponentBuilder>().addComponents(categoryInput)
    const fifthRow = new ActionRowBuilder<ModalActionRowComponentBuilder>().addComponents(linksInput)

    modal.addComponents(firstRow, secondRow, thirdRow, fourthRow, fifthRow)

    await interaction.showModal(modal)

    // Listen for the modal response
    interaction.client.on('interactionCreate', async interaction => {
      if (!interaction.isModalSubmit()) return

      const term: string = interaction.fields.getTextInputValue('termInput')
      const definition: string = interaction.fields.getTextInputValue('definitionInput')
      const usage: string = interaction.fields.getTextInputValue('usageInput') || ''
      const category: string = interaction.fields.getTextInputValue('categoryInput') || ''
      const links: string = interaction.fields.getTextInputValue('linksInput')
      const search: string = makeSearchable(term)

      // Check if nothing was changed
      if (term === termData.term && definition === termData.definition && usage === termData.usage && category === termData.category && links === termData.links.join(', ')) {
        return interaction.reply({ content: 'You did not change anything!', ephemeral: true })
      }

      // Check if the updated term already exists besides the original term
      const termExists = await database.glossary.findUnique({
        where: {
          search
        }
      })

      if ((termExists != null) && termExists.id !== termData.id) {
        return interaction.reply({ content: `The term **${term}** already exists.`, ephemeral: true })
      }

      // Ensure that links are separated by commas, unless there are no links or only one link
      if (links) {
        const linksArray = links.split(',')

        // Ensure that all links are valid URLs
        for (let i = 0; i < linksArray.length; i++) {
          linksArray[i] = linksArray[i].trim()
          try {
            new URL(linksArray[i])
          } catch (error) {
            return interaction.reply({ content: 'One or more of your links is not a valid URL!', ephemeral: true })
          }
        }

        // Add the term to the database
        await database.glossary.create({
          data: {
            term,
            search,
            definition,
            usage,
            category,
            links: linksArray
          }
        })

        return interaction.reply({ content: 'Term added!', ephemeral: true })
      }

      // Add the term to the database
      await database.glossary.update({
        where: {
          id: termData.id
        },
        data: {
          term,
          search,
          definition,
          usage,
          category,
          links: []
        }
      })

      return interaction.reply({ content: 'Term updated!', ephemeral: true })
    })

    //   const term: string = interaction.options.getString('term');
    //   const search: string = makeSearchable(term);

    //   const terms = await database.glossary.findMany({
    //     where: {
    //       search: {
    //         contains: search
    //       }
    //     }
    //   });

    //   const suggestions = terms.map(term => {
    //     return {
    //       name: term.term,
    //       value: term.search
    //     };
    //   })

    //   await interaction.respond(suggestions)
    // },
    // async execute(interaction) {
    //   const oldterm: string = interaction.options.getString('term');
    //   const oldsearch: string = makeSearchable(oldterm);

    //   // Ensure that the term exists
    //   const term = await database.glossary.findUnique({
    //     where: {
    //       search: oldsearch
    //     }
    //   });

    //   if (!term) {
    //     return interaction.reply({ content: 'That term does not exist!', ephemeral: true });
    //   }

    //   const newterm: string = interaction.options.getString('name') || oldterm;
    //   const newsearch: string = makeSearchable(newterm) || oldsearch;
    //   const definition: string = interaction.options.getString('definition') || term.definition;
    //   const usage: string = interaction.options.getString('usage') || term.usage;
    //   const category: string = interaction.options.getString('category') || term.category;
    //   const links: string = interaction.options.getString('links') || term.links.join(', ');

    //   if (oldsearch !== newsearch) {
    //     // Ensure that the new term does not already exist
    //     const oldtermDb = await database.glossary.findUnique({
    //       where: {
    //         search: newsearch
    //       }
    //     });

    //     if (oldtermDb && oldtermDb.search === newsearch) {
    //       return interaction.reply({ content: 'That term already exists!', ephemeral: true });
    //     }
    //   }

    //   // Warn the database that nothing was updated if the user did not provide any new information
    //   if (oldsearch === newsearch && definition === term.definition && usage === term.usage && category === term.category && links === term.links.join(', ')) {
    //     return interaction.reply({ content: 'You did not provide any new information!', ephemeral: true });
    //   }

    //   // Ensure that links are separated by commas, unless there are no links or only one link
    //   if (links) {
    //     const linksArray = links.split(',');

    //     // Ensure that all links are valid URLs
    //     for (let i = 0; i < linksArray.length; i++) {
    //       linksArray[i] = linksArray[i].trim();
    //       try {
    //         new URL(linksArray[i]);
    //       } catch (error) {
    //         return interaction.reply({ content: 'One or more of your links is not a valid URL!', ephemeral: true });
    //       }
    //     }

    //     // Add the term to the database
    //     await database.glossary.update({
    //       where: {
    //         search: oldsearch
    //       },
    //       data: {
    //         term: newterm,
    //         definition: definition,
    //         usage: usage,
    //         category: category,
    //         links: linksArray

    //       }
    //     });
    //   } else {
    //     // Add the term to the database
    //     await database.glossary.update({
    //       where: {
    //         search: oldsearch
    //       },
    //       data: {
    //         term: newterm,
    //         definition: definition,
    //         usage: usage,
    //         category: category,
    //         links: []
    //       }
    //     });
    //   }

  //   return interaction.reply({ content: 'Term updated!', ephemeral: true });
  }
}
