-- Rebuild with foreign_keys disabled before the migration transaction (SQLite's 12-step procedure).
DROP TRIGGER select_sealed_audio_insert;
DROP TRIGGER select_sealed_audio_update;
CREATE TABLE new_speech_versions (
    locator TEXT PRIMARY KEY CHECK(length(locator)=64 AND locator NOT GLOB '*[^0-9a-f]*'),
    semantic_key TEXT REFERENCES speech_assets(semantic_key),
    content_hash TEXT NOT NULL CHECK(length(content_hash)=64 AND content_hash NOT GLOB '*[^0-9a-f]*'),
    byte_length INTEGER CHECK(byte_length BETWEEN 4 AND 10000000),
    generated_at INTEGER,
    created_at INTEGER NOT NULL,
    vocabulary_id INTEGER,
    expected_revision INTEGER NOT NULL DEFAULT 0,
    generation INTEGER NOT NULL DEFAULT 0,
    sealed INTEGER NOT NULL DEFAULT 0 CHECK(sealed IN (0,1)),
    execution_generation INTEGER,
    physical_locator TEXT NOT NULL CHECK(length(physical_locator)=64 AND physical_locator NOT GLOB '*[^0-9a-f]*'),
    context TEXT NOT NULL CHECK(context IN ('word','sentence')),
    verification TEXT NOT NULL DEFAULT 'unverified' CHECK(verification IN ('unverified','verified','suspect','missing','invalid')),
    gc_state TEXT NOT NULL DEFAULT 'live' CHECK(gc_state IN ('live','marked','deleted')),
    gc_generation INTEGER NOT NULL DEFAULT 0 CHECK(gc_generation>=0),
    expires_at INTEGER,
    orphan_until INTEGER,
    last_used INTEGER NOT NULL,
    gc_error TEXT,
    import_locator TEXT,
    UNIQUE(physical_locator,content_hash)
);
INSERT INTO new_speech_versions
SELECT locator,semantic_key,content_hash,byte_length,generated_at,created_at,vocabulary_id,expected_revision,generation,sealed,execution_generation,
       locator,'word',CASE WHEN sealed=1 THEN 'verified' ELSE 'unverified' END,'live',0,
       CASE WHEN generated_at<=9223372036854171007 THEN generated_at+604800 END,
       CASE WHEN generated_at IS NULL AND created_at<=9223372036854171007 THEN created_at+604800 END,
       created_at,NULL,NULL
FROM speech_versions;
DROP TABLE speech_versions;
ALTER TABLE new_speech_versions RENAME TO speech_versions;
CREATE TRIGGER immutable_speech_version BEFORE UPDATE OF locator,semantic_key,content_hash,generated_at,created_at,vocabulary_id,expected_revision,generation,execution_generation,physical_locator,context,expires_at ON speech_versions
BEGIN SELECT RAISE(ABORT,'Audio content versions are immutable'); END;
CREATE TRIGGER immutable_audio_length BEFORE UPDATE OF byte_length ON speech_versions
WHEN OLD.byte_length IS NOT NULL AND NEW.byte_length IS NOT OLD.byte_length
BEGIN SELECT RAISE(ABORT,'Audio content length is immutable'); END;
CREATE TRIGGER sealed_speech_version BEFORE UPDATE OF sealed ON speech_versions
WHEN OLD.sealed=1 AND NEW.sealed<>1
BEGIN SELECT RAISE(ABORT,'Audio content versions cannot be unsealed'); END;
CREATE TRIGGER terminal_audio_gc BEFORE UPDATE OF gc_state ON speech_versions
WHEN OLD.gc_state<>'live' AND NEW.gc_state<>OLD.gc_state AND NOT(OLD.gc_state='marked' AND NEW.gc_state='deleted')
BEGIN SELECT RAISE(ABORT,'Retired audio versions cannot be revived'); END;
CREATE INDEX pending_speech_version_locator ON speech_versions(sealed,locator);
CREATE INDEX audio_version_maintenance ON speech_versions(gc_state,expires_at,last_used,locator);
CREATE TABLE word_audio_publications (
    locator TEXT PRIMARY KEY REFERENCES speech_versions(locator),
    vocabulary_id INTEGER NOT NULL REFERENCES vocabulary(id),
    expected_revision INTEGER NOT NULL,
    generation INTEGER NOT NULL,
    execution_generation INTEGER,
    error TEXT
);
INSERT INTO word_audio_publications(locator,vocabulary_id,expected_revision,generation,execution_generation)
SELECT locator,vocabulary_id,expected_revision,generation,execution_generation FROM speech_versions WHERE sealed=0;
CREATE INDEX resumable_speech_versions ON word_audio_publications(vocabulary_id,expected_revision,generation);
CREATE INDEX current_word_audio_version ON word_audio_selections(current_version);
CREATE INDEX previous_word_audio_version ON word_audio_selections(previous_version,previous_until);
ALTER TABLE word_audio_selections ADD COLUMN current_generation INTEGER NOT NULL DEFAULT 0 CHECK(current_generation>=0);
UPDATE word_audio_selections SET current_generation=COALESCE((SELECT generation FROM speech_versions WHERE locator=current_version),0);
CREATE TRIGGER select_sealed_audio_insert BEFORE INSERT ON word_audio_selections
WHEN NEW.current_version IS NOT NULL AND NOT EXISTS(SELECT 1 FROM speech_versions WHERE locator=NEW.current_version AND sealed=1 AND context='word' AND gc_state='live' AND verification='verified')
BEGIN SELECT RAISE(ABORT,'Only verified live word audio can be selected'); END;
CREATE TRIGGER select_sealed_audio_update BEFORE UPDATE OF current_version ON word_audio_selections
WHEN NEW.current_version IS NOT NULL AND NOT EXISTS(SELECT 1 FROM speech_versions WHERE locator=NEW.current_version AND sealed=1 AND context='word' AND gc_state='live' AND verification='verified')
BEGIN SELECT RAISE(ABORT,'Only verified live word audio can be selected'); END;
ALTER TABLE speech_assets ADD COLUMN current_version TEXT REFERENCES speech_versions(locator);
ALTER TABLE speech_assets ADD COLUMN pending_version TEXT REFERENCES speech_versions(locator);
CREATE INDEX current_source_audio_version ON speech_assets(current_version);
CREATE INDEX pending_source_audio_version ON speech_assets(pending_version);
-- Existing key/hash paths remain readable; new publications always use a fresh physical locator.
INSERT INTO speech_versions(locator,semantic_key,content_hash,generated_at,created_at,physical_locator,context,gc_state,gc_generation,expires_at,last_used)
SELECT lower(hex(randomblob(32))),a.semantic_key,a.content_hash,a.generated_at,COALESCE(a.generated_at,0),a.semantic_key,a.context,
       CASE WHEN a.gc_pending=1 AND NOT(a.context='word' AND EXISTS(SELECT 1 FROM speech_owners o WHERE o.semantic_key=a.semantic_key)) THEN 'marked' ELSE 'live' END,
       CASE WHEN a.gc_pending=1 THEN 1 ELSE 0 END,a.expires_at,COALESCE(a.generated_at,0)
FROM speech_assets a WHERE a.content_hash IS NOT NULL;
UPDATE speech_assets SET current_version=(SELECT locator FROM speech_versions v WHERE v.semantic_key=speech_assets.semantic_key AND v.physical_locator=speech_assets.semantic_key AND v.content_hash=speech_assets.content_hash),gc_pending=CASE WHEN context='word' AND EXISTS(SELECT 1 FROM speech_owners o WHERE o.semantic_key=speech_assets.semantic_key) THEN 0 ELSE gc_pending END;
CREATE VIEW audio_version_protection AS
SELECT v.*,
       EXISTS(SELECT 1 FROM word_audio_selections s WHERE s.current_version=v.locator) AS owned,
       MAX(COALESCE(v.orphan_until,0),COALESCE((SELECT MAX(s.previous_until) FROM word_audio_selections s WHERE s.previous_version=v.locator),0)) AS protected_until,
       EXISTS(SELECT 1 FROM word_audio_publications p WHERE p.locator=v.locator) OR EXISTS(SELECT 1 FROM speech_assets a WHERE a.pending_version=v.locator) AS publishing,
       EXISTS(SELECT 1 FROM speech_assets a JOIN speech_owners o USING(semantic_key) LEFT JOIN word_audio_selections s USING(vocabulary_id) LEFT JOIN word_audio_imports i USING(vocabulary_id) WHERE a.current_version=v.locator AND a.context='word' AND s.current_version IS NULL AND (a.retain=1 OR i.state IN ('pending','failed'))) AS migrating
FROM speech_versions v;
WITH nonces AS MATERIALIZED (SELECT semantic_key,lower(hex(randomblob(32))) AS nonce FROM speech_assets WHERE pending_hash IS NOT NULL)
INSERT INTO speech_versions(locator,semantic_key,content_hash,generated_at,created_at,physical_locator,context,execution_generation,expires_at,last_used,import_locator)
SELECT nonce,a.semantic_key,a.pending_hash,a.pending_generated_at,COALESCE(a.pending_generated_at,0),nonce,a.context,a.pending_execution,
       CASE WHEN a.pending_generated_at<=9223372036854171007 THEN a.pending_generated_at+604800 END,COALESCE(a.pending_generated_at,0),a.semantic_key
FROM speech_assets a JOIN nonces n USING(semantic_key)
WHERE a.pending_hash IS NOT NULL;
UPDATE speech_assets SET pending_version=(SELECT locator FROM speech_versions v WHERE v.semantic_key=speech_assets.semantic_key AND v.import_locator=speech_assets.semantic_key AND v.content_hash=speech_assets.pending_hash AND v.generated_at IS speech_assets.pending_generated_at) WHERE pending_hash IS NOT NULL;
PRAGMA user_version = 15;
