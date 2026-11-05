CREATE TABLE review_progress (
    card_id INTEGER PRIMARY KEY REFERENCES review_cards(id),
    revision INTEGER NOT NULL,
    state TEXT NOT NULL
);
CREATE TABLE review_ratings (
    operation TEXT PRIMARY KEY,
    card_id INTEGER NOT NULL REFERENCES review_cards(id),
    request TEXT NOT NULL,
    before_state TEXT NOT NULL,
    result TEXT NOT NULL,
    reviewed_at INTEGER NOT NULL,
    learning_day TEXT NOT NULL,
    quota TEXT NOT NULL CHECK (quota IN ('new', 'review'))
);
PRAGMA user_version = 4;
