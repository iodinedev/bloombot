/*
  Warnings:

  - The primary key for the `CommandUsage` table will be changed. If it partially fails, the table could be left without primary key constraint.
  - The primary key for the `Courses` table will be changed. If it partially fails, the table could be left without primary key constraint.
  - The primary key for the `Glossary` table will be changed. If it partially fails, the table could be left without primary key constraint.
  - The primary key for the `Meditations` table will be changed. If it partially fails, the table could be left without primary key constraint.
  - The primary key for the `QuoteBook` table will be changed. If it partially fails, the table could be left without primary key constraint.
  - The primary key for the `Stars` table will be changed. If it partially fails, the table could be left without primary key constraint.

*/
-- AlterTable
ALTER TABLE "CommandUsage" DROP CONSTRAINT "CommandUsage_pkey",
ALTER COLUMN "id" DROP DEFAULT,
ALTER COLUMN "id" SET DATA TYPE TEXT,
ADD CONSTRAINT "CommandUsage_pkey" PRIMARY KEY ("id");
DROP SEQUENCE "CommandUsage_id_seq";

-- AlterTable
ALTER TABLE "Courses" DROP CONSTRAINT "Courses_pkey",
ALTER COLUMN "id" DROP DEFAULT,
ALTER COLUMN "id" SET DATA TYPE TEXT,
ADD CONSTRAINT "Courses_pkey" PRIMARY KEY ("id");
DROP SEQUENCE "Courses_id_seq";

-- AlterTable
ALTER TABLE "Glossary" DROP CONSTRAINT "Glossary_pkey",
ALTER COLUMN "id" DROP DEFAULT,
ALTER COLUMN "id" SET DATA TYPE TEXT,
ADD CONSTRAINT "Glossary_pkey" PRIMARY KEY ("id");
DROP SEQUENCE "Glossary_id_seq";

-- AlterTable
ALTER TABLE "Meditations" DROP CONSTRAINT "Meditations_pkey",
ALTER COLUMN "id" DROP DEFAULT,
ALTER COLUMN "id" SET DATA TYPE TEXT,
ADD CONSTRAINT "Meditations_pkey" PRIMARY KEY ("id");
DROP SEQUENCE "Meditations_id_seq";

-- AlterTable
ALTER TABLE "QuoteBook" DROP CONSTRAINT "QuoteBook_pkey",
ALTER COLUMN "id" DROP DEFAULT,
ALTER COLUMN "id" SET DATA TYPE TEXT,
ADD CONSTRAINT "QuoteBook_pkey" PRIMARY KEY ("id");
DROP SEQUENCE "QuoteBook_id_seq";

-- AlterTable
ALTER TABLE "Stars" DROP CONSTRAINT "Stars_pkey",
ALTER COLUMN "id" DROP DEFAULT,
ALTER COLUMN "id" SET DATA TYPE TEXT,
ADD CONSTRAINT "Stars_pkey" PRIMARY KEY ("id");
DROP SEQUENCE "Stars_id_seq";
