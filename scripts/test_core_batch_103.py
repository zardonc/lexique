"""Frozen membership, original-source and meaningful-content checks for #103."""

import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-103"
RESOURCE = ROOT / "resources/core-pilot-79"


class Batch103Tests(unittest.TestCase):
    def assert_source_span(self, raw, binding):
        start_line = binding["line_1_based"] - 1
        end_line = binding.get("end_line_1_based", start_line + 1)
        selected = "\n".join(raw[binding["source_id"]][start_line:end_line])
        start_column = binding["column_1_based"] - 1
        excerpt = binding["raw_source_excerpt"]
        self.assertEqual(selected[start_column:start_column + len(excerpt)], excerpt)

    def test_exact_frozen_groups_have_four_fields_bound_to_original_sources(self):
        with (RESOURCE / "scope-manifest.csv").open(encoding="utf-8-sig", newline="") as handle:
            scope = list(csv.DictReader(handle))
        candidates = json.loads((EVIDENCE / "review-candidates.json").read_text(encoding="utf-8"))
        groups = candidates["groups"]
        self.assertEqual(candidates["scope_denominator_groups"], 4087)
        self.assertEqual([group["frozen_row"] for group in groups], list(range(1101, 1151)))
        excerpts = json.loads((EVIDENCE / "source-excerpts.json").read_text(encoding="utf-8"))
        bindings = {(record["sense_id"], record["field"]): record
                    for record in excerpts["fields"] + excerpts.get("candidate_fields", [])}
        sources = json.loads((EVIDENCE / "sources.json").read_text(encoding="utf-8"))
        captures = json.loads((EVIDENCE / "source-captures.json").read_text(encoding="utf-8"))["sources"]
        raw = {}
        for capture in captures:
            page = next(page for page in json.loads(Path(capture["api_path"]).read_text(encoding="utf-8"))["query"]["pages"]
                        if page["pageid"] == capture["pageid"])
            revision = next(rev for rev in page["revisions"] if rev["revid"] == capture["revid"])
            body = Path(capture["raw_path"]).read_text(encoding="utf-8")
            self.assertEqual(body, revision["slots"]["main"]["content"])
            self.assertEqual(revision["timestamp"], capture["timestamp"])
            self.assertEqual(page["title"], capture["title"])
            raw[capture["source_id"]] = body.splitlines()
        self.assertEqual(len(bindings), 200)
        self.assertTrue({source["id"] for source in sources}.issubset(raw))
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
                    self.assertEqual(field["source_id"], binding["source_id"])
                    self.assertEqual(field["source_span"], {key: binding[key] for key in ("line_1_based", "column_1_based", "raw_source_excerpt")})
                    self.assertGreater(binding["revid"], 0)
                    self.assertIn("oldid=" + str(binding["revid"]), binding["fixed_url"])
                    line = raw[binding["source_id"]][binding["line_1_based"] - 1]
                    start = binding["column_1_based"] - 1
                    self.assertEqual(line[start:start + len(binding["raw_source_excerpt"])], binding["raw_source_excerpt"])

    def test_reviewed_handoff_preserves_content_and_rejected_original(self):
        candidates = json.loads((EVIDENCE / "review-candidates.json").read_text(encoding="utf-8"))["groups"]
        replacement = json.loads((EVIDENCE / "replacement-1140.json").read_text(encoding="utf-8"))
        rejected = json.loads((EVIDENCE / "rejected-original-1140.json").read_text(encoding="utf-8"))
        groups = json.loads((EVIDENCE / "groups.json").read_text(encoding="utf-8"))
        review = json.loads((EVIDENCE / "field-review.json").read_text(encoding="utf-8"))
        admission = json.loads((EVIDENCE / "admission.json").read_text(encoding="utf-8"))
        captures = json.loads((EVIDENCE / "source-captures.json").read_text(encoding="utf-8"))["sources"]
        raw = {capture["source_id"]: Path(capture["raw_path"]).read_text(encoding="utf-8").splitlines()
               for capture in captures}
        self.assertTrue(review["independence"])
        self.assertEqual([group["frozen_row"] for group in groups], list(range(1101, 1151)))
        fields = {(record["sense_id"], record["field"]): record for record in review["fields"]}
        usage = {record["sense_id"]: record for record in review["usage"]}
        for group, candidate in zip(groups, candidates, strict=True):
            for key in ("frozen_row", "headword", "lexique_cgram", "subtitle_lemma_per_million",
                        "subtitle_pos_per_million"):
                self.assertEqual(group[key], candidate[key])
            if group["frozen_row"] == 1140:
                self.assertEqual(rejected["candidate_group"], candidate)
                self.assertEqual({record["field"] for record in rejected["field_decisions"]},
                                 {"french_definition", "ipa", "english", "chinese"})
                candidate = replacement["group"]
                for key in ("frozen_row", "headword", "lexique_cgram", "subtitle_lemma_per_million",
                            "subtitle_pos_per_million"):
                    self.assertEqual(group[key], candidate[key])
                self.assertEqual(replacement["pending_fields"], 4)
                self.assertEqual(replacement["admitted_fields"], 0)
                self.assertEqual(candidate["senses"][0]["independent_review"], "pending")
            sense, original = group["senses"][0], candidate["senses"][0]
            self.assertEqual(sense["sense_id"], original["sense_id"])
            self.assertEqual(sense["use"], original["use"])
            self.assertEqual(sense["excluded_uses"], original["excluded_uses"])
            self.assertEqual(usage[sense["sense_id"]]["accepted_usage"], sense["use"])
            self.assertEqual(sense["independent_review"], "passed_after_exact_accepted_replacements")
            for support in sense["supporting_evidence"]:
                self.assert_source_span(raw, support)
            for name, field in sense["fields"].items():
                for key in ("text", "language", "source_id", "source_span", "content_license", "modification"):
                    self.assertEqual(field[key], original["fields"][name][key])
                record = fields[sense["sense_id"], name]
                self.assertEqual(record["accepted_text"], field["text"])
                self.assertEqual(record["accepted_text_quality_status"], "passed")
                self.assertEqual(record["rights_eligibility_status"], "passed_at_selected_text_boundary")
                self.assertEqual(record["exact_source"]["reviewed_excerpt"], field["source_span"]["raw_source_excerpt"])
                self.assert_source_span(raw, {"source_id": field["source_id"], **field["source_span"]})
                if group["frozen_row"] == 1140:
                    self.assertEqual(original["fields"][name]["independent_review"], "pending")
                    if name != "ipa":
                        self.assertEqual(field["source_span"]["line_1_based"], 118)
                        self.assertEqual(field["source_span"]["end_line_1_based"], 119)
                        self.assertTrue(field["source_span"]["raw_source_excerpt"].endswith("vente et d’achat"))
                        self.assertNotIn(";", field["source_span"]["raw_source_excerpt"])
        self.assertEqual(len(fields), 200)
        self.assertEqual((admission["accepted_groups"], admission["accepted_fields"],
                          admission["missing_admitted_fields"]), (50, 200, 0))
        self.assertEqual(review["unresolved"], [])
        self.assertEqual(review["unresolved_fields"], [])
        self.assertEqual(admission["approved_annotation_exceptions"], [])
        self.assertEqual(admission["runtime_aliases_added"], [])


if __name__ == "__main__":
    unittest.main()
