CREATE TABLE review_queue_changes (
    revision INTEGER PRIMARY KEY AUTOINCREMENT,
    kind TEXT NOT NULL CHECK(kind IN ('collection','progress','position','audio','record','settings','quota','context')),
    card_id INTEGER,
    vocabulary_id INTEGER,
    locator TEXT
);
CREATE TRIGGER queue_revision_prune AFTER INSERT ON review_queue_changes
BEGIN DELETE FROM review_queue_changes WHERE revision<=NEW.revision-1024; END;
CREATE TRIGGER queue_revision_card_insert AFTER INSERT ON review_cards
BEGIN INSERT INTO review_queue_changes(kind,card_id,vocabulary_id) VALUES('collection',NEW.id,NEW.vocabulary_id); END;
CREATE TRIGGER queue_revision_card_delete AFTER DELETE ON review_cards
BEGIN INSERT INTO review_queue_changes(kind,card_id,vocabulary_id) VALUES('collection',OLD.id,OLD.vocabulary_id); END;
CREATE TRIGGER queue_revision_card_update AFTER UPDATE ON review_cards
WHEN NEW.id IS NOT OLD.id OR NEW.vocabulary_id IS NOT OLD.vocabulary_id OR NEW.kind IS NOT OLD.kind
BEGIN INSERT INTO review_queue_changes(kind,card_id,vocabulary_id) VALUES('collection',NEW.id,NEW.vocabulary_id); END;
CREATE TRIGGER queue_revision_progress_insert AFTER INSERT ON review_progress
BEGIN INSERT INTO review_queue_changes(kind,card_id) VALUES('progress',NEW.card_id); END;
CREATE TRIGGER queue_revision_progress_update AFTER UPDATE ON review_progress
WHEN NEW.revision IS NOT OLD.revision OR NEW.state IS NOT OLD.state
BEGIN INSERT INTO review_queue_changes(kind,card_id) VALUES('progress',NEW.card_id); END;
CREATE TRIGGER queue_revision_progress_delete AFTER DELETE ON review_progress
BEGIN INSERT INTO review_queue_changes(kind,card_id) VALUES('progress',OLD.card_id); END;
CREATE TRIGGER queue_revision_pause_insert AFTER INSERT ON review_paused
BEGIN INSERT INTO review_queue_changes(kind,vocabulary_id) VALUES('collection',NEW.vocabulary_id); END;
CREATE TRIGGER queue_revision_pause_delete AFTER DELETE ON review_paused
BEGIN INSERT INTO review_queue_changes(kind,vocabulary_id) VALUES('collection',OLD.vocabulary_id); END;
CREATE TRIGGER queue_revision_position_insert AFTER INSERT ON review_queue_entries
BEGIN INSERT INTO review_queue_changes(kind,card_id) VALUES('position',NEW.card_id); END;
CREATE TRIGGER queue_revision_position_update AFTER UPDATE ON review_queue_entries WHEN NEW.position IS NOT OLD.position
BEGIN INSERT INTO review_queue_changes(kind,card_id) VALUES('position',NEW.card_id); END;
CREATE TRIGGER queue_revision_position_delete AFTER DELETE ON review_queue_entries
BEGIN INSERT INTO review_queue_changes(kind,card_id) VALUES('position',OLD.card_id); END;
CREATE TRIGGER queue_revision_selection_insert AFTER INSERT ON word_audio_selections
BEGIN INSERT INTO review_queue_changes(kind,vocabulary_id) VALUES('audio',NEW.vocabulary_id); END;
CREATE TRIGGER queue_revision_selection_update AFTER UPDATE ON word_audio_selections
WHEN NEW.current_version IS NOT OLD.current_version OR NEW.revision IS NOT OLD.revision OR NEW.desired_generation IS NOT OLD.desired_generation
BEGIN INSERT INTO review_queue_changes(kind,vocabulary_id) VALUES('audio',NEW.vocabulary_id); END;
CREATE TRIGGER queue_revision_selection_delete AFTER DELETE ON word_audio_selections
BEGIN INSERT INTO review_queue_changes(kind,vocabulary_id) VALUES('audio',OLD.vocabulary_id); END;
CREATE TRIGGER queue_revision_version_update AFTER UPDATE ON speech_versions
WHEN NEW.verification IS NOT OLD.verification OR NEW.gc_state IS NOT OLD.gc_state OR NEW.sealed IS NOT OLD.sealed
BEGIN INSERT INTO review_queue_changes(kind,locator) VALUES('audio',NEW.locator); END;
CREATE TRIGGER queue_revision_word_update AFTER UPDATE ON vocabulary WHEN NEW.revision IS NOT OLD.revision
BEGIN INSERT INTO review_queue_changes(kind,vocabulary_id) VALUES('record',NEW.id); END;
CREATE TRIGGER queue_revision_settings_update AFTER UPDATE ON review_settings
WHEN NEW.retention_percent IS NOT OLD.retention_percent OR NEW.new_limit IS NOT OLD.new_limit OR NEW.review_limit IS NOT OLD.review_limit
BEGIN INSERT INTO review_queue_changes(kind) VALUES('settings'); END;
CREATE TRIGGER queue_revision_ignore_insert AFTER INSERT ON app_metadata WHEN NEW.key='new-cards-ignore-review-limit'
BEGIN INSERT INTO review_queue_changes(kind) VALUES('settings'); END;
CREATE TRIGGER queue_revision_ignore_update AFTER UPDATE ON app_metadata WHEN NEW.key='new-cards-ignore-review-limit' AND NEW.value IS NOT OLD.value
BEGIN INSERT INTO review_queue_changes(kind) VALUES('settings'); END;
CREATE TRIGGER queue_revision_ignore_delete AFTER DELETE ON app_metadata WHEN OLD.key='new-cards-ignore-review-limit'
BEGIN INSERT INTO review_queue_changes(kind) VALUES('settings'); END;
CREATE TRIGGER queue_revision_rating_insert AFTER INSERT ON review_ratings
BEGIN INSERT INTO review_queue_changes(kind,card_id) VALUES('quota',NEW.card_id); END;
CREATE TRIGGER queue_revision_rating_update AFTER UPDATE ON review_ratings WHEN NEW.undo_operation IS NOT OLD.undo_operation
BEGIN INSERT INTO review_queue_changes(kind,card_id) VALUES('quota',NEW.card_id); END;
CREATE TRIGGER queue_revision_context_insert AFTER INSERT ON review_queue_state
BEGIN INSERT INTO review_queue_changes(kind) VALUES('context'); END;
CREATE TRIGGER queue_revision_context_update AFTER UPDATE ON review_queue_state
WHEN NEW.learning_day IS NOT OLD.learning_day OR NEW.timezone IS NOT OLD.timezone
BEGIN INSERT INTO review_queue_changes(kind) VALUES('context'); END;
CREATE TRIGGER queue_revision_context_delete AFTER DELETE ON review_queue_state
BEGIN INSERT INTO review_queue_changes(kind) VALUES('context'); END;
PRAGMA user_version = 16;
