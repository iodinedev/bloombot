-- CreateTable
CREATE TABLE "Glossary" (
    "id" SERIAL NOT NULL,
    "term" TEXT NOT NULL,
    "search" TEXT NOT NULL,
    "definition" TEXT NOT NULL,
    "usage" TEXT NOT NULL,
    "links" TEXT[],
    "category" TEXT NOT NULL,
    "aliases" TEXT[],

    CONSTRAINT "Glossary_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "Meditations" (
    "id" SERIAL NOT NULL,
    "session_user" TEXT NOT NULL,
    "occurred_at" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "session_time" INTEGER NOT NULL,
    "session_guild" TEXT NOT NULL,

    CONSTRAINT "Meditations_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "Stars" (
    "id" SERIAL NOT NULL,
    "messageID" TEXT NOT NULL,
    "embedID" TEXT NOT NULL,
    "messageChannelID" TEXT NOT NULL,

    CONSTRAINT "Stars_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "SteamKeys" (
    "key" TEXT NOT NULL,
    "reserved" TEXT,
    "used" BOOLEAN NOT NULL
);

-- CreateTable
CREATE TABLE "QuoteBook" (
    "id" SERIAL NOT NULL,
    "quote" TEXT NOT NULL,
    "author" TEXT NOT NULL DEFAULT 'Anonymous',
    "date" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "QuoteBook_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "Courses" (
    "id" SERIAL NOT NULL,
    "name" TEXT NOT NULL,
    "search" TEXT NOT NULL,
    "participant_role" TEXT NOT NULL,
    "graduate_role" TEXT NOT NULL,
    "guild" TEXT NOT NULL,
    "date" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "Courses_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "CommandUsage" (
    "id" SERIAL NOT NULL,
    "command" TEXT NOT NULL,
    "user" TEXT NOT NULL,
    "guild" TEXT NOT NULL,
    "date" TIMESTAMP(3) NOT NULL DEFAULT CURRENT_TIMESTAMP,

    CONSTRAINT "CommandUsage_pkey" PRIMARY KEY ("id")
);

-- CreateIndex
CREATE UNIQUE INDEX "Glossary_term_key" ON "Glossary"("term");

-- CreateIndex
CREATE UNIQUE INDEX "Glossary_search_key" ON "Glossary"("search");

-- CreateIndex
CREATE UNIQUE INDEX "Stars_messageID_key" ON "Stars"("messageID");

-- CreateIndex
CREATE UNIQUE INDEX "Stars_embedID_key" ON "Stars"("embedID");

-- CreateIndex
CREATE UNIQUE INDEX "SteamKeys_key_key" ON "SteamKeys"("key");

-- CreateIndex
CREATE UNIQUE INDEX "Courses_name_key" ON "Courses"("name");

-- CreateIndex
CREATE UNIQUE INDEX "Courses_search_key" ON "Courses"("search");
