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
        self.assertEqual([group["frozen_row"] for group in blocked], [1051])
        self.assertEqual(sum(len(group["senses"]) for group in groups), 49)
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
        self.assertEqual(len(bindings), 196)
        self.assertEqual(len(bindings), len(excerpts["fields"]))
        self.assertEqual(len(source_by_id), len(sources))
        reviewed = json.loads((EVIDENCE / "groups.json").read_text(encoding="utf-8"))
        content_keys = ("language", "text", "source_id", "source_span", "content_license", "modification")
        for draft, current in zip(candidates["groups"], reviewed, strict=True):
            self.assertEqual(draft["frozen_row"], current["frozen_row"])
            self.assertEqual(len(draft["senses"]), len(current["senses"]))
            for proposed, actual in zip(draft["senses"], current["senses"], strict=True):
                self.assertEqual(proposed["sense_id"], actual["sense_id"])
                self.assertEqual(proposed["use"], actual["use"])
                for name, field in proposed["fields"].items():
                    self.assertEqual({key: field[key] for key in content_keys},
                                     {key: actual["fields"][name][key] for key in content_keys})
        review = json.loads((EVIDENCE / "field-review.json").read_text(encoding="utf-8"))
        reviewed_fields = {(sense["sense_id"], name): field for group in reviewed
                           for sense in group["senses"] for name, field in sense["fields"].items()}
        self.assertEqual(len(review["fields"]), 192)
        for decision in review["fields"]:
            field = reviewed_fields[decision["sense_id"], decision["field"]]
            self.assertEqual(field["text"], decision["accepted_text"])
            self.assertEqual(field["source_id"], decision["source_id"])
            self.assertEqual(field["source_span"]["raw_source_excerpt"],
                             decision["exact_source"]["reviewed_excerpt"])
        admission = json.loads((EVIDENCE / "admission.json").read_text(encoding="utf-8"))
        self.assertEqual(admission["status"], "pending")
        self.assertTrue(admission["independence"])
        self.assertEqual((admission["accepted_groups"], admission["accepted_fields"]), (48, 192))
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
                        binding = bindings[sense["sense_id"], name]
                        self.assertEqual(field["source_id"], binding["source_id"])
                        self.assertEqual(field["source_span"]["raw_source_excerpt"], binding["raw_source_excerpt"])
                        self.assertEqual(field["source_span"]["line_1_based"], binding["line_1_based"])
                        self.assertEqual(field["source_span"]["column_1_based"], binding["column_1_based"])
                        self.assertGreater(binding["revid"], 0)
                        self.assertIn("oldid=" + str(binding["revid"]), binding["fixed_url"])
                        self.assertTrue(field["modification"])

    def test_mme_title_has_four_pending_fields_without_admitting_bare_s(self):
        groups = json.loads((EVIDENCE / "groups.json").read_text(encoding="utf-8"))
        blocked, mme = groups[0], groups[6]
        self.assertEqual((blocked["frozen_row"], blocked["headword"], blocked["lexique_cgram"]),
                         (1051, "s", "PRO:per"))
        self.assertEqual(blocked["senses"], [])
        self.assertEqual((mme["frozen_row"], mme["headword"], mme["lexique_cgram"]),
                         (1057, "mme", "NOM"))
        self.assertEqual((mme["admitted_fields"], mme["missing_fields"]), (0, 4))
        self.assertEqual(mme["independent_review"], "pending")
        self.assertEqual(len(mme["senses"]), 1)
        sense = mme["senses"][0]
        self.assertEqual(sense["independent_review"], "pending")
        self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
        self.assertEqual(sense["fields"]["ipa"]["text"], "/ma.dam/")
        for name, field in sense["fields"].items():
            self.assertEqual(field["source_id"], "b102_fr_193650")
            expected = "{{pron|ma.dam|fr}}" if name == "ipa" else "# [[Madame]] (comme [[titre]])."
            self.assertEqual(field["source_span"]["raw_source_excerpt"], expected)
            self.assertEqual(field["independent_review"], "pending")
            self.assertEqual(field["rights_eligibility_status"],
                             "pending_independent_selected_text_boundary_review")
            self.assertIsNone(field["reviewer"])
            self.assertFalse(field["independence"])
        admission = json.loads((EVIDENCE / "admission.json").read_text(encoding="utf-8"))
        self.assertEqual(admission["missing_admitted_fields"], 8)
        self.assertEqual(admission["runtime_aliases_added"], [])


if __name__ == "__main__":
    unittest.main()
