import { database } from "../helpers/database";
import { backup } from "../helpers/backup";

export = async (client, message) => {
  // Backup the database
  setInterval(() => {
    backup(client);
  }, 1000 * 60 * 60 * 24);
  
  // Prints the database name
  const current_database: any = await database.$queryRaw`select current_database()`;
  console.log(`Current database: ${current_database[0].current_database}`);

  console.log("Bot is ready; logged in as '" + client.user.tag + "'.");
}