"""Resource-boundary check of the issue #108 author handoff."""

import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-108"


class Batch108Tests(unittest.TestCase):
    def test_exact_frozen_groups_have_four_bound_candidate_fields(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            scope = list(csv.DictReader(handle))
        candidates = json.loads((EVIDENCE / "review-candidates.json").read_text(encoding="utf-8"))
        groups = candidates["groups"]
        self.assertEqual([group["frozen_row"] for group in groups], list(range(1351, 1401)))
        self.assertEqual(candidates["scope_denominator_groups"], 4087)
        excerpts = json.loads((EVIDENCE / "source-excerpts.json").read_text(encoding="utf-8"))
        bindings = {(record["sense_id"], record["field"]): record for record in excerpts["fields"]}
        self.assertEqual(len(bindings), 200)
        for group in groups:
            row = group["frozen_row"]
            with self.subTest(row=row):
                frozen = scope[row - 1]
                self.assertEqual((group["headword"], group["lexique_cgram"]),
                                 (frozen["word"], frozen["lexique_cgram"]))
                for name in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                    self.assertEqual(group[name], frozen[name])
                self.assertEqual(len(group["senses"]), 1)
                sense = group["senses"][0]
                self.assertEqual(sense["independent_review"], "pending")
                self.assertIsNone(sense["cefr"])
                self.assertTrue(sense["use"])
                self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                for name, field in sense["fields"].items():
                    self.assertTrue(field["text"].strip())
                    self.assertEqual(field["independent_review"], "pending")
                    binding = bindings[sense["sense_id"], name]
                    self.assertEqual(field["source_id"], binding["source_id"])
                    self.assertEqual(field["source_span"]["raw_source_excerpt"], binding["raw_source_excerpt"])
                    self.assertEqual(field["source_span"]["line_1_based"], binding["line_1_based"])
                    self.assertGreater(binding["revid"], 0)
                    self.assertIn("oldid=" + str(binding["revid"]), binding["fixed_url"])


if __name__ == "__main__":
    unittest.main()
