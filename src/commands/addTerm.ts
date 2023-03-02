import { SlashCommandBuilder, ModalBuilder, TextInputBuilder, TextInputStyle, ActionRowBuilder, ModalActionRowComponentBuilder } from "discord.js";
import { makeSearchable } from "../helpers/strings";
import { database } from "../helpers/database";
import { modCommand } from "../helpers/commandPermissions";

export = {
	data: new SlashCommandBuilder()
		.setName('addterm')
		.setDescription('Adds a new term to the glossary. Uses a modal to collect information.')
    // .addStringOption(option =>
    //   option.setName('term')
    //     .setDescription('The term you want to add.')
    //     .setRequired(true))
    // .addStringOption(option =>
    //   option.setName('definition')
    //     .setDescription('The definition of the term.')
    //     .setRequired(true))
    // .addStringOption(option =>
    //   option.setName('usage')
    //     .setDescription('An example sentence showing how to use the term.')
    //     .setRequired(true))
    // .addStringOption(option =>
    //   option.setName('category')
    //     .setDescription('The category the term belongs to.')
    //     .setRequired(true))
    // .addStringOption(option =>
    //   option.setName('links')
    //     .setDescription('Links to resources that explain the term. (Must be separated by commas.)')
    //     .setRequired(false))
    .setDefaultMemberPermissions(modCommand())
    .setDMPermission(false),
	async execute(interaction) {
    const modal = new ModalBuilder()
			.setCustomId('termModal')
			.setTitle('Add a new term');

    const termInput = new TextInputBuilder()
      .setCustomId('termInput')
      .setLabel('Term')
      .setPlaceholder('Enter the term you want to add.')
      .setStyle(TextInputStyle.Short);

    const definitionInput = new TextInputBuilder()
      .setCustomId('definitionInput')
      .setLabel('Definition')
      .setPlaceholder('Enter the definition of the term. (Max 1000 characters. For longer definitions, use a link to further reading.)')
      .setMaxLength(1000)
      .setStyle(TextInputStyle.Paragraph);

    const usageInput = new TextInputBuilder()
      .setCustomId('usageInput')
      .setLabel('Usage')
      .setPlaceholder('Enter an example sentence showing how to use the term.')
      .setRequired(false)
      .setStyle(TextInputStyle.Short);

    const categoryInput = new TextInputBuilder()
      .setCustomId('categoryInput')
      .setLabel('Category')
      .setPlaceholder('Enter the category the term belongs to.')
      .setRequired(false)
      .setStyle(TextInputStyle.Short);

    const linksInput = new TextInputBuilder()
      .setCustomId('linksInput')
      .setLabel('Links')
      .setPlaceholder('Enter links to resources that explain the term. (Must be separated by commas.)')
      .setRequired(false)
      .setStyle(TextInputStyle.Short);

    const firstRow = new ActionRowBuilder<ModalActionRowComponentBuilder>().addComponents(termInput);
    const secondRow = new ActionRowBuilder<ModalActionRowComponentBuilder>().addComponents(definitionInput);
    const thirdRow = new ActionRowBuilder<ModalActionRowComponentBuilder>().addComponents(usageInput);
    const fourthRow = new ActionRowBuilder<ModalActionRowComponentBuilder>().addComponents(categoryInput);
    const fifthRow = new ActionRowBuilder<ModalActionRowComponentBuilder>().addComponents(linksInput);

    modal.addComponents(firstRow, secondRow, thirdRow, fourthRow, fifthRow);

    await interaction.showModal(modal);
  },
};