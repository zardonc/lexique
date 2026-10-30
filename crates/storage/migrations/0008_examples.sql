ALTER TABLE vocabulary ADD COLUMN example_fr TEXT NOT NULL DEFAULT '';
ALTER TABLE vocabulary ADD COLUMN example_zh TEXT NOT NULL DEFAULT '';
PRAGMA user_version = 8;
