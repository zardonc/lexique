"""Resource-boundary membership check for the issue #99 review handoff."""

import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-99"
RESOURCE = ROOT / "resources/core-pilot-79"


class Batch99Tests(unittest.TestCase):
    def test_exact_frozen_groups_have_sourced_fields_or_explicit_gaps(self):
        with (RESOURCE / "scope-manifest.csv").open(encoding="utf-8-sig", newline="") as handle:
            scope = list(csv.DictReader(handle))
        candidates = json.loads((EVIDENCE / "review-candidates.json").read_text(encoding="utf-8"))
        groups = candidates["groups"]
        self.assertEqual([group["frozen_row"] for group in groups], list(range(901, 951)))
        self.assertEqual(candidates["scope_denominator_groups"], 4087)
        excerpts = json.loads((EVIDENCE / "source-excerpts.json").read_text(encoding="utf-8"))
        bindings = {(record["sense_id"], record["field"]): record for record in excerpts["fields"]}
        for group in groups:
            row = group["frozen_row"]
            with self.subTest(row=row):
                self.assertEqual((group["headword"], group["lexique_cgram"]),
                                 (scope[row - 1]["word"], scope[row - 1]["lexique_cgram"]))
                self.assertEqual(group["subtitle_lemma_per_million"], scope[row - 1]["subtitle_lemma_per_million"])
                self.assertEqual(group["subtitle_pos_per_million"], scope[row - 1]["subtitle_pos_per_million"])
                if row == 934:
                    self.assertEqual(group["senses"], [])
                    self.assertTrue(group["unresolved_reason"])
                    continue
                self.assertEqual(len(group["senses"]), 1)
                sense = group["senses"][0]
                self.assertEqual(sense["independent_review"], "pending")
                self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                for name, field in sense["fields"].items():
                    self.assertTrue(field["text"].strip())
                    self.assertEqual(field["independent_review"], "pending")
                    binding = bindings[sense["sense_id"], name]
                    self.assertEqual(field["source_id"], binding["source_id"])
                    self.assertEqual(field["source_span"]["raw_source_excerpt"], binding["reviewed_excerpt"])
                    self.assertEqual(field["source_span"]["line_1_based"], binding["line_1_based"])
                    self.assertGreater(binding["revid"], 0)
                    self.assertIn("oldid=" + str(binding["revid"]), binding["url"])


if __name__ == "__main__":
    unittest.main()
