ALTER TABLE speech_tasks ADD COLUMN demand_class INTEGER NOT NULL DEFAULT 3 CHECK(demand_class BETWEEN 1 AND 3);
ALTER TABLE speech_tasks ADD COLUMN requested_at INTEGER NOT NULL DEFAULT 0 CHECK(requested_at>=0);
CREATE INDEX speech_demand_order ON speech_tasks(state,priority,demand_class,requested_at,id);
CREATE TABLE word_audio_repairs (
    vocabulary_id INTEGER NOT NULL REFERENCES vocabulary(id) ON DELETE CASCADE,
    failure_identity TEXT NOT NULL,
    semantic_key TEXT REFERENCES speech_assets(semantic_key),
    generation INTEGER,
    error TEXT,
    PRIMARY KEY(vocabulary_id,failure_identity)
);
PRAGMA user_version = 17;
