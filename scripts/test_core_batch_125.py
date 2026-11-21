"""Check #125 candidates at the approved resource/admission seam."""
import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-125"


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch125Tests(unittest.TestCase):
    def test_exact_frozen_groups_have_four_bound_candidate_fields(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(encoding="utf-8-sig") as handle:
            scope = list(csv.DictReader(handle))
        self.assertEqual(len(scope), 4087)
        groups = read("groups.json")
        self.assertEqual(len(groups), 50)
        for row, group in zip(range(2201, 2251), groups, strict=True):
            with self.subTest(row=row):
                frozen = scope[row - 1]
                self.assertEqual((group["frozen_row"], group["headword"], group["lexique_cgram"]),
                                 (row, frozen["word"], frozen["lexique_cgram"]))
                for key in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                    self.assertEqual(group[key], frozen[key])
                self.assertTrue(group["senses"])
                for sense in group["senses"]:
                    self.assertTrue(sense["use"])
                    self.assertIsNone(sense["cefr"])
                    self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                    for name, language in (("french_definition", "fr"), ("ipa", "und"),
                                           ("english", "en"), ("chinese", "zh")):
                        field = sense["fields"][name]
                        self.assertEqual(field["language"], language)
                        self.assertTrue(field["text"].strip())
                        self.assertTrue(field["source_id"])
                        self.assertTrue(field["source_span"]["raw_source_excerpt"])
        self.assertEqual(read("reuse.json")["groups"], [])

    def test_sources_rights_and_reviews_keep_author_proposals_pending(self):
        sources = {source["id"]: source for source in read("sources.json")["sources"]}
        contexts = {source["id"]: source for source in read("source-context.json")["current_content"]}
        rights = read("rights.json")
        review = read("field-review.json")
        admission = read("admission.json")
        self.assertEqual(len(sources), 51)
        self.assertFalse(review["independent_acceptance"])
        self.assertIsNone(review["reviewer"])
        self.assertIsNone(rights["reviewer"])
        self.assertEqual(admission["independently_accepted_new_fields"], 0)
        self.assertEqual(admission["proposed_fields"], 200)
        self.assertEqual(len(rights["fields"]), 200)
        self.assertEqual(len(review["fields"]), 200)
        self.assertEqual({(f["sense_id"], f["field"]) for f in rights["fields"]},
                         {(f["sense_id"], f["field"]) for f in review["fields"]})
        for field in rights["fields"]:
            with self.subTest(sense=field["sense_id"], field=field["field"]):
                source = sources[field["source_id"]]
                self.assertIn(str(source["revid"]), source["fixed_url"])
                self.assertTrue(source["history_url"])
                self.assertEqual(source["content_license"], "CC-BY-SA-4.0")
                context = contexts[source["id"]]
                self.assertEqual(context["current_registry_names"], len(context["current_contributors"]))
                self.assertTrue(context["contributor_capture_parts"])
                self.assertTrue(field["selected_rights_boundary"])
                self.assertEqual(field["independent_review"], "pending")
                self.assertIsNone(field["reviewer"])
        rang = next(source for source in sources.values() if source["title"] == "rang")
        self.assertIn("Émile Littré", {layer["original_authors"] for layer in rang["original_historical_dictionary_layers"]})
        for source in sources.values():
            for layer in source["original_historical_dictionary_layers"]:
                if layer["original_authors"] == "Académie française":
                    self.assertTrue(layer["conservative_expiry"].startswith("2006-01-01"))
        self.assertTrue((EVIDENCE / "COPYING").read_text(encoding="utf-8").strip())


if __name__ == "__main__":
    unittest.main()
