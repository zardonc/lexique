"""Resource-boundary check of #118 authored candidates and fixed captures."""

import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-118"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch118Tests(unittest.TestCase):
    def test_exact_frozen_groups_have_four_bound_candidate_fields(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            scope = list(csv.DictReader(handle))
        groups = read("groups.json")
        self.assertEqual([group["frozen_row"] for group in groups], list(range(1851, 1901)))
        admission = read("admission.json")
        self.assertIsNone(admission["independent_reviewer"])
        self.assertEqual(groups, read("review-candidates.json"))
        self.assertEqual(len(scope), 4087)
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual(admission["independently_accepted_new_fields"], 0)
        self.assertEqual(admission["resource_admission"],
                         "pending_separate_independent_review")
        sources = {source["id"]: source for source in read("sources.json")["sources"]}
        contexts = {item["id"]: item for item in read("source-context.json")["current_content"]}
        bindings = {(item["sense_id"], item["field"]): item
                    for item in read("source-excerpts.json")["fields"]}
        reviews = read("field-review.json")
        self.assertFalse(reviews["independent_acceptance"])
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
                                 "pending")
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
                    self.assertEqual((review["frozen_row"], review["headword"], review["pos"]),
                                     (row, group["headword"], group["lexique_cgram"]))
                    self.assertEqual(review["exact_source"],
                                     {key: value for key, value in binding.items()
                                      if key not in ("sense_id", "field")})
                    self.assertEqual(review["accepted_text_quality_status"], "pending")
                    self.assertEqual(review["reviewer"], admission["independent_reviewer"])
                    self.assertEqual(field["language"], LANGUAGES[name])
                    self.assertTrue(field["modification"])
                    self.assertEqual(field["independent_review"], "pending")
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

    def test_rights_and_constructions_remain_a_complete_author_handoff(self):
        rights = read("rights.json")
        self.assertEqual(rights["status"], "author_proposal_pending_independent_selected_span_review")
        self.assertFalse(rights["independent_acceptance"])
        self.assertIsNone(rights["reviewer"])
        self.assertEqual(rights["adaptation_license"], "CC-BY-SA-4.0")
        self.assertTrue((EVIDENCE / rights["complete_legal_text"]).resolve().is_file())
        self.assertTrue(rights["authority_captures"])
        self.assertTrue(rights["redistribution_obligations"])
        proofs = {(item["sense_id"], item["field"]): item
                  for item in rights["selected_field_rights_proof"]}
        self.assertEqual(len(proofs), 200)
        sources = {item["id"]: item for item in read("sources.json")["sources"]}
        contexts = {item["id"]: item for item in read("source-context.json")["current_content"]}
        for group in read("groups.json"):
            sense = group["senses"][0]
            for name, field in sense["fields"].items():
                proof = proofs[sense["sense_id"], name]
                self.assertEqual(proof["source_id"], field["source_id"])
                self.assertEqual(proof["selected_span"], field["source_span"])
                self.assertEqual(proof["revid"], sources[field["source_id"]]["revid"])
                self.assertEqual(proof["independent_review"], "pending")
                self.assertTrue(proof["authority_fixed_url"])
                self.assertTrue(contexts[field["source_id"]]["current_contributors"])
                self.assertTrue(proof["excluded_content"])
                self.assertTrue(proof["supporting_source_rights"])
                for support in proof["supporting_source_rights"]:
                    self.assertEqual(support["revid"], sources[support["source_id"]]["revid"])
                    self.assertEqual(support["independent_review"], "pending")
        admission = read("admission.json")
        self.assertEqual((admission["missing_authored_fields"], admission["missing_admitted_fields"]), (0, 200))
        self.assertEqual(admission["approved_annotations"], [])
        self.assertEqual(admission["runtime_aliases_added"], [])
        self.assertEqual(admission["independently_accepted_new_groups"], 0)
        groups = {group["frozen_row"]: group for group in read("groups.json")}
        relations = {item["frozen_row"]: item for item in read("field-review.json")["spelling_relations"]}
        for relation in relations.values():
            self.assertEqual(relation["relation_role"], "source_binding_only")
            self.assertEqual(relation["alias_admission"], "not_admitted")
            self.assertEqual(relation["independent_review"], "pending")
        for row, word, pos, ipa in ((1870, "vive", "ONO", "/viv/"),
                                    (1884, "l'une", "PRO:ind", "/l‿yn/"),
                                    (1893, "lors", "ADV", "/lɔʁ/")):
            self.assertEqual((groups[row]["headword"], groups[row]["lexique_cgram"]), (word, pos))
            self.assertEqual(groups[row]["senses"][0]["fields"]["ipa"]["text"], ipa)
            self.assertTrue(relations[row]["proposed_binding_reason"])
            self.assertTrue(relations[row]["construction_evidence"])
        for language in ("french_definition", "english", "chinese"):
            self.assertIn("vivent", groups[1870]["senses"][0]["fields"][language]["text"])
            self.assertIn("l’une", groups[1884]["senses"][0]["fields"][language]["text"])
            self.assertIn("lors", groups[1893]["senses"][0]["fields"][language]["text"])
            self.assertIn("du", groups[1893]["senses"][0]["fields"][language]["text"])
            self.assertIn("des", groups[1893]["senses"][0]["fields"][language]["text"])
        for row, pos in ((1873, "ADJ"), (1874, "NOM"), (1878, "ADJ"), (1879, "NOM")):
            self.assertEqual(groups[row]["lexique_cgram"], pos)



if __name__ == "__main__":
    unittest.main()
