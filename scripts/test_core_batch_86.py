"""Focused checks at issue #86's curation and produced-resource boundaries."""

import json
from pathlib import Path
import tempfile
import unittest

from core_pilot import prepare

ROOT = Path(__file__).resolve().parents[1]
RESOURCE = ROOT / "resources/core-pilot-79"


class Batch86Tests(unittest.TestCase):
    def test_pending_batch_cannot_become_an_admitted_resource(self):
        curated = json.loads((RESOURCE / 'curated-inputs.json').read_text(encoding='utf-8'))
        pending = [g for g in curated['groups'] if 251 <= g['frozen_row'] <= 300
                   and any(s['independent_review'] == 'pending' for s in g['senses'])]
        if not pending:
            self.skipTest('Independent review resolved this draft gate; admitted/native checks run centrally')
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / 'core'
            with self.assertRaisesRegex(ValueError, 'Unreviewed field source'):
                prepare(output)
            self.assertFalse(output.exists(), 'Pending review must not create an admitted artifact')

    def test_batch_keeps_all_fifty_frozen_word_pos_groups(self):
        expected = [
            ('plaire', 'VER'), ('maison', 'NOM'), ('gens', 'NOM'), ('nuit', 'NOM'), ('ah', 'ONO'),
            ('soir', 'NOM'), ('nom', 'NOM'), ('bonjour', 'NOM'), ('jouer', 'VER'), ('leur', 'PRO:per'),
            ('leur', 'PRO:pos'), ('finir', 'VER'), ('peur', 'NOM'), ('mort', 'ADJ'), ('mort', 'NOM'),
            ('parce que', 'CON'), ('perdre', 'VER'), ('maman', 'NOM'), ('sentir', 'VER'), ('ouais', 'ADV'),
            ('rentrer', 'VER'), ('nos', 'ADJ:pos'), ('argent', 'NOM'), ('vivre', 'NOM'), ('vivre', 'VER'),
            ('premier', 'ADJ'), ('premier', 'NOM'), ('problème', 'NOM'), ('quelle', 'ADJ:int'), ('rendre', 'VER'),
            ('dernier', 'ADJ'), ('dernier', 'NOM'), ('tenir', 'VER'), ('cet', 'ADJ:dem'), ('main', 'NOM'),
            ('cela', 'PRO:dem'), ('vite', 'ADV'), ('moins', 'ADV'), ('moins', 'NOM'), ('oublier', 'VER'),
            ('air', 'NOM'), ('salut', 'NOM'), ('salut', 'ONO'), ('fils', 'NOM'), ('travailler', 'VER'),
            ('demain', 'ADV'), ('demain', 'NOM'), ('tête', 'NOM'), ('manger', 'VER'), ('coup', 'NOM'),
        ]
        curated = json.loads((RESOURCE / "curated-inputs.json").read_text(encoding="utf-8"))
        groups = [g for g in curated["groups"] if 251 <= g["frozen_row"] <= 300]
        self.assertEqual([g["frozen_row"] for g in groups], list(range(251, 301)))
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], expected)

    def test_plaire_has_four_sourced_fields_for_pleasing_someone(self):
        curated = json.loads((RESOURCE / "curated-inputs.json").read_text(encoding="utf-8"))
        matches = [g for g in curated["groups"] if g["frozen_row"] == 251]
        self.assertEqual(len(matches), 1, "The frozen plaire/VER group must be present")
        group = matches[0]
        self.assertEqual((group["headword"], group["lexique_cgram"]), ("plaire", "VER"))
        fields = group["senses"][0]["fields"]
        self.assertEqual(set(fields), {"french_definition", "ipa", "english", "chinese"})
        self.assertIn("à", fields["english"]["text"])
        self.assertIn("某人", fields["chinese"]["text"])
        for field in fields.values():
            self.assertTrue(field["text"].strip())
            self.assertTrue(field["source_span"]["raw_source_excerpt"].strip())
            self.assertIn(field["independent_review"], ("pending", "passed"))


if __name__ == "__main__":
    unittest.main()
