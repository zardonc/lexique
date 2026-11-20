"""Check #117's independently accepted content and fixed source bindings."""

import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-117"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch117Tests(unittest.TestCase):
    def test_exact_frozen_groups_have_four_independently_accepted_bound_fields(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            scope = list(csv.DictReader(handle))
        groups = read("groups.json")
        self.assertEqual([group["frozen_row"] for group in groups], list(range(1801, 1851)))
        admission = read("admission.json")
        self.assertTrue(admission["independent_reviewer"])
        self.assertTrue(admission["independence"])
        drafts = {group["frozen_row"]: group for group in read("review-candidates.json")}
        self.assertEqual(len(scope), 4087)
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual(admission["independently_accepted_new_fields"], 200)
        self.assertEqual(admission["independently_accepted_new_groups"], 50)
        self.assertEqual(admission["newly_admitted_fields"], 200)
        self.assertEqual(admission["canonical_pending_fields"], 0)
        self.assertEqual(admission["pending_independent_fields"], 0)
        self.assertEqual(admission["corrected_fields"], [])
        self.assertEqual(admission["resource_admission"],
                         "passed_combined_conversion_rights_notices_and_native_gates")
        sources = {source["id"]: source for source in read("sources.json")["sources"]}
        contexts = {item["id"]: item for item in read("source-context.json")["current_content"]}
        bindings = {(item["sense_id"], item["field"]): item
                    for item in read("source-excerpts.json")["fields"]}
        reviews = read("field-review.json")
        self.assertTrue(reviews["independent_acceptance"])
        self.assertTrue(reviews["independence"])
        self.assertEqual(reviews["unresolved_fields"], [])
        self.assertEqual(reviews["accepted_unchanged_fields"], 200)
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
                draft = drafts[row]["senses"][0]
                self.assertEqual(sense["independent_review"],
                                 "passed")
                self.assertEqual(draft["independent_review"], "pending")
                self.assertEqual(sense["use"], draft["use"])
                self.assertIsNone(sense["cefr"])
                self.assertTrue(sense["use"].strip())
                self.assertEqual(set(sense["fields"]), set(LANGUAGES))
                for name, field in sense["fields"].items():
                    binding = bindings[sense["sense_id"], name]
                    review = proposed[sense["sense_id"], name]
                    source = sources[field["source_id"]]
                    context = contexts[source["id"]]
                    self.assertTrue(field["text"].strip())
                    self.assertEqual(review["proposed_text"], field["text"])
                    self.assertEqual(review["accepted_text"], field["text"])
                    self.assertEqual(draft["fields"][name]["text"], field["text"])
                    self.assertEqual(draft["fields"][name]["independent_review"], "pending")
                    self.assertEqual((review["frozen_row"], review["headword"], review["pos"]),
                                     (row, group["headword"], group["lexique_cgram"]))
                    self.assertEqual(review["exact_source"],
                                     {key: value for key, value in binding.items()
                                      if key not in ("sense_id", "field")})
                    self.assertEqual(review["accepted_text_quality_status"], "passed")
                    self.assertEqual(review["rights_eligibility_status"], "passed_at_selected_text_boundary")
                    self.assertTrue(review["independence"])
                    self.assertTrue(review["review_reason"])
                    self.assertTrue(review["selected_text_rights_reason"])
                    self.assertEqual(review["reviewer"], admission["independent_reviewer"])
                    self.assertEqual(field["language"], LANGUAGES[name])
                    self.assertTrue(field["modification"])
                    self.assertEqual(field["independent_review"], "passed")
                    self.assertEqual(field["rights_eligibility_status"], "passed_at_selected_text_boundary")
                    self.assertEqual(field["content_license"], "CC-BY-SA-4.0")
                    self.assertEqual(field["source_id"], binding["source_id"])
                    self.assertEqual(field["source_span"]["line_1_based"], binding["line_1_based"])
                    self.assertEqual(field["source_span"]["raw_source_excerpt"], binding["reviewed_excerpt"])
                    self.assertEqual(source["revid"], binding["revid"])
                    self.assertEqual(source["fixed_url"], binding["url"])
                    self.assertIn("oldid=" + str(binding["revid"]), binding["url"])
                    self.assertTrue(context["contributor_capture_parts"])
                    self.assertEqual(context["current_content_revid"], source["revid"])
                    self.assertTrue(context["selected_locations"])
                    capture = captures[source["title"]]
                    raw_path = ROOT / capture["raw_path"]
                    # Raw captures are author evidence; detached/CI checkouts retain bindings.
                    if raw_path.exists():
                        lines = raw_path.read_text(encoding="utf-8").splitlines()
                        self.assertEqual(lines[binding["line_1_based"] - 1], binding["reviewed_excerpt"])
        self.assertEqual(read("reuse.json")["reused_groups"], [])

    def test_selected_rights_and_exact_flexions_preserve_independent_history_and_final_admission(self):
        rights = read("rights.json")
        self.assertEqual(rights["status"], "selected_span_rights_review_passed_packaging_pending")
        self.assertTrue(rights["independent_acceptance"])
        self.assertEqual(rights["reviewer"], read("admission.json")["independent_reviewer"])
        self.assertEqual(rights["final_combined_package_status"], "pending_parent_conversion_and_actual_notices")
        self.assertTrue((EVIDENCE / rights["complete_legal_text"]).resolve().is_file())
        self.assertTrue(rights["authority_captures"])
        self.assertTrue(rights["redistribution_obligations"])
        admission = read("admission.json")
        self.assertEqual((admission["missing_authored_fields"], admission["missing_admitted_fields"]), (0, 0))
        self.assertEqual(admission["approved_annotations"], [])
        self.assertEqual(admission["runtime_aliases_added"], [])
        groups = {item["frozen_row"]: item for item in read("groups.json")}
        relations = {item["frozen_row"]: item for item in read("field-review.json")["spelling_relations"]}
        for row, pos in ((1813, "PRO:dem"), (1822, "ADJ"), (1823, "NOM"),
                         (1824, "ADJ"), (1825, "NOM"), (1826, "ADJ"), (1827, "ADV"),
                         (1847, "ADJ:ind"), (1848, "PRO:ind")):
            self.assertEqual(groups[row]["lexique_cgram"], pos)
        for row, semantic_title, form, ipa in ((1813, "celui-ci", "celle-ci", "/sɛl.si/"),
                                             (1836, "professeur", "prof", "/pʁɔf/"),
                                             (1847, "tout", "toutes", "/tut/"),
                                             (1848, "tout", "toutes", "/tut/")):
            sense = groups[row]["senses"][0]
            self.assertEqual((groups[row]["headword"], sense["source_headword"]), (form, semantic_title))
            self.assertEqual(sense["fields"]["ipa"]["text"], ipa)
            self.assertEqual(relations[row]["alias_admission"], "not_admitted")
            self.assertTrue(relations[row]["proposed_binding_reason"])
            self.assertTrue(relations[row]["proposed_evidence"])
        for relation in relations.values():
            self.assertEqual(relation["relation_role"], "source_binding_only")
            self.assertEqual(relation["independent_review"], "passed_for_source_binding_only")
            self.assertTrue(relation["independence"])
            self.assertTrue(relation["review_reason"])
            self.assertEqual(relation["alias_admission"], "not_admitted")
        proposals = rights["selected_field_rights"]
        self.assertEqual(len(proposals), 200)
        self.assertEqual({(item["sense_id"], item["field"]) for item in proposals},
                         {(item["sense_id"], item["field"]) for item in read("field-review.json")["fields"]})
        for item in proposals:
            self.assertEqual(item["independent_review"], "passed_at_selected_text_boundary")
            self.assertTrue(item["independence"])
            self.assertTrue(item["independent_rights_reason"])
            self.assertTrue(item["fixed_source"])
            self.assertTrue(item["contributor_context"])
            self.assertTrue(item["adaptation_authors"])
            self.assertEqual(item["adaptation_license"], "CC-BY-SA-4.0")
        for item in read("field-review.json")["usage"]:
            self.assertEqual(item["accepted_usage_quality_status"], "passed")
            self.assertEqual(item["everyday_sense_status"], "passed")
            self.assertTrue(item["everyday_sense_reason"])
        contexts = read("source-context.json")["current_content"]
        self.assertEqual(len(contexts), 50)
        self.assertEqual(sum(bool(item["original_historical_dictionary_layers"]) for item in contexts), 38)
        for item in contexts:
            self.assertEqual(item["selected_text_rights_status"], "passed_at_selected_text_boundary")
            self.assertTrue(item["independent_rights_reason"])
        captures = {item["revid"]: item for item in read("source-captures.json")["sources"]}
        for item in read("source-excerpts.json")["supporting_evidence"]:
            raw_path = ROOT / captures[item["revid"]]["raw_path"]
            if raw_path.exists():
                line = raw_path.read_text(encoding="utf-8").splitlines()[item["line_1_based"] - 1]
                start = item["column_1_based"] - 1
                self.assertEqual(line[start:start + len(item["reviewed_excerpt"])], item["reviewed_excerpt"])


if __name__ == '__main__':
    unittest.main()
