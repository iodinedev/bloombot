CREATE TABLE IF NOT EXISTS erases (
  record_id          TEXT PRIMARY KEY,
  user_id            TEXT NOT NULL,
  guild_id           TEXT NOT NULL,
  message_link       TEXT,
  occurred_at        TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);