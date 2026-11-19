"""Batch #84 at the frozen content and produced-resource boundaries."""

import csv
import json
from pathlib import Path
import shutil
import tempfile
import unittest

import core_pilot


ROOT = Path(__file__).resolve().parents[1]
RESOURCE = ROOT / "resources/core-pilot-79"


class CoreBatch84Tests(unittest.TestCase):
    def test_all_fifty_frozen_groups_have_four_scoped_candidate_fields(self):
        curated = json.loads((RESOURCE / "curated-inputs.json").read_text(encoding="utf-8"))
        with (RESOURCE / "scope-manifest.csv").open(encoding="utf-8-sig", newline="") as handle:
            scope = list(csv.DictReader(handle))
        groups = [group for group in curated["groups"] if 151 <= group["frozen_row"] <= 200]
        self.assertEqual([group["frozen_row"] for group in groups], list(range(151, 201)))
        for group in groups:
            original = scope[group["frozen_row"] - 1]
            self.assertEqual((group["headword"], group["lexique_cgram"]),
                             (original["word"], original["lexique_cgram"]))
            self.assertEqual(len(group["senses"]), 1)
            sense = group["senses"][0]
            self.assertTrue(sense["use"])
            self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
            for field in sense["fields"].values():
                self.assertTrue(field["text"].strip())
                self.assertTrue(field["source_span"]["raw_source_excerpt"].strip())
                self.assertGreater(field["source_span"]["line_1_based"], 0)

    def test_pending_author_draft_cannot_become_an_admitted_resource(self):
        with tempfile.TemporaryDirectory() as directory:
            inputs = Path(directory) / "inputs"
            shutil.copytree(RESOURCE, inputs)
            curated = json.loads((inputs / "curated-inputs.json").read_text(encoding="utf-8"))
            source = next(source for source in curated["sources"] if source["id"] == "b84_fr_2244")
            source["selected_text_review"] = "pending_independent_review"
            (inputs / "curated-inputs.json").write_text(json.dumps(curated, ensure_ascii=False), encoding="utf-8")
            destination = Path(directory) / "unreviewed-resource"
            with self.assertRaisesRegex(ValueError, "Unreviewed field source|not independently reviewed"):
                core_pilot.prepare(destination, source_dir=inputs)
            self.assertFalse(destination.exists())

    def test_difficult_uses_retain_their_selected_constructions(self):
        curated = json.loads((RESOURCE / "curated-inputs.json").read_text(encoding="utf-8"))
        senses = {group["frozen_row"]: group["senses"][0] for group in curated["groups"]
                  if 151 <= group["frozen_row"] <= 200}
        for row, word in [(151, "un merci"), (155, "l’un et l’autre"), (163, "sujet avant le verbe"),
                          (165, "uniquement au singulier"), (181, "un seul"), (186, "numéral")]:
            self.assertIn(word, senses[row]["fields"]["french_definition"]["text"])
        self.assertIn("approximately", senses[186]["fields"]["english"]["text"])
        self.assertEqual(senses[176]["fields"]["ipa"]["text"], "/dø/")


if __name__ == "__main__":
    unittest.main()
