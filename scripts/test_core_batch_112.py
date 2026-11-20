"""Resource-boundary check of #112 content, independent decisions and bindings."""

import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-112"
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


class Batch112Tests(unittest.TestCase):
    def test_exact_frozen_groups_have_four_bound_candidate_fields(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            scope = list(csv.DictReader(handle))
        groups = read("groups.json")
        self.assertEqual([group["frozen_row"] for group in groups], list(range(1551, 1601)))
        admission = read("admission.json")
        self.assertTrue(admission["independent_reviewer"])
        reviewed = admission["independent_reviewer"] is not None
        status = "passed" if reviewed else "pending"
        if reviewed:
            self.assertEqual(content(groups), content(read("review-candidates.json")))
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual(admission["independently_accepted_new_fields"], 200 if reviewed else 0)
        self.assertEqual(admission["resource_admission"],
                         "passed_combined_conversion_rights_notices_and_native_gates")
        sources = {source["id"]: source for source in read("sources.json")["sources"]}
        contexts = {item["id"]: item for item in read("source-context.json")["current_content"]}
        bindings = {(item["sense_id"], item["field"]): item
                    for item in read("source-excerpts.json")["fields"]}
        reviews = read("field-review.json")
        self.assertEqual(reviews["independent_acceptance"], reviewed)
        self.assertEqual(reviews["independence"], reviewed)
        self.assertEqual(reviews["reviewer"], admission["independent_reviewer"])
        if reviewed:
            self.assertEqual(reviews["review_scope"]["groups_reviewed"], 50)
            self.assertEqual(reviews["review_scope"]["fields_reviewed"], 200)
            self.assertEqual(reviews["accepted_replacements"], [])
            self.assertEqual(admission["pending_independent_fields"], 0)
            self.assertEqual(admission["unresolved_exact_groups"], [])
            self.assertEqual(admission["approved_annotations"], [])
            self.assertEqual(admission["runtime_aliases_added"], [])
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
                if reviewed:
                    usage = next(item for item in reviews["usage"]
                                 if item["sense_id"] == sense["sense_id"])
                    self.assertEqual(usage["accepted_usage"], sense["use"])
                    self.assertEqual(usage["accepted_usage_quality_status"], "passed")
                    self.assertEqual(usage["everyday_sense_status"], "passed")
                    self.assertTrue(usage["review_reason"])
                    spelling = next(item for item in reviews["spelling_relations"]
                                    if item["sense_id"] == sense["sense_id"])
                    self.assertEqual(spelling["independent_review"], "passed_for_source_binding")
                    self.assertEqual(spelling["alias_admission"], "not_admitted")
                    self.assertEqual(spelling["new_runtime_aliases"], [])
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
                    if reviewed:
                        self.assertEqual(review["accepted_text"], review["proposed_text"])
                        self.assertTrue(review["independence"])
                        self.assertTrue(review["review_reason"])
                        self.assertTrue(review["selected_text_rights_reason"])
                        self.assertEqual(review["rights_eligibility_status"],
                                         "passed_at_selected_text_boundary")
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

    def test_selected_rights_and_constructions_have_explicit_author_evidence(self):
        rights = read("rights.json")
        admission = read("admission.json")
        reviewed = admission["independent_reviewer"] is not None
        self.assertEqual(rights["independent_acceptance"], reviewed)
        self.assertEqual(rights["reviewer"], admission["independent_reviewer"])
        if reviewed:
            self.assertTrue(rights["independence"])
            self.assertEqual(len(rights["selected_source_reviews"]), 46)
            self.assertEqual(rights["assembled_redistribution_validation"], "integration_owned_not_run")
        bindings = {(item["sense_id"], item["field"]): item
                    for item in read("source-excerpts.json")["fields"]}
        self.assertEqual(len(rights["fields"]), 200)
        self.assertEqual({(item["sense_id"], item["field"]) for item in rights["fields"]},
                         set(bindings))
        for item in rights["fields"]:
            with self.subTest(sense=item["sense_id"], field=item["field"]):
                binding = bindings[item["sense_id"], item["field"]]
                self.assertEqual(item["source_id"], binding["source_id"])
                self.assertEqual(item["source_revision"], binding["revid"])
                self.assertEqual(item["license"], "CC-BY-SA-4.0")
                self.assertEqual(item["independent_review"],
                                 "passed_at_selected_text_boundary" if reviewed else "pending")
                self.assertEqual(item["reviewer"], admission["independent_reviewer"])
                if reviewed:
                    self.assertTrue(item["review_reason"])
                self.assertTrue(item["author_rights_basis"])
                self.assertTrue(item["attribution"])
                self.assertTrue(item["modification"])
        self.assertTrue((EVIDENCE / rights["license_file"]).is_file())
        captures = read("source-captures.json")
        self.assertTrue(captures["contributor_registry"]["pagination_complete"])
        self.assertFalse(captures["contributor_registry"]["selected_sentence_authorship_proved"])
        contexts = read("source-context.json")["current_content"]
        self.assertEqual({item["title"] for item in contexts if item["excluded_import_layers"]},
                         {"rose", "source", "chasse"})
        if reviewed:
            self.assertEqual(sum(bool(item["import_notices"]) for item in contexts), 25)
            for item in contexts:
                self.assertEqual(item["selected_text_rights_status"], "passed_at_selected_text_boundary")
                self.assertTrue(item["independent_selected_rights_reason"])
        groups = {group["frozen_row"]: group for group in read("groups.json")}
        self.assertEqual(groups[1580]["headword"], "quelles")
        self.assertEqual(groups[1580]["lexique_cgram"], "ADJ:int")
        self.assertEqual(groups[1580]["senses"][0]["source_headword"], "quel")
        self.assertIn("feminine plural", groups[1580]["senses"][0]["use"])
        self.assertEqual(groups[1586]["lexique_cgram"], "ADV")
        self.assertIn("invariable", groups[1586]["senses"][0]["use"])
        self.assertIn("peser lourd", groups[1586]["senses"][0]["fields"]["french_definition"]["text"])
        self.assertEqual(read("reuse.json")["canonical_assigned_rows_checked"],
                         list(range(1551, 1601)))


if __name__ == "__main__":
    unittest.main()
