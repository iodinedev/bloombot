CREATE TABLE Glossary (
  id SERIAL PRIMARY KEY,
  term VARCHAR(255) UNIQUE,
  term_definition TEXT,
  usage TEXT,
  links TEXT[],
  category VARCHAR(255),
  aliases TEXT[]
);

CREATE TABLE Meditations (
  id SERIAL PRIMARY KEY,
  session_user_id VARCHAR(255),
  occurred_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
  session_minutes INT,
  session_guild_id VARCHAR(255)
);

CREATE TABLE Stars (
  id SERIAL PRIMARY KEY,
  messageID VARCHAR(255) UNIQUE,
  embedID VARCHAR(255) UNIQUE,
  messageChannelID VARCHAR(255)
);

CREATE TABLE SteamKeys (
  steam_key VARCHAR(255) UNIQUE,
  reserved VARCHAR(255),
  used BOOLEAN
);

CREATE TABLE QuoteBook (
  id SERIAL PRIMARY KEY,
  quote TEXT,
  author VARCHAR(255) DEFAULT 'Anonymous',
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE Courses (
  id SERIAL PRIMARY KEY,
  course_name VARCHAR(255) UNIQUE,
  participant_role VARCHAR(255),
  graduate_role VARCHAR(255),
  guild VARCHAR(255),
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE CommandUsage (
  id SERIAL PRIMARY KEY,
  command_name VARCHAR(255),
  user_id VARCHAR(255),
  guild_id VARCHAR(255),
  created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
