ALTER TABLE speech_tasks ADD COLUMN successor_requested INTEGER NOT NULL DEFAULT 0 CHECK(successor_requested IN (0,1));
ALTER TABLE speech_tasks ADD COLUMN interactive_requested INTEGER NOT NULL DEFAULT 0 CHECK(interactive_requested IN (0,1));
PRAGMA user_version = 13;
