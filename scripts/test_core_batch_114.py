"""Check #114's independently reviewed content at the resource/admission seam."""
import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-114"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch114Tests(unittest.TestCase):
    def test_selected_explanations_do_not_expand_authority_or_misread_police_rules(self):
        groups = {item['frozen_row']: item for item in read('groups.json')}
        powerful = groups[1667]['senses'][0]['fields']
        police = groups[1685]['senses'][0]['fields']
        self.assertNotIn('力量', powerful['chinese']['text'])
        self.assertIn('authority', powerful['english']['text'])
        self.assertNotIn('警察规章', police['chinese']['text'])
        self.assertIn('公共秩序', police['chinese']['text'])
        self.assertIn('public-order', police['english']['text'])

    def test_exact_frozen_groups_match_separate_accepted_field_and_source_decisions(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            scope = list(csv.DictReader(handle))
        groups = read("groups.json")
        self.assertEqual([item["frozen_row"] for item in groups], list(range(1651, 1701)))
        admission = read("admission.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertTrue(admission["independent_reviewer"])
        self.assertTrue(admission["independence"])
        self.assertEqual(admission["independently_accepted_new_fields"], 200)
        self.assertEqual(admission["independently_accepted_new_groups"], 50)
        self.assertEqual(admission["resource_admission"], "passed_combined_conversion_rights_notices_and_native_gates")
        self.assertEqual(admission["newly_admitted_fields"], 200)
        self.assertEqual(admission["approved_annotations"], [])
        self.assertEqual(admission["runtime_aliases_added"], [])
        sources = {item["id"]: item for item in read("sources.json")["sources"]}
        contexts = {item["id"]: item for item in read("source-context.json")["current_content"]}
        bindings = {(item["sense_id"], item["field"]): item
                    for item in read("source-excerpts.json")["fields"]}
        reviews = read("field-review.json")
        self.assertTrue(reviews["independent_acceptance"])
        self.assertTrue(reviews["independence"])
        self.assertEqual(reviews["kind"], "separate_independent_field_review")
        self.assertEqual(reviews["reviewer"], admission["independent_reviewer"])
        proposed = {(item["sense_id"], item["field"]): item for item in reviews["fields"]}
        self.assertEqual(len(proposed), 200)
        self.assertEqual(set(proposed), set(bindings))
        self.assertEqual(len(reviews["usage"]), 50)
        self.assertEqual(len(reviews["spelling_relations"]), 50)
        usages = {item["sense_id"]: item for item in reviews["usage"]}
        spellings = {item["sense_id"]: item for item in reviews["spelling_relations"]}
        drafts = {item["frozen_row"]: item for item in read("review-candidates.json")}
        self.assertEqual(list(drafts), list(range(1651, 1701)))
        changes = set()
        use_changes = set()
        raw_checked = set()
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
                self.assertEqual(sense["independent_review"], "passed_after_exact_accepted_replacements")
                self.assertIsNone(sense["cefr"])
                self.assertTrue(sense["use"].strip())
                draft = drafts[row]["senses"][0]
                self.assertEqual(draft["independent_review"], "pending")
                self.assertEqual(sense["sense_id"], draft["sense_id"])
                usage = usages[sense["sense_id"]]
                self.assertEqual(usage["proposed_usage"], draft["use"])
                self.assertEqual(usage["accepted_usage"], sense["use"])
                self.assertEqual(usage["accepted_usage_quality_status"], "passed")
                self.assertEqual(usage["everyday_sense_status"], "passed")
                self.assertTrue(usage["everyday_sense_reason"])
                spelling = spellings[sense["sense_id"]]
                self.assertEqual(spelling["alias_admission"], "not_admitted")
                self.assertEqual(spelling["relation_role"], "source_binding_only")
                self.assertEqual(spelling["independent_review"], "passed_for_source_binding_only")
                self.assertTrue(spelling["review_reason"])
                if sense["use"] != draft["use"]:
                    use_changes.add(row)
                self.assertEqual(set(sense["fields"]), set(LANGUAGES))
                for name, field in sense["fields"].items():
                    binding = bindings[sense["sense_id"], name]
                    review = proposed[sense["sense_id"], name]
                    source = sources[field["source_id"]]
                    context = contexts[source["id"]]
                    self.assertTrue(field["text"].strip())
                    self.assertEqual(review["proposed_text"], draft["fields"][name]["text"])
                    self.assertEqual(review["accepted_text"], field["text"])
                    if review["proposed_text"] != field["text"]:
                        changes.add((row, name))
                        self.assertTrue(review["replacement_reason"])
                    self.assertEqual((review["frozen_row"], review["headword"], review["pos"]),
                                     (row, group["headword"], group["lexique_cgram"]))
                    self.assertEqual(review["exact_source"],
                                     {key: value for key, value in binding.items()
                                      if key not in ("sense_id", "field")})
                    self.assertEqual(review["accepted_text_quality_status"], "passed")
                    self.assertEqual(review["rights_eligibility_status"], "passed_at_selected_text_boundary")
                    self.assertEqual(review["reviewer"], admission["independent_reviewer"])
                    self.assertTrue(review["independence"])
                    self.assertTrue(review["review_reason"])
                    self.assertTrue(review["selected_text_rights_reason"])
                    self.assertEqual(field["language"], LANGUAGES[name])
                    self.assertTrue(field["modification"])
                    self.assertEqual(field["independent_review"], "passed")
                    self.assertEqual(field["content_license"], "CC-BY-SA-4.0")
                    self.assertEqual(field["source_id"], binding["source_id"])
                    self.assertEqual(field["source_span"]["line_1_based"], binding["line_1_based"])
                    self.assertEqual(field["source_span"]["raw_source_excerpt"], binding["reviewed_excerpt"])
                    self.assertEqual(source["revid"], binding["revid"])
                    self.assertEqual(source["fixed_url"], binding["url"])
                    self.assertIn("oldid=" + str(binding["revid"]), binding["url"])
                    self.assertEqual(context["current_content_revid"], source["revid"])
                    self.assertEqual(source["selected_text_review"], "passed_for_selected_spans")
                    self.assertEqual(context["selected_text_rights_status"], "passed_at_selected_text_boundary")
                    self.assertTrue(context["contributor_capture_parts"])
                    self.assertTrue(context["current_contributors"])
                    self.assertEqual(context["current_registry_names"], len(context["current_contributors"]))
                    self.assertTrue(context["selected_locations"])
                    if name == "ipa":
                        self.assertEqual(field["text"], "/" + binding["ipa_variant_excerpt"] + "/")
                        column = binding["ipa_variant_column_1_based"] - 1
                        self.assertEqual(binding["reviewed_excerpt"][column:column + len(binding["ipa_variant_excerpt"])],
                                         binding["ipa_variant_excerpt"])
                    raw_path = ROOT / captures[source["title"]]["raw_path"]
                    author_root = Path(admission["author_evidence_root"]).parents[1]
                    if not raw_path.exists():
                        raw_path = author_root / captures[source["title"]]["raw_path"]
                    # Fixed captures remain with the author when this checkout is detached.
                    if raw_path.exists():
                        lines = raw_path.read_text(encoding="utf-8").splitlines()
                        self.assertEqual(lines[binding["line_1_based"] - 1], binding["reviewed_excerpt"])
                        raw_checked.add(source["id"])
                    elif author_root.exists():
                        self.fail(f"Missing supplied fixed capture: {raw_path}")
        self.assertEqual(changes, {(1667, "english"), (1667, "chinese"),
                                   (1685, "english"), (1685, "chinese")})
        self.assertEqual(use_changes, {1667})
        if Path(admission["author_evidence_root"]).exists():
            self.assertEqual(raw_checked, set(sources))
        self.assertTrue(read("source-captures.json")["contributor_registry"]["pagination_complete"])
        rights = read("rights.json")
        self.assertEqual(rights["independent_review"], "passed_at_selected_text_boundary")
        self.assertEqual(rights["reviewer"], admission["independent_reviewer"])
        self.assertEqual(rights["independent_rights_fields_accepted"], 200)
        self.assertEqual(rights["final_combined_package_status"], "pending_parent_conversion_and_actual_notices")
        self.assertEqual({item["source_id"] for item in rights["sources"]}, set(sources))
        self.assertTrue((ROOT / rights["complete_legal_text"]).is_file())
        self.assertTrue(rights["authority_captures"])
        self.assertTrue(rights["redistribution_obligations"])
        self.assertEqual(admission["pending_independent_review_rows"], [])
        self.assertEqual(reviews["unresolved_fields"], [])
        for source_rights in rights["sources"]:
            self.assertEqual(source_rights["independent_review"], "passed_at_selected_text_boundary")
            self.assertTrue(source_rights["independent_rights_reason"])
        self.assertEqual(read("reuse.json")["reused_groups"], [])


if __name__ == "__main__":
    unittest.main()
