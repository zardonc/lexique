CREATE TABLE grammar_exchange_identities (
    record_id INTEGER PRIMARY KEY REFERENCES vocabulary(id) ON DELETE CASCADE,
    exchange_id TEXT NOT NULL UNIQUE DEFAULT (lower(hex(randomblob(16))))
        CHECK(length(exchange_id) BETWEEN 1 AND 128)
);
INSERT INTO grammar_exchange_identities(record_id)
SELECT id FROM vocabulary WHERE record_kind = 'grammar';
CREATE TRIGGER grammar_exchange_identity_insert AFTER INSERT ON vocabulary
WHEN NEW.record_kind = 'grammar'
BEGIN
    INSERT INTO grammar_exchange_identities(record_id) VALUES (NEW.id);
END;
PRAGMA user_version = 21;
