"""Frozen membership and source bindings of the pending #97 review artifact."""

import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
BATCH = ROOT / "resources/core-pilot-79/batch-97"


class Batch97Tests(unittest.TestCase):
    def test_frozen_membership_and_selected_field_bindings(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
                encoding="utf-8-sig", newline="") as handle:
            scope = list(csv.DictReader(handle))
        groups = json.loads((BATCH / "groups.json").read_text(encoding="utf-8"))
        unresolved = json.loads((BATCH / "unresolved.json").read_text(encoding="utf-8"))
        records = {group["frozen_row"]: group for group in groups + unresolved}
        self.assertEqual(len(records), len(groups) + len(unresolved))
        self.assertEqual(set(records), set(range(801, 851)))
        excerpts = json.loads((BATCH / "source-excerpts.json").read_text(encoding="utf-8"))
        fields = {(item["sense_id"], item["field"]): item for item in excerpts}
        for row, group in records.items():
            with self.subTest(row=row):
                self.assertEqual(
                    (group["headword"], group["lexique_cgram"],
                     group["subtitle_lemma_per_million"], group["subtitle_pos_per_million"]),
                    tuple(scope[row - 1].values()))
                if group in unresolved:
                    self.assertEqual(group["senses"], [])
                    self.assertTrue(group["evidence"])
                    continue
                self.assertTrue(group["senses"])
                for sense in group["senses"]:
                    self.assertEqual(set(sense["fields"]),
                                     {"french_definition", "ipa", "english", "chinese"})
                    self.assertEqual(sense["independent_review"], "pending")
                    for name, field in sense["fields"].items():
                        self.assertTrue(field["text"].strip())
                        excerpt = fields[(sense["sense_id"], name)]
                        self.assertEqual(field["source_id"], excerpt["source_id"])
                        self.assertEqual(field["source_span"]["raw_source_excerpt"],
                                         excerpt["raw_source_excerpt"])
                        self.assertEqual(field["source_span"]["line_1_based"],
                                         excerpt["line_1_based"])
                        self.assertGreater(excerpt["revid"], 0)
                        self.assertIn(f"oldid={excerpt['revid']}", excerpt["fixed_url"])
                        self.assertEqual(field["independent_review"], "pending")


if __name__ == "__main__":
    unittest.main()
