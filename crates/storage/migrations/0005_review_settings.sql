CREATE TABLE review_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    retention_percent INTEGER NOT NULL CHECK (retention_percent BETWEEN 70 AND 99),
    new_limit INTEGER NOT NULL CHECK (new_limit BETWEEN 0 AND 1000000),
    review_limit INTEGER NOT NULL CHECK (review_limit BETWEEN 0 AND 1000000)
);
INSERT INTO review_settings VALUES (1, 90, 20, 50);
CREATE TABLE review_queue_state (id INTEGER PRIMARY KEY CHECK (id = 1), learning_day TEXT NOT NULL, timezone TEXT NOT NULL);
CREATE TABLE review_queue_entries (card_id INTEGER PRIMARY KEY REFERENCES review_cards(id), position INTEGER NOT NULL UNIQUE);
ALTER TABLE review_ratings ADD COLUMN queue_before TEXT;
PRAGMA user_version = 5;
