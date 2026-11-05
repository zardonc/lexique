CREATE TABLE review_cards (
    id INTEGER PRIMARY KEY,
    vocabulary_id INTEGER NOT NULL REFERENCES vocabulary(id),
    kind TEXT NOT NULL CHECK (kind IN ('recognition', 'listening')),
    UNIQUE (vocabulary_id, kind)
);
PRAGMA user_version = 3;
