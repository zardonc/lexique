CREATE TABLE review_paused (
    vocabulary_id INTEGER PRIMARY KEY REFERENCES vocabulary(id)
);
PRAGMA user_version = 7;
