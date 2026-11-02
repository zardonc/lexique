"""V04/V05 contract checks; controlled clock/worker, never product integration."""
import hashlib
import sqlite3
import tempfile
import unittest
from pathlib import Path

from preflight_audio import AudioLedger, semantic_key


class LedgerFixture(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.path = Path(self.directory.name) / "ledger.sqlite3"
        self.ledger = AudioLedger(self.path)
        self.key = semantic_key("école")
        self.digest = hashlib.sha256(b"OggSfixture").hexdigest()

    def tearDown(self):
        self.ledger.close()
        self.directory.cleanup()


class CacheContract(LedgerFixture):
    def test_v04_absolute_ttl_boundaries_and_access_never_renews(self):
        self.ledger.publish(self.key, self.digest, 1000)
        for now, usable in ((1000, True), (605799, True), (605800, False), (605801, False)):
            for access in ("play", "download", "inspect"):
                self.assertEqual(self.ledger.usable(self.key, now, access), usable)
        self.assertEqual(self.ledger.asset(self.key)["expires_at"], 605800)

    def test_v04_semantic_identity_uses_nfc_and_preserves_distinctions(self):
        self.assertEqual(self.key, semantic_key("e\u0301cole"))
        for text in ("École", "ecole", "école ", "aujourd'hui", "aujourd’hui"):
            self.assertNotEqual(self.key, semantic_key(text))
        self.assertNotEqual(semantic_key("aujourd'hui"), semantic_key("aujourd’hui"))
        for field, value in (("voice", "male"), ("model", "different"), ("context", "sentence"),
                             ("language", "en"), ("preprocessing", "v2"), ("steps", 4), ("bitrate", 24000)):
            self.assertNotEqual(self.key, semantic_key("école", **{field: value}))

    def test_v04_favorite_without_cards_and_shared_owners_protect_word_only(self):
        self.ledger.publish(self.key, self.digest, 1000)
        self.ledger.attach(self.key, "word:1", "word")
        self.ledger.attach(self.key, "word:2", "word")
        self.assertTrue(self.ledger.usable(self.key, 605800, "inspect"))
        self.ledger.detach(self.key, "word:1")
        self.assertTrue(self.ledger.usable(self.key, 605800, "play"))
        sentence = semantic_key("Une école.", context="sentence")
        self.ledger.publish(sentence, self.digest, 1000, context="sentence")
        self.ledger.attach(sentence, "word:2", "word")
        self.assertFalse(self.ledger.usable(sentence, 605800, "play"))
        self.ledger.detach(self.key, "word:2")
        self.assertEqual(self.ledger.gc_candidates(605800), [sentence])

    def test_v04_legacy_unknown_clock_is_preserved_and_registration_rolls_back(self):
        self.ledger.publish("legacy:1", self.digest, None)
        self.assertTrue(self.ledger.usable("legacy:1", 9999999, "inspect"))
        self.assertEqual(self.ledger.gc_candidates(9999999), [])
        with self.assertRaises(sqlite3.IntegrityError):
            self.ledger.register_legacy([(self.key, self.digest), (self.key, "invalid")])
        self.assertIsNone(self.ledger.asset(self.key))
        self.assertIsNotNone(self.ledger.asset("legacy:1"))

    def test_v04_gc_rechecks_owner_and_active_use(self):
        self.ledger.publish(self.key, self.digest, 1000)
        self.assertEqual(self.ledger.gc_candidates(605800), [self.key])
        self.ledger.attach(self.key, "word:1", "word")
        self.assertFalse(self.ledger.may_delete(self.key, 605800))
        sentence = semantic_key("Une école.", context="sentence")
        self.ledger.publish(sentence, self.digest, 1000, context="sentence")
        self.assertFalse(self.ledger.may_delete(sentence, 605800, in_use=True))


class TaskContract(LedgerFixture):
    def test_v05_dedup_promotion_reserved_interactive_and_cancel(self):
        background = self.ledger.demand("background", 1, 1000)
        promoted = self.ledger.demand("promoted", 1, 1000)
        self.assertEqual(self.ledger.demand("promoted", 0, 1000), promoted)
        self.assertEqual(self.ledger.claim(1000, background_limit=1), promoted)
        self.assertEqual(self.ledger.claim(1000, background_limit=1), background)
        queued = self.ledger.demand("queued", 1, 1000)
        self.assertIsNone(self.ledger.claim(1000, background_limit=1))
        self.ledger.cancel_remaining()
        self.assertEqual(self.ledger.task(queued)["state"], "cancelled")
        self.assertEqual(self.ledger.task(background)["state"], "running")
        self.ledger.complete(background, valid_published=True)
        self.assertEqual(self.ledger.task(background)["state"], "ready")

    def test_v05_four_calls_retry_after_and_budget_survive_restart(self):
        task = self.ledger.demand("retry", 0, 1000)
        now = 1000
        for attempt, delay in enumerate((5, 8, 20, None), 1):
            self.assertEqual(self.ledger.claim(now), task)
            self.ledger.failure(task, now, "transport unavailable", retry_after=5)
            self.assertEqual(self.ledger.task(task)["attempts"], attempt)
            if delay is not None:
                self.assertIsNone(self.ledger.claim(now + delay - 1))
                now += delay
                self.ledger.close()
                self.ledger = AudioLedger(self.path)
        self.assertEqual(self.ledger.task(task)["state"], "failed")
        self.assertIsNone(self.ledger.claim(now + 999))
        budget = self.ledger.demand("budget", 0, 1000, budget_seconds=3)
        self.assertEqual(self.ledger.claim(1000), budget)
        self.ledger.failure(budget, 1000, "worker unavailable", retry_after=5)
        self.assertEqual(self.ledger.task(budget)["state"], "failed")

    def test_v05_publish_failure_and_crash_gap_do_not_simulate_success(self):
        task = self.ledger.demand("crash", 0, 1000)
        self.ledger.claim(1000)
        with self.assertRaises(ValueError):
            self.ledger.complete(task, valid_published=False)
        self.assertEqual(self.ledger.task(task)["state"], "running")
        self.ledger.close()
        self.ledger = AudioLedger(self.path)
        self.ledger.recover(1001, lambda key: key == "crash")
        self.assertEqual(self.ledger.task(task)["state"], "ready")
        unknown = self.ledger.demand("bad-file", 0, 1000)
        self.ledger.claim(1000)
        self.ledger.recover(1001, lambda _: False)
        self.assertEqual(self.ledger.task(unknown)["state"], "queued")
        self.assertEqual(self.ledger.task(unknown)["attempts"], 1)

    def test_v05_cancel_and_hard_failures_survive_restart(self):
        task = self.ledger.demand("queued", 1, 1000)
        self.ledger.cancel_remaining()
        self.ledger.close()
        self.ledger = AudioLedger(self.path)
        self.ledger.recover(1001, lambda _: False)
        self.assertIsNone(self.ledger.claim(1001))
        self.assertEqual(self.ledger.task(task)["state"], "cancelled")
        task = self.ledger.demand("disk", 0, 1000)
        self.ledger.claim(1000)
        self.ledger.failure(task, 1001, "disk denied", permanent=True)
        self.assertEqual(self.ledger.task(task)["state"], "failed")


if __name__ == "__main__":
    unittest.main()
