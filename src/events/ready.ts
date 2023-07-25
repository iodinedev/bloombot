import { backup } from "../helpers/backup"
import { probeServer } from "../helpers/probeServer"
import type { Message, Client } from "discord.js"

// 'Ready' event fired from Discord.js client.on
export = async (client: Client, message: Message) => {
  try {
    // Backup the database
    setInterval(() => {
      backup(client)
    }, 1000 * 60 * 60 * 24)
  } catch (err: any) {
    console.error("[ERROR] " + err)
  }

  // Set the ready state to true
  probeServer.setReady(true)
  probeServer.verifyLive()

  console.log("[INFO] Bot is ready; logged in as '" + client.user?.tag + "'.")
}
