"""Resource-boundary check of #113 authored candidates and fixed captures."""

import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-113"
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


class Batch113Tests(unittest.TestCase):
    def test_exact_frozen_groups_have_four_bound_candidate_fields(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            scope = list(csv.DictReader(handle))
        groups = read("groups.json")
        self.assertEqual([group["frozen_row"] for group in groups], list(range(1601, 1651)))
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

    def test_author_rights_and_construction_evidence_is_not_independent_admission(self):
        rights = read("rights.json")
        self.assertEqual(rights["status"], "author_proposal_pending_independent_selected_span_review")
        self.assertFalse(rights["independent_acceptance"])
        self.assertIsNone(rights["reviewer"])
        self.assertEqual(rights["adaptation_license"], "CC-BY-SA-4.0")
        self.assertTrue((EVIDENCE / rights["complete_legal_text"]).resolve().is_file())
        self.assertTrue(rights["redistribution_obligations"])
        self.assertTrue(rights["authority_captures"])
        self.assertEqual({item["title"] for item in rights["unselected_import_boundaries"]},
                         {"objet", "valeur"})
        self.assertTrue(all(item["status"] == "excluded_from_selected_fields"
                            for item in rights["unselected_import_boundaries"]))
        groups = {group["frozen_row"]: group for group in read("groups.json")}
        self.assertEqual(groups[1610]["headword"], "mlle")
        title = groups[1610]["senses"][0]
        self.assertEqual(title["source_headword"], "Mlle")
        self.assertEqual(title["fields"]["ipa"]["text"], "/mad.mwa.zɛl/")
        for language, words in {"french_definition": ("Mlle", "Mlles", "Québec", "adresse"),
                                "english": ("Mlle", "Mlles", "Quebec", "address"),
                                "chinese": ("Mlle", "Mlles", "魁北克", "称呼")}.items():
            for word in words:
                self.assertIn(word, title["fields"][language]["text"])
        self.assertEqual(groups[1633]["senses"][0]["fields"]["ipa"]["text"], "/mɛ/")
        relation = next(item for item in read("field-review.json")["spelling_relations"]
                        if item["frozen_row"] == 1610)
        self.assertEqual(relation["alias_admission"], "not_admitted")
        self.assertEqual(len(relation["corroboration"]), 2)


if __name__ == "__main__":
    unittest.main()
