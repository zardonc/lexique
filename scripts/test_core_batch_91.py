"""Frozen membership check for the issue #91 content handoff, before admission."""

import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
RESOURCE = ROOT / "resources/core-pilot-79"
EVIDENCE = ROOT / "docs/evidence/issue-91"


class Batch91Tests(unittest.TestCase):
    def test_every_frozen_group_has_its_own_four_field_selected_use(self):
        with (RESOURCE / "scope-manifest.csv").open(encoding="utf-8-sig", newline="") as handle:
            scope = list(csv.DictReader(handle))
        self.assertEqual(len(scope), 4087)
        def additions(name):
            path = EVIDENCE / name
            return json.loads(path.read_text(encoding="utf-8")) if path.exists() else []

        records = additions("groups.json")
        groups = {group["frozen_row"]: group for group in records}
        self.assertEqual(set(groups), set(range(501, 551)))
        self.assertEqual(len(groups), len(records), "Duplicate frozen group")
        sources = {source["id"]: source for source in additions("sources.json")}
        bindings = {(field["sense_id"], field["field"]): field for field in additions("source-excerpts.json")}
        for row in range(501, 551):
            with self.subTest(row=row):
                self.assertTrue(row in groups, f"Missing frozen row {row}")
                group = groups[row]
                self.assertEqual((group["headword"], group["lexique_cgram"]),
                                 (scope[row - 1]["word"], scope[row - 1]["lexique_cgram"]))
                self.assertEqual(group["subtitle_lemma_per_million"], scope[row - 1]["subtitle_lemma_per_million"])
                self.assertEqual(group["subtitle_pos_per_million"], scope[row - 1]["subtitle_pos_per_million"])
                self.assertTrue(group["senses"])
                for sense in group["senses"]:
                    self.assertTrue(sense["use"].strip())
                    self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                    for name, field in sense["fields"].items():
                        self.assertTrue(field["text"].strip())
                        self.assertEqual(field["language"], {"french_definition": "fr", "ipa": "und",
                                                            "english": "en", "chinese": "zh"}[name])
                        source = sources[field["source_id"]]
                        binding = bindings[sense["sense_id"], name]
                        self.assertEqual(binding["revid"], source["revid"])
                        self.assertEqual(binding["fixed_url"], source["fixed_url"])
                        self.assertEqual(binding["line_1_based"], field["source_span"]["line_1_based"])
                        self.assertEqual(binding["raw_source_excerpt"], field["source_span"]["raw_source_excerpt"])


if __name__ == "__main__":
    unittest.main()
