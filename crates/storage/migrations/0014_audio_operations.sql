ALTER TABLE speech_tasks ADD COLUMN execution_generation INTEGER NOT NULL DEFAULT 0 CHECK(execution_generation>=0);
ALTER TABLE speech_assets ADD COLUMN pending_execution INTEGER;
ALTER TABLE speech_versions ADD COLUMN execution_generation INTEGER;
UPDATE speech_assets SET pending_execution=(SELECT execution_generation FROM speech_tasks t WHERE t.semantic_key=speech_assets.semantic_key) WHERE pending_hash IS NOT NULL;
UPDATE speech_versions SET execution_generation=(SELECT execution_generation FROM speech_tasks t WHERE t.semantic_key=speech_versions.semantic_key);
CREATE TRIGGER immutable_audio_execution BEFORE UPDATE OF execution_generation ON speech_versions
BEGIN SELECT RAISE(ABORT,'Audio execution identity is immutable'); END;
CREATE TABLE word_audio_operations (
    operation TEXT PRIMARY KEY CHECK(length(operation) BETWEEN 1 AND 128),
    vocabulary_id INTEGER NOT NULL REFERENCES vocabulary(id),
    semantic_key TEXT NOT NULL REFERENCES speech_assets(semantic_key),
    generation INTEGER NOT NULL CHECK(generation>0),
    refresh INTEGER NOT NULL CHECK(refresh IN (0,1))
);
PRAGMA user_version = 14;
