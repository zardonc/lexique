"""Resource-boundary membership check for the owner's amended issue-98 scope."""

import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
RESOURCE = ROOT / "resources/core-pilot-79"
EVIDENCE = ROOT / "docs/evidence/issue-98"


class Batch98Tests(unittest.TestCase):
    def test_exact_frozen_membership_has_sourced_content_or_approved_annotations(self):
        with (RESOURCE / "scope-manifest.csv").open(encoding="utf-8-sig", newline="") as handle:
            scope = list(csv.DictReader(handle))
        self.assertEqual(len(scope), 4087)

        def records(name):
            path = EVIDENCE / name
            return json.loads(path.read_text(encoding="utf-8")) if path.exists() else []

        candidates = records("groups.json")
        annotations = records("annotations.json")
        approval = records("owner-approval.json")
        self.assertEqual(approval["approved_by"], "project_owner")
        self.assertEqual({item["frozen_row"] for item in approval["annotations"]}, {859, 860})
        groups = {group["frozen_row"]: group for group in candidates + annotations}
        self.assertEqual(set(groups), set(range(851, 901)))
        self.assertEqual(len(groups), len(candidates) + len(annotations))
        sources = {source["id"]: source for source in records("sources.json")}
        excerpts = {(field["sense_id"], field["field"]): field
                    for field in records("source-excerpts.json")}
        for row, group in groups.items():
            with self.subTest(row=row):
                self.assertEqual((group["headword"], group["lexique_cgram"]),
                                 (scope[row - 1]["word"], scope[row - 1]["lexique_cgram"]))
                self.assertEqual(group["subtitle_lemma_per_million"],
                                 scope[row - 1]["subtitle_lemma_per_million"])
                self.assertEqual(group["subtitle_pos_per_million"],
                                 scope[row - 1]["subtitle_pos_per_million"])
                if row in (859, 860):
                    self.assertEqual(group["senses"], [])
                    self.assertEqual(group["status"], "approved_annotation_exception")
                    self.assertEqual(group["admitted_fields"], 0)
                    self.assertEqual(group["missing_fields"], 4)
                    approved = next(item for item in approval["annotations"] if item["frozen_row"] == row)
                    self.assertEqual((approved["issue"], approved["headword"], approved["pos"], approved["admitted_fields"]),
                                     (98, group["headword"], group["lexique_cgram"], 0))
                    self.assertTrue(group["evidence"])
                    continue
                self.assertTrue(group["senses"])
                for sense in group["senses"]:
                    self.assertTrue(sense["use"].strip())
                    self.assertEqual(set(sense["fields"]),
                                     {"french_definition", "ipa", "english", "chinese"})
                    for name, field in sense["fields"].items():
                        self.assertTrue(field["text"].strip())
                        self.assertEqual(field["language"],
                                         {"french_definition": "fr", "ipa": "und",
                                          "english": "en", "chinese": "zh"}[name])
                        source = sources[field["source_id"]]
                        self.assertGreater(source["revid"], 0)
                        self.assertIn(f"oldid={source['revid']}", source["fixed_url"])
                        self.assertTrue(source["attribution"])
                        excerpt = excerpts[sense["sense_id"], name]
                        self.assertEqual(excerpt["revid"], source["revid"])
                        self.assertEqual(excerpt["fixed_url"], source["fixed_url"])
                        self.assertEqual(excerpt["line_1_based"], field["source_span"]["line_1_based"])
                        self.assertEqual(excerpt["raw_source_excerpt"],
                                         field["source_span"]["raw_source_excerpt"])


if __name__ == "__main__":
    unittest.main()
