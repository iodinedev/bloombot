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
  if (user_time >= 50) {
    lvl_role = 'I_Star';

    if (user_time >= 100) lvl_role = 'II_Star';
    if (user_time >= 150) lvl_role = 'III_Star';
    if (user_time >= 250) lvl_role = 'I_S_Star';
    if (user_time >= 500) lvl_role = 'II_S_Star';
    if (user_time >= 1000) lvl_role = 'III_S_Star';
    if (user_time >= 2000) lvl_role = 'I_M_Star';
    if (user_time >= 5000) lvl_role = 'II_M_Star';
    if (user_time >= 10000) lvl_role = 'III_M_Star';
    if (user_time >= 20000) lvl_role = 'I_Star_S';
    if (user_time >= 50000) lvl_role = 'II_Star_S';
    if (user_time >= 100000) lvl_role = 'III_Star_S';
  }

  // Streak tests
  if (streak >= 7) {
    streak_role = 'egg';

    if (streak >= 14) streak_role = 'hatching_chick';
    if (streak >= 28) streak_role = 'baby_chick';
    if (streak >= 35) streak_role = 'chicken';
    if (streak >= 56) streak_role = 'dove';
    if (streak >= 70) streak_role = 'owl';
    if (streak >= 140) streak_role = 'eagle';
    if (streak >= 365) streak_role = 'dragon';
    if (streak >= 730) streak_role = 'alien';
  }

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

  console.log(remove_roles)

  // Add and remove roles
  if (add_roles.length > 0) await member.roles.add(add_roles);
  // if (remove_roles.length > 0) await member.roles.remove(remove_roles);
}

export const getStreak = async (client: Client, guild: Guild, user: User) => {
  // The streak is the number of days in a row that the user has meditated. It is calculated by finding the number of days between the first meditation and the last meditation, and then adding one.
  const user_id = user.id;
  const guild_id = guild.id;

  const streak_entries: any = await database.$queryRaw`
    WITH cte AS (
      SELECT "session_user", "session_guild", "occurred_at"::date AS "date", 
      date_part('day', NOW() - DATE_TRUNC('day', "occurred_at")) AS "days_ago"
      FROM "Meditations" 
      WHERE "session_user" = ${user_id} AND "session_guild" = ${guild_id} 
      AND "occurred_at"::date <= NOW()::date
      GROUP BY "days_ago", "session_user", "session_guild", "date"
    )
    SELECT * FROM cte;`

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
