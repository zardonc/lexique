"""Check batch #87 at the frozen-input and converter admission boundaries."""

import csv
import json
from pathlib import Path
import shutil
import tempfile
import unittest

import core_pilot


class CoreBatch87Tests(unittest.TestCase):
    def test_pending_source_review_cannot_publish_a_dictionary_resource(self):
        with tempfile.TemporaryDirectory() as temporary:
            inputs = Path(temporary) / "inputs"
            shutil.copytree(core_pilot.RESOURCE, inputs)
            curated = core_pilot.read_json(inputs / "curated-inputs.json")
            next(source for source in curated["sources"] if source["id"].startswith("b87_"))["selected_text_review"] = "pending"
            (inputs / "curated-inputs.json").write_text(json.dumps(curated, ensure_ascii=False), encoding="utf-8")
            destination = Path(temporary) / "core"
            with self.assertRaisesRegex(ValueError, "Unreviewed field source or incompatible licence"):
                core_pilot.prepare(destination, source_dir=inputs)
            self.assertFalse((destination / "core.sqlite3").exists())

    def test_admitted_fields_and_owner_annotation_keep_distinct_counts(self):
        curated = core_pilot.read_json(core_pilot.RESOURCE / "curated-inputs.json")
        batch = [group for group in curated["groups"] if 301 <= group["frozen_row"] <= 350]
        selected = [sense for group in batch for sense in group["senses"]]
        self.assertEqual(len(selected), 49)
        self.assertEqual(sum(len(sense["fields"]) for sense in selected), 196)
        for sense in selected:
            self.assertEqual(set(sense["fields"]), set(core_pilot.FIELDS))
            self.assertEqual(sense["independent_review"], "passed_after_exact_accepted_replacements")
            for field in sense["fields"].values():
                self.assertTrue(field["text"].strip())
                self.assertEqual(field["independent_review"], "passed")
                self.assertTrue(field["source_span"]["raw_source_excerpt"].strip())
        with tempfile.TemporaryDirectory() as temporary:
            report = core_pilot.prepare(Path(temporary) / "core")
            batch_report = next(row for row in report["content_batches"] if row["issue"] == 87)
            gap = next(group for group in batch_report["groups"] if group["frozen_row"] == 305)
            self.assertEqual(gap["complete_fields"], 0)
            self.assertEqual(gap["incomplete_fields"], list(core_pilot.FIELDS))
            self.assertEqual(gap["annotation_exception_decision"]["issue"], "https://github.com/zardonc/lexique/issues/87")

    def test_listen_fields_keep_the_selected_sound_attention_use(self):
        curated = core_pilot.read_json(core_pilot.RESOURCE / "curated-inputs.json")
        group = next(group for group in curated["groups"] if group["frozen_row"] == 301)
        self.assertEqual(len(group["senses"]), 1)
        fields = group["senses"][0]["fields"]
        self.assertEqual(fields["french_definition"]["text"], "Faire attention, prêter l’oreille pour entendre.")
        self.assertEqual(fields["ipa"]["text"], "/e.ku.te/")
        self.assertEqual(fields["english"]["text"], "Listen; pay attention and use one's ears to hear.")
        self.assertEqual(fields["chinese"]["text"], "听、倾听；集中注意力，用耳朵去听。")

    def test_batch_retains_every_frozen_word_and_pos_without_substitution(self):
        with (core_pilot.RESOURCE / "scope-manifest.csv").open(encoding="utf-8-sig", newline="") as handle:
            expected = list(csv.DictReader(handle))[300:350]
        curated = core_pilot.read_json(core_pilot.RESOURCE / "curated-inputs.json")
        batch = [group for group in curated["groups"] if 301 <= group["frozen_row"] <= 350]
        self.assertEqual([group["frozen_row"] for group in batch], [*range(301, 305), *range(306, 351)])
        self.assertEqual(
            [(group["headword"], group["lexique_cgram"], group["subtitle_lemma_per_million"], group["subtitle_pos_per_million"]) for group in batch],
            [tuple(row.values()) for index, row in enumerate(expected, 301) if index != 305],
        )


if __name__ == "__main__":
    unittest.main()
