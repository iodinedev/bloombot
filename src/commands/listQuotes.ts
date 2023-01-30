import { database } from '../helpers/database';
import Discord from 'discord.js';
import { modCommand } from "../helpers/commandPermissions";

export = {
  data: new Discord.SlashCommandBuilder()
    .setName('listquotes')
    .setDescription('Shows a list of all quotes.')
    .addIntegerOption(option =>
      option.setName('id')
        .setDescription('The ID of the quote you want to see.')
        .setRequired(false))
    .setDefaultMemberPermissions(modCommand())
    .setDMPermission(false),
  async execute(interaction) {
    const id = interaction.options.getInteger('id');
    if (id) {
      const quote = await database.quoteBook.findUnique({
        where: {
          id: id
        }
      });

      if (!quote) {
        return interaction.reply({ content: 'That quote doesn\'t exist!', ephemeral: true });
      }

      const embed = new Discord.EmbedBuilder()
        .setTitle(`Quote ID: ${quote.id}`)
        .setDescription(quote.quote)
        .addFields([
          { name: 'Author', value: quote.author },
        ]);

      return interaction.reply({ embeds: [embed], ephemeral: true });
    } else {
      // Max 10 fields, uses buttons to paginate
      const terms = await database.quoteBook.findMany();
      const embeds: any[] = [];
      let embed = new Discord.EmbedBuilder()
        .setTitle('Quotes')
        .setDescription('Here\'s a list of all the quotes:');

      if (terms.length === 0) {
        embed.setDescription('There are no quotes yet!');
        return interaction.reply({ embeds: [embed], ephemeral: true });
      }

      for (let i = 0; i < terms.length; i++) {
        const fields = embed.toJSON().fields;
        if (fields && fields.length === 10) {
          embeds.push(embed);
          embed = new Discord.EmbedBuilder()
            .setTitle('Quotes')
            .setDescription('Here\'s a list of all the quotes:');
        }

        const cut = terms[i].quote.slice(0, 150);
        const value = terms[i].quote.length > 150 ? `${cut}...` : terms[i].quote;

        embed.addFields({ name: `ID: ${terms[i].id}`, value: value });
        embed.setFooter({ text: `Page ${embeds.length + 1} of ${Math.ceil(terms.length / 10)}` });
      }

      embeds.push(embed);

      const row = new Discord.ActionRowBuilder()
        .addComponents(
          new Discord.ButtonBuilder()
            .setCustomId('previous')
            .setLabel('Previous')
            .setStyle(Discord.ButtonStyle.Primary)
            .setDisabled(true),
          new Discord.ButtonBuilder()
            .setCustomId('next')
            .setLabel('Next')
            .setStyle(Discord.ButtonStyle.Primary)
        );

      let page = 0;

      if (embeds.length > 1) {
        const msg = await interaction.reply({ embeds: [embeds[page]], components: [row], fetchReply: true, ephemeral: true });

        const filter = (i: any) => i.customId === 'previous' || i.customId === 'next';
        const collector = msg.createMessageComponentCollector({ filter, time: 60000 });

        collector.on('collect', async (i: any) => {
          if (i.customId === 'previous') {
            page--;
            if (page === 0) {
              (<any>row.components[0]).setDisabled(true);
            }
            (<any>row.components[1]).setDisabled(false);
          } else if (i.customId === 'next') {
            page++;
            if (page === embeds.length - 1) {
              (<any>row.components[1]).setDisabled(true);
            }
            (<any>row.components[0]).setDisabled(false);
          }
          await i.update({ embeds: [embeds[page]], components: [row], ephemeral: true });
        });
      } else {
        await interaction.reply({ embeds: [embeds[page]], ephemeral: true })
      }
    }
  },
};