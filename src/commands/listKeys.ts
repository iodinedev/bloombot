import { SlashCommandBuilder, EmbedBuilder, ActionRowBuilder, ButtonBuilder, ButtonStyle } from "discord.js";
import { adminCommand } from "../helpers/commandPermissions";
import { database } from "../helpers/database";

export = {
	data: new SlashCommandBuilder()
		.setName('listkeys')
		.setDescription('Lists all the Playne keys.')
    .addIntegerOption(
      option => option.setName('page')
        .setDescription('The page you want to see.')
        .setRequired(false))
    .setDefaultMemberPermissions(adminCommand())
    .setDMPermission(false),
	async execute(interaction) {
    let page = 0;

    if (interaction.options.getInteger('page') && interaction.options.getInteger('page') > 0) {
      page = interaction.options.getInteger('page') - 1;
    }

		const keys = await database.steamKeys.findMany();
    const embeds: any[] = [];
    let embed = new EmbedBuilder()
      .setTitle('Playne Keys')
      .setDescription('Here\'s a list of all the keys:');

    if (keys.length === 0) {
      embed.setDescription('There are no keys yet!');
      return interaction.reply({ embeds: [embed], ephemeral: true });
    }

    if (page > Math.ceil(keys.length / 10)) return interaction.reply({ content: `That's not a valid page! Last page is \`${Math.ceil(keys.length / 10)}\`.`, ephemeral: true });

    for (let i = 0; i < keys.length; i++) {
      const fields = embed.toJSON().fields;
      if (fields && fields.length === 10) {
        embeds.push(embed);
        embed = new EmbedBuilder()
          .setTitle('Playne Keys')
          .setDescription('Here\'s a list of all the keys:');
      }

      embed.addFields({ name: `\`\`\`${keys[i].key}\`\`\``, value: keys[i].used ? 'Used' : 'Unused', inline: true });
      embed.setFooter({ text: `Page ${embeds.length + 1} of ${Math.ceil(keys.length / 10)}` });
    }

    embeds.push(embed);

    const row = new ActionRowBuilder()
      .addComponents(
        new ButtonBuilder()
          .setCustomId('previous')
          .setLabel('Previous')
          .setStyle(ButtonStyle.Primary)
          .setDisabled(true),
        new ButtonBuilder()
          .setCustomId('next')
          .setLabel('Next')
          .setStyle(ButtonStyle.Primary)
      );

    if (embeds.length > 1) {
      const msg = await interaction.reply({ embeds: [embeds[page]], components: [row], fetchReply: true, ephemeral: true });

      const filter = (i: any) => i.customId === 'previous' || i.customId === 'next';
      const collector = msg.createMessageComponentCollector({ filter, time: 60000 });

      collector.on('collect', async (i: any) => {
        if (i.customId === 'previous') {
          collector.resetTimer();
          
          page--;
          if (page === 0) {
            (<any>row.components[0]).setDisabled(true);
          }
          (<any>row.components[1]).setDisabled(false);
        } else if (i.customId === 'next') {
          collector.resetTimer();
          
          page++;
          if (page === embeds.length - 1) {
            (<any>row.components[1]).setDisabled(true);
          }
          (<any>row.components[0]).setDisabled(false);
        }
        await i.update({ embeds: [embeds[page]], components: [row], ephemeral: true });
      });

      collector.on('end', async () => {
        (<any>row.components[0]).setDisabled(true);
        (<any>row.components[1]).setDisabled(true);
      });
    } else {
      await interaction.reply({ embeds: [embeds[page]], ephemeral: true })
    }
	},
};