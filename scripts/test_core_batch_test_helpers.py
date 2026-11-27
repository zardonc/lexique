"""Tests for shared core batch test helpers."""
import json
from pathlib import Path
import shutil
import tempfile
import unittest

from core_batch_test_helpers import (
    assert_core_load_inputs_rejected,
    assert_french_pos_frame,
    verify_archived_contributor_captures,
)
from core_pilot import MATERIALS, RESOURCE, load_inputs


class FrenchPosFrameTests(unittest.TestCase):
    def test_french_pos_frame_ignores_non_pos_sections_and_accepts_pronunciation(self):
        lines = ["== {{langue|fr}} ==", "=== {{S|nom|fr}} ===",
                 "=== {{S|prononciation}} ===", "* {{pron|sœʁ|fr}}"]
        assert_french_pos_frame(self, lines, 4, "NOM")
        assert_french_pos_frame(self, lines[:1], 1)

    def test_foreign_language_and_wrong_pos_frames_are_rejected(self):
        for language, pos in (("en", "nom"), ("fr", "verbe")):
            with self.subTest(language=language, pos=pos), self.assertRaises(AssertionError):
                assert_french_pos_frame(self,
                                       ["== {{langue|" + language + "}} ==", "=== {{S|" + pos + "|fr}} ==="],
                                       2, "NOM")


class ContributorCaptureTests(unittest.TestCase):
    def setUp(self):
        folder = tempfile.TemporaryDirectory()
        self.addCleanup(folder.cleanup)
        self.archive = Path(folder.name)
        self.parts = [self.archive / f"contributors-new-1-{part}.api.json" for part in (1, 2)]
        self.responses = [
            {"query": {"pages": [{"title": "pull", "contributors": [{"name": "Alice"}], "anoncontributors": 2}]},
             "continue": {"continue": "||", "pccontinue": "10|20"}},
            {"query": {"pages": [{"title": "pull", "contributors": [{"name": "Alice"}, {"name": "Bob"}]},
                                  {"title": "cavalier", "contributors": []}]}},
        ]
        self.request = "https://fr.wiktionary.org/w/api.php?continue=%7C%7C&pccontinue=10%7C20"

    def verify(self):
        for path, response in zip(self.parts, self.responses):
            path.write_text(json.dumps(response), encoding="utf-8")
        self.parts[1].with_suffix(".request.txt").write_text(self.request, encoding="utf-8")
        return verify_archived_contributor_captures(self, list(reversed(self.parts)), Path)

    def test_complete_pagination_aggregates_unique_names_and_anonymous_counts(self):
        names, anonymous, chains = self.verify()
        self.assertEqual(names, {"pull": {"Alice", "Bob"}, "cavalier": set()})
        self.assertEqual(anonymous, {"pull": 2})
        self.assertEqual(set(chains), {"contributors-new-1"})

    def test_incomplete_pagination_and_mismatched_requests_are_rejected(self):
        with self.subTest("missing continuation"):
            continuation = self.responses[0].pop("continue")
            with self.assertRaises(AssertionError):
                self.verify()
            self.responses[0]["continue"] = continuation
        with self.subTest("empty continuation"):
            self.responses[0]["continue"] = {}
            with self.assertRaises(AssertionError):
                self.verify()
            self.responses[0]["continue"] = continuation
        with self.subTest("missing intermediate part"):
            terminal = self.parts[1]
            self.parts[1] = self.archive / "contributors-new-1-3.api.json"
            with self.assertRaises(AssertionError):
                self.verify()
            self.parts[1] = terminal
        with self.subTest("unfinished final part"):
            self.responses[1]["continue"] = continuation
            with self.assertRaises(AssertionError):
                self.verify()
            del self.responses[1]["continue"]
        with self.subTest("wrong next request token"):
            self.request = self.request.replace("10%7C20", "10%7C21")
            with self.assertRaises(AssertionError):
                self.verify()
        with self.subTest("missing next request token"):
            self.request = "https://fr.wiktionary.org/w/api.php?continue=%7C%7C"
            with self.assertRaises(KeyError):
                self.verify()


class CoreLoadInputsHelperTests(unittest.TestCase):
    def test_assert_core_load_inputs_rejected_checks_mutation_and_unmutated_failure(self):
        def mutate(stage):
            review = json.loads((stage / "field-review.json").read_text(encoding="utf-8"))
            review["independence"] = False
            (stage / "field-review.json").write_text(json.dumps(review), encoding="utf-8")

        assert_core_load_inputs_rejected(
            self, RESOURCE, MATERIALS, load_inputs, mutate,
            "A separate independent field review is required", "helper-reject-",
        )
        with self.assertRaises(AssertionError):
            assert_core_load_inputs_rejected(
                self, RESOURCE, MATERIALS, load_inputs, lambda _stage: None,
                "A separate independent field review is required", "helper-pass-",
            )


if __name__ == "__main__":
    unittest.main()
