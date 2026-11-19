"""Exact issue #96 resource-handoff membership; independent admission is separate."""

import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EXPECTED = (
    (751, "réussir", "VER"), (752, "bizarre", "ADJ"),
    (753, "or", "CON"), (754, "or", "NOM"), (755, "voix", "NOM"),
    (756, "protéger", "VER"), (757, "honneur", "NOM"), (758, "tôt", "ADV"),
    (759, "équipe", "NOM"), (760, "prévenir", "VER"), (761, "cheval", "NOM"),
    (762, "ailleurs", "ADV"), (763, "lequel", "PRO:int"), (764, "lequel", "PRO:rel"),
    (765, "avion", "NOM"), (766, "bout", "NOM"), (767, "habiter", "VER"),
    (768, "faim", "NOM"), (769, "ensuite", "ADV"), (770, "l'un", "PRO:ind"),
    (771, "déranger", "VER"), (772, "reprendre", "VER"), (773, "oncle", "NOM"),
    (774, "toute", "ADJ:ind"), (775, "prix", "NOM"), (776, "retard", "NOM"),
    (777, "détruire", "VER"), (778, "cadeau", "NOM"), (779, "pousser", "VER"),
    (780, "face", "NOM"), (781, "amoureux", "ADJ"), (782, "amoureux", "NOM"),
    (783, "gueule", "NOM"), (784, "chemin", "NOM"), (785, "bateau", "NOM"),
    (786, "million", "NOM"), (787, "sac", "NOM"), (788, "seconde", "NOM"),
    (789, "sujet", "NOM"), (790, "bête", "ADJ"), (791, "bête", "NOM"),
    (792, "découvrir", "VER"), (793, "erreur", "NOM"), (794, "soleil", "NOM"),
    (795, "voyage", "NOM"), (796, "sauter", "VER"), (797, "rêver", "VER"),
    (798, "détester", "VER"), (799, "paraître", "VER"), (800, "balle", "NOM"),
)


class Batch96Tests(unittest.TestCase):
    def test_exact_frozen_groups_have_sourced_four_field_selected_uses(self):
        resource = json.loads((ROOT / "resources/core-pilot-79/curated-inputs.json").read_text(encoding="utf-8"))
        path = ROOT / "docs/evidence/issue-96/groups.json"
        drafts = json.loads(path.read_text(encoding="utf-8")) if path.exists() else []
        if path.exists():
            self.assertEqual(tuple((g["frozen_row"], g["headword"], g["lexique_cgram"]) for g in drafts), EXPECTED)
        sources = {source["id"]: source for source in resource["sources"]}
        if path.exists():
            sources.update({source["id"]: source for source in json.loads((path.parent / "sources.json").read_text(encoding="utf-8"))})
        groups = {group["frozen_row"]: group for group in resource["groups"]}
        for group in drafts:
            groups.setdefault(group["frozen_row"], group)
        selected = [groups[row] for row, _, _ in EXPECTED if row in groups]
        self.assertEqual(tuple((g["frozen_row"], g["headword"], g["lexique_cgram"]) for g in selected), EXPECTED)
        self.assertEqual(resource["scope_denominator_groups"], 4087)
        for group in selected:
            self.assertTrue(group["senses"])
            for sense in group["senses"]:
                self.assertTrue(sense["use"].strip())
                self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                for field in sense["fields"].values():
                    self.assertTrue(field["text"].strip())
                    self.assertTrue(field["source_span"]["raw_source_excerpt"].strip())
                    source = sources[field["source_id"]]
                    self.assertGreater(source["revid"], 0)
                    self.assertIn(f"oldid={source['revid']}", source["fixed_url"])
                    self.assertTrue(source["attribution"].strip())


if __name__ == "__main__":
    unittest.main()
