import { database } from '../helpers/database';
import Discord from 'discord.js';
import { makeSearchable } from '../helpers/glossary';

export = {
  data: new Discord.SlashCommandBuilder()
    .setName('glossary')
    .setDescription('Shows a list of all glossary terms, or info about a specific term.')
    .addStringOption(option =>
      option.setName('term')
        .setDescription('The term you want info about.')
        .setAutocomplete(true)
        .setRequired(false)),
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

    await interaction.respond(suggestions);
  },
  async execute(interaction) {
    const term = interaction.options.getString('term');
    if (term) {
      const search = makeSearchable(term);
      const termData = await database.glossary.findFirst({
        where: {
          search: search
        }
      });

      if (!termData) return interaction.reply({ content: 'That\'s not a valid term!', ephemeral: true });
      const embed = new Discord.EmbedBuilder()
        .setTitle(`Term: ${termData.term}`)
        .setDescription(termData.definition)
        .addFields([
          { name: 'Usage', value: termData.usage },
          { name: 'Category', value: termData.category },
          { name: 'Links', value: termData.links.length > 0 ? termData.links.join("\n") : 'None' }
        ]);
      return interaction.reply({ embeds: [embed] });
    } else {
      // Max 10 fields, uses buttons to paginate
      const terms = await database.glossary.findMany();
      const embeds: any[] = [];
      let embed = new Discord.EmbedBuilder()
        .setTitle('Glossary')
        .setDescription('Here\'s a list of all my glossary terms:');

      for (let i = 0; i < terms.length; i++) {
        const fields = embed.toJSON().fields;
        if (fields && fields.length === 10) {
          embeds.push(embed);
          embed = new Discord.EmbedBuilder()
            .setTitle('Glossary')
            .setDescription('Here\'s a list of all my glossary terms:');
        }

        embed.addFields({ name: terms[i].term, value: terms[i].definition });
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
        const msg = await interaction.reply({ embeds: [embeds[page]], components: [row], fetchReply: true });

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
          await i.update({ embeds: [embeds[page]], components: [row] });
        });
      } else {
        await interaction.reply({ embeds: [embeds[page]] })
      }
    }
  },
};