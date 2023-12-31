-- Add migration script here
CREATE TABLE IF NOT EXISTS tracking_profile (
  record_id          TEXT PRIMARY KEY,
  user_id            TEXT NOT NULL,
  guild_id           TEXT NOT NULL,
  utc_offset         SMALLINT DEFAULT 0 NOT NULL,
  anonymous_tracking BOOLEAN DEFAULT FALSE NOT NULL,
  streaks_active     BOOLEAN DEFAULT TRUE NOT NULL,
  streaks_private    BOOLEAN DEFAULT FALSE NOT NULL,
  stats_private      BOOLEAN DEFAULT FALSE NOT NULL
);