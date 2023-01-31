import Discord, { SlashCommandBuilder } from "discord.js";
import { database } from "../helpers/database";
import { config } from "../config";
import { channelGuard } from "../helpers/guards";

export = {
	data: new SlashCommandBuilder()
		.setName('recent')
		.setDescription('Gets your recent meditation entries.')
    .addIntegerOption(option =>
      option.setName('page')
        .setDescription('The page you want to see.')
        .setRequired(false))
    .setDMPermission(false),
	async execute(interaction) {
    if (!(await channelGuard)(interaction, [config.channels.meditation, config.channels.commands], interaction.channelId)) return;

    let page = 0;

    if (interaction.options.getInteger('page') !== null) {
      page = interaction.options.getInteger('page') - 1;
    }

    if (page < 0) return interaction.reply({ content: 'That\'s not a valid page!', ephemeral: true });

		const sessions = await database.meditations.findMany({
      where: {
        session_user: interaction.user.id,
        session_guild: interaction.guild.id
      },
      orderBy: [
        {
          id: 'desc'
        }
      ],
    });

    
    const embeds: any[] = [];
    let embed = new Discord.EmbedBuilder()
      .setTitle('Entries')
      .setDescription('Here\'s a list of your meditation sessions:');
      
    if (sessions.length === 0) {
      embed.setDescription('There are no entries yet!');
      return interaction.reply({ embeds: [embed], ephemeral: true });
    }

    if (page > Math.ceil(sessions.length / 10)) return interaction.reply({ content: `That's not a valid page! Last page is \`${Math.ceil(sessions.length / 10)}\`.`, ephemeral: true });

    const today = new Date();

    for (let i = 0; i < sessions.length; i++) {
      const fields = embed.toJSON().fields;
      if (fields && fields.length === 10) {
        embeds.push(embed);
        embed = new Discord.EmbedBuilder()
          .setTitle('Entries')
          .setDescription('Here\'s a list of your meditation sessions:');
      }

      // Show time if the date is today, otherwise show the date
      const date = new Date(sessions[i].occurred_at);
      var dateTime = `\`${sessions[i].id}\` - ${date.toLocaleDateString('en-US', { weekday: 'long', year: 'numeric', month: 'long', day: 'numeric' })}`;

      if (date.getDate() === today.getDate() && date.getMonth() === today.getMonth() && date.getFullYear() === today.getFullYear()) {
        dateTime = `\`${sessions[i].id}\` - ${date.toLocaleTimeString('en-US', { hour: 'numeric', minute: 'numeric', hour12: true })}`;
      }

      embed.addFields({ name: dateTime, value: `\`\`\`${sessions[i].session_time}\`\`\`` });
      embed.setFooter({ text: `Page ${embeds.length + 1} of ${Math.ceil(sessions.length / 10)}` });
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