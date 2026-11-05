CREATE TABLE speech_assets (
    semantic_key TEXT PRIMARY KEY,
    spec TEXT NOT NULL,
    context TEXT NOT NULL CHECK(context IN ('word','sentence')),
    content_hash TEXT,
    generated_at INTEGER,
    expires_at INTEGER,
    pending_hash TEXT,
    pending_generated_at INTEGER,
    gc_pending INTEGER NOT NULL DEFAULT 0 CHECK(gc_pending IN (0,1)),
    retain INTEGER NOT NULL DEFAULT 0 CHECK(retain IN (0,1))
);
CREATE INDEX speech_expiry ON speech_assets(expires_at);
CREATE TABLE speech_owners (
    semantic_key TEXT NOT NULL REFERENCES speech_assets(semantic_key),
    vocabulary_id INTEGER NOT NULL REFERENCES vocabulary(id),
    PRIMARY KEY(semantic_key,vocabulary_id)
);
CREATE TABLE speech_tasks (
    id INTEGER PRIMARY KEY,
    semantic_key TEXT NOT NULL UNIQUE REFERENCES speech_assets(semantic_key),
    priority INTEGER NOT NULL CHECK(priority IN (0,1)),
    state TEXT NOT NULL CHECK(state IN ('queued','running','retry_wait','ready','failed','cancelled')),
    attempts INTEGER NOT NULL DEFAULT 0 CHECK(attempts BETWEEN 0 AND 4),
    wake_at INTEGER NOT NULL,
    deadline_at INTEGER,
    cancel_requested INTEGER NOT NULL DEFAULT 0 CHECK(cancel_requested IN (0,1)),
    refresh_requested INTEGER NOT NULL DEFAULT 0 CHECK(refresh_requested IN (0,1)),
    error TEXT
);
CREATE INDEX speech_pending ON speech_tasks(state,priority,wake_at);
CREATE TABLE legacy_speech (
    vocabulary_id INTEGER PRIMARY KEY REFERENCES vocabulary(id),
    content_hash TEXT NOT NULL
);
CREATE TABLE review_batches (
    operation TEXT PRIMARY KEY,
    request TEXT NOT NULL,
    voice TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    cancelled INTEGER NOT NULL DEFAULT 0 CHECK(cancelled IN (0,1))
);
CREATE TABLE review_batch_items (
    operation TEXT NOT NULL REFERENCES review_batches(operation),
    ordinal INTEGER NOT NULL,
    vocabulary_id INTEGER NOT NULL,
    semantic_key TEXT REFERENCES speech_assets(semantic_key),
    created_cards INTEGER NOT NULL,
    error TEXT,
    PRIMARY KEY(operation,vocabulary_id)
);
CREATE INDEX batch_audio_demand ON review_batch_items(semantic_key,operation);
PRAGMA user_version = 11;
