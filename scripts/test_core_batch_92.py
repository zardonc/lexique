"""Resource-boundary membership check; candidate presence is not admission."""

import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
RESOURCE = ROOT / "resources/core-pilot-79"
EXPECTED = (
    (551, "livre", "NOM"), (552, "agir", "VER"), (553, "jeter", "VER"),
    (554, "imaginer", "VER"), (555, "tour", "NOM"), (556, "adorer", "VER"),
    (557, "vérité", "NOM"), (558, "instant", "NOM"), (559, "recevoir", "VER"),
    (560, "pleurer", "VER"), (561, "bébé", "NOM"), (562, "partie", "NOM"),
    (563, "nouvelle", "ADJ"), (564, "prochain", "ADJ"), (565, "prochain", "NOM"),
    (566, "jeu", "NOM"), (567, "amener", "VER"), (568, "coucher", "VER"),
    (569, "derrière", "ADV"), (570, "derrière", "PRE"), (571, "service", "NOM"),
    (572, "haut", "ADJ"), (573, "haut", "ADV"), (574, "haut", "NOM"),
    (575, "plaisir", "NOM"), (576, "long", "ADJ"), (577, "long", "NOM"),
    (578, "content", "ADJ"), (579, "con", "ADJ"), (580, "con", "NOM"),
    (581, "promettre", "VER"), (582, "mentir", "VER"), (583, "soeur", "NOM"),
    (584, "bientôt", "ADV"), (585, "lit", "NOM"), (586, "tellement", "ADV"),
    (587, "bas", "ADJ"), (588, "bas", "ADV"), (589, "bas", "NOM"),
    (590, "utiliser", "VER"), (591, "lieu", "NOM"), (592, "presque", "ADV"),
    (593, "passé", "ADJ"), (594, "passé", "NOM"), (595, "préférer", "VER"),
    (596, "rire", "NOM"), (597, "rire", "VER"), (598, "café", "ADJ"),
    (599, "café", "NOM"), (600, "roi", "NOM"),
)


class Batch92Tests(unittest.TestCase):
    def test_exact_frozen_groups_have_sourced_four_field_selected_uses(self):
        curated = json.loads((RESOURCE / "curated-inputs.json").read_text(encoding="utf-8"))
        candidate_path = ROOT / "docs/evidence/issue-92/review-candidates.json"
        candidate = json.loads(candidate_path.read_text(encoding="utf-8")) if candidate_path.exists() else {}
        if candidate_path.exists():
            self.assertEqual(tuple((group["frozen_row"], group["headword"], group["lexique_cgram"])
                                   for group in candidate["groups"]), EXPECTED)
        groups = {group["frozen_row"]: group for group in curated["groups"]}
        for group in candidate.get("groups", []):
            groups.setdefault(group["frozen_row"], group)
        sources = {source["id"]: source for source in curated["sources"]}
        sources.update({source["id"]: source for source in candidate.get("sources", [])})
        excerpts = json.loads((RESOURCE / "selected-source-excerpts.json").read_text(encoding="utf-8"))
        excerpts = {(item["sense_id"], item["field"]): item for item in excerpts["fields"]}
        if candidate_path.exists():
            draft_excerpts = json.loads((candidate_path.parent / "source-excerpts.json").read_text(encoding="utf-8"))
            excerpts.update({(item["sense_id"], item["field"]): item for item in draft_excerpts["fields"]})
        self.assertEqual(curated["scope_denominator_groups"], 4087)
        for row, headword, pos in EXPECTED:
            with self.subTest(row=row):
                self.assertIn(row, set(groups))
                group = groups[row]
                self.assertEqual((group["headword"], group["lexique_cgram"]), (headword, pos))
                self.assertTrue(group["senses"])
                for sense in group["senses"]:
                    self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                    self.assertTrue(sense["use"].strip())
                    for name, field in sense["fields"].items():
                        self.assertTrue(field["text"].strip())
                        source = sources[field["source_id"]]
                        self.assertGreater(source["revid"], 0)
                        self.assertIn(f"oldid={source['revid']}", source["fixed_url"])
                        self.assertTrue(source["attribution"])
                        excerpt = excerpts[sense["sense_id"], name]
                        self.assertEqual(excerpt["revid"], source["revid"])
                        self.assertEqual(excerpt["raw_source_excerpt"], field["source_span"]["raw_source_excerpt"])


if __name__ == "__main__":
    unittest.main()
