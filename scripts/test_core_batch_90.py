"""Content-handoff membership check for issue #90; independent admission is separate."""

import json
from pathlib import Path
import unittest


RESOURCE = Path(__file__).resolve().parents[1] / "resources/core-pilot-79"
EVIDENCE = Path(__file__).resolve().parents[1] / "docs/evidence/issue-90"
EXPECTED = (
    (451, "importer", "VER"), (452, "papa", "NOM"),
    (453, "vers", "NOM"), (454, "vers", "PRE"),
    (455, "heureux", "ADJ"), (456, "feu", "ADJ"),
    (457, "feu", "NOM"), (458, "train", "NOM"),
    (459, "manquer", "VER"), (460, "raconter", "VER"),
    (461, "film", "NOM"), (462, "car", "CON"),
    (463, "car", "NOM"), (464, "répondre", "VER"),
    (465, "garçon", "NOM"), (466, "chéri", "ADJ"),
    (467, "chéri", "NOM"), (468, "leurs", "ADJ:pos"),
    (469, "corps", "NOM"), (470, "celui", "PRO:dem"),
    (471, "autres", "PRO:ind"), (472, "loin", "ADV"),
    (473, "aucun", "ADJ:ind"), (474, "aucun", "PRO:ind"),
    (475, "pauvre", "ADJ"), (476, "pauvre", "NOM"),
    (477, "sauver", "VER"), (478, "chaque", "ADJ:ind"),
    (479, "retourner", "VER"), (480, "fin", "ADJ"),
    (481, "fin", "NOM"), (482, "rencontrer", "VER"),
    (483, "coeur", "NOM"), (484, "voler", "VER"),
    (485, "fermer", "VER"), (486, "valoir", "VER"),
    (487, "descendre", "VER"), (488, "docteur", "NOM"),
    (489, "important", "ADJ"), (490, "suffire", "VER"),
    (491, "façon", "NOM"), (492, "sembler", "VER"),
    (493, "dehors", "ADV"), (494, "dehors", "NOM"),
    (495, "dehors", "ONO"), (496, "super", "ADJ"),
    (497, "super", "NOM"), (498, "compter", "VER"),
    (499, "aucune", "ADJ:ind"), (500, "aucune", "PRO:ind"),
)


class Batch90Tests(unittest.TestCase):
    def test_frozen_membership_has_complete_selected_content(self):
        artifact = json.loads((RESOURCE / "curated-inputs.json").read_text(encoding="utf-8"))
        groups = json.loads((EVIDENCE / "groups.json").read_text(encoding="utf-8"))
        groups += [group for group in artifact["groups"] if group["frozen_row"] == 483]
        groups.sort(key=lambda group: group["frozen_row"])
        self.assertEqual(tuple((group["frozen_row"], group["headword"], group["lexique_cgram"])
                              for group in groups), EXPECTED)
        for group in groups:
            self.assertTrue(group["senses"], group["headword"])
            for sense in group["senses"]:
                self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                for field in sense["fields"].values():
                    self.assertTrue(field["text"].strip())
                    self.assertTrue(field["source_span"]["raw_source_excerpt"].strip())


if __name__ == "__main__":
    unittest.main()
