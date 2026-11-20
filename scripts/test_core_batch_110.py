"""Resource-boundary check of #110 independent outcomes and fixed captures."""

import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-110"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch110Tests(unittest.TestCase):
    def test_selected_chinese_support_and_symbol_system(self):
        groups = {group["frozen_row"]: group for group in read("groups.json")}
        expected = {
            1489: "把某物挂或固定在钩子、钉子或衣帽架上。",
            1495: "符号系统；编码：用于交换信息的一套符号体系。",
        }
        for row, text in expected.items():
            with self.subTest(row=row):
                field = groups[row]["senses"][0]["fields"]["chinese"]
                self.assertEqual(field["text"], text)

    def test_selected_rights_and_attribution_have_independent_evidence(self):
        rights = read("rights.json")
        sources = read("sources.json")["sources"]
        contexts = read("source-context.json")["current_content"]
        self.assertEqual(len(sources), 51)
        self.assertEqual(rights["independent_review"], "passed_at_selected_text_boundary")
        self.assertTrue(rights["independent_reviewer"])
        self.assertIs(rights["independence"], True)
        self.assertEqual(rights["unresolved_selected_fields"], [])
        self.assertEqual(rights["final_combined_package_status"],
                         "pending_parent_conversion_notices_and_native_gates")
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
                                 "passed_at_selected_text_boundary")
                boundary = context["independent_selected_boundary_review"]
                self.assertEqual((boundary["source_id"], boundary["revid"]),
                                 (source["id"], source["revid"]))
                self.assertIs(boundary["independence"], True)
                self.assertTrue(boundary["selected_lines"])
                self.assertTrue(boundary["reason"])
                self.assertEqual(boundary["selected_sentence_authorship"], "not_proved")
                self.assertEqual(source["selected_text_review"], "passed_for_selected_spans")
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
        candidates = {group["frozen_row"]: group for group in read("review-candidates.json")}
        self.assertEqual(list(candidates), list(range(1451, 1501)))
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual(admission["independently_accepted_new_fields"], 200)
        self.assertEqual(admission["independently_accepted_new_groups"], 50)
        self.assertEqual(admission["resource_admission"],
                         "passed_combined_conversion_rights_notices_and_native_gates")
        self.assertEqual(admission["unresolved_field_blockers"], [])
        self.assertEqual(admission["canonical_newly_admitted_fields"], 200)
        self.assertEqual(admission["missing_admitted_fields"], 0)
        self.assertEqual(admission["conversion_and_packaging"], "passed_core_119_v1_and_verified_portable_package")
        self.assertEqual(admission["native_checks"],
                         "passed_windows_11_x64_exact_batch_lookup_trusted_save_reopen_and_upgrade")
        sources = {source["id"]: source for source in read("sources.json")["sources"]}
        contexts = {item["id"]: item for item in read("source-context.json")["current_content"]}
        bindings = {(item["sense_id"], item["field"]): item
                    for item in read("source-excerpts.json")["fields"]}
        reviews = read("field-review.json")
        self.assertEqual(reviews["kind"], "separate_independent_field_review")
        self.assertIs(reviews["independence"], True)
        self.assertIs(reviews["independent_acceptance"], True)
        self.assertEqual(reviews["reviewer"], admission["independent_reviewer"])
        self.assertEqual(len(reviews["fields"]), 200)
        self.assertEqual(len(bindings), 200)
        proposed = {(item["sense_id"], item["field"]): item for item in reviews["fields"]}
        self.assertEqual(set(proposed), set(bindings))
        self.assertEqual(len(reviews["usage"]), 50)
        self.assertEqual(len(reviews["spelling_relations"]), 50)
        self.assertEqual({(item["frozen_row"], item["field"])
                          for item in reviews["accepted_replacements"]},
                         {(1489, "chinese"), (1495, "chinese")})
        for relation in reviews["spelling_relations"]:
            self.assertEqual(relation["alias_admission"], "not_admitted")
            self.assertEqual(relation["independent_review"], "passed_for_source_binding")
            self.assertEqual(relation["new_runtime_aliases"], [])
            self.assertIs(relation["independence"], True)
            self.assertTrue(relation["proposed_evidence"])
            self.assertTrue(relation["review_reason"])
        captures = {item["title"]: item for item in read("source-captures.json")["sources"]}
        capture_root = Path(read("source-captures.json")["capture_root"])
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
                                 "passed_after_exact_accepted_replacements")
                self.assertIsNone(sense["cefr"])
                self.assertTrue(sense["use"].strip())
                self.assertEqual(set(sense["fields"]), set(LANGUAGES))
                original = candidates[row]
                self.assertEqual({key: value for key, value in group.items() if key != "senses"},
                                 {key: value for key, value in original.items() if key != "senses"})
                usage = next(item for item in reviews["usage"] if item["sense_id"] == sense["sense_id"])
                self.assertEqual(usage["proposed_usage"], original["senses"][0]["use"])
                self.assertEqual(usage["accepted_usage"], sense["use"])
                self.assertEqual(usage["accepted_usage_quality_status"], "passed")
                self.assertEqual(usage["everyday_sense_status"], "passed")
                self.assertTrue(usage["everyday_sense_reason"])
                self.assertIs(usage["independence"], True)
                for name, field in sense["fields"].items():
                    binding = bindings[sense["sense_id"], name]
                    review = proposed[sense["sense_id"], name]
                    source = sources[field["source_id"]]
                    context = contexts[source["id"]]
                    self.assertTrue(field["text"].strip())
                    self.assertEqual(review["accepted_text"], field["text"])
                    self.assertEqual(review["proposed_text"], original["senses"][0]["fields"][name]["text"])
                    if (row, name) not in {(1489, "chinese"), (1495, "chinese")}:
                        self.assertEqual(review["accepted_text"], review["proposed_text"])
                    self.assertEqual((review["frozen_row"], review["headword"], review["pos"]),
                                     (row, group["headword"], group["lexique_cgram"]))
                    self.assertEqual(review["exact_source"],
                                     {key: value for key, value in binding.items()
                                      if key not in ("sense_id", "field")})
                    self.assertEqual(review["accepted_text_quality_status"], "passed")
                    self.assertEqual(review["rights_eligibility_status"], "passed_at_selected_text_boundary")
                    self.assertIs(review["independence"], True)
                    self.assertTrue(review["review_reason"])
                    self.assertTrue(review["selected_text_rights_reason"])
                    self.assertEqual(review["reviewed_context"]["revid"], source["revid"])
                    self.assertEqual(review["reviewer"], admission["independent_reviewer"])
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
                    self.assertTrue(context["contributor_capture_parts"])
                    self.assertTrue(context["selected_locations"])
                    capture = captures[source["title"]]
                    raw_path = capture_root / "sources" / Path(capture["raw_path"].replace("\\", "/")).name
                    # Detached/CI checkouts retain bindings; supplied author captures are checked locally.
                    if raw_path.exists():
                        lines = raw_path.read_text(encoding="utf-8").splitlines()
                        self.assertEqual(lines[binding["line_1_based"] - 1], binding["reviewed_excerpt"])
        self.assertEqual(read("reuse.json")["reused_groups"], [])


if __name__ == "__main__":
    unittest.main()
