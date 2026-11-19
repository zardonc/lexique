"""Frozen membership and source-boundary check for the #103 author handoff."""

import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-103"
RESOURCE = ROOT / "resources/core-pilot-79"


class Batch103Tests(unittest.TestCase):
    def test_exact_frozen_groups_have_four_fields_bound_to_original_sources(self):
        with (RESOURCE / "scope-manifest.csv").open(encoding="utf-8-sig", newline="") as handle:
            scope = list(csv.DictReader(handle))
        candidates = json.loads((EVIDENCE / "review-candidates.json").read_text(encoding="utf-8"))
        groups = candidates["groups"]
        self.assertEqual(candidates["scope_denominator_groups"], 4087)
        self.assertEqual([group["frozen_row"] for group in groups], list(range(1101, 1151)))
        excerpts = json.loads((EVIDENCE / "source-excerpts.json").read_text(encoding="utf-8"))
        bindings = {(record["sense_id"], record["field"]): record for record in excerpts["fields"]}
        sources = json.loads((EVIDENCE / "sources.json").read_text(encoding="utf-8"))
        self.assertEqual(groups, json.loads((EVIDENCE / "groups.json").read_text(encoding="utf-8")))
        review = json.loads((EVIDENCE / "field-review.json").read_text(encoding="utf-8"))
        self.assertFalse(review["independence"])
        self.assertIsNone(review["independent_reviewer"])
        reviewed = {(record["sense_id"], record["field"]): record for record in review["fields"]}
        self.assertEqual(len(reviewed), 200)
        captures = json.loads((EVIDENCE / "source-captures.json").read_text(encoding="utf-8"))["sources"]
        raw = {}
        for capture in captures:
            page = next(page for page in json.loads(Path(capture["api_path"]).read_text(encoding="utf-8"))["query"]["pages"]
                        if page["pageid"] == capture["pageid"])
            revision = next(rev for rev in page["revisions"] if rev["revid"] == capture["revid"])
            body = Path(capture["raw_path"]).read_text(encoding="utf-8")
            self.assertEqual(body, revision["slots"]["main"]["content"])
            self.assertEqual(revision["timestamp"], capture["timestamp"])
            raw[capture["source_id"]] = body.splitlines()
        self.assertEqual(len(bindings), 200)
        self.assertEqual({source["id"] for source in sources}, set(raw))
        for group in groups:
            row = group["frozen_row"]
            with self.subTest(row=row):
                self.assertEqual((group["headword"], group["lexique_cgram"]),
                                 (scope[row - 1]["word"], scope[row - 1]["lexique_cgram"]))
                for frequency in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                    self.assertEqual(group[frequency], scope[row - 1][frequency])
                self.assertEqual(len(group["senses"]), 1)
                sense = group["senses"][0]
                self.assertEqual(sense["independent_review"], "pending")
                self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                self.assertTrue(sense["use"])
                for support in sense["supporting_evidence"]:
                    self.assertIn(support["raw_source_excerpt"], raw[support["source_id"]][support["line_1_based"] - 1])
                for name, field in sense["fields"].items():
                    self.assertTrue(field["text"].strip())
                    self.assertEqual(field["independent_review"], "pending")
                    self.assertTrue(field["modification"])
                    self.assertEqual(field["content_license"], "CC-BY-SA-4.0")
                    binding = bindings[sense["sense_id"], name]
                    record = reviewed[sense["sense_id"], name]
                    self.assertEqual(record["proposed_text"], field["text"])
                    self.assertEqual(record["accepted_text_quality_status"], "pending")
                    self.assertEqual(record["rights_eligibility_status"], "pending_selected_span_audit")
                    self.assertIsNone(record["reviewer"])
                    self.assertEqual(record["exact_source"]["reviewed_excerpt"], binding["raw_source_excerpt"])
                    self.assertEqual(field["source_id"], binding["source_id"])
                    self.assertEqual(field["source_span"], {key: binding[key] for key in ("line_1_based", "column_1_based", "raw_source_excerpt")})
                    self.assertGreater(binding["revid"], 0)
                    self.assertIn("oldid=" + str(binding["revid"]), binding["fixed_url"])
                    line = raw[binding["source_id"]][binding["line_1_based"] - 1]
                    start = binding["column_1_based"] - 1
                    self.assertEqual(line[start:start + len(binding["raw_source_excerpt"])], binding["raw_source_excerpt"])


if __name__ == "__main__":
    unittest.main()
