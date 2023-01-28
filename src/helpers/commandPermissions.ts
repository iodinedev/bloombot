import Discord from 'discord.js';

export const adminCommand = () => {
  const permissions = Discord.PermissionFlagsBits.Administrator;

  return permissions;
}

export const modCommand = () => {
  const permissions = Discord.PermissionFlagsBits.ManageMessages;

  return permissions;
}