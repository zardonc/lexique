"""Check batch #87 at the frozen-input and converter admission boundaries."""

import csv
from pathlib import Path
import tempfile
import unittest

import core_pilot


class CoreBatch87Tests(unittest.TestCase):
    def test_pending_source_review_cannot_publish_a_dictionary_resource(self):
        with tempfile.TemporaryDirectory() as temporary:
            destination = Path(temporary) / "core"
            with self.assertRaisesRegex(ValueError, "Unreviewed field source or incompatible licence"):
                core_pilot.prepare(destination)
            self.assertFalse((destination / "core.sqlite3").exists())

    def test_candidates_keep_four_pending_fields_and_the_interrogative_gap(self):
        curated = core_pilot.read_json(core_pilot.RESOURCE / "curated-inputs.json")
        batch = [group for group in curated["groups"] if 301 <= group["frozen_row"] <= 350]
        selected = [sense for group in batch for sense in group["senses"]]
        self.assertEqual(len(selected), 49)
        self.assertEqual(sum(len(sense["fields"]) for sense in selected), 196)
        for sense in selected:
            self.assertEqual(set(sense["fields"]), set(core_pilot.FIELDS))
            self.assertEqual(sense["independent_review"], "pending")
            for field in sense["fields"].values():
                self.assertTrue(field["text"].strip())
                self.assertEqual(field["independent_review"], "pending")
                self.assertTrue(field["source_span"]["raw_source_excerpt"].strip())
        gap = next(group for group in batch if group["frozen_row"] == 305)
        self.assertEqual(gap["senses"], [])
        self.assertEqual(gap["incomplete_fields"], list(core_pilot.FIELDS))
        self.assertIn("obsolete", gap["unresolved_reason"])

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
        self.assertEqual([group["frozen_row"] for group in batch], list(range(301, 351)))
        self.assertEqual(
            [(group["headword"], group["lexique_cgram"], group["subtitle_lemma_per_million"], group["subtitle_pos_per_million"]) for group in batch],
            [tuple(row.values()) for row in expected],
        )


if __name__ == "__main__":
    unittest.main()
