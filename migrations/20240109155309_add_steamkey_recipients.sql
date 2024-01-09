CREATE TABLE IF NOT EXISTS steamkey_recipients (
  record_id          TEXT PRIMARY KEY,
  user_id            TEXT NOT NULL,
  guild_id           TEXT NOT NULL,
  challenge_prize    BOOLEAN,
  donator_perk       BOOLEAN,
  total_keys         SMALLINT NOT NULL
);