"""Resource-boundary check of #110 authored candidates and fixed captures."""

import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-110"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


def content(value):
    if isinstance(value, list):
        return [content(item) for item in value]
    if isinstance(value, dict):
        return {key: content(item) for key, item in value.items()
                if key not in {"independent_review", "resource_admission", "rights_eligibility_status"}}
    return value


class Batch110Tests(unittest.TestCase):
    def test_authored_rights_and_attribution_keep_independent_review_pending(self):
        rights = read("rights.json")
        sources = read("sources.json")["sources"]
        contexts = read("source-context.json")["current_content"]
        self.assertEqual(len(sources), 51)
        self.assertEqual(rights["independent_review"], "pending")
        self.assertIsNone(rights["independent_reviewer"])
        self.assertEqual(rights["proposed_adaptation_license"], "CC-BY-SA-4.0")
        self.assertTrue((EVIDENCE / rights["complete_legal_text"]).is_file())
        self.assertEqual({item["source_id"] for item in rights["source_assessments"]},
                         {source["id"] for source in sources})
        self.assertTrue(rights["redistribution_obligations"])
        self.assertEqual(len(rights["original_daf8_boundary_sources"]), 29)
        for source, context in zip(sources, contexts, strict=True):
            with self.subTest(source=source["title"]):
                self.assertEqual(source["id"], context["id"])
                self.assertTrue(source["attribution"])
                self.assertTrue(source["history_url"].endswith("&action=history"))
                self.assertGreater(context["current_registry_names"], 0)
                self.assertEqual(context["current_registry_names"],
                                 len(set(context["current_contributors"])))
                self.assertEqual(context["selected_text_rights_status"],
                                 "pending_independent_selected_text_and_import_boundary_audit")
        registry = read("source-captures.json")["contributor_registry"]
        self.assertTrue(registry["pagination_complete"])
        self.assertFalse(registry["selected_sentence_authorship_proved"])
        self.assertEqual(registry["names"], 3315)
        self.assertEqual(registry["titles"], 51)

    def test_exact_frozen_groups_have_four_bound_candidate_fields(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            scope = list(csv.DictReader(handle))
        groups = read("groups.json")
        self.assertEqual([group["frozen_row"] for group in groups], list(range(1451, 1501)))
        admission = read("admission.json")
        reviewed = admission["independent_reviewer"] is not None
        status = "passed" if reviewed else "pending"
        if reviewed:
            self.assertEqual(content(groups), content(read("review-candidates.json")))
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual(admission["independently_accepted_new_fields"], 200 if reviewed else 0)
        self.assertEqual(admission["resource_admission"],
                         "passed_independent_selected_content_and_rights_only" if reviewed
                         else "pending_separate_independent_review")
        sources = {source["id"]: source for source in read("sources.json")["sources"]}
        contexts = {item["id"]: item for item in read("source-context.json")["current_content"]}
        bindings = {(item["sense_id"], item["field"]): item
                    for item in read("source-excerpts.json")["fields"]}
        reviews = read("field-review.json")
        self.assertEqual(reviews["independent_acceptance"], reviewed)
        self.assertEqual(reviews["reviewer"], admission["independent_reviewer"])
        self.assertEqual(len(reviews["fields"]), 200)
        self.assertEqual(len(bindings), 200)
        proposed = {(item["sense_id"], item["field"]): item for item in reviews["fields"]}
        self.assertEqual(set(proposed), set(bindings))
        self.assertEqual(len(reviews["usage"]), 50)
        self.assertEqual(len(reviews["spelling_relations"]), 50)
        captures = {item["title"]: item for item in read("source-captures.json")["sources"]}
        for group in groups:
            row = group["frozen_row"]
            with self.subTest(row=row):
                frozen = scope[row - 1]
                self.assertEqual((group["headword"], group["lexique_cgram"]),
                                 (frozen["word"], frozen["lexique_cgram"]))
                for frequency in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                    self.assertEqual(group[frequency], frozen[frequency])
                self.assertEqual(len(group["senses"]), 1)
                sense = group["senses"][0]
                self.assertEqual(sense["independent_review"],
                                 "passed_after_exact_accepted_replacements" if reviewed else "pending")
                self.assertIsNone(sense["cefr"])
                self.assertTrue(sense["use"].strip())
                self.assertEqual(set(sense["fields"]), set(LANGUAGES))
                for name, field in sense["fields"].items():
                    binding = bindings[sense["sense_id"], name]
                    review = proposed[sense["sense_id"], name]
                    source = sources[field["source_id"]]
                    context = contexts[source["id"]]
                    self.assertTrue(field["text"].strip())
                    self.assertEqual(review["accepted_text" if reviewed else "proposed_text"], field["text"])
                    self.assertEqual((review["frozen_row"], review["headword"], review["pos"]),
                                     (row, group["headword"], group["lexique_cgram"]))
                    self.assertEqual(review["exact_source"],
                                     {key: value for key, value in binding.items()
                                      if key not in ("sense_id", "field")})
                    self.assertEqual(review["accepted_text_quality_status"], status)
                    self.assertEqual(review["reviewer"], admission["independent_reviewer"])
                    self.assertEqual(field["language"], LANGUAGES[name])
                    self.assertTrue(field["modification"])
                    self.assertEqual(field["independent_review"], status)
                    self.assertEqual(field["content_license"], "CC-BY-SA-4.0")
                    self.assertEqual(field["source_id"], binding["source_id"])
                    self.assertEqual(field["source_span"]["line_1_based"], binding["line_1_based"])
                    self.assertEqual(field["source_span"]["raw_source_excerpt"], binding["reviewed_excerpt"])
                    self.assertEqual(source["revid"], binding["revid"])
                    self.assertEqual(source["fixed_url"], binding["url"])
                    self.assertIn("oldid=" + str(binding["revid"]), binding["url"])
                    self.assertEqual(context["current_content_revid"], source["revid"])
                    self.assertTrue(context["contributor_capture_parts"])
                    self.assertTrue(context["selected_locations"])
                    capture = captures[source["title"]]
                    raw_path = ROOT / capture["raw_path"]
                    # Raw captures are author evidence; detached/CI checkouts retain bindings.
                    if raw_path.exists():
                        lines = raw_path.read_text(encoding="utf-8").splitlines()
                        self.assertEqual(lines[binding["line_1_based"] - 1], binding["reviewed_excerpt"])
        self.assertEqual(read("reuse.json")["reused_groups"], [])


if __name__ == "__main__":
    unittest.main()
