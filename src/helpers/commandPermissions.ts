import { PermissionFlagsBits } from 'discord.js'

export const adminCommand = () => {
  const permissions = PermissionFlagsBits.Administrator

  return permissions
}

export const modCommand = () => {
  const permissions = PermissionFlagsBits.ManageRoles

  return permissions
}
