import { Client, Guild, User } from 'discord.js';
import { database } from './database';
import { config } from '../config';

export const updateRoles = async (client: Client, guild: Guild, user: User) => {
  const user_id = user.id;
  const guild_id = guild.id;

  const user_info = await database.meditations.aggregate({
    where: {
      session_user: user_id,
      session_guild: guild_id
    },
    _sum: {
      session_time: true
    }
  });

  const member = await guild.members.fetch(user_id);
  const user_time = user_info._sum.session_time || 0;
  const streak = await getStreak(client, guild, user);

  const member_roles = member.roles.cache.map(role => role.id);

  let add_roles: string[] = [];
  let remove_roles: string[] = [];

  var lvl_role = '';
  var streak_role = '';

  // Level tests
  if (user_time >= 100000) lvl_role = 'III_Star_S';
  else if (user_time >= 50000) lvl_role = 'II_Star_S';
  else if (user_time >= 20000) lvl_role = 'I_Star_S';
  else if (user_time >= 10000) lvl_role = 'III_M_Star';
  else if (user_time >= 5000) lvl_role = 'II_M_Star';
  else if (user_time >= 2000) lvl_role = 'I_M_Star';
  else if (user_time >= 1000) lvl_role = 'III_S_Star';
  else if (user_time >= 500) lvl_role = 'II_S_Star';
  else if (user_time >= 250) lvl_role = 'I_S_Star';
  else if (user_time >= 150) lvl_role = 'III_Star';
  else if (user_time >= 100) lvl_role = 'II_Star';
  else if (user_time >= 50) lvl_role = 'I_Star';

  // Streak tests
  if (streak >= 730) streak_role = 'alien';
  else if (streak >= 365) streak_role = 'dragon';
  else if (streak >= 140) streak_role = 'eagle';
  else if (streak >= 70) streak_role = 'owl';
  else if (streak >= 56) streak_role = 'dove';
  else if (streak >= 35) streak_role = 'chicken';
  else if (streak >= 28) streak_role = 'baby_chick';
  else if (streak >= 14) streak_role = 'hatching_chick';
  else if (streak >= 7) streak_role = 'egg';

  // Get role IDs
  const lvl_role_id = config.time_roles[lvl_role];
  const streak_role_id = config.streak_roles[streak_role];

  // Add roles if the user doesn't have them and remove the old ones
  if (lvl_role_id && !member_roles.includes(lvl_role_id)) add_roles.push(lvl_role_id);
  if (streak_role_id && !member_roles.includes(streak_role_id)) add_roles.push(streak_role_id);

  // Remove roles if the user has them and they are not the new ones, but only if they exist in the config
  const relevant_role_ids = Object.values(config.time_roles).concat(Object.values(config.streak_roles));

  for (const role of member_roles) {
    if (role !== lvl_role_id && role !== streak_role_id && relevant_role_ids.includes(role)) {
      remove_roles.push(role);
    }
  }

  // Removes roles that exist in both add_roles and remove_roles
  const duplicates: string[] = [];
  add_roles = add_roles.filter(role => {
    if (remove_roles.includes(role)) {
      duplicates.push(role);
    } else {
      return role;
    }
  });
  remove_roles = remove_roles.filter(role => !duplicates.includes(role));

  // Add and remove roles
  try {
  if (add_roles.length > 0) await member.roles.add(add_roles);
  if (remove_roles.length > 0) await member.roles.remove(remove_roles);
  } catch (error: any) {
    if (error.code === 50013) {
      console.log('Missing permissions to manage roles');
    } else {
      throw error;
    }
  }

  // new_streak returns the streak role if it was added, otherwise it returns an empty array
  return {
    new_streak: add_roles.filter(role => Object.values(config.streak_roles).includes(role)),
    new_level: add_roles.filter(role => Object.values(config.time_roles).includes(role))
  }
}

export const getStreak = async (client: Client, guild: Guild, user: User) => {
  // The streak is the number of days in a row that the user has meditated. It is calculated by finding the number of days between the first meditation and the last meditation, and then adding one.
  const user_id = user.id;
  const guild_id = guild.id;

  const streak_entries: any = await database.$queryRaw`
    WITH cte AS (
      SELECT date_part('day', NOW() - DATE_TRUNC('day', "occurred_at")) AS "days_ago"
      FROM "Meditations" 
      WHERE "session_user" = ${user_id} AND "session_guild" = ${guild_id} 
      AND "occurred_at"::date <= NOW()::date
    )
    SELECT * FROM cte
    GROUP BY "days_ago"
    ORDER BY "days_ago" ASC;
    `;

  var streak = 0;

  if (streak_entries.length > 0 && (streak_entries[0].days_ago === 0 || streak_entries[0].days_ago === 1)) {
    var curr = streak_entries[0].days_ago;

    for await (const entry of streak_entries) {
      if (entry.days_ago === curr) {
        streak++;
        curr++;
      } else {
        break;
      }
    }
  }

  return streak;
}
