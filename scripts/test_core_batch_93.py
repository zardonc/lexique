"""Resource-boundary membership check for issue #93's pending content slice."""

import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
RESOURCE = ROOT / "resources/core-pilot-79"
SLICE = ROOT / "docs/evidence/issue-93"
EXPECTED = (
    ("offrir", "VER"), ("verre", "NOM"), ("réveiller", "VER"), ("aide", "NOM"),
    ("d'abord", "ADV"), ("journée", "NOM"), ("préparer", "VER"), ("numéro", "NOM"),
    ("propre", "ADJ"), ("propre", "NOM"), ("permettre", "VER"), ("ramener", "VER"),
    ("enlever", "VER"), ("lâcher", "VER"), ("calme", "ADJ"), ("calme", "NOM"),
    ("lever", "VER"), ("choisir", "VER"), ("musique", "NOM"), ("conduire", "VER"),
    ("faute", "NOM"), ("calmer", "VER"), ("dîner", "NOM"), ("dîner", "VER"),
    ("mariage", "NOM"), ("bureau", "NOM"), ("route", "NOM"), ("chanter", "VER"),
    ("secret", "ADJ"), ("secret", "NOM"), ("général", "ADJ"), ("général", "NOM"),
    ("disparaître", "VER"), ("dessus", "ADV"), ("dessus", "NOM"), ("présenter", "VER"),
    ("accepter", "VER"), ("blanc", "ADJ"), ("blanc", "NOM"), ("baiser", "NOM"),
    ("baiser", "VER"), ("sinon", "CON"), ("idiot", "ADJ"), ("idiot", "NOM"),
    ("drôle", "ADJ"), ("drôle", "NOM"), ("confiance", "NOM"), ("cinq", "ADJ:num"),
    ("bonsoir", "NOM"), ("compte", "NOM"),
)


class Batch93Tests(unittest.TestCase):
    def test_frozen_groups_have_complete_source_bound_selected_fields(self):
        with (RESOURCE / "scope-manifest.csv").open(encoding="utf-8-sig", newline="") as handle:
            scope = list(csv.DictReader(handle))
        records = json.loads((SLICE / "groups.json").read_text(encoding="utf-8"))
        groups = {g["frozen_row"]: g for g in records}
        self.assertEqual(set(groups), set(range(601, 651)))
        self.assertEqual(len(records), 50)
        sources = {s["id"]: s for s in json.loads((SLICE / "sources.json").read_text(encoding="utf-8"))}
        excerpts = json.loads((SLICE / "source-excerpts.json").read_text(encoding="utf-8"))
        selected = {(e["sense_id"], e["field"]): e for e in excerpts["fields"]}
        for row, expected in enumerate(EXPECTED, 601):
            group = groups[row]
            self.assertEqual(expected, (scope[row - 1]["word"], scope[row - 1]["lexique_cgram"]))
            self.assertEqual(expected, (group["headword"], group["lexique_cgram"]))
            self.assertEqual(group["subtitle_lemma_per_million"], scope[row - 1]["subtitle_lemma_per_million"])
            self.assertEqual(group["subtitle_pos_per_million"], scope[row - 1]["subtitle_pos_per_million"])
            self.assertTrue(group["senses"], f"Missing selected use at row {row}")
            for sense in group["senses"]:
                self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                for name, field in sense["fields"].items():
                    self.assertTrue(field["text"].strip())
                    self.assertEqual(field["language"], {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}[name])
                    self.assertEqual(field["content_license"], "CC-BY-SA-4.0")
                    source = sources[field["source_id"]]
                    excerpt = selected[sense["sense_id"], name]
                    self.assertGreater(source["revid"], 0)
                    self.assertIn(f"oldid={source['revid']}", source["fixed_url"])
                    self.assertEqual(excerpt["revid"], source["revid"])
                    self.assertEqual(excerpt["source_id"], field["source_id"])
                    self.assertEqual(excerpt["raw_source_excerpt"], field["source_span"]["raw_source_excerpt"])
                    self.assertEqual(excerpt["line_1_based"], field["source_span"]["line_1_based"])


if __name__ == "__main__":
    unittest.main()
