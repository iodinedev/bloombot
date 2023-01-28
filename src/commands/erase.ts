import { SlashCommandBuilder } from "discord.js";
import { modCommand } from "../helpers/commandPermissions";

export = {
	data: new SlashCommandBuilder()
		.setName('erase')
		.setDescription('Deletes a message.')
    .addStringOption(option =>
      option.setName('message')
        .setDescription('The message ID of the message you want to delete.')
        .setRequired(true))
    .addStringOption(option =>
      option.setName('reason')
        .setDescription('The reason for deleting the message.')
        .setRequired(false))
    .setDefaultMemberPermissions(modCommand())
    .setDMPermission(false),
	async execute(interaction) {
		const messageID: string = interaction.options.getString('message');
    const reason: string = interaction.options.getString('reason');

    const message = await interaction.channel.messages.fetch(messageID);
    var sent = false;

    if (!message) {
      return interaction.reply({ content: 'That message does not exist!', ephemeral: true });
    }

    if (reason) {
      try {
        await message.author.send(`Your message in ${interaction.channel} was deleted for the following reason: ${reason}`);
        sent = true;
      } catch (error) {
        console.error(error);
      }
    }

    try {
      await message.delete();
    } catch (error) {
      console.error(error);
      return interaction.reply({ content: 'Message could not be deleted.', ephemeral: true });
    }

    if (reason) {
      if (sent) {
        return interaction.reply({ content: 'Message deleted and user notified.', ephemeral: true });
      }

      return interaction.reply({ content: 'Message deleted, but user could not be notified.', ephemeral: true });
    }

    return interaction.reply({ content: 'Message deleted.', ephemeral: true });
	},
};