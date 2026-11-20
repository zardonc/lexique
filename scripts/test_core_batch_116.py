"""Check #116's independently reviewed package at the resource/admission seam."""
import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-116"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch116Tests(unittest.TestCase):
    def test_nombre_retains_the_selected_unspecified_quantity(self):
        sense = next(group for group in read("groups.json")
                     if group["frozen_row"] == 1756)["senses"][0]
        expected = {
            "french_definition": "Quantité non précisée de personnes ou de choses.",
            "english": "An unspecified number or quantity of people or things.",
            "chinese": "若干，一定数量：未指明具体数目的人或事物。",
        }
        for name, text in expected.items():
            with self.subTest(field=name):
                self.assertEqual(sense["fields"][name]["text"], text)

    def test_exact_frozen_groups_have_complete_independent_review(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            scope = list(csv.DictReader(handle))
        groups = read("groups.json")
        candidates = {item["frozen_row"]: item for item in read("review-candidates.json")}
        self.assertEqual([item["frozen_row"] for item in groups], list(range(1751, 1801)))
        admission = read("admission.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertTrue(admission["independent_reviewer"])
        self.assertEqual(admission["independently_accepted_new_groups"], 50)
        self.assertEqual(admission["independently_accepted_new_fields"], 200)
        self.assertEqual(admission["newly_admitted_fields"], 0)
        self.assertEqual(admission["resource_admission"], "pending_parent_conversion_and_packaged_notices")
        sources = {item["id"]: item for item in read("sources.json")["sources"]}
        contexts = {item["id"]: item for item in read("source-context.json")["current_content"]}
        bindings = {(item["sense_id"], item["field"]): item
                    for item in read("source-excerpts.json")["fields"]}
        reviews = read("field-review.json")
        self.assertTrue(reviews["independent_acceptance"])
        self.assertTrue(reviews["independence"])
        self.assertEqual(reviews["reviewer"], admission["independent_reviewer"])
        self.assertEqual(reviews["unresolved_fields"], [])
        self.assertEqual({(item["frozen_row"], item["field"])
                          for item in reviews["accepted_replacements"]},
                         {(1756, "french_definition"), (1756, "english"), (1756, "chinese")})
        proposed = {(item["sense_id"], item["field"]): item for item in reviews["fields"]}
        self.assertEqual(len(proposed), 200)
        self.assertEqual(set(proposed), set(bindings))
        self.assertEqual(len(reviews["usage"]), 50)
        self.assertEqual(len(reviews["spelling_relations"]), 50)
        usage = {item["frozen_row"]: item for item in reviews["usage"]}
        relations = {item["frozen_row"]: item for item in reviews["spelling_relations"]}
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
                candidate = candidates[row]["senses"][0]
                self.assertEqual(sense["sense_id"], candidate["sense_id"])
                self.assertEqual(sense["excluded_uses"], candidate["excluded_uses"])
                self.assertEqual(sense["source_headword"], candidate["source_headword"])
                self.assertEqual(usage[row]["proposed_usage"], candidate["use"])
                self.assertEqual(usage[row]["accepted_usage"], sense["use"])
                self.assertEqual(usage[row]["accepted_usage_quality_status"], "passed")
                self.assertTrue(usage[row]["review_reason"])
                self.assertEqual(usage[row]["reviewer"], reviews["reviewer"])
                self.assertTrue(usage[row]["author_construction_and_flexion_interpretation"])
                self.assertTrue(usage[row]["construction_and_flexion_evidence"])
                self.assertEqual(relations[row]["relation_role"], "source_binding_only")
                self.assertEqual(relations[row]["alias_admission"], "not_admitted")
                self.assertEqual(relations[row]["independent_review"], "passed_for_source_binding_only")
                self.assertTrue(relations[row]["review_reason"])
                self.assertEqual(sense["independent_review"], "passed")
                self.assertIsNone(sense["cefr"])
                self.assertTrue(sense["use"].strip())
                self.assertEqual(set(sense["fields"]), set(LANGUAGES))
                for name, field in sense["fields"].items():
                    binding = bindings[sense["sense_id"], name]
                    review = proposed[sense["sense_id"], name]
                    source = sources[field["source_id"]]
                    context = contexts[source["id"]]
                    self.assertTrue(field["text"].strip())
                    self.assertEqual(review["proposed_text"], candidate["fields"][name]["text"])
                    self.assertEqual(review["accepted_text"], field["text"])
                    self.assertEqual(field["source_id"], candidate["fields"][name]["source_id"])
                    self.assertEqual(field["source_span"], candidate["fields"][name]["source_span"])
                    self.assertEqual(field["modification"], candidate["fields"][name]["modification"])
                    if row != 1756 or name == "ipa":
                        self.assertEqual(field["text"], candidate["fields"][name]["text"])
                    self.assertEqual((review["frozen_row"], review["headword"], review["pos"]),
                                     (row, group["headword"], group["lexique_cgram"]))
                    self.assertEqual(review["exact_source"],
                                     {key: value for key, value in binding.items()
                                      if key not in ("sense_id", "field")})
                    self.assertEqual(review["accepted_text_quality_status"], "passed")
                    self.assertEqual(review["reviewer"], reviews["reviewer"])
                    self.assertTrue(review["independence"])
                    self.assertTrue(review["review_reason"])
                    self.assertTrue(review["selected_text_rights_reason"])
                    self.assertEqual(review["rights_eligibility_status"], "passed_at_selected_text_boundary")
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
                    self.assertEqual(context["current_content_revid"], source["revid"])
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
                    # Detached checkouts retain tracked bindings; raw captures stay local.
                    if raw_path.exists():
                        lines = raw_path.read_text(encoding="utf-8").splitlines()
                        self.assertEqual(lines[binding["line_1_based"] - 1], binding["reviewed_excerpt"])
        self.assertTrue(read("source-captures.json")["contributor_registry"]["pagination_complete"])
        rights = read("rights.json")
        field_rights = {(item["sense_id"], item["field"]): item for item in rights["fields"]}
        self.assertEqual(set(field_rights), set(bindings))
        for group in groups:
            sense = group["senses"][0]
            for name, field in sense["fields"].items():
                boundary = field_rights[sense["sense_id"], name]
                self.assertEqual(boundary["source_id"], field["source_id"])
                self.assertEqual(boundary["exact_source_span"], field["source_span"])
                self.assertTrue(boundary["attribution"])
                self.assertTrue(boundary["author_rights_basis"])
                self.assertEqual(boundary["independent_review"], "passed_at_selected_text_boundary")
                self.assertEqual(boundary["reviewer"], reviews["reviewer"])
                self.assertTrue(boundary["selected_text_rights_reason"])
        self.assertIn(rights["selected_lexical_example_boundary"]["source_id"], sources)
        self.assertEqual(rights["independent_review"], "passed_at_selected_text_boundary")
        self.assertEqual(rights["reviewer"], reviews["reviewer"])
        self.assertEqual(rights["independent_rights_fields_accepted"], 200)
        self.assertEqual({item["source_id"] for item in rights["sources"]}, set(sources))
        self.assertTrue((ROOT / rights["complete_legal_text"]).is_file())
        self.assertEqual((EVIDENCE / "COPYING").read_text(encoding="utf-8").rstrip(),
                         (ROOT / rights["complete_legal_text"]).read_text(encoding="utf-8").rstrip())
        self.assertTrue(rights["authority_captures"])
        self.assertTrue(rights["redistribution_obligations"])
        for source in rights["sources"]:
            self.assertEqual(source["independent_review"], "passed_at_selected_text_boundary")
            self.assertTrue(source["selected_text_rights_reason"])
        self.assertEqual(relations[1751]["source_headword"], "le tien")
        self.assertEqual(relations[1782]["source_headword"], "œuf")
        self.assertIn("pronominal", usage[1793]["proposed_usage"])
        self.assertIn("masculine", usage[1784]["proposed_usage"])
        self.assertIn("feminine", usage[1789]["proposed_usage"].lower())
        self.assertIn("misses", usage[1798]["author_construction_and_flexion_interpretation"])
        self.assertEqual(admission["pending_independent_review_rows"], [])
        self.assertEqual(admission["pending_independent_review_fields"], 0)
        self.assertEqual(admission["pending_independent_rights_fields"], 0)
        self.assertEqual(read("reuse.json")["reused_groups"], [])


if __name__ == "__main__":
    unittest.main()
