-- Keep the existing record identity namespace and all learning references.
CREATE TABLE vocabulary_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    headword TEXT NOT NULL,
    snapshot TEXT,
    notes TEXT NOT NULL,
    revision INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    example_fr TEXT NOT NULL DEFAULT '',
    example_zh TEXT NOT NULL DEFAULT '',
    more_examples TEXT NOT NULL DEFAULT '[]',
    example_sense INTEGER,
    example_source TEXT,
    record_kind TEXT NOT NULL DEFAULT 'vocabulary' CHECK (record_kind IN ('vocabulary', 'grammar')),
    CHECK ((record_kind = 'vocabulary' AND snapshot IS NOT NULL) OR (record_kind = 'grammar' AND snapshot IS NULL))
);
INSERT INTO vocabulary_new (id, headword, snapshot, notes, revision, created_at, updated_at,
    example_fr, example_zh, more_examples, example_sense, example_source)
SELECT id, headword, snapshot, notes, revision, created_at, updated_at,
    example_fr, example_zh, more_examples, example_sense, example_source FROM vocabulary;
DROP TABLE vocabulary;
ALTER TABLE vocabulary_new RENAME TO vocabulary;
CREATE UNIQUE INDEX vocabulary_headword ON vocabulary(headword COLLATE BINARY) WHERE record_kind = 'vocabulary';

CREATE TABLE review_cards_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    vocabulary_id INTEGER NOT NULL REFERENCES vocabulary(id),
    kind TEXT NOT NULL CHECK (kind IN ('recognition', 'listening', 'grammar')),
    UNIQUE (vocabulary_id, kind)
);
INSERT INTO review_cards_new SELECT id, vocabulary_id, kind FROM review_cards;
DROP TABLE review_cards;
ALTER TABLE review_cards_new RENAME TO review_cards;
CREATE TRIGGER queue_revision_card_insert AFTER INSERT ON review_cards
BEGIN INSERT INTO review_queue_changes(kind,card_id,vocabulary_id) VALUES('collection',NEW.id,NEW.vocabulary_id); END;
CREATE TRIGGER queue_revision_card_delete AFTER DELETE ON review_cards
BEGIN INSERT INTO review_queue_changes(kind,card_id,vocabulary_id) VALUES('collection',OLD.id,OLD.vocabulary_id); END;
CREATE TRIGGER queue_revision_card_update AFTER UPDATE ON review_cards
WHEN NEW.id IS NOT OLD.id OR NEW.vocabulary_id IS NOT OLD.vocabulary_id OR NEW.kind IS NOT OLD.kind
BEGIN INSERT INTO review_queue_changes(kind,card_id,vocabulary_id) VALUES('collection',NEW.id,NEW.vocabulary_id); END;
CREATE TRIGGER queue_revision_word_update AFTER UPDATE ON vocabulary WHEN NEW.revision IS NOT OLD.revision
BEGIN INSERT INTO review_queue_changes(kind,vocabulary_id) VALUES('record',NEW.id); END;
CREATE TABLE grammar_cards (
    card_id INTEGER PRIMARY KEY REFERENCES review_cards(id) ON DELETE CASCADE,
    question TEXT NOT NULL,
    answer TEXT NOT NULL
);
CREATE TABLE learning_categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    name_key TEXT NOT NULL UNIQUE
);
CREATE TABLE record_categories (
    record_id INTEGER NOT NULL REFERENCES vocabulary(id) ON DELETE CASCADE,
    category_id INTEGER NOT NULL REFERENCES learning_categories(id) ON DELETE CASCADE,
    PRIMARY KEY (record_id, category_id)
);
CREATE INDEX category_records ON record_categories(category_id, record_id);
CREATE TABLE deleted_grammar_operations (
    operation TEXT PRIMARY KEY NOT NULL
);
CREATE TABLE retained_review_usage (
    learning_day TEXT NOT NULL,
    reviewed_at INTEGER NOT NULL,
    new_cards INTEGER NOT NULL CHECK (new_cards >= 0),
    review_cards INTEGER NOT NULL CHECK (review_cards >= 0),
    PRIMARY KEY (learning_day, reviewed_at)
);
CREATE TRIGGER queue_revision_retained_insert AFTER INSERT ON retained_review_usage
BEGIN INSERT INTO review_queue_changes(kind) VALUES('quota'); END;
CREATE TRIGGER queue_revision_retained_update AFTER UPDATE ON retained_review_usage
BEGIN INSERT INTO review_queue_changes(kind) VALUES('quota'); END;
PRAGMA user_version = 19;
