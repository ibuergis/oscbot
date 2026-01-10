-- Add migration script here
PRAGMA foreign_keys = ON;

CREATE TABLE "user" (
  "id" INTEGER PRIMARY KEY,
  "osu_id" INTEGER NOT NULL,
  "discord_id" INTEGER NOT NULL,
  "is_blacklisted" INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE "skin" (
  "id" INTEGER PRIMARY KEY,
  "user" INTEGER NOT NULL,
  "identifier" TEXT NOT NULL,
  "url" TEXT NOT NULL,
  "default" TEXT DEFAULT NULL,
  
  FOREIGN KEY ("user") REFERENCES "user"("id") ON DELETE CASCADE,
  UNIQUE ("user", "default")
);

CREATE TABLE "score" (
    "identifier" TEXT PRIMARY KEY
);