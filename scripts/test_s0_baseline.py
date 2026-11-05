"""Behavior checks for the #25 independent baseline and sampling tools."""
import json
import io
from contextlib import redirect_stdout
from pathlib import Path
import sqlite3
import tempfile
import threading
import unittest
from unittest.mock import patch

from s0_baseline import oracle, summarize, seed_rows
from s0_faults import Rendezvous, Worker, files


class BaselineTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.connection = sqlite3.connect(Path(self.directory.name) / "fixture.sqlite3")
        self.addCleanup(self.connection.close)
        self.connection.executescript("""
            CREATE TABLE vocabulary(id INTEGER PRIMARY KEY, headword TEXT, snapshot TEXT,
                notes TEXT, created_at INTEGER, updated_at INTEGER);
            CREATE TABLE review_cards(id INTEGER PRIMARY KEY, vocabulary_id INTEGER, kind TEXT);
            CREATE TABLE review_progress(card_id INTEGER PRIMARY KEY, revision INTEGER, state TEXT);
            CREATE TABLE review_paused(vocabulary_id INTEGER PRIMARY KEY);
            CREATE TABLE review_settings(id INTEGER, retention_percent INTEGER, new_limit INTEGER, review_limit INTEGER);
            INSERT INTO review_settings VALUES(1,90,20,50);
            CREATE TABLE app_metadata(key TEXT PRIMARY KEY,value TEXT);
            CREATE TABLE review_ratings(quota TEXT,before_state TEXT,request TEXT,
                learning_day TEXT,reviewed_at INTEGER,undo_operation TEXT);
            CREATE TABLE review_queue_entries(card_id INTEGER PRIMARY KEY,position INTEGER);
            CREATE TABLE review_queue_state(id INTEGER,learning_day TEXT,timezone TEXT);
        """)

    def result(self, now=1_700_000_000, timezone="UTC", health=None, skipped=()):
        return oracle(self.connection, now, timezone, health or {}, skipped)

    def test_new_cards_without_progress_and_null_due_are_discoverable(self):
        seed_rows(self.connection, 2, "new", 1_700_000_000)
        self.connection.execute("INSERT INTO review_progress VALUES(1,4,?)", [json.dumps(
            {"phase": "new", "memory": None, "last_review_at": None, "due_at": None})])
        result = self.result(health={1: "healthy", 2: "missing"})
        self.assertEqual(result["ids"], [1, 2, 3])
        self.assertEqual(result["classes"], {"new": 3, "review": 0, "interday": 0, "intraday": 0})

    def test_zero_quotas_keep_both_intraday_cards(self):
        seed_rows(self.connection, 1, "intraday", 1_700_000_000)
        self.connection.execute("UPDATE review_settings SET new_limit=0,review_limit=0")
        self.assertEqual(self.result(health={1: "healthy"})["ids"], [1, 2])
        self.assertEqual(self.result(health={1: "corrupt"})["ids"], [1])

    def test_pause_skip_unknown_health_and_saved_positions_are_independent(self):
        seed_rows(self.connection, 3, "new", 1_700_000_000)
        self.connection.execute("INSERT INTO review_paused VALUES(2)")
        self.connection.executemany("INSERT INTO review_queue_entries VALUES(?,?)", [(6, 10), (1, 20), (5, 30)])
        self.connection.execute("INSERT INTO review_queue_state VALUES(1,'2023-11-14','UTC')")
        result = self.result(health={1: "unknown", 3: "healthy"}, skipped=[5])
        self.assertEqual(result["ids"], [6, 1])
        self.assertEqual(result["unknown_audio"], 1)

    def test_local_four_am_and_dst_use_absolute_due_times(self):
        seed_rows(self.connection, 1, "new", 1_700_000_000)
        # Paris 2026-03-29: DST advances at 01:00 UTC; 04:00 local is 02:00 UTC.
        before, boundary = 1_774_749_599, 1_774_749_600
        self.assertEqual(self.result(before, "Europe/Paris")["learning_day"], "2026-03-28")
        self.assertEqual(self.result(boundary, "Europe/Paris")["learning_day"], "2026-03-29")
        self.assertEqual(self.result(before, "Europe/Paris")["next_day_at"], boundary)

    def test_minute_learning_does_not_consume_daily_review_usage(self):
        seed_rows(self.connection, 1, "intraday", 1_700_000_000)
        state = self.connection.execute("SELECT state FROM review_progress WHERE card_id=1").fetchone()[0]
        self.connection.execute("INSERT INTO review_ratings VALUES('review',?,?,?, ?,NULL)",
            [state, json.dumps({"timezone": "UTC"}), "2023-11-14", 1_699_999_990])
        self.assertEqual(self.result(health={1: "healthy"})["used"], {"new": 0, "review": 0})
        self.assertEqual(self.result(1_699_999_989, health={1: "healthy"})["reason"], "clock")

    def test_tail_report_requires_one_hundred_attempts_and_retains_failures(self):
        with self.assertRaises(ValueError):
            summarize([{"ms": 1, "status": "passed"}] * 3)
        rows = [{"ms": value, "status": "passed"} for value in range(1, 100)]
        rows.append({"ms": 60_000, "status": "timeout"})
        result = summarize(rows)
        self.assertEqual(result["attempts"], 100)
        self.assertEqual(result["failures"], 1)
        self.assertEqual(result["p95_ms"], 95)
        self.assertEqual(result["max_ms"], 60_000)
        self.assertFalse(result["accepted"])

    def test_worker_barrier_holds_completion_until_competing_action_finishes(self):
        barrier = Rendezvous("renamed")
        result = []
        worker = Worker(barrier)
        thread = threading.Thread(target=lambda: result.append(worker("synthetic", "female")))
        thread.start()
        try:
            barrier.wait()
            self.assertEqual(result, [])
            barrier.release()
        finally:
            barrier.release()
            thread.join(timeout=5)
        self.assertFalse(thread.is_alive())
        self.assertEqual(result, [b"OggSfixture"])
        with self.assertRaisesRegex(RuntimeError, "controlled failure"):
            Worker(error=RuntimeError("controlled failure"))("synthetic", "male")

    def test_file_faults_are_real_and_header_fixture_is_never_playback_proof(self):
        root = Path(self.directory.name) / "audio"
        manifest = files(root)
        self.assertNotEqual(manifest["wrong_sha"]["sha256"], manifest["wrong_sha"]["expected_sha256"])
        self.assertGreater((root / "oversize.ogg").stat().st_size, 10_000_000)
        self.assertFalse((root / "missing.ogg").exists())
        self.assertFalse(any(item.get("playback_evidence") for item in manifest.values()))

    def test_matrix_failure_on_first_request_is_not_hidden_by_healthy_warm_samples(self):
        import measure_s0
        root = Path(self.directory.name)
        report = {"first": {"status": "failed", "ms": 60000},
            "steady": {"attempts": 100, "failures": 0}}
        with patch.object(measure_s0, "new_directory", side_effect=lambda path: path), \
                patch.object(measure_s0, "queue_case", return_value=report), \
                patch("sys.argv", ["measure_s0", str(root), "--binary", "unused", "--matrix"]), \
                redirect_stdout(io.StringIO()):
            with self.assertRaises(SystemExit) as result:
                measure_s0.main()
        self.assertEqual(result.exception.code, 1)


if __name__ == "__main__":
    unittest.main()
