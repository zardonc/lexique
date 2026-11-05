"""Isolated V04/V05 SQLite contract candidate, not a product service or cache."""
import hashlib
import json
import sqlite3
import unicodedata


def semantic_key(text, **parameters):
    spec = {"text": unicodedata.normalize("NFC", text), "context": "word", "language": "fr",
            "engine": "sherpa-onnx/2.0.1", "model": "supertonic-3/int8/admitted",
            "preprocessing": "v1-y-i-grec", "voice": "female", "speed": 1, "steps": 8,
            "encoding": "ogg-opus", "bitrate": 48000, **parameters}
    return hashlib.sha256(json.dumps(spec, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()).hexdigest()


class AudioLedger:
    def __init__(self, path):
        self.connection = sqlite3.connect(path)
        self.connection.row_factory = sqlite3.Row
        self.connection.execute("PRAGMA foreign_keys=ON")
        self.connection.executescript("""
            CREATE TABLE IF NOT EXISTS assets(
                key TEXT PRIMARY KEY, digest TEXT NOT NULL CHECK(length(digest)=64),
                context TEXT NOT NULL, generated_at INTEGER, expires_at INTEGER,
                retain INTEGER NOT NULL DEFAULT 0);
            CREATE TABLE IF NOT EXISTS owners(
                key TEXT NOT NULL REFERENCES assets(key), owner TEXT NOT NULL, kind TEXT NOT NULL,
                PRIMARY KEY(key,owner));
            CREATE INDEX IF NOT EXISTS expiry ON assets(expires_at);
            CREATE TABLE IF NOT EXISTS tasks(
                id INTEGER PRIMARY KEY, key TEXT UNIQUE NOT NULL, priority INTEGER NOT NULL,
                state TEXT NOT NULL CHECK(state IN ('queued','running','retry_wait','ready','failed','cancelled')),
                attempts INTEGER NOT NULL DEFAULT 0, deadline INTEGER NOT NULL, wake_at INTEGER NOT NULL,
                error TEXT);
            CREATE INDEX IF NOT EXISTS pending ON tasks(state,priority,wake_at);
        """)

    def close(self):
        self.connection.close()

    def publish(self, key, digest, now, context="word"):
        with self.connection:
            self.connection.execute("INSERT INTO assets(key,digest,context,generated_at,expires_at) VALUES(?,?,?,?,?)",
                                    (key, digest, context, now, None if now is None else now + 604800))

    def asset(self, key):
        row = self.connection.execute("SELECT * FROM assets WHERE key=?", (key,)).fetchone()
        return dict(row) if row else None

    def attach(self, key, owner, kind):
        with self.connection:
            self.connection.execute("INSERT OR IGNORE INTO owners VALUES(?,?,?)", (key, owner, kind))

    def detach(self, key, owner):
        with self.connection:
            self.connection.execute("DELETE FROM owners WHERE key=? AND owner=?", (key, owner))
            # Last-owner and old-version deletion are deliberately conservative.
            self.connection.execute("UPDATE assets SET retain=1 WHERE key=? AND NOT EXISTS(SELECT 1 FROM owners WHERE key=?)", (key, key))

    def usable(self, key, now, access):
        if access not in ("play", "download", "inspect"):
            raise ValueError("Unknown access")
        row = self.connection.execute("""SELECT a.*, EXISTS(SELECT 1 FROM owners o WHERE o.key=a.key AND o.kind='word') AS persistent
            FROM assets a WHERE key=?""", (key,)).fetchone()
        return bool(row and (row["expires_at"] is None or row["context"] == "word" and row["persistent"] or now < row["expires_at"]))

    def gc_candidates(self, now):
        return [row[0] for row in self.connection.execute("""SELECT key FROM assets a WHERE expires_at<=? AND retain=0
            AND NOT (context='word' AND EXISTS(SELECT 1 FROM owners o WHERE o.key=a.key AND o.kind='word')) ORDER BY key LIMIT 100""", (now,))]

    def may_delete(self, key, now, in_use=False):
        return not in_use and key in self.gc_candidates(now)

    def register_legacy(self, entries):
        with self.connection:
            for key, digest in entries:
                self.connection.execute("INSERT INTO assets(key,digest,context) VALUES(?,?,'legacy')", (key, digest))

    def demand(self, key, priority, now, budget_seconds=60):
        with self.connection:
            self.connection.execute("""INSERT INTO tasks(key,priority,state,deadline,wake_at) VALUES(?,?,'queued',?,?)
                ON CONFLICT(key) DO UPDATE SET priority=MIN(priority,excluded.priority) WHERE state IN ('queued','retry_wait')""",
                                    (key, priority, now + budget_seconds, now))
            return self.connection.execute("SELECT id FROM tasks WHERE key=?", (key,)).fetchone()[0]

    def task(self, identity):
        row = self.connection.execute("SELECT * FROM tasks WHERE id=?", (identity,)).fetchone()
        return dict(row) if row else None

    def claim(self, now, background_limit=1):
        with self.connection:
            self.connection.execute("UPDATE tasks SET state='failed',error='budget exhausted' WHERE state IN ('queued','retry_wait') AND deadline<=?", (now,))
            running = {row[0]: row[1] for row in self.connection.execute("SELECT priority,COUNT(*) FROM tasks WHERE state='running' GROUP BY priority")}
            row = self.connection.execute("""SELECT id FROM tasks WHERE state IN ('queued','retry_wait') AND wake_at<=?
                AND attempts<4 AND ((priority=0 AND ?=0) OR (priority=1 AND ?<?)) ORDER BY priority,id LIMIT 1""",
                                          (now, running.get(0, 0), running.get(1, 0), background_limit)).fetchone()
            if row:
                self.connection.execute("UPDATE tasks SET state='running',attempts=attempts+1 WHERE id=?", (row[0],))
                return row[0]
        return None

    def complete(self, identity, valid_published):
        if not valid_published:
            raise ValueError("No validated publication")
        with self.connection:
            if self.connection.execute("UPDATE tasks SET state='ready',error=NULL WHERE id=? AND state='running'", (identity,)).rowcount != 1:
                raise ValueError("Stale completion")

    def failure(self, identity, now, error, retry_after=0, permanent=False):
        task = self.task(identity)
        if task["state"] != "running":
            raise ValueError("Stale failure")
        attempts = task["attempts"]
        delay = max((2, 8, 20)[min(attempts - 1, 2)], retry_after)
        state = "failed" if permanent or attempts >= 4 or now + delay >= task["deadline"] else "retry_wait"
        with self.connection:
            self.connection.execute("UPDATE tasks SET state=?,wake_at=?,error=? WHERE id=?", (state, now + delay, error, identity))

    def cancel_remaining(self):
        with self.connection:
            self.connection.execute("UPDATE tasks SET state='cancelled' WHERE state IN ('queued','retry_wait')")

    def recover(self, now, validate_published):
        for row in self.connection.execute("SELECT * FROM tasks WHERE state='running'").fetchall():
            if validate_published(row["key"]):
                self.complete(row["id"], True)
            else:
                state = "failed" if row["attempts"] >= 4 or now >= row["deadline"] else "queued"
                with self.connection:
                    self.connection.execute("UPDATE tasks SET state=?,wake_at=?,error='worker interrupted' WHERE id=?", (state, now, row["id"]))
