"""Exercise the public offline conjugation CLI with the pinned real resource."""

import json
import gzip
from pathlib import Path
import subprocess
import sqlite3
import shutil
import sys
import tempfile
import unittest

CLI = Path(__file__).with_name("conjugation_sample.py")


def run_cli(*args):
    return subprocess.run([sys.executable, "-X", "utf8", str(CLI), *map(str, args)],
                          capture_output=True, text=True, encoding="utf-8", timeout=60)


class ConjugationCliTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.temporary = tempfile.TemporaryDirectory()
        cls.addClassCleanup(cls.temporary.cleanup)
        cls.output = Path(cls.temporary.name) / "resource"
        result = run_cli("build", cls.output)
        if result.returncode:
            raise AssertionError(result.stderr)
        cls.database = cls.output / "conjugation.sqlite3"

    def read(self, command, value):
        result = run_cli(command, self.database, value)
        self.assertEqual(result.returncode, 0, result.stderr)
        return json.loads(result.stdout)

    def test_sommes_preserves_both_lemmas_and_all_slots(self):
        result = self.read("query", "sommes")
        self.assertEqual([(r["lemma"], r["tense"], r["person"], r["number"])
                          for r in result["matches"]],
                         [("sommer", "P", "2", "s"),
                          ("sommer", "S", "2", "s"), ("être", "P", "1", "p")])
        highlighted = [c for t in result["tables"] for c in t["cells"] if c["matched"]]
        self.assertEqual(len(highlighted), 3)
        self.assertTrue(all(c["form"] == "sommes" for c in highlighted))

    def test_suis_and_aime_expand_every_explicit_person_and_tense(self):
        result = self.read("query", "suis")
        self.assertEqual([(r["lemma"], r["tense"], r["person"]) for r in result["matches"]],
                         [("suivre", "P", "1"), ("suivre", "P", "2"),
                          ("suivre", "Y", "2"), ("être", "P", "1")])
        self.assertEqual(len(self.read("query", "aime")["matches"]), 5)

    def test_regular_irregular_auxiliary_pronominal_and_defective_tables(self):
        for lemma, form, tense in [("aimer", "aimons", "P"), ("finir", "finissons", "P"),
                                   ("aller", "vais", "P"), ("avoir", "avons", "P"),
                                   ("être", "sommes", "P"), ("enfuir", "enfuis", "P"),
                                   ("falloir", "faut", "P")]:
            with self.subTest(lemma=lemma):
                table = self.read("table", lemma)["tables"][0]
                self.assertEqual(table["status"], "source_only")
                self.assertTrue(any(c["form"] == form and c["tense"] == tense for c in table["cells"]))
        falloir = self.read("table", "falloir")["tables"][0]["cells"]
        self.assertFalse(any(c["person"] in ("1", "2") for c in falloir))
        self.assertEqual(self.read("table", "s'enfuir")["status"], "not_found")
        self.assertEqual(self.read("query", "avons aimé")["status"], "not_found")

    def test_unspecified_agreement_and_tags_are_not_invented(self):
        ete = self.read("query", "été")["matches"]
        self.assertEqual([(r["raw_tag"], r["gender"], r["number"]) for r in ete], [("K", None, None)])
        unresolved = self.read("query", "voilà")["matches"]
        self.assertEqual([(r["raw_tag"], r["tense"]) for r in unresolved], [("", None)])

    def test_spelling_variants_share_a_slot_without_overwriting(self):
        table = self.read("table", "payer")["tables"][0]["cells"]
        self.assertEqual([c["form"] for c in table if c["tense"] == "P"
                          and c["person"] == "1" and c["number"] == "s"], ["paie", "paye"])

    def test_missing_form_and_exact_spelling(self):
        for form in ("lexique-inexistant", "SOMMES", "ete", "x' OR 1=1 --"):
            result = self.read("query", form)
            self.assertEqual(result["status"], "not_found")
            self.assertEqual(result["matches"], [])

    def test_missing_corrupt_and_truncated_resources_fail(self):
        with tempfile.TemporaryDirectory() as temporary:
            path = Path(temporary) / "resource.sqlite3"
            for content in (None, b"not sqlite", self.database.read_bytes()[:4096]):
                if content is not None:
                    path.write_bytes(content)
                result = run_cli("query", path, "sommes")
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("Conjugation resource error:", result.stderr)
                self.assertEqual(result.stdout, "")
            shutil.copyfile(self.database, path)
            with sqlite3.connect(path) as connection:
                connection.execute("DELETE FROM slots WHERE form = 'sommes'")
            connection.close()
            self.assertIn("count mismatch", run_cli("query", path, "sommes").stderr)

    def test_source_integrity_and_existing_output_are_enforced(self):
        self.assertNotEqual(run_cli("build", self.output).returncode, 0)
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary) / "source.gz"
            output = Path(temporary) / "output"
            for content in (None, gzip.compress(b"sommes\tv\twrong\tP1p\n"), b"corrupt"):
                if content is not None:
                    source.write_bytes(content)
                result = run_cli("build", output, "--source", source)
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(output.exists())

    def test_size_report_accounts_for_both_indices(self):
        report = json.loads((self.output / "report.json").read_text(encoding="utf-8"))
        self.assertEqual(report["source_verb_rows"], 342771)
        self.assertEqual(report["lemmas"], 7826)
        self.assertEqual(report["database_bytes"], report["table_bytes"]
                         + report["lemma_index_bytes"] + report["reverse_index_bytes"])
        for name in ("LICENSE-LGPLLR", "provenance.json", "lefff-3.4.mlex.gz", "README.md"):
            self.assertTrue((self.output / name).is_file())


if __name__ == "__main__":
    unittest.main()
