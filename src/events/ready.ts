import { database } from "../helpers/database";

export = async (client, message) => {

  // Prints the database name
  const current_database: any = await database.$queryRaw`select current_database()`;
  console.log(`Current database: ${current_database[0].current_database}`);

  console.log("Bot is ready; logged in as '" + client.user.tag + "'.");
}