"""Resource-boundary author handoff check for issue #102; no admission claim."""

import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-102"


class Batch102Tests(unittest.TestCase):
    def test_exact_ticket_groups_have_fields_or_visible_source_blockers(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            scope = list(csv.DictReader(handle))
        candidates = json.loads((EVIDENCE / "review-candidates.json").read_text(encoding="utf-8"))
        self.assertEqual(candidates["ticket"], 102)
        self.assertEqual(candidates["scope_denominator_groups"], 4087)
        groups = candidates["groups"]
        self.assertEqual([group["frozen_row"] for group in groups], list(range(1051, 1101)))
        blocked = [group for group in groups if not group["senses"]]
        self.assertEqual([group["frozen_row"] for group in blocked], [1051, 1057])
        self.assertEqual(sum(len(group["senses"]) for group in groups), 48)
        for group in groups:
            row = group["frozen_row"]
            with self.subTest(row=row):
                expected = scope[row - 1]
                self.assertEqual((group["headword"], group["lexique_cgram"]),
                                 (expected["word"], expected["lexique_cgram"]))
                for name in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                    self.assertEqual(group[name], expected[name])
                self.assertEqual(group["independent_review"], "pending")
                self.assertEqual(group["admitted_fields"], 0)
                self.assertTrue(group["senses"] or group["unresolved_reason"])

    def test_candidate_fields_bind_fixed_sources_without_claiming_admission(self):
        candidates = json.loads((EVIDENCE / "review-candidates.json").read_text(encoding="utf-8"))
        excerpts = json.loads((EVIDENCE / "source-excerpts.json").read_text(encoding="utf-8"))
        sources = json.loads((EVIDENCE / "sources.json").read_text(encoding="utf-8"))
        bindings = {(b["sense_id"], b["field"]): b for b in excerpts["fields"]}
        source_by_id = {s["id"]: s for s in sources}
        self.assertEqual(len(bindings), 192)
        self.assertEqual(len(bindings), len(excerpts["fields"]))
        self.assertEqual(len(source_by_id), len(sources))
        self.assertEqual(json.loads((EVIDENCE / "groups.json").read_text(encoding="utf-8")), candidates["groups"])
        admission = json.loads((EVIDENCE / "admission.json").read_text(encoding="utf-8"))
        self.assertEqual(admission["status"], "pending")
        self.assertFalse(admission["independence"])
        self.assertEqual((admission["accepted_groups"], admission["accepted_fields"]), (0, 0))
        self.assertEqual(admission["approved_annotation_exceptions"], [])
        languages = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
        for group in candidates["groups"]:
            for sense in group["senses"]:
                with self.subTest(row=group["frozen_row"]):
                    self.assertEqual(set(sense["fields"]), set(languages))
                    self.assertTrue(sense["use"])
                    self.assertEqual(sense["independent_review"], "pending")
                    for name, field in sense["fields"].items():
                        self.assertTrue(field["text"].strip())
                        self.assertEqual(field["language"], languages[name])
                        self.assertEqual(field["independent_review"], "pending")
                        self.assertIsNone(field["reviewer"])
                        source = source_by_id[field["source_id"]]
                        self.assertTrue(source["attribution"])
                        self.assertTrue(source["redistribution_obligations"])
                        self.assertEqual(source["independent_review"], "pending")
                        binding = bindings[sense["sense_id"], name]
                        self.assertEqual(field["source_id"], binding["source_id"])
                        self.assertEqual(field["source_span"]["raw_source_excerpt"], binding["raw_source_excerpt"])
                        self.assertEqual(field["source_span"]["line_1_based"], binding["line_1_based"])
                        self.assertGreater(binding["revid"], 0)
                        self.assertIn("oldid=" + str(binding["revid"]), binding["fixed_url"])
                        self.assertTrue(field["modification"])



if __name__ == "__main__":
    unittest.main()
