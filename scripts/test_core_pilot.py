import json
import os
from contextlib import closing
from pathlib import Path
import shutil
import sqlite3
import tempfile
import unittest

from core_pilot import prepare


RESOURCE = Path(__file__).resolve().parents[1] / "resources/core-pilot-79"
FIELD_NAMES = {"french_definition": "definition_fr", "ipa": "ipa", "english": "meaning_en", "chinese": "meaning_zh"}


class CorePilotPreparationTests(unittest.TestCase):
    def test_first_content_batch_keeps_all_fifty_frozen_word_pos_pairs(self):
        expected = [
            ("être", "AUX"), ("être", "NOM"), ("être", "VER"),
            ("avoir", "AUX"), ("avoir", "VER"), ("je", "PRO:per"),
            ("de", "PRE"), ("ne", "ADV"), ("pas", "ADV"), ("pas", "NOM"),
            ("le", "ART:def"), ("le", "PRO:per"), ("la", "ART:def"),
            ("la", "PRO:per"), ("tu", "PRO:per"), ("vous", "PRO:per"),
            ("il", "PRO:per"), ("et", "CON"), ("un", "ADJ:num"),
            ("un", "ART:ind"), ("un", "PRO:ind"), ("à", "PRE"),
            ("l'", "ART:def"), ("l'", "PRO:per"), ("qui", "PRO:int"),
            ("qui", "PRO:rel"), ("aller", "NOM"), ("aller", "VER"),
            ("les", "ART:def"), ("les", "PRO:per"), ("en", "ADV"),
            ("en", "PRE"), ("en", "PRO:per"), ("ça", "NOM"),
            ("ça", "PRO:dem"), ("faire", "VER"), ("tout", "ADJ"),
            ("tout", "ADJ:ind"), ("tout", "ADV"), ("tout", "NOM"),
            ("tout", "PRO:ind"), ("on", "PRO:per"), ("que", "PRO:int"),
            ("que", "PRO:rel"), ("une", "ADJ:num"), ("une", "ART:ind"),
            ("une", "PRO:ind"), ("ce", "ADJ:dem"), ("ce", "PRO:dem"),
            ("d'", "PRE"),
        ]
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "prepared"
            report = prepare(output)
            with closing(sqlite3.connect(output / "core.sqlite3")) as connection:
                batch = [json.loads(row[0]) for row in connection.execute(
                    "SELECT payload FROM senses WHERE frozen_row BETWEEN 1 AND 50 ORDER BY frozen_row,sense_index")]
            self.assertEqual([(sense["headword"], sense["pos"]) for sense in batch], expected)
            self.assertTrue(all(sense[field].strip() for sense in batch for field in FIELD_NAMES.values()))
            content_batch = report["content_batches"][0]
            self.assertEqual((content_batch["issue"], content_batch["target_groups"], content_batch["complete_groups"],
                              content_batch["complete_fields"], content_batch["reused_pilot_groups"], content_batch["new_groups"]),
                             (81, 50, 50, 200, 5, 45))
            self.assertEqual([(group["headword"], group["pos"]) for group in content_batch["groups"]], expected)

    def test_repeat_preparation_preserves_verified_resource_timestamps(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "prepared"
            first = prepare(output)
            for path in output.iterdir():
                os.utime(path, ns=(1_700_000_000_000_000_000, 1_700_000_000_000_000_000))
            before = {path.name: (path.read_bytes(), path.stat().st_mtime_ns) for path in output.iterdir()}
            self.assertEqual(prepare(output), first)
            self.assertEqual({path.name: (path.read_bytes(), path.stat().st_mtime_ns)
                              for path in output.iterdir()}, before)
            (output / "core.sqlite3").write_bytes(b"damaged generated resource")
            self.assertEqual(prepare(output), first)
            self.assertEqual((output / "core.sqlite3").read_bytes(), before["core.sqlite3"][0])

    def test_preparation_is_reproducible_and_binds_all_reviewed_fields_to_frozen_groups(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            report = prepare(root / "first")
            second = prepare(root / "second")
            self.assertEqual(report["sqlite_sha256"], second["sqlite_sha256"])
            self.assertEqual((root / "first/core.sqlite3").read_bytes(), (root / "second/core.sqlite3").read_bytes())
            self.assertEqual(report["admission"]["complete_groups"], 57)
            self.assertEqual(report["admission"]["complete_fields"], 228)
            self.assertEqual(report["admission"]["remaining_groups"], 4030)
            self.assertFalse(report["learner_acceptance"]["accepted"])
            connection = sqlite3.connect((root / "first/core.sqlite3").as_uri() + "?mode=ro", uri=True)
            with connection:
                senses = [json.loads(row[0]) for row in connection.execute("SELECT payload FROM senses ORDER BY frozen_row,sense_index")]
                self.assertEqual([row["frozen_row"] for row in senses], [*range(1, 51), 70, 71, 360, 422, 423, 483, 1911])
                self.assertEqual([row["pos"] for row in senses[:5]], ["AUX", "NOM", "VER", "AUX", "VER"])
                self.assertIn("all pronominal verbs", senses[0]["meaning_en"])
                self.assertIn("past participle", senses[3]["meaning_en"])
                self.assertIn("être", senses[3]["meaning_en"])
                self.assertEqual(dict(connection.execute("SELECT query,headword FROM aliases")),
                                 {"cœur": "coeur", "aujourd’hui": "aujourd'hui", "l’": "l'", "d’": "d'"})
                reviewed = json.loads((RESOURCE / "field-review.json").read_text(encoding="utf-8"))
                by_id = {sense["sense_id"]: sense for sense in senses}
                for field in reviewed["fields"]:
                    sense = by_id[field["sense_id"]]
                    self.assertEqual(sense[FIELD_NAMES[field["field"]]], field["accepted_text"])
                    source = sense["field_sources"][FIELD_NAMES[field["field"]]]
                    self.assertEqual(source["source_url"], field["exact_source"]["url"])
                    self.assertIn(str(field["exact_source"]["line_1_based"]), source["span"])
                with self.assertRaises(sqlite3.OperationalError):
                    connection.execute("DELETE FROM senses")
            connection.close()
            coverage = json.loads((root / "first/coverage.json").read_text(encoding="utf-8"))
            self.assertEqual(len(coverage["incomplete_groups"]), 4030)
            self.assertTrue(all(set(group["incomplete_fields"]) == set(FIELD_NAMES) for group in coverage["incomplete_groups"]))
            self.assertIn("COPYING", report["inputs"])
            self.assertIn("core_pilot.py", report["inputs"])
            self.assertIn("LEXIQUE-UPSTREAM-README.txt", report["inputs"])

    def test_incomplete_unreviewed_or_wrongly_bound_fields_never_produce_a_resource(self):
        for change in ("missing", "unreviewed", "binding"):
            with self.subTest(change=change), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                source = root / "inputs"
                shutil.copytree(RESOURCE, source)
                value = json.loads((source / "curated-inputs.json").read_text(encoding="utf-8"))
                if change == "missing":
                    del value["groups"][0]["senses"][0]["fields"]["chinese"]
                elif change == "unreviewed":
                    value["groups"][0]["senses"][0]["fields"]["chinese"]["independent_review"] = "pending"
                else:
                    value["groups"][0]["lexique_cgram"] = "VER"
                (source / "curated-inputs.json").write_text(json.dumps(value, ensure_ascii=False), encoding="utf-8")
                with self.assertRaisesRegex(ValueError, "field|reviewed|scope"):
                    prepare(root / "output", source_dir=source)
                self.assertFalse((root / "output/core.sqlite3").exists())

    def test_missing_legal_material_or_changed_spelling_evidence_fails_admission(self):
        for change in ("license", "redirect"):
            with self.subTest(change=change), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                source = root / "inputs"
                shutil.copytree(RESOURCE, source)
                if change == "license":
                    (source / "COPYING").unlink()
                else:
                    value = json.loads((source / "curated-inputs.json").read_text(encoding="utf-8"))
                    value["spelling_relations"][1]["source_span"]["raw_source_excerpt"] = "#"
                    (source / "curated-inputs.json").write_text(json.dumps(value, ensure_ascii=False), encoding="utf-8")
                with self.assertRaises(ValueError):
                    prepare(root / "output", source_dir=source)
                self.assertFalse((root / "output/core.sqlite3").exists())


if __name__ == "__main__":
    unittest.main()
