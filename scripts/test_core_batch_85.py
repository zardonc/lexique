"""Frozen #85 content and selected-source evidence at the resource boundary."""

import csv
import json
from pathlib import Path
import shutil
import tempfile
import unittest

import core_pilot

RESOURCE = Path(__file__).resolve().parents[1] / "resources/core-pilot-79"


def material(name):
    return json.loads((RESOURCE / name).read_text(encoding="utf-8"))


class Batch85ContentTests(unittest.TestCase):
    def test_all_fifty_frozen_groups_remain_present_with_exact_membership(self):
        with (RESOURCE / "scope-manifest.csv").open(encoding="utf-8-sig", newline="") as handle:
            scope = list(csv.DictReader(handle))
        groups = [g for g in material("curated-inputs.json")["groups"] if 201 <= g["frozen_row"] <= 250]
        self.assertEqual([g["frozen_row"] for g in groups], list(range(201, 251)))
        for group in groups:
            original = scope[group["frozen_row"] - 1]
            self.assertEqual((group["headword"], group["lexique_cgram"]),
                             (original["word"], original["lexique_cgram"]))
        self.assertEqual((groups[17]["headword"], groups[17]["lexique_cgram"]), ("avant", "ADJ"))
        self.assertEqual((groups[37]["headword"], groups[37]["lexique_cgram"]), ("déjà", "ONO"))

    def test_admitted_fields_retain_exact_spans_and_passed_quality(self):
        curated = material("curated-inputs.json")
        sources = {s["id"]: s for s in curated["sources"]}
        excerpts = {(e["sense_id"], e["field"]): e for e in material("selected-source-excerpts.json")["fields"]}
        reviews = {(r["sense_id"], r["field"]): r for r in material("field-review.json")["fields"]}
        groups = [g for g in curated["groups"] if 201 <= g["frozen_row"] <= 250]
        self.assertEqual([g["frozen_row"] for g in groups if not g["senses"]], [])
        for group in groups:
            for sense in group["senses"]:
                self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                self.assertEqual(sense["independent_review"], "passed_after_exact_accepted_replacements")
                for name, field in sense["fields"].items():
                    with self.subTest(row=group["frozen_row"], field=name):
                        self.assertTrue(field["text"].strip())
                        self.assertEqual(field["independent_review"], "passed")
                        source = sources[field["source_id"]]
                        excerpt = excerpts[sense["sense_id"], name]
                        self.assertGreater(source["revid"], 0)
                        self.assertIn(f"oldid={source['revid']}", source["fixed_url"])
                        self.assertEqual(excerpt["raw_source_excerpt"], field["source_span"]["raw_source_excerpt"])
                        self.assertEqual(excerpt["line_1_based"], field["source_span"]["line_1_based"])
                        self.assertEqual(reviews[sense["sense_id"], name]["accepted_text_quality_status"], "passed")

    def test_pending_source_review_cannot_produce_an_admitted_resource(self):
        with tempfile.TemporaryDirectory(prefix="lexique-batch85-") as temporary:
            inputs = Path(temporary) / "inputs"
            shutil.copytree(RESOURCE, inputs)
            curated = material("curated-inputs.json")
            next(source for source in curated["sources"] if source["id"].startswith("b85_"))["selected_text_review"] = "pending"
            (inputs / "curated-inputs.json").write_text(json.dumps(curated, ensure_ascii=False), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "Unreviewed field source"):
                core_pilot.prepare(Path(temporary) / "core", inputs, RESOURCE.parents[1] / "LICENSE")
            self.assertFalse((Path(temporary) / "core" / "core.sqlite3").exists())


if __name__ == "__main__":
    unittest.main()
