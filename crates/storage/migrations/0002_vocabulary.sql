CREATE TABLE vocabulary (
    id INTEGER PRIMARY KEY,
    headword TEXT NOT NULL UNIQUE COLLATE BINARY,
    snapshot TEXT NOT NULL,
    notes TEXT NOT NULL,
    revision INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
CREATE TABLE save_operations (
    operation TEXT PRIMARY KEY,
    request TEXT NOT NULL,
    outcome TEXT NOT NULL,
    record_id INTEGER NOT NULL REFERENCES vocabulary(id)
);
PRAGMA user_version = 2;
