ALTER TABLE review_ratings ADD COLUMN undo_operation TEXT;
ALTER TABLE review_ratings ADD COLUMN undo_queue_rebuilt INTEGER NOT NULL DEFAULT 0 CHECK (undo_queue_rebuilt IN (0, 1));
CREATE UNIQUE INDEX review_ratings_undo_operation ON review_ratings(undo_operation);
PRAGMA user_version = 6;
