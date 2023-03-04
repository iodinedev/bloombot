export const channelGuard = async (interaction, channels: string[], current: string) => {
  if (process.env.PRODUCTION === 'false') {
    console.log('Channel guard bypassed (development mode).')
    return true
  }

  if (!channels.includes(current)) {
    await interaction.reply({
      content: `You can only use this command in <#${channels.join('>, <#')}>`,
      ephemeral: true
    })

    return false
  }

  return true
}
