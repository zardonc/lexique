"""Content-boundary membership check for ticket #94's frozen groups."""

import csv
import json
from pathlib import Path
import unittest

RESOURCE = Path(__file__).resolve().parents[1] / "resources/core-pilot-79"
BATCH_GROUPS = RESOURCE.parents[1] / "docs/evidence/issue-94/groups.json"


class Batch94Tests(unittest.TestCase):
    def test_every_frozen_group_has_its_own_complete_selected_use(self):
        with (RESOURCE / "scope-manifest.csv").open(encoding="utf-8-sig", newline="") as handle:
            scope = list(csv.DictReader(handle))
        curated = json.loads((RESOURCE / "curated-inputs.json").read_text(encoding="utf-8"))
        additions = json.loads(BATCH_GROUPS.read_text(encoding="utf-8")) if BATCH_GROUPS.exists() else []
        self.assertEqual(len({group["frozen_row"] for group in additions}), len(additions),
                         "Duplicate frozen groups in the batch evidence")
        groups = {group["frozen_row"]: group for group in curated["groups"]}
        for group in additions:
            # Independently accepted replacements take precedence after central admission.
            groups.setdefault(group["frozen_row"], group)
        for row in range(651, 701):
            with self.subTest(row=row):
                self.assertTrue(row in groups, f"Missing frozen row {row}")
                group = groups[row]
                self.assertEqual((group["headword"], group["lexique_cgram"]),
                                 (scope[row - 1]["word"], scope[row - 1]["lexique_cgram"]))
                self.assertTrue(group["senses"])
                for sense in group["senses"]:
                    self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                    self.assertTrue(all(field["text"].strip() for field in sense["fields"].values()))


if __name__ == "__main__":
    unittest.main()
