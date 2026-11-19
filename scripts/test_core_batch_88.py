"""Resource-boundary checks for the exact issue #88 content handoff."""

import csv
import json
from pathlib import Path
import re
import shutil
import subprocess
import tempfile
import unittest

from core_pilot import prepare

ROOT = Path(__file__).resolve().parents[1]
RESOURCE = ROOT / "resources/core-pilot-79"


class Batch88Tests(unittest.TestCase):
    def test_every_frozen_group_has_its_own_selected_use(self):
        with (RESOURCE / "scope-manifest.csv").open(encoding="utf-8-sig", newline="") as handle:
            scope = list(csv.DictReader(handle))
        curated = json.loads((RESOURCE / "curated-inputs.json").read_text(encoding="utf-8"))
        self.assertEqual(sum(351 <= group["frozen_row"] <= 400 for group in curated["groups"]), 50)
        groups = {group["frozen_row"]: group for group in curated["groups"]}
        for row in range(351, 401):
            with self.subTest(row=row):
                self.assertTrue(row in groups, f"Frozen row {row} has no selected use")
                group = groups[row]
                self.assertEqual((group["headword"], group["lexique_cgram"]),
                                 (scope[row - 1]["word"], scope[row - 1]["lexique_cgram"]))
                self.assertTrue(group["senses"])
                for sense in group["senses"]:
                    self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                    self.assertTrue(all(field["text"].strip() for field in sense["fields"].values()))

    def test_pending_fields_cannot_be_prepared_as_admitted_content(self):
        with tempfile.TemporaryDirectory() as temporary:
            fixture = Path(temporary) / "inputs"
            shutil.copytree(RESOURCE, fixture)
            path = fixture / "curated-inputs.json"
            curated = json.loads(path.read_text(encoding="utf-8"))
            source = next(source for source in curated["sources"] if source["id"].startswith("b88_"))
            source["selected_text_review"] = "pending"
            path.write_text(json.dumps(curated, ensure_ascii=False), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "Unreviewed field source"):
                prepare(Path(temporary) / "resource", source_dir=fixture)

    def test_preexisting_content_and_metadata_are_preserved(self):
        base = "42c720556fd1195cca39ba704761877d1c0df48a"
        for name in ("curated-inputs.json", "field-review.json", "selected-source-context.json", "selected-source-excerpts.json"):
            with self.subTest(file=name):
                original = json.loads(subprocess.check_output(
                    ["git", "show", f"{base}:resources/core-pilot-79/{name}"], cwd=ROOT))
                current = json.loads((RESOURCE / name).read_text(encoding="utf-8"))
                for key, value in original.items():
                    if isinstance(value, list) and key != "material_limits":
                        # New records may be inserted; every prior record remains exact.
                        retained = [record for record in current[key] if record in value]
                        self.assertEqual(retained, value, f"Prior {name}/{key} records changed")
        original_text = subprocess.check_output(
            ["git", "show", f"{base}:resources/core-pilot-79/curated-inputs.json"], cwd=ROOT).decode("utf-8")
        current_text = (RESOURCE / "curated-inputs.json").read_text(encoding="utf-8")
        for row in (360,):
            records = []
            for document in (original_text, current_text):
                start = re.search(r'(?m)^    \{\n      "frozen_row": ' + str(row) + ',', document).start() + 4
                _, end = json.JSONDecoder().raw_decode(document, start)
                records.append(document[start:end])
            self.assertEqual(*records, f"Reused row {row} was reformatted or changed")


if __name__ == "__main__":
    unittest.main()
