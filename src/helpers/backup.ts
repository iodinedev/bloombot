import { database } from "./database";
import { config } from "../config";
import { Prisma } from "@prisma/client";
import { AttachmentBuilder } from "discord.js";

// Takes a snapshot of the current state of the database and saves it to the backup channel
// This is used to restore the database in case of a crash
export async function backup(client) {
  // Get the backup channel
  const backup_channel = client.channels.cache.get(config.channels.backup);

  // Gets all tables
  const tables: any = await database.$queryRaw`SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'`;

  tables.forEach(async (table) => {
    // Gets all rows from the table, changing keys to progessive alphabet letters to avoid Discord's 2000 character limit
    // For example, if the table has 3 columns, the first row will be { a: 'value1', b: 'value2', c: 'value3' }
    console.log(table.table_name)
    const rows: any = await database.$queryRaw(Prisma.raw(`SELECT * FROM "${table.table_name}"`));

    if (rows.length === 0) return;

    const keys = Object.keys(rows[0]);
    
    // Convert to array of arrays, converting Date objects to numbers
    const minimized = rows.map((row) => {
      const newRow: any[] = [];
      keys.forEach((key) => {
        if (row[key] instanceof Date) {
          newRow.push(row[key].getTime());
          return;
        }

        newRow.push(row[key]);
      });

      return newRow;
    });

    // const compressed = brotli.compress(JSON.stringify(minimized), { mode: 1, quality: 11, lgwin: 22, lgblock: 0, extention: 'br' });
    const compressed = JSON.stringify(minimized)
    
    try {
      const attachment = new AttachmentBuilder(Buffer.from(compressed), { name: table.table_name + '.json' });

      await backup_channel.send({ content: `${keys}`, files: [attachment]})
    } catch (e) {
      const logChannel = client.channels.cache.get(config.channels.logs);
      await logChannel.send({ content: `Error while backing up ${table.table_name}: ${e}` });
    }
  });
}