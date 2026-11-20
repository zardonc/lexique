"""Check #119 at the parent-approved resource/admission boundary."""
import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-119"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch119Tests(unittest.TestCase):
    def test_exact_frozen_slice_preserves_accepted_cote_and_proposes_196_new_fields(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            scope = list(csv.DictReader(handle))
        groups = read("groups.json")
        self.assertEqual([group["frozen_row"] for group in groups], list(range(1901, 1951)))
        for group in groups:
            frozen = scope[group["frozen_row"] - 1]
            self.assertEqual((group["headword"], group["lexique_cgram"]),
                             (frozen["word"], frozen["lexique_cgram"]))
            for frequency in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                self.assertEqual(group[frequency], frozen[frequency])
            self.assertEqual(len(group["senses"]), 1)
            self.assertEqual(set(group["senses"][0]["fields"]), set(LANGUAGES))
        curated = json.loads((ROOT / "resources/core-pilot-79/curated-inputs.json").read_text(encoding="utf-8"))
        accepted = next(group for group in curated["groups"] if group["frozen_row"] == 1911)
        self.assertEqual(groups[10], accepted)
        reuse = read("reuse.json")
        self.assertEqual([group["frozen_row"] for group in reuse["groups"]], [1911])
        self.assertEqual(reuse["groups"][0]["sense_id"], "cote-noun-rib")
        self.assertEqual(reuse["groups"][0]["field_identifiers"],
                         {name: {"sense_id": "cote-noun-rib", "field": name, "source_id": "fr_cote"}
                          for name in LANGUAGES})
        admission = read("admission.json")
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (49, 196))
        self.assertEqual((admission["reused_groups"], admission["reused_fields"]), (1, 4))
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual(admission["pending_independent_review_rows"],
                         [row for row in range(1901, 1951) if row != 1911])
        self.assertEqual(admission["independently_accepted_new_fields"], 0)
        self.assertIsNone(admission["independent_reviewer"])

    def test_new_field_sources_rights_and_quality_handoff_are_complete_without_reused_copies(self):
        groups = [group for group in read("groups.json") if group["frozen_row"] != 1911]
        sources = {source["id"]: source for source in read("sources.json")["sources"]}
        contexts = {source["id"]: source for source in read("source-context.json")["current_content"]}
        captures = {source["source_id"]: source for source in read("source-captures.json")["sources"]}
        bindings = {(item["sense_id"], item["field"]): item for item in read("source-excerpts.json")["fields"]}
        review = read("field-review.json")
        reviews = {(item["sense_id"], item["field"]): item for item in review["fields"]}
        rights = read("rights.json")
        boundaries = {(item["sense_id"], item["field"]): item for item in rights["fields"]}
        self.assertEqual(len(bindings), 196)
        self.assertEqual(set(bindings), set(reviews))
        self.assertEqual(set(bindings), set(boundaries))
        self.assertFalse(review["independent_acceptance"])
        self.assertIsNone(review["reviewer"])
        self.assertEqual(rights["independent_rights_fields_accepted"], 0)
        self.assertIsNone(rights["reviewer"])
        self.assertEqual(rights["independent_review"], "pending")
        self.assertEqual({item["source_id"] for item in rights["sources"]}, set(sources))
        self.assertEqual((EVIDENCE / "COPYING").read_text(encoding="utf-8").rstrip(),
                         (ROOT / rights["complete_legal_text"]).read_text(encoding="utf-8").rstrip())
        self.assertTrue(rights["authority_captures"])
        self.assertTrue(rights["redistribution_obligations"])
        self.assertTrue(read("source-captures.json")["contributor_registry"]["pagination_complete"])
        self.assertEqual(len(review["usage"]), 49)
        self.assertEqual(len(review["spelling_relations"]), 49)
        for group in groups:
            sense = group["senses"][0]
            with self.subTest(row=group["frozen_row"]):
                self.assertEqual(sense["independent_review"], "pending")
                self.assertEqual(sense["resource_admission"], "pending_separate_independent_review")
                self.assertIsNone(sense["cefr"])
                self.assertTrue(sense["use"].strip())
                for name, field in sense["fields"].items():
                    key = sense["sense_id"], name
                    binding, proposed, boundary = bindings[key], reviews[key], boundaries[key]
                    source = sources[field["source_id"]]
                    context = contexts[source["id"]]
                    self.assertTrue(field["text"].strip())
                    self.assertEqual(field["language"], LANGUAGES[name])
                    self.assertEqual(field["content_license"], "CC-BY-SA-4.0")
                    self.assertTrue(field["modification"])
                    self.assertEqual(field["independent_review"], "pending")
                    self.assertEqual(proposed["proposed_text"], field["text"])
                    self.assertEqual((proposed["frozen_row"], proposed["headword"], proposed["pos"]),
                                     (group["frozen_row"], group["headword"], group["lexique_cgram"]))
                    self.assertEqual(proposed["exact_source"],
                                     {key: value for key, value in binding.items() if key not in ("sense_id", "field")})
                    self.assertEqual(proposed["accepted_text_quality_status"], "pending")
                    self.assertIsNone(proposed["reviewer"])
                    self.assertEqual(binding["source_id"], field["source_id"])
                    self.assertEqual(binding["line_1_based"], field["source_span"]["line_1_based"])
                    self.assertEqual(binding["reviewed_excerpt"], field["source_span"]["raw_source_excerpt"])
                    self.assertEqual(binding["revid"], source["revid"])
                    self.assertEqual(binding["url"], source["fixed_url"])
                    self.assertIn("oldid=" + str(source["revid"]), source["fixed_url"])
                    self.assertEqual(context["current_content_revid"], source["revid"])
                    self.assertTrue(context["current_contributors"])
                    self.assertEqual(context["current_registry_names"], len(context["current_contributors"]))
                    self.assertTrue(context["contributor_capture_parts"])
                    self.assertEqual(boundary["exact_source_span"], field["source_span"])
                    self.assertEqual(boundary["source_id"], field["source_id"])
                    self.assertTrue(boundary["attribution"])
                    self.assertTrue(boundary["author_rights_basis"])
                    self.assertEqual(boundary["independent_review"], "pending")
                    self.assertIsNone(boundary["reviewer"])
                    if name == "ipa":
                        self.assertEqual(field["text"], "/" + binding["ipa_variant_excerpt"] + "/")
                        column = binding["ipa_variant_column_1_based"] - 1
                        self.assertEqual(binding["reviewed_excerpt"][column:column + len(binding["ipa_variant_excerpt"])],
                                         binding["ipa_variant_excerpt"])
                    raw_path = ROOT / captures[source["id"]]["raw_path"]
                    if raw_path.exists():
                        self.assertEqual(raw_path.read_text(encoding="utf-8").splitlines()[binding["line_1_based"] - 1],
                                         binding["reviewed_excerpt"])
        for item in review["fields"] + review["usage"] + review["spelling_relations"]:
            self.assertNotEqual(item["frozen_row"], 1911)
            self.assertNotEqual(item["sense_id"], "cote-noun-rib")
        for relation in review["spelling_relations"]:
            self.assertEqual(relation["relation_role"], "source_binding_only")
            self.assertEqual(relation["alias_admission"], "not_admitted")
            self.assertTrue(relation["proposed_evidence"])
        for usage in review["usage"]:
            self.assertTrue(usage["construction_and_flexion_evidence"])
            self.assertTrue(usage["author_construction_and_flexion_interpretation"])
            self.assertIsNone(usage["reviewer"])
        relations = {item["frozen_row"]: item for item in review["spelling_relations"]}
        self.assertEqual(relations[1908]["source_headword"], "continu")
        self.assertEqual(relations[1910]["source_headword"], "euh")
        self.assertEqual(relations[1930]["source_headword"], "celui")
        self.assertEqual(relations[1934]["source_headword"], "vieux")
        self.assertEqual(relations[1936]["source_headword"], "le vôtre")
        spelling = rights["heu_spelling_boundary"]
        self.assertEqual(spelling["frozen_row"], 1910)
        self.assertEqual(spelling["exact_source_span"]["raw_source_excerpt"], "# {{alternative spelling of|fr|euh}}")
        self.assertTrue(spelling["fixed_url"].startswith("https://en.wiktionary.org/"))
        self.assertIn(spelling["source_id"], sources)
        self.assertEqual(read("admission.json")["pending_independent_review_fields"], 196)
        self.assertEqual(read("admission.json")["newly_admitted_fields"], 0)


if __name__ == "__main__":
    unittest.main()
