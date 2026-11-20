"""Resource-boundary check of #115 independent review and fixed captures."""

import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-115"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch115Tests(unittest.TestCase):
    def test_exact_frozen_groups_have_four_bound_candidate_fields(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            scope = list(csv.DictReader(handle))
        groups = read("groups.json")
        self.assertEqual([group["frozen_row"] for group in groups], list(range(1701, 1751)))
        admission = read("admission.json")
        self.assertTrue(admission["independent_reviewer"])
        candidates = {group["frozen_row"]: group for group in read("review-candidates.json")}
        self.assertEqual(len(scope), 4087)
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual(admission["independently_accepted_new_fields"], 200)
        self.assertEqual(admission["resource_admission"],
                         "passed_combined_conversion_rights_notices_and_native_gates")
        sources = {source["id"]: source for source in read("sources.json")["sources"]}
        contexts = {item["id"]: item for item in read("source-context.json")["current_content"]}
        bindings = {(item["sense_id"], item["field"]): item
                    for item in read("source-excerpts.json")["fields"]}
        reviews = read("field-review.json")
        self.assertTrue(reviews["independent_acceptance"])
        self.assertTrue(reviews["independence"])
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
                candidate_sense = candidates[row]["senses"][0]
                for preserved in ("sense_id", "source_headword", "excluded_uses", "cefr"):
                    self.assertEqual(sense[preserved], candidate_sense[preserved])
                self.assertEqual(sense["independent_review"],
                                 "passed_after_exact_accepted_replacements")
                self.assertIsNone(sense["cefr"])
                self.assertTrue(sense["use"].strip())
                self.assertEqual(set(sense["fields"]), set(LANGUAGES))
                for name, field in sense["fields"].items():
                    binding = bindings[sense["sense_id"], name]
                    review = proposed[sense["sense_id"], name]
                    source = sources[field["source_id"]]
                    context = contexts[source["id"]]
                    self.assertTrue(field["text"].strip())
                    candidate = candidates[row]["senses"][0]["fields"][name]
                    self.assertEqual(review["proposed_text"], candidate["text"])
                    self.assertEqual(review["accepted_text"], field["text"])
                    self.assertEqual(candidate["independent_review"], "pending")
                    self.assertTrue(review["review_reason"])
                    self.assertTrue(review["selected_text_rights_reason"])
                    self.assertEqual((review["frozen_row"], review["headword"], review["pos"]),
                                     (row, group["headword"], group["lexique_cgram"]))
                    self.assertEqual(review["exact_source"],
                                     {key: value for key, value in binding.items()
                                      if key not in ("sense_id", "field")})
                    self.assertEqual(review["accepted_text_quality_status"], "passed")
                    self.assertEqual(review["rights_eligibility_status"], "passed_at_selected_text_boundary")
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
                    if source["source_frame"] == "fixed_publication_edition_capture":
                        self.assertEqual(source["edition"], "DAF8-1935-tome2-page659")
                        self.assertTrue(context["edition_contributors"])
                    else:
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

    def test_rights_and_nominal_constructions_have_separate_acceptance(self):
        rights = read("rights.json")
        self.assertEqual(rights["status"], "passed_independent_selected_span_rights_review")
        self.assertTrue(rights["independent_acceptance"])
        self.assertEqual(rights["reviewer"], read("field-review.json")["reviewer"])
        self.assertEqual(rights["adaptation_license"], "CC-BY-SA-4.0")
        self.assertTrue((EVIDENCE / rights["complete_legal_text"]).resolve().is_file())
        self.assertTrue(rights["authority_captures"])
        self.assertTrue(rights["redistribution_obligations"])
        self.assertEqual(len(rights["selected_source_reviews"]), 51)
        self.assertTrue(all(item["selected_text_rights_status"] == "passed_at_selected_text_boundary"
                            and item["reason"] for item in rights["selected_source_reviews"]))
        self.assertEqual([item["title"] for item in rights["unselected_import_boundaries"]], ["titre"])
        self.assertEqual(rights["unselected_import_boundaries"][0]["status"], "excluded_from_selected_fields")
        admission = read("admission.json")
        self.assertEqual((admission["missing_authored_fields"], admission["missing_admitted_fields"]), (0, 200))
        self.assertEqual(admission["approved_annotations"], [])
        self.assertEqual(admission["runtime_aliases_added"], [])
        self.assertEqual(admission["independently_accepted_new_groups"], 50)
        self.assertEqual((admission["canonical_newly_admitted_groups"],
                          admission["canonical_newly_admitted_fields"]), (0, 0))
        groups = {group["frozen_row"]: group for group in read("groups.json")}
        relations = {item["frozen_row"]: item for item in read("field-review.json")["spelling_relations"]}
        sources = {item["id"]: item for item in read("sources.json")["sources"]}
        for row in (1704, 1705, 1746, 1747):
            self.assertEqual(groups[row]["lexique_cgram"], "ADJ" if row in (1704, 1746) else "NOM")
        work = groups[1714]["senses"][0]
        self.assertEqual((groups[1714]["headword"], work["source_headword"]), ("oeuvre", "œuvre"))
        self.assertEqual(relations[1714]["alias_admission"], "not_admitted")
        self.assertTrue(any("contrainte typographique" in item["reviewed_excerpt"]
                            for item in relations[1714]["proposed_evidence"]))
        noun = groups[1750]["senses"][0]
        self.assertEqual(groups[1750]["lexique_cgram"], "NOM")
        self.assertEqual(noun["fields"]["ipa"]["text"], "/tjɛ̃/")
        semantic_source = sources[noun["fields"]["french_definition"]["source_id"]]
        self.assertEqual(semantic_source["edition"], "DAF8-1935-tome2-page659")
        self.assertTrue(any("nom masculin" in item["reviewed_excerpt"]
                            for item in relations[1750]["proposed_evidence"]))
        self.assertTrue(relations[1750]["modern_construction_corroboration"])
        self.assertEqual(relations[1750]["alias_admission"], "not_admitted")
        for language in ("french_definition", "english", "chinese"):
            self.assertIn("y mettre du tien", noun["fields"][language]["text"])
            self.assertIn("de + le", noun["fields"][language]["text"])
            self.assertIn("œuvre", work["fields"][language]["text"])
            bone = groups[1745]["senses"][0]["fields"][language]["text"]
            self.assertIn("/ɔs/", bone)
            self.assertIn("/o/", bone)
        self.assertEqual(groups[1745]["senses"][0]["fields"]["ipa"]["text"], "/ɔs/")
        for name, gender in (("french_definition", "féminin"),
                             ("english", "Feminine"), ("chinese", "阴性")):
            self.assertIn(gender, groups[1743]["senses"][0]["fields"][name]["text"])

    def test_accepted_meanings_retain_disagreement_and_title_prominence(self):
        groups = {group["frozen_row"]: group for group in read("groups.json")}
        for row, expected in {
            1729: {"french_definition": "désaccord", "english": "disagreement", "chinese": "异议"},
            1732: {"french_definition": "mis en valeur", "english": "visually distinguished", "chinese": "突出显示"},
        }.items():
            for language, phrase in expected.items():
                with self.subTest(row=row, language=language):
                    self.assertIn(phrase, groups[row]["senses"][0]["fields"][language]["text"])
        reviews = read("field-review.json")
        self.assertEqual(sum(item["proposed_text"] != item["accepted_text"]
                             for item in reviews["fields"]), 6)
        self.assertEqual(read("admission.json")["corrected_fields"], 6)
        self.assertEqual(reviews["unresolved_fields"], [])
        self.assertTrue(all(item["accepted_usage_quality_status"] == "passed"
                            and item["review_reason"] for item in reviews["usage"]))
        self.assertTrue(all(item["accepted_relation_status"] == "passed"
                            and item["alias_admission"] == "not_admitted"
                            for item in reviews["spelling_relations"]))




if __name__ == "__main__":
    unittest.main()
