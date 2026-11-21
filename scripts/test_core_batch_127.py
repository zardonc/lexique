"""Check #127 reviewed content/rights at the approved admission boundary."""
import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-127"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch127Tests(unittest.TestCase):
    def test_monseigneur_explanation_retains_the_selected_address_scope(self):
        group = next(group for group in read("groups.json") if group["frozen_row"] == 2322)
        self.assertEqual(group["senses"][0]["fields"]["english"]["text"],
                         "An honorific used when addressing certain people of high birth or dignity, especially bishops.")

    def test_exact_frozen_groups_have_four_independently_reviewed_fields(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(encoding="utf-8-sig") as handle:
            scope = list(csv.DictReader(handle))
        self.assertEqual(len(scope), 4087)
        groups = read("groups.json")
        self.assertEqual(len(groups), 50)
        for row, group in zip(range(2301, 2351), groups, strict=True):
            with self.subTest(row=row):
                frozen = scope[row - 1]
                self.assertEqual((group["frozen_row"], group["headword"], group["lexique_cgram"]),
                                 (row, frozen["word"], frozen["lexique_cgram"]))
                for key in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                    self.assertEqual(group[key], frozen[key])
                self.assertEqual(len(group["senses"]), 1)
                sense = group["senses"][0]
                self.assertTrue(sense["use"])
                self.assertIsNone(sense["cefr"])
                self.assertEqual(sense["independent_review"], "passed_after_exact_accepted_replacements")
                self.assertEqual(set(sense["fields"]), set(LANGUAGES))
                for name, field in sense["fields"].items():
                    self.assertEqual(field["language"], LANGUAGES[name])
                    self.assertTrue(field["text"].strip())
                    self.assertEqual(field["independent_review"], "passed")
                    self.assertEqual(field["rights_eligibility_status"], "passed_at_selected_text_boundary")
        admission = read("admission.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["accepted_groups"], admission["accepted_fields"]), (50, 200))
        self.assertEqual(admission["independently_accepted_new_fields"] + admission["reused_fields"], 200)
        self.assertEqual(admission["independently_accepted_new_groups"] + admission["reused_groups"], 50)
        for name in ("missing_fields", "pending_independent_review_fields", "pending_independent_rights_fields",
                     "annotation_exception_groups", "specialist_exception_groups"):
            self.assertEqual(admission[name], 0)
        self.assertEqual(admission["pending_independent_review_rows"], [])
        self.assertEqual(admission["unresolved_field_blockers"], [])
        for name in ("independent_quality", "selected_span_rights", "construction_and_spelling_review"):
            self.assertEqual(admission[name], "passed")

    def test_every_accepted_field_retains_its_proposal_and_selected_rights(self):
        groups = read("groups.json")
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        contexts = {s["id"]: s for s in read("source-context.json")["current_content"]}
        review, rights = read("field-review.json"), read("rights.json")
        reviews = {(f["sense_id"], f["field"]): f for f in review["fields"]}
        boundaries = {(f["sense_id"], f["field"]): f for f in rights["fields"]}
        excerpts = {(f["sense_id"], f["field"]): f for f in read("source-excerpts.json")["fields"]}
        proposals = {(s["sense_id"], name): field["text"] for g in read("review-candidates.json")
                     for s in g["senses"] for name, field in s["fields"].items()}
        self.assertEqual(len(reviews), 200)
        self.assertEqual(set(reviews), set(boundaries))
        self.assertEqual(set(reviews), set(excerpts))
        self.assertEqual(review["kind"], "separate_independent_field_review")
        self.assertIs(review["independence"], True)
        self.assertIs(review["independent_acceptance"], True)
        self.assertTrue(review["reviewer"])
        self.assertEqual(review["unresolved_fields"], [])
        self.assertEqual(rights["independent_review"], "passed_at_selected_text_boundary")
        self.assertEqual(rights["independent_rights_fields_accepted"], 200)
        self.assertTrue(rights["redistribution_obligations"])
        uses = {u["sense_id"]: u for u in review["usage"]}
        spellings = {s["frozen_row"]: s for s in review["spelling_relations"]}
        self.assertEqual((len(uses), len(spellings), len(sources)), (50, 50, 49))
        for group in groups:
            sense = group["senses"][0]
            use, spelling = uses[sense["sense_id"]], spellings[group["frozen_row"]]
            self.assertEqual(use["accepted_usage"], sense["use"])
            self.assertEqual(use["everyday_sense_status"], "passed")
            self.assertTrue(use["everyday_sense_reason"])
            self.assertEqual(spelling["accepted_spelling_quality_status"], "passed")
            self.assertEqual(spelling["alias_admission"], "not_admitted")
            for name, field in sense["fields"].items():
                with self.subTest(row=group["frozen_row"], field=name):
                    key = sense["sense_id"], name
                    decision, boundary = reviews[key], boundaries[key]
                    source, context = sources[field["source_id"]], contexts[field["source_id"]]
                    self.assertEqual(decision["proposed_text"], proposals[key])
                    self.assertEqual(decision["accepted_text"], field["text"])
                    self.assertEqual(decision["accepted_text_quality_status"], "passed")
                    self.assertIs(decision["independence"], True)
                    self.assertTrue(decision["review_reason"])
                    self.assertTrue(decision["selected_text_rights_reason"])
                    self.assertEqual(boundary["accepted_text"], field["text"])
                    self.assertEqual(boundary["exact_source_span"], field["source_span"])
                    self.assertEqual(boundary["independent_review"], "passed_at_selected_text_boundary")
                    self.assertEqual(boundary["reviewer"], review["reviewer"])
                    self.assertTrue(boundary["independent_rights_reason"])
                    self.assertEqual(source["selected_text_review"], "passed_at_selected_text_boundary")
                    self.assertEqual(context["selected_text_rights_status"], "passed_at_selected_text_boundary")
                    self.assertEqual(source["revid"], boundary["revid"])
                    self.assertEqual(context["current_registry_names"], len(context["current_contributors"]))
                    self.assertTrue(source["attribution"])
                    self.assertTrue(field["modification"])
                    self.assertEqual(field["content_license"], "CC-BY-SA-4.0")
                    self.assertEqual(excerpts[key]["reviewed_excerpt"], field["source_span"]["raw_source_excerpt"])
        replacements = {key for key, f in reviews.items() if f["proposed_text"] != f["accepted_text"]}
        self.assertEqual(replacements, {("b127-2322-monseigneur-nom-selected", "english")})
        copie = next(c for c in contexts.values() if c["title"] == "copie")
        self.assertIn((204, "* {{Import:Littré}}"),
                      {(s["line_1_based"], s["raw_marker"]) for s in copie["import_notices"]})
        for source in sources.values():
            for layer in source["original_historical_dictionary_layers"]:
                self.assertEqual(layer["independent_review"], "passed_at_original_work_boundary")
                if layer["original_authors"] == "Académie française":
                    self.assertTrue(layer["conservative_expiry"].startswith("2006-01-01"))
        self.assertEqual(read("admission-author-proposal.json")["independently_accepted_new_fields"], 0)
        self.assertFalse(read("authored-review-proposals.json")["field-review.json"]["independent_acceptance"])

    def test_existing_canonical_overlap_preserves_identical_reviewed_content(self):
        canonical = json.loads((ROOT / "resources/core-pilot-79/curated-inputs.json").read_text(encoding="utf-8"))
        indexed = {(g["headword"], g["lexique_cgram"]): g for g in canonical["groups"]}
        for group in read("groups.json"):
            existing = indexed.get((group["headword"], group["lexique_cgram"]))
            if existing is None:
                continue
            with self.subTest(row=group["frozen_row"]):
                for name in ("frozen_row", "headword", "lexique_cgram", "subtitle_lemma_per_million", "subtitle_pos_per_million"):
                    self.assertEqual(existing[name], group[name])
                self.assertEqual(len(existing["senses"]), len(group["senses"]))
                for actual, expected in zip(existing["senses"], group["senses"], strict=True):
                    for name in ("sense_id", "source_headword", "use", "excluded_uses", "cefr"):
                        self.assertEqual(actual[name], expected[name])
                    for name in LANGUAGES:
                        for key in ("language", "text", "source_id", "source_span", "content_license", "modification"):
                            self.assertEqual(actual["fields"][name][key], expected["fields"][name][key])


if __name__ == "__main__":
    unittest.main()
