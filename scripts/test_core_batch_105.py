"""Resource-boundary check for the issue #105 candidate handoff."""

import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-105"


class Batch105Tests(unittest.TestCase):
    def test_exact_frozen_groups_have_sourced_candidates_or_explicit_gaps(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            scope = list(csv.DictReader(handle))
        candidates = json.loads((EVIDENCE / "review-candidates.json").read_text(encoding="utf-8"))
        self.assertEqual(candidates["scope_denominator_groups"], 4087)
        self.assertEqual([g["frozen_row"] for g in candidates["groups"]], list(range(1201, 1251)))
        excerpts = json.loads((EVIDENCE / "source-excerpts.json").read_text(encoding="utf-8"))
        bindings = {(r["sense_id"], r["field"]): r for r in excerpts["fields"]}
        for group in candidates["groups"]:
            with self.subTest(row=group["frozen_row"]):
                frozen = scope[group["frozen_row"] - 1]
                self.assertEqual((group["headword"], group["lexique_cgram"]),
                                 (frozen["word"], frozen["lexique_cgram"]))
                for name in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                    self.assertEqual(group[name], frozen[name])
                if not group["senses"]:
                    self.assertEqual(group["status"], "unresolved_fixed_source_blocker")
                    self.assertTrue(group["unresolved_reason"])
                    self.assertEqual((group["admitted_fields"], group["missing_fields"]), (0, 4))
                    self.assertTrue(group["blocker_evidence"])
                    continue
                self.assertEqual(len(group["senses"]), 1)
                sense = group["senses"][0]
                self.assertEqual(sense["independent_review"], "pending")
                self.assertTrue(sense["use"].strip())
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
