CREATE EXTENSION pg_trgm;
CREATE EXTENSION vector;

CREATE TABLE term (
  record_id   TEXT PRIMARY KEY,
  term_name   TEXT UNIQUE NOT NULL,
  meaning     TEXT NOT NULL,
  usage       TEXT,
  links       TEXT[] DEFAULT ARRAY[]::TEXT[],
  category    TEXT,
  aliases     TEXT[] DEFAULT ARRAY[]::TEXT[],
  guild_id    TEXT NOT NULL,
  embedding   vector(1536)
);

CREATE INDEX ON term (LOWER(term_name));
CREATE INDEX ON term USING GIN (term_name gin_trgm_ops);

CREATE TABLE meditation (
  record_id          TEXT PRIMARY KEY,
  user_id            TEXT NOT NULL,
  occurred_at        TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
  meditation_minutes INTEGER NOT NULL,
  guild_id           TEXT NOT NULL
);

CREATE TABLE star (
  record_id          TEXT PRIMARY KEY NOT NULL,
  starred_message_id TEXT UNIQUE NOT NULL,
  board_message_id   TEXT UNIQUE NOT NULL,
  starred_channel_id TEXT NOT NULL
);

CREATE TABLE steamkey (
  record_id TEXT PRIMARY KEY,
  steam_key TEXT UNIQUE NOT NULL,
  reserved  TEXT,
  used      BOOLEAN NOT NULL,
  guild_id  TEXT NOT NULL
);

CREATE TABLE quote (
  record_id   TEXT PRIMARY KEY,
  quote       TEXT NOT NULL,
  author      TEXT DEFAULT 'Anonymous',
  guild_id    TEXT NOT NULL,
  created_at  TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE course (
  record_id        TEXT PRIMARY KEY,
  course_name      TEXT UNIQUE NOT NULL,
  participant_role TEXT NOT NULL,
  graduate_role    TEXT NOT NULL,
  guild_id         TEXT,
  created_at       TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX ON course (LOWER(course_name));
CREATE INDEX ON course USING GIN (course_name gin_trgm_ops);

CREATE TABLE commmand (
  record_id     TEXT PRIMARY KEY,
  command_name  TEXT NOT NULL,
  user_id       TEXT NOT NULL,
  guild_id      TEXT NOT NULL,
  created_at    TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
