import { SlashCommandBuilder, ActionRowBuilder, ButtonBuilder, ButtonStyle } from "discord.js";
import { database } from "../helpers/database";
import { updateRoles } from "../helpers/streaks";
import { config } from "../config";
import { getStreak } from "../helpers/streaks";
import { channelGuard } from "../helpers/guards";
import { alphanumeric } from "../helpers/strings";

export = {
	data: new SlashCommandBuilder()
		.setName('add')
		.setDescription('Adds minutes to your meditation time.')
    .addIntegerOption(option =>
      option.setName('minutes')
        .setDescription('The number of minutes you want to add.')
        .setMinValue(1)
        .setRequired(true))
    .setDMPermission(false),
	async execute(interaction) {
    if (!(await channelGuard)(interaction, [config.channels.meditation, config.channels.commands], interaction.channelId)) return;

		const minutes: number = interaction.options.getInteger('minutes');

    // Check with user if they want to add the minutes if they are over 300
    if (minutes > 300) {
      const row = new ActionRowBuilder()
      .addComponents(
        new ButtonBuilder()
          .setCustomId('yes')
          .setLabel('Yes')
          .setStyle(ButtonStyle.Danger),
        new ButtonBuilder()
          .setCustomId('no')
          .setLabel('No')
          .setStyle(ButtonStyle.Primary)
      );

      interaction.reply({ content: `Are you sure you want to add ${minutes} minutes?`, components: [row] });

      const filter = i => i.user.id === interaction.user.id;
      const collector = interaction.channel.createMessageComponentCollector({ filter, time: 15000 });

      collector.on('collect', async i => {
        if (i.customId === 'yes') {
          collector.resetTimer();

          await addMinutes(interaction, minutes, true);
        } else if (i.customId === 'no') {
          collector.resetTimer();

          interaction.editReply({ content: 'Time not added.', components: [] });
        }
      })

      collector.on('end', collected => {
        if (collected.size === 0) {
          interaction.editReply({ content: 'You did not respond in time. Time not added.', components: [] });
        }
      })
    } else {
      await addMinutes(interaction, minutes, false);
    }    
	},
};

async function addMinutes(interaction, minutes, replied) {
  const user = interaction.user.id;
  const guild = interaction.guild.id;

  await database.meditations.create({
    data: {
      session_user: user,
      session_time: minutes,
      session_guild: guild
    }
  })

  
  const total = await database.meditations.aggregate({
    where: {
      session_user: user,
      session_guild: guild
    },
    _sum: {
      session_time: true
    }
  });

  const guild_total = await database.meditations.aggregate({
    where: {
      session_guild: guild
    },
    _sum: {
      session_time: true
    },
    _count: {
      session_time: true
    }
  });

  const motivation_messages = (await database.quoteBook.findMany()).map((quote) => alphanumeric(quote.quote));
  
  const motivation = motivation_messages.length > 0 ? `\n*${motivation_messages[Math.floor(Math.random() * motivation_messages.length)]}*` : "";
  const update = await updateRoles(interaction.client, interaction.guild, interaction.user);

  if (!replied) {
    await interaction.reply({ content: `Added **${minutes} minutes** to your meditation time! Your total meditation time is ${(total._sum.session_time ?? 0).toLocaleString()} minutes :tada:${motivation}` });
  } else {
    await interaction.editReply({ content: `Added **${minutes} minutes** to your meditation time! Your total meditation time is ${(total._sum.session_time ?? 0).toLocaleString()} minutes :tada:${motivation}`, components: [] });
  }

  if (guild_total._count.session_time % 10 === 0 && guild_total._count.session_time > 0) {
    var time_in_hours = Math.round((guild_total._sum.session_time ?? 0) / 60);

    await interaction.channel
      .send(
        `Awesome sauce! This server has collectively generated ${time_in_hours.toLocaleString()} hours of realmbreaking meditation!`
      );
  }

  if (update.new_streak.length > 0) {
    return interaction.followUp({
      content: `:tada: Congrats to <@${interaction.user.id}>, your hard work is paying off! Your current streak is ${await getStreak(interaction.client, interaction.guild, interaction.user)}, giving you the <@&${update.new_streak}> role!`,
      allowedMentions: {
        roles: [],
        users: [interaction.user.id]
      }
    });
  } else if (update.new_level.length > 0) {
    return interaction.followUp({
      content: `:tada: Congrats to <@${interaction.user.id}>, your hard work is paying off! Your total meditation minutes have given you the <@&${update.new_level}> role!`,
      allowedMentions: {
        roles: [],
        users: [interaction.user.id]
      },
    });
  }
}