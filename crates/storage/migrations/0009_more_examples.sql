ALTER TABLE vocabulary ADD COLUMN more_examples TEXT NOT NULL DEFAULT '[]';
PRAGMA user_version = 9;
