import { SlashCommandBuilder, EmbedBuilder, ActionRowBuilder, ButtonBuilder, ButtonStyle } from "discord.js";
import { adminCommand } from "../helpers/commandPermissions";
import { database } from "../helpers/database";

export = {
	data: new SlashCommandBuilder()
		.setName('listkeys')
		.setDescription('Lists all the Playne keys.')
    .setDefaultMemberPermissions(adminCommand())
    .setDMPermission(false),
	async execute(interaction) {
		const keys = await database.steamKeys.findMany();
      const embeds: any[] = [];
      let embed = new EmbedBuilder()
        .setTitle('Playne Keys')
        .setDescription('Here\'s a list of all the keys:');

      for (let i = 0; i < keys.length; i++) {
        const fields = embed.toJSON().fields;
        if (fields && fields.length === 10) {
          embeds.push(embed);
          embed = new EmbedBuilder()
            .setTitle('Playne Keys')
            .setDescription('Here\'s a list of all the keys:');
        }

        embed.addFields({ name: `\`\`\`${keys[i].key}\`\`\``, value: keys[i].used ? 'Used' : 'Unused', inline: true });
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
	},
};