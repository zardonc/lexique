"""Resource-boundary check for the issue #100 reviewed handoff."""

import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-100"


class Batch100Tests(unittest.TestCase):
    def test_first_frozen_group_has_a_sourced_candidate(self):
        path = EVIDENCE / "review-candidates.json"
        self.assertTrue(path.is_file(), "Missing first candidate: row 951 connerie/NOM")
        group = json.loads(path.read_text(encoding="utf-8"))["groups"][0]
        self.assertEqual((group["frozen_row"], group["headword"], group["lexique_cgram"]),
                         (951, "connerie", "NOM"))
        self.assertEqual(set(group["senses"][0]["fields"]),
                         {"french_definition", "ipa", "english", "chinese"})

    def test_exact_frozen_membership_and_reviewed_content_preserve_candidates(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(encoding="utf-8-sig", newline="") as handle:
            scope = list(csv.DictReader(handle))
        candidates = json.loads((EVIDENCE / "review-candidates.json").read_text(encoding="utf-8"))
        groups = json.loads((EVIDENCE / "groups.json").read_text(encoding="utf-8"))
        originals = {group["frozen_row"]: group for group in candidates["groups"]}
        self.assertEqual([group["frozen_row"] for group in groups], list(range(951, 1001)))
        self.assertEqual(candidates["scope_denominator_groups"], 4087)
        excerpts = json.loads((EVIDENCE / "source-excerpts.json").read_text(encoding="utf-8"))["fields"]
        bindings = {(record["sense_id"], record["field"]): record for record in excerpts}
        sources = {source["id"]: source for source in json.loads((EVIDENCE / "sources.json").read_text(encoding="utf-8"))}
        contexts = {record["id"]: record for record in json.loads((EVIDENCE / "source-context.json").read_text(encoding="utf-8"))["current_content"]}
        review = json.loads((EVIDENCE / "field-review.json").read_text(encoding="utf-8"))
        self.assertTrue(review["independence"])
        self.assertTrue(review["independent_reviewer"])
        self.assertEqual(review["status"], "passed")
        reviews = {(record["sense_id"], record["field"]): record for record in review["fields"]}
        self.assertEqual(len(review["usage"]), 50)
        self.assertEqual(len(review["spelling_relations"]), 50)
        admission = json.loads((EVIDENCE / "admission.json").read_text(encoding="utf-8"))
        self.assertEqual(admission["frozen_rows"], list(range(951, 1001)))
        self.assertEqual((admission["accepted_groups"], admission["accepted_fields"], admission["missing_admitted_fields"]), (50, 200, 0))
        self.assertEqual(admission["annotation_exceptions"], [])
        self.assertEqual(len(bindings), 200)
        self.assertEqual(set(reviews), set(bindings))
        for group in groups:
            row = group["frozen_row"]
            with self.subTest(row=row):
                expected = scope[row - 1]
                self.assertEqual((group["headword"], group["lexique_cgram"]),
                                 (expected["word"], expected["lexique_cgram"]))
                for frequency in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                    self.assertEqual(group[frequency], expected[frequency])
                self.assertEqual((group["admitted_fields"], group["missing_fields"]), (4, 0))
                self.assertEqual(len(group["senses"]), 1)
                sense = group["senses"][0]
                original = originals[row]["senses"][0]
                for key in ("sense_id", "source_headword", "use", "use_language", "excluded_uses", "cefr", "spelling_relation", "source_context_bindings"):
                    self.assertEqual(sense[key], original[key])
                self.assertEqual(sense["independent_review"], "passed")
                self.assertEqual(sense["resource_admission"], "passed_independent_content_review")
                self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                for name, field in sense["fields"].items():
                    self.assertTrue(field["text"].strip())
                    for key in ("language", "text", "source_id", "source_span", "content_license", "modification"):
                        self.assertEqual(field[key], original["fields"][name][key])
                    self.assertEqual(field["independent_review"], "passed")
                    self.assertEqual(field["language"], {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}[name])
                    binding = bindings[sense["sense_id"], name]
                    self.assertEqual(field["source_id"], binding["source_id"])
                    self.assertEqual(field["source_span"]["raw_source_excerpt"], binding["raw_source_excerpt"])
                    self.assertEqual(field["source_span"]["line_1_based"], binding["line_1_based"])
                    self.assertEqual(field["source_span"]["column_1_based"], binding["column_1_based"])
                    source = sources[field["source_id"]]
                    self.assertEqual(binding["revid"], source["revid"])
                    self.assertIn("oldid=" + str(binding["revid"]), binding["fixed_url"])
                    self.assertTrue(source["attribution"])
                    self.assertTrue(source["redistribution_obligations"])
                    self.assertTrue(field["modification"])
                    reviewed = reviews[sense["sense_id"], name]
                    self.assertEqual(reviewed["selected_text"], field["text"])
                    self.assertEqual(reviewed["exact_source"]["reviewed_excerpt"], binding["raw_source_excerpt"])
                    self.assertEqual(reviewed["accepted_text"], field["text"])
                    self.assertEqual(reviewed["accepted_text_quality_status"], "passed")
                    self.assertEqual(reviewed["rights_eligibility_status"], "passed_at_selected_text_boundary")
                    self.assertTrue(reviewed["independence"])
                    self.assertTrue(reviewed["reviewer"])
                    context = contexts[field["source_id"]]
                    self.assertEqual(context["revid"], source["revid"])
                    self.assertIn(binding, context["selected_locations"])
                self.assertEqual(sense["spelling_relation"]["source_headword"], group["headword"])
                self.assertEqual(sense["spelling_relation"]["query_aliases"], [])

    def test_retained_original_api_snapshots_match_exact_field_locations(self):
        captures = json.loads((EVIDENCE / "source-captures.json").read_text(encoding="utf-8"))["sources"]
        if not all(Path(record["api_path"]).is_file() and Path(record["raw_path"]).is_file() for record in captures):
            self.skipTest("Original captures require the owning author evidence root from source-captures.json")
        originals = {}
        for record in captures:
            response = json.loads(Path(record["api_path"]).read_bytes())
            page = next(page for page in response["query"]["pages"] if page["pageid"] == record["pageid"])
            revision = next(rev for rev in page["revisions"] if rev["revid"] == record["revid"])
            original = Path(record["raw_path"]).read_text(encoding="utf-8")
            self.assertEqual(original, revision["slots"]["main"]["content"])
            self.assertEqual(record["timestamp"], revision["timestamp"])
            originals[f"b100_fr_{record['pageid']}"] = original.splitlines()
        excerpts = json.loads((EVIDENCE / "source-excerpts.json").read_text(encoding="utf-8"))["fields"]
        for binding in excerpts:
            with self.subTest(sense=binding["sense_id"], field=binding["field"]):
                line = originals[binding["source_id"]][binding["line_1_based"] - 1]
                column = binding["column_1_based"] - 1
                self.assertEqual(line[column:column + len(binding["raw_source_excerpt"])], binding["raw_source_excerpt"])


if __name__ == "__main__":
    unittest.main()
