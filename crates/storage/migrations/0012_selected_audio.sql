CREATE TABLE speech_versions (
    locator TEXT PRIMARY KEY CHECK(length(locator)=64 AND locator NOT GLOB '*[^0-9a-f]*'),
    semantic_key TEXT REFERENCES speech_assets(semantic_key),
    content_hash TEXT NOT NULL CHECK(length(content_hash)=64 AND content_hash NOT GLOB '*[^0-9a-f]*'),
    byte_length INTEGER NOT NULL CHECK(byte_length BETWEEN 4 AND 10000000),
    generated_at INTEGER,
    created_at INTEGER NOT NULL,
    vocabulary_id INTEGER NOT NULL REFERENCES vocabulary(id),
    expected_revision INTEGER NOT NULL,
    generation INTEGER NOT NULL,
    sealed INTEGER NOT NULL DEFAULT 0 CHECK(sealed IN (0,1))
);
CREATE TRIGGER immutable_speech_version BEFORE UPDATE OF locator,semantic_key,content_hash,byte_length,generated_at,created_at,vocabulary_id,expected_revision,generation ON speech_versions
BEGIN SELECT RAISE(ABORT,'Audio content versions are immutable'); END;
CREATE TRIGGER sealed_speech_version BEFORE UPDATE OF sealed ON speech_versions
WHEN OLD.sealed=1 AND NEW.sealed<>1
BEGIN SELECT RAISE(ABORT,'Audio content versions cannot be unsealed'); END;
CREATE TABLE word_audio_selections (
    vocabulary_id INTEGER PRIMARY KEY REFERENCES vocabulary(id),
    current_version TEXT REFERENCES speech_versions(locator),
    previous_version TEXT REFERENCES speech_versions(locator),
    previous_until INTEGER,
    revision INTEGER NOT NULL DEFAULT 0 CHECK(revision>=0),
    desired_generation INTEGER NOT NULL DEFAULT 0 CHECK(desired_generation>=0),
    desired_key TEXT REFERENCES speech_assets(semantic_key),
    CHECK((previous_version IS NULL)=(previous_until IS NULL)),
    CHECK(current_version IS NULL OR previous_version IS NULL OR current_version<>previous_version)
);
CREATE INDEX pending_speech_version_locator ON speech_versions(sealed,locator);
CREATE INDEX resumable_speech_versions ON speech_versions(vocabulary_id,expected_revision,generation) WHERE sealed=0;
CREATE TRIGGER select_sealed_audio_insert BEFORE INSERT ON word_audio_selections
WHEN NEW.current_version IS NOT NULL AND NOT EXISTS(SELECT 1 FROM speech_versions WHERE locator=NEW.current_version AND sealed=1 AND vocabulary_id=NEW.vocabulary_id)
BEGIN SELECT RAISE(ABORT,'Only verified word audio can be selected'); END;
CREATE TRIGGER select_sealed_audio_update BEFORE UPDATE OF current_version ON word_audio_selections
WHEN NEW.current_version IS NOT NULL AND NOT EXISTS(SELECT 1 FROM speech_versions WHERE locator=NEW.current_version AND sealed=1 AND vocabulary_id=NEW.vocabulary_id)
BEGIN SELECT RAISE(ABORT,'Only verified word audio can be selected'); END;
CREATE TABLE word_audio_imports (
    vocabulary_id INTEGER PRIMARY KEY REFERENCES vocabulary(id),
    state TEXT NOT NULL DEFAULT 'pending' CHECK(state IN ('pending','finished','failed')),
    error TEXT
);
INSERT INTO word_audio_imports(vocabulary_id) SELECT id FROM vocabulary;
CREATE INDEX pending_word_audio_imports ON word_audio_imports(state,vocabulary_id);
PRAGMA user_version = 12;
