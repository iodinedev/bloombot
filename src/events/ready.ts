import { database } from '../helpers/database'
import { backup } from '../helpers/backup'
import { rollbar } from '../helpers/rollbar'

export = async (client, message) => {
  try {
    // Backup the database
    setInterval(() => {
      backup(client)
    }, 1000 * 60 * 60 * 24)
    // Prints the database name
    const current_database: any = await database.$queryRaw`select current_database()`
    console.log(`Current database: ${current_database[0].current_database}`)

    console.log("Bot is ready; logged in as '" + client.user.tag + "'.")
  } catch (err: any) {
    rollbar.error(err)
  }
}
