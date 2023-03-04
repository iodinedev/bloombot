import { SlashCommandBuilder } from 'discord.js'

export = {
  data: new SlashCommandBuilder()
    .setName('ping')
    .setDescription('Replies with the bot\'s latency.')
    .setDMPermission(false),
  async execute (interaction) {
    await interaction.reply('Getting ping...').then(async () => {
      await interaction.editReply(`Pong! Latency is ${Date.now() - interaction.createdTimestamp}ms. API Latency is ${Math.round(interaction.client.ws.ping)}ms.`)
    })
  }
}
