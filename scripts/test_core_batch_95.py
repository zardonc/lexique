"""Frozen resource-handoff membership and four-field checks for issue #95."""

import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
RESOURCE = ROOT / "resources/core-pilot-79"
EVIDENCE = ROOT / "docs/evidence/issue-95"


class Batch95Tests(unittest.TestCase):
    def test_every_frozen_group_has_its_own_selected_use(self):
        with (RESOURCE / "scope-manifest.csv").open(encoding="utf-8-sig", newline="") as handle:
            scope = list(csv.DictReader(handle))[700:750]
        path = EVIDENCE / "groups.json"
        self.assertTrue(path.is_file(), "Issue #95 content handoff is missing")
        groups = json.loads(path.read_text(encoding="utf-8"))
        self.assertEqual([group["frozen_row"] for group in groups], list(range(701, 751)))
        for group, row in zip(groups, scope, strict=True):
            with self.subTest(row=group["frozen_row"]):
                self.assertEqual((group["headword"], group["lexique_cgram"]),
                                 (row["word"], row["lexique_cgram"]))
                self.assertTrue(group["senses"])
                for sense in group["senses"]:
                    self.assertEqual(set(sense["fields"]),
                                     {"french_definition", "ipa", "english", "chinese"})
                    for field in sense["fields"].values():
                        self.assertTrue(field["text"].strip())
                        self.assertTrue(field["source_id"])
                        self.assertGreater(field["source_span"]["line_1_based"], 0)
                        self.assertTrue(field["source_span"]["raw_source_excerpt"])


if __name__ == "__main__":
    unittest.main()
