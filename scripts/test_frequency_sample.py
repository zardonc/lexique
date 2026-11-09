"""Checks at the TSV-to-resource boundary using independently known source values."""

import gzip
import json
import sqlite3
import tempfile
import unittest
from contextlib import closing
from pathlib import Path

from frequency_sample import build_database, prepare


class FrequencySampleTests(unittest.TestCase):
    def test_missing_part_of_speech_keeps_source_value_and_withholds_the_total(self):
        source = "ortho\tlemme\tcgram\tfreqlemfilms2\nteam\tteam\t\t0.03\n"
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            archive = root / "source.tsv.gz"
            archive.write_bytes(gzip.compress(source.encode("utf-8")))
            database = root / "frequency.sqlite3"
            report = build_database(archive, database)
            with closing(sqlite3.connect(database)) as connection:
                value, reason, parts = connection.execute(
                    "SELECT per_million, reason, parts FROM lemmas WHERE lemma = 'team'"
                ).fetchone()
            self.assertIsNone(value)
            self.assertEqual(reason, "missing_pos")
            self.assertEqual(json.loads(parts), [
                {"part_of_speech": None, "per_million": 0.03, "reason": "missing_pos"}
            ])
            self.assertEqual(report["missing_part_of_speech_groups"], 1)

    def test_preparation_reuses_verified_resource_and_preserves_corrupt_output(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "frequency"
            first = prepare(output)
            self.assertEqual(first["rows"], 142694)
            self.assertEqual(first["lemmas"], 46947)
            self.assertEqual(first["lemma_pos_groups"], 51126)
            self.assertEqual(first["lemmas_without_islem"], 3385)
            self.assertEqual(first["unknown_lemmas"], 8)
            self.assertEqual(first["missing_part_of_speech_groups"], 8)
            database = output / "frequency.sqlite3"
            before = database.stat().st_mtime_ns
            self.assertEqual(prepare(output), first)
            self.assertEqual(database.stat().st_mtime_ns, before)
            self.assertTrue((output / "COPYING").is_file())
            self.assertTrue((output / "LICENSE-MIT").is_file())
            database.write_bytes(b"corrupt")
            with self.assertRaises(ValueError):
                prepare(output)
            self.assertEqual(database.read_bytes(), b"corrupt")

    def test_source_mapping_preserves_accents_and_all_possible_lemmas(self):
        source = (
            "ortho\tlemme\tcgram\tfreqlemfilms2\tfreqlemlivres\n"
            "cote\tcote\tNOM\t8.48\t900.00\n"
            "cote\tcoter\tVER\t20.00\t901.00\n"
            "côte\tcôte\tNOM\t35.16\t902.00\n"
            "côtes\tcôte\tNOM\t35.16\t902.00\n"
            "côté\tcôté\tNOM\t285.99\t903.00\n"
            "abaissa\tabaisser\tVER\t4.93\t904.00\n"
        )
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            archive = root / "source.tsv.gz"
            archive.write_bytes(gzip.compress(source.encode("utf-8")))
            database = root / "frequency.sqlite3"
            build_database(archive, database)
            with closing(sqlite3.connect(database)) as connection:
                mapping = list(connection.execute("SELECT key, lemma FROM forms ORDER BY key, lemma"))
                values = dict(connection.execute("SELECT lemma, per_million FROM lemmas"))
                metric = connection.execute("SELECT value FROM metadata WHERE key = 'metric'").fetchone()[0]
            self.assertEqual(metric, "freqlemfilms2")
            self.assertEqual(mapping, [
                ("abaissa", "abaisser"), ("cote", "cote"), ("cote", "coter"),
                ("côte", "côte"), ("côtes", "côte"), ("côté", "côté"),
            ])
            self.assertEqual(values["cote"], 8.48)
            self.assertEqual(values["côte"], 35.16)
            self.assertEqual(values["côté"], 285.99)

    def test_zero_is_known_but_missing_invalid_and_conflicting_values_stay_unknown(self):
        source = (
            "ortho\tlemme\tcgram\tfreqlemfilms2\n"
            "zéro\tzéro\tNOM\t0.00\n"
            "manquant\tmanquant\tNOM\t\n"
            "négatif\tnégatif\tADJ\t-1\n"
            "infini\tinfini\tADJ\tInfinity\n"
            "invalide\tinvalide\tADJ\tNaN\n"
            "conflit\tconflit\tNOM\t2.00\n"
            "conflits\tconflit\tNOM\t3.00\n"
            "partiel\tpartiel\tNOM\t2.50\n"
            "partiel\tpartiel\tADJ\t\n"
        )
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            archive = root / "source.tsv.gz"
            archive.write_bytes(gzip.compress(source.encode("utf-8")))
            database = root / "frequency.sqlite3"
            build_database(archive, database)
            with closing(sqlite3.connect(database)) as connection:
                values = {lemma: (value, reason) for lemma, value, reason in
                          connection.execute("SELECT lemma, per_million, reason FROM lemmas")}
            self.assertEqual(values, {
                "zéro": (0.0, None), "manquant": (None, "missing"),
                "négatif": (None, "invalid"), "infini": (None, "invalid"),
                "invalide": (None, "invalid"), "conflit": (None, "conflict"),
                "partiel": (None, "missing"),
            })

    def test_standard_headword_combines_parts_of_speech_without_recounting_inflections(self):
        source = (
            "ortho\tlemme\tcgram\tfreqlemfilms2\tislem\n"
            "a priori\ta priori\tADV\t1.04\t1\n"
            "a priori\ta priori\tNOM\t0.41\t1\n"
            "abaissa\tabaisser\tVER\t4.93\t0\n"
            "abaisser\tabaisser\tVER\t4.93\t1\n"
            "abaissait\tabaisser\tVER\t4.93\t0\n"
            "égards\tégard\tNOM\t10.00\t0\n"
        )
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            archive = root / "source.tsv.gz"
            archive.write_bytes(gzip.compress(source.encode("utf-8")))
            database = root / "frequency.sqlite3"
            report = build_database(archive, database)
            self.assertEqual(report["rows"], 6)
            self.assertEqual(report["lemmas"], 3)
            self.assertEqual(report["lemma_pos_groups"], 4)
            with closing(sqlite3.connect(database)) as connection:
                values = dict(connection.execute("SELECT lemma, per_million FROM lemmas"))
            self.assertEqual(values, {"a priori": 1.45, "abaisser": 4.93, "égard": 10.0})


if __name__ == "__main__":
    unittest.main()
