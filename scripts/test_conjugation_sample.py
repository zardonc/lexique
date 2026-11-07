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
import zlib

CLI = Path(__file__).with_name("conjugation_sample.py")
VERB_RESOURCE = CLI.parent.parent / "resources/grammalecte-2.3.0"


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

    def test_regular_coloring_spans_follow_the_spec_independently_of_storage_cuts(self):
        for query, form, expected in (
                ("nous avions lavé", "avions lavé", {"verbal": [[5, 16]], "main": [12, 16], "ending": [15, 16]}),
                ("lavais", "lavais", {"verbal": [[0, 6]], "main": [0, 6], "ending": [3, 6]})):
            with self.subTest(query=query):
                table = self.read("query", query)["tables"][0]
                cell = next(cell for cell in table["cells"] if cell["form"] == form and cell["matched"])
                self.assertEqual(cell["coloring"][query], expected)
                self.assertEqual(cell["form"], form)

    def test_coloring_validates_regular_families_variants_and_pronoun_boundaries(self):
        expectations = {
            "laver": [("P", "lave", "e"), ("I", "lavais", "ais"), ("J", "lavâmes", "âmes"),
                      ("F", "laverai", "rai"), ("C", "laverions", "rions"), ("S", "laviez", "iez"),
                      ("T", "lavassions", "assions"), ("Y", "lavez", "ez"), ("W", "laver", "er"),
                      ("G", "lavant", "ant"), ("K", "lavées", "ées"), ("PQP", "avions lavé", "é")],
            "finir": [("P", "finis", "s"), ("P", "finissons", "ons"), ("I", "finissais", "ais"),
                      ("J", "finîmes", "îmes"), ("F", "finirai", "rai"), ("C", "finirions", "rions"),
                      ("S", "finisse", "e"), ("T", "finît", "ît"), ("Y", "finissez", "ez"),
                      ("W", "finir", "ir"), ("G", "finissant", "ant"), ("K", "finies", "ies")],
            "haïr": [("P", "hais", "s"), ("J", "haïmes", "ïmes"), ("W", "haïr", "ïr"), ("K", "haï", "ï")],
            "payer": [("P", "paie", "e"), ("P", "paye", "e"), ("F", "paierai", "rai"), ("F", "payerai", "rai")],
            "manger": [("I", "mangeais", "ais"), ("P", "mangeons", "ons")],
            "placer": [("I", "plaçais", "ais")],
            "mûrir": [("P", "mûris", "s"), ("K", "mûri", "i")],
            "rassir": [("K", "rassise", "ise"), ("K", "rassis", "is")],
        }
        for lemma, cases in expectations.items():
            table = self.read("table", lemma)["tables"][0]
            for tense, form, ending in cases:
                with self.subTest(lemma=lemma, tense=tense, form=form):
                    cells = [cell for cell in table["cells"] if (cell["tense"], cell["form"]) == (tense, form)]
                    self.assertTrue(cells)
                    for cell in cells:
                        start, end = cell["coloring"][form]["ending"]
                        self.assertEqual(form[start:end], ending)
        pron = self.read("table", "se laver")["tables"][0]
        for form, expected in (
                ("lave-toi", {"verbal": [[0, 4]], "main": [0, 4], "ending": [3, 4]}),
                ("en se lavant", {"verbal": [[6, 12]], "main": [6, 12], "ending": [9, 12]}),
                ("sois-toi lavé", {"verbal": [[0, 4], [9, 13]], "main": [9, 13], "ending": [12, 13]})):
            self.assertEqual(next(cell for cell in pron["cells"] if cell["form"] == form)["coloring"][form], expected)
        irregular = self.read("table", "s’en aller")["tables"][0]
        cell = next(cell for cell in irregular["cells"] if cell["form"] == "va-t’en")
        self.assertEqual(cell["coloring"]["va-t’en"], {"verbal": [[0, 2]], "main": [0, 2], "ending": None})
        self.assertTrue(all(span["ending"] is None for cell in irregular["cells"] for span in cell["coloring"].values()))
        coverage = json.loads((self.output / "report.json").read_text(encoding="utf-8"))["coloring_coverage"]
        self.assertEqual(coverage["regular_unconfirmed"], [])
        self.assertGreater(coverage["groups"]["2"]["endings"], 1000)
        self.assertGreater(coverage["groups"]["3"]["cells"], 1000)
        self.assertEqual(coverage["groups"]["3"]["endings"], 0)
        self.assertEqual({row[0] for row in coverage["retained_source_only"]}, {"courbaturer", "précéder", "égaler"})
        self.assertEqual(self.read("table", "mûrir")["tables"][0]["verb"]["group"], 2)
        for lemma in ("messeoir", "reparcourir"):
            table = self.read("table", lemma)["tables"][0]
            self.assertEqual(table["verb"]["group"], 3)
            self.assertTrue(all(span["ending"] is None for cell in table["cells"] for span in cell["coloring"].values()))

    def test_case_variations_preserve_every_match_and_query(self):
        result = self.read("query", "SOMMES")
        self.assertEqual(result["status"], "found")
        self.assertEqual(result["query"], "SOMMES")
        self.assertEqual([table["lemma"] for table in result["tables"]], ["sommer", "être"])
        self.assertEqual([(cell["lemma"], cell["tense"], cell["person"], cell["number"])
                          for cell in result["matches"]],
                         [("sommer", "P", "2", "s"), ("sommer", "S", "2", "s"), ("être", "P", "1", "p")])
        self.assertTrue(all(cell["form"] == "sommes" for cell in result["matches"]))

    def test_typographic_expressions_preserve_original_forms_and_identities(self):
        for entered, lemma, expression in (
                ("\u00a0E\u0302TRE\u202f", "être", "être"),
                ("  J'AI  AIME\u0301  ", "aimer", "j’ai aimé"),
                (" QUE  J'AIME ", "aimer", "que j’aime"),
                (" ELLES\u00a0SE   SONT ABSTENUES ", "s’abstenir", "elles se sont abstenues"),
                (" VA-T'EN ", "s’en aller", "va-t’en"),
                (" S'ABSTENIR ", "s’abstenir", "s’abstenir"),
                ("V’LÀ", "voilà", "v'là")):
            with self.subTest(query=entered):
                result = self.read("query", entered)
                self.assertEqual(result["query"], entered)
                self.assertEqual([table["lemma"] for table in result["tables"]], [lemma])
                self.assertTrue(any(expression in cell["expressions"] for cell in result["matches"]))
                self.assertTrue(any(cell["matched"] and expression in cell["expressions"]
                                    for cell in result["tables"][0]["cells"]))

    def test_compact_resource_reports_shared_models_and_preserves_expressions(self):
        report = json.loads((self.output / "report.json").read_text(encoding="utf-8"))
        self.assertIn("models", report, "Conjugation build must store shared simple-form models")
        self.assertLess(report["models"], 250)
        self.assertEqual(report["slots"], 1451539)
        self.assertLess(report["database_bytes"], 16 * 1024 * 1024)
        self.assertEqual(report["lemmas"], 10781)
        self.assertEqual(self.read("query", "nous nous sommes écrites")["tables"][0]["lemma"], "s’écrire")

    def test_pronominal_lookup_keeps_a_distinct_identity_and_complete_simple_forms(self):
        table = self.read("query", "se laver")["tables"]
        self.assertEqual([t["lemma"] for t in table], ["se laver"])
        pron = table[0]
        self.assertEqual(pron["verb"]["pronominal"]["base"], "laver")
        self.assertEqual(len(pron["verb"]["blocks"]), 23)
        self.assertTrue(any("je me lave" in c["expressions"] for c in pron["cells"] if c["tense"] == "P"))
        self.assertTrue(any(c["form"] == "lave-toi" for c in pron["cells"] if c["tense"] == "Y"))
        self.assertTrue(any(c["form"] == "en se lavant" for c in pron["cells"] if c["tense"] == "GP"))
        ordinary = self.read("query", "je lave")["tables"]
        self.assertEqual([t["lemma"] for t in ordinary], ["laver"])

    def test_english_meanings_preserve_source_senses_and_missing_tables(self):
        aimer = self.read("table", "aimer")["tables"][0]
        self.assertEqual(aimer["meanings"]["status"], "available")
        self.assertEqual([s["glosses"] for s in aimer["meanings"]["senses"]], [["love"], ["appreciate", "like"]])
        candidates = self.read("query", "sommes")["tables"]
        self.assertEqual([t["lemma"] for t in candidates], ["sommer", "être"])
        self.assertEqual(candidates[1]["meanings"]["senses"][0]["glosses"], ["be"])
        missing = self.read("table", "abéliser")["tables"][0]
        self.assertEqual(missing["meanings"], {"status": "unavailable", "senses": []})
        self.assertEqual(len(missing["verb"]["blocks"]), 23)
        abstain = self.read("table", "abstenir")["tables"][0]["meanings"]["senses"]
        self.assertEqual(abstain[0]["headword"], "s'abstenir")
        self.assertTrue(all(s["use"] == "pronominal" for s in abstain))
        self.assertEqual(abstain[0]["glosses"], ["abstain"])
        self.assertEqual(self.read("table", "avenir")["tables"][0]["meanings"]["status"], "unavailable")
        retain = self.read("table", "retenir")["tables"][0]["meanings"]["senses"]
        self.assertTrue(any("keep" in s["glosses"] for s in retain))
        self.assertFalse(any("thunder" in s["glosses"] for s in retain))

    def test_pronominal_compounds_label_agreement_and_preserve_whole_expressions(self):
        for query, lemma in (("elles se sont abstenues", "s’abstenir"), ("elles se sont lavées", "se laver"),
                             ("elles se sont parlé", "se parler"), ("elle s’est arrogé", "s’arroger"),
                             ("va-t’en", "s’en aller"), ("je m’en suis allé", "s’en aller")):
            tables = self.read("query", query)["tables"]
            self.assertEqual([t["lemma"] for t in tables], [lemma], query)
            self.assertTrue(any(query in c["expressions"] for c in tables[0]["cells"]))
        for lemma, agreement in (("s’abstenir", "subject"), ("se laver", "contextual"),
                                 ("se parler", "invariable"), ("s’arroger", "contextual")):
            table = self.read("table", lemma)["tables"][0]
            self.assertEqual(table["verb"]["pronominal"]["agreement"], agreement)
        for query in ("elles se sont abstenus", "elles se sont parlées", "je se lave", "elle se suis lavée",
                      "se falloir", "se abéliser"):
            self.assertEqual(self.read("query", query)["tables"], [], query)

    def test_pronominal_source_restrictions_and_resource_wide_coverage(self):
        for query, lemma in (("il s’agit", "s’agir"), ("on s’entraide", "s’entraider"),
                             ("ils s’entraident", "s’entraider"), ("il s’est ensuivi", "s’ensuivre"),
                             ("je me hais", "se haïr"), ("je m’habille", "s’habiller")):
            self.assertEqual([t["lemma"] for t in self.read("query", query)["tables"]], [lemma], query)
        for query in ("je m’agis", "je m’entraide", "je m’ensuis", "elle s’est abstenu", "s’en découdre"):
            self.assertEqual(self.read("query", query)["tables"], [], query)
        report = json.loads((self.output / "report.json").read_text(encoding="utf-8"))
        coverage = report["pronominal_coverage"]
        self.assertGreater(coverage["identities"], 2500)
        self.assertEqual(coverage["base_lemmas"] + len(coverage["excluded"]), report["ordinary_lemmas"])
        self.assertEqual(coverage["incomplete"], [])
        for counts in coverage["blocks"].values():
            self.assertEqual(sum(counts.values()), coverage["identities"])
            self.assertEqual(counts.get("missing", 0) + counts.get("unconfirmed", 0), 0)

    def test_indirect_reflexives_keep_preceding_direct_object_agreement_contextual(self):
        for query, lemma in (("elles se sont dites", "se dire"), ("nous nous sommes écrites", "s’écrire")):
            tables = self.read("query", query)["tables"]
            self.assertEqual([t["lemma"] for t in tables], [lemma], query)
            self.assertEqual(tables[0]["verb"]["pronominal"]["agreement"], "contextual")
            self.assertTrue(any(query in c["expressions"] for c in tables[0]["cells"]))

    def test_collective_on_agrees_without_licensing_plural_impersonal_or_il_forms(self):
        for query in ("on s’est abstenus", "on s’est abstenues"):
            self.assertEqual([t["lemma"] for t in self.read("query", query)["tables"]], ["s’abstenir"])
        self.assertEqual([t["lemma"] for t in self.read("query", "il s’est agi")["tables"]], ["s’agir"])
        for query in ("il s’est abstenues", "il s’est abstenus", "s’étant agie", "s’étant agis"):
            self.assertEqual(self.read("query", query)["tables"], [], query)

    def test_en_uses_keep_explicit_meanings_and_other_pronominal_uses_distinct(self):
        for query in ("se vouloir", "se découdre"):
            self.assertEqual([t["lemma"] for t in self.read("query", query)["tables"]], [query])
        departure = self.read("table", "s’en aller")["tables"][0]
        self.assertTrue(any("go away" in s["glosses"] for s in departure["meanings"]["senses"]))
        self.assertTrue(all(s["headword"] == "s'en aller" for s in departure["meanings"]["senses"]))
        self.assertEqual(self.read("query", "s’en découdre")["tables"], [])

    def test_english_meaning_inputs_and_tampered_payloads_fail_visibly(self):
        source = Path(self.temporary.name) / "english-source"
        shutil.copytree(self.output / "freedict-fra-eng-0.4.1", source)
        for name in ("freedict-fra-eng-0.4.1.src.tar.xz", "COPYING", "provenance.json"):
            path = source / name
            original = path.read_bytes()
            path.write_bytes(b"corrupt")
            output = Path(self.temporary.name) / "bad-english"
            result = run_cli("build", output, "--meanings-source", source)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("Conjugation resource error:", result.stderr)
            self.assertFalse(output.exists())
            path.write_bytes(original)
        altered = Path(self.temporary.name) / "altered-english"
        shutil.copytree(self.output, altered)
        with sqlite3.connect(altered / "conjugation.sqlite3") as connection:
            connection.execute("UPDATE verbs SET meanings = ? WHERE lemma = ?",
                               (json.dumps({"status": "available", "senses": []}), "aimer"))
        connection.close()
        result = run_cli("table", altered / "conjugation.sqlite3", "aimer")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("English meanings differ", result.stderr)
        self.assertEqual(self.read("table", "falloir")["tables"][0]["meanings"]["status"], "unavailable")
        coverage = json.loads((self.output / "report.json").read_text(encoding="utf-8"))["english_coverage"]
        self.assertEqual(coverage["available"] + len(coverage["unavailable"]), coverage["verbs"])
        self.assertIn("avenir", coverage["unavailable"])

    def test_sommes_preserves_both_lemmas_and_all_slots(self):
        result = self.read("query", "sommes")
        self.assertEqual([(r["lemma"], r["tense"], r["person"], r["number"])
                          for r in result["matches"]],
                         [("sommer", "P", "2", "s"),
                          ("sommer", "S", "2", "s"), ("être", "P", "1", "p")])
        highlighted = [c for t in result["tables"] for c in t["cells"] if c["matched"]]
        self.assertEqual(len(highlighted), 3)
        self.assertTrue(all(c["form"] == "sommes" for c in highlighted))

    def test_ordinary_compounds_cover_each_block_and_whole_subject_expressions(self):
        expected = {"P": "aimons", "PC": "avons aimé", "I": "aimions", "PQP": "avions aimé",
                    "J": "aimâmes", "PA": "eûmes aimé", "F": "aimerons", "FA": "aurons aimé",
                    "C": "aimerions", "CP": "aurions aimé", "S": "aimions", "SP": "ayons aimé",
                    "T": "aimassions", "SQP": "eussions aimé", "Y": "aime", "YP": "aie aimé",
                    "W": "aimer", "WP": "avoir aimé", "G": "aimant", "K": "aimé",
                    "KP": "ayant aimé", "GP": "en aimant", "GPP": "en ayant aimé"}
        cells = self.read("table", "aimer")["tables"][0]["cells"]
        for tense, form in expected.items():
            with self.subTest(tense=tense):
                self.assertTrue(any(c["tense"] == tense and c["form"] == form for c in cells))
        for query in ("avons aimé", "nous avons aimé", "que nous ayons aimé", "ayant aimé", "en ayant aimé"):
            result = self.read("query", query)
            self.assertIn("aimer", [t["lemma"] for t in result["tables"]])
            self.assertTrue(all(c["form"] == query or query in c["expressions"] for c in result["matches"]))
        for query in ("je avons aimé", "avons aimé inconnu", "avons eu aimé", "elle est allé"):
            self.assertEqual(self.read("query", query)["status"], "not_found")
        for query in ("je suis allé", "je suis allée", "elle est allée", "vous êtes allées"):
            self.assertIn("aller", [t["lemma"] for t in self.read("query", query)["tables"]])

    def test_compound_interpretations_and_lexically_licensed_agreement(self):
        matches = self.read("query", "aie aimé")["matches"]
        self.assertEqual({(c["tense"], c["person"]) for c in matches}, {("SP", "1"), ("YP", "2")})
        for query, lemma in (("ai descendu", "descendre"), ("suis descendu", "descendre"),
                             ("suis rentré", "rentrer"), ("ai rentré", "rentrer")):
            table = next(t for t in self.read("query", query)["tables"] if t["lemma"] == lemma)
            self.assertTrue(any(c["matched"] for c in table["cells"]))
        # The admitted syntactic frames preserve both ordinary uses.
        verb = self.read("table", "aboyer")["tables"][0]["verb"]
        self.assertEqual(verb["auxiliaries"], [{"lemma": "avoir", "use": "intransitive; transitive"}])
        self.assertTrue(verb["source"]["syntax"]["entries"])

    def test_syntactic_source_closes_rare_and_multiple_auxiliary_gaps(self):
        for query, lemma in (("j’ai abélisé", "abéliser"), ("nous avons abondé", "abonder"),
                             ("j’ai différentié", "différentier"), ("j’ai doué", "douer"),
                             ("je suis monté", "monter"), ("j’ai monté", "monter"),
                             ("je suis remontée", "remonter"), ("j’ai remonté", "remonter")):
            with self.subTest(query=query):
                result = self.read("query", query)
                self.assertIn(lemma, [t["lemma"] for t in result["tables"]])
                verb = next(t["verb"] for t in result["tables"] if t["lemma"] == lemma)
                self.assertTrue(verb["source"]["syntax"]["entries"])
        for lemma in ("agenouiller", "abstenir"):
            verb = self.read("table", lemma)["tables"][0]["verb"]
            self.assertEqual(verb["ordinary_status"], "not_applicable")
        self.assertEqual(self.read("query", "j’ai agenouillé")["status"], "not_found")
        for query in ("j’abstiens", "j’agenouille", "je voilà"):
            self.assertEqual(self.read("query", query)["status"], "not_found")
        for query, lemma in (("je suis parvenue", "parvenir"), ("elle est née", "naître"), ("je suis resté", "rester")):
            self.assertIn(lemma, [t["lemma"] for t in self.read("query", query)["tables"]])
        for query in ("j’ai parvenu", "j’ai né", "j’ai resté"):
            self.assertEqual(self.read("query", query)["status"], "not_found")
        for query, lemma in (("j’ai démoralisé", "démoraliser"), ("j’ai scandalisé", "scandaliser"),
                             ("j’ai évaporé", "évaporer"), ("j’ai métabolisé", "métaboliser"), ("j’ai ranci", "rancir")):
            self.assertIn(lemma, [t["lemma"] for t in self.read("query", query)["tables"]])

    def test_full_resource_has_no_unresolved_ordinary_block_admission(self):
        report = json.loads((self.output / "report.json").read_text(encoding="utf-8"))
        for code, counts in report["block_coverage"].items():
            with self.subTest(block=code):
                self.assertEqual(sum(counts.values()), 7826)
                self.assertEqual(counts.get("unconfirmed", 0), 0)
                self.assertEqual(counts.get("missing", 0), 0)
        for lemma in ("accroire", "quérir", "ravoir", "gésir"):
            verb = self.read("table", lemma)["tables"][0]["verb"]
            self.assertEqual(verb["blocks"]["PC"], "not_applicable")

    def test_subject_elision_uses_lexical_h_and_y_evidence(self):
        for query, lemma in (("j’habite", "habiter"), ("je hais", "haïr"),
                             ("je hurle", "hurler"), ("je harcèle", "harceler"),
                             ("je hèle", "héler"), ("je yorubaïse", "yorubaïser")):
            with self.subTest(query=query):
                self.assertIn(lemma, [t["lemma"] for t in self.read("query", query)["tables"]])
        for query in ("je habite", "j’hais", "j’hurle", "j’harcèle", "j’hèle", "j’yorubaïse"):
            self.assertEqual(self.read("query", query)["status"], "not_found")

    def test_positively_attested_rare_forms_override_source_omissions_with_use_labels(self):
        for query, lemma, tense in (("j’occirai", "occire", "F"), ("j’occirais", "occire", "C"),
                                    ("je suis chue", "choir", "PC"), ("ayons chu", "choir", "YP"),
                                    ("il est avenu", "avenir", "PC"), ("elles sont avenues", "avenir", "PC"),
                                    ("il aviendra", "avenir", "F"), ("aie occis", "occire", "YP"),
                                    ("il stupéfait", "stupéfaire", "P"), ("aie stupéfait", "stupéfaire", "YP"),
                                    ("aie forclos", "forclore", "YP")):
            with self.subTest(query=query):
                matches = self.read("query", query)["matches"]
                self.assertTrue(any(c["lemma"] == lemma and c["tense"] == tense for c in matches))
        self.assertEqual({(c["tense"], c["person"], c["number"]) for c in self.read("query", "aie occis")["matches"]},
                         {("SP", "1", "s"), ("YP", "2", "s")})
        tables = self.read("query", "j’ai pu")["tables"]
        self.assertEqual({t["lemma"] for t in tables}, {"paître", "pouvoir"})
        verb = next(t["verb"] for t in tables if t["lemma"] == "paître")
        self.assertIn("falconry_participle", verb["usage_labels"])
        self.assertEqual(verb["auxiliaries"][0]["use"], "falconry")
        self.assertEqual(self.read("query", "je suis avenu")["status"], "not_found")

    def test_syntax_inputs_and_stale_converters_fail_without_replacing_output(self):
        source = Path(self.temporary.name) / "syntax"
        shutil.copytree(self.output / "lefff-syntax", source)
        for name in ("v_new.ilex.gz", "morpho.fr", "rosaenlg-hmuet.ts", "provenance.json", "LICENSE"):
            path = source / name
            original = path.read_bytes()
            path.write_bytes(b"corrupt")
            output = Path(self.temporary.name) / "bad-syntax"
            result = run_cli("build", output, "--syntax-source", source)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("Conjugation resource error:", result.stderr)
            self.assertFalse(output.exists())
            path.write_bytes(original)
        stale = Path(self.temporary.name) / "stale-converter"
        shutil.copytree(self.output, stale)
        (stale / "conjugation_generation.py").write_text("stale", encoding="utf-8")
        self.assertIn("converter differs", run_cli("prepare", stale).stderr)
        self.assertEqual((stale / "conjugation_generation.py").read_text(), "stale")

    def test_prepare_reuses_the_resource_and_refuses_damaged_output(self):
        prepared = Path(self.temporary.name) / "prepared"
        result = run_cli("prepare", prepared)
        self.assertEqual(result.returncode, 0, result.stderr)
        for name in ("conjugation_sample.py", "conjugation_metadata.py", "conjugation_generation.py",
                     "conjugation_syntax.py", "conjugation_meanings.py", "conjugation_pronominal.py",
                     "conjugation_runtime.py"):
            self.assertEqual((prepared / name).read_bytes(), CLI.with_name(name).read_bytes())
        before = {p.relative_to(self.output): p.read_bytes() for p in self.output.rglob("*") if p.is_file()}
        result = run_cli("prepare", self.output)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(json.loads(result.stdout)["database_bytes"], json.loads((self.output / "report.json").read_text())["database_bytes"])
        self.assertEqual({p.relative_to(self.output): p.read_bytes() for p in self.output.rglob("*") if p.is_file()}, before)
        broken = Path(self.temporary.name) / "damaged-prepare"
        shutil.copytree(self.output, broken)
        with (broken / "conjugation.sqlite3").open("ab") as stream:
            stream.write(b"damaged")
        result = run_cli("prepare", broken)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("checksum", result.stderr)
        self.assertTrue((broken / "conjugation.sqlite3").read_bytes().endswith(b"damaged"))

    def test_traditional_groups_and_ordinary_auxiliaries_have_source_evidence(self):
        # Literal expectations independently checked against the cited grammar tables.
        for lemma, group, auxiliaries in [
            ("aimer", 1, ["avoir"]), ("finir", 2, ["avoir"]),
            ("aller", 3, ["être"]), ("partir", 3, ["être"]),
            ("avoir", 3, ["avoir"]), ("être", 3, ["avoir"]),
            ("descendre", 3, ["avoir", "être"]), ("falloir", 3, ["avoir"]),
        ]:
            with self.subTest(lemma=lemma):
                verb = self.read("table", lemma)["tables"][0]["verb"]
                self.assertEqual(verb["group"], group)
                self.assertEqual(verb["group_status"], "supported")
                self.assertEqual([a["lemma"] for a in verb["auxiliaries"]], auxiliaries)
                self.assertEqual(verb["auxiliary_status"], "supported")
                self.assertTrue(verb["source"]["raw_type"])
        descendre = self.read("table", "descendre")["tables"][0]["verb"]
        self.assertEqual([a["use"] for a in descendre["auxiliaries"]],
                         ["transitive", "intransitive"])
        self.assertTrue(descendre["correction"]["references"])

    def test_invalid_metadata_inputs_fail_before_publishing_and_preserve_valid_output(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            source = directory / "metadata"
            shutil.copytree(VERB_RESOURCE, source)
            output = directory / "output"
            for name in ("conj_data.json.gz", "corrections.json", "provenance.json", "LICENSE-GPL3", "README.md"):
                path = source / name
                original = path.read_bytes()
                for content in (None, b"corrupt", original[:16]):
                    with self.subTest(name=name, content=content is None):
                        if content is None:
                            path.unlink()
                        else:
                            path.write_bytes(content)
                        result = run_cli("build", output, "--metadata-source", source)
                        self.assertNotEqual(result.returncode, 0)
                        self.assertIn("Conjugation resource error:", result.stderr)
                        self.assertEqual(result.stdout, "")
                        self.assertFalse(output.exists())
                        path.write_bytes(original)
            result = run_cli("build", self.output, "--metadata-source", source)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(self.read("table", "finir")["tables"][0]["verb"]["group"], 2)

    def test_unknown_metadata_is_not_guessed_and_pronominal_only_is_not_ordinary(self):
        pronominal = self.read("table", "abstenir")["tables"][0]["verb"]
        self.assertEqual(pronominal["ordinary_status"], "not_applicable")
        self.assertEqual(pronominal["auxiliary_status"], "not_applicable")
        self.assertEqual(pronominal["auxiliaries"], [])
        self.assertIn("pronominal_only", pronominal["usage_labels"])
        unknown = self.read("table", "voilà")["tables"][0]
        self.assertTrue(unknown["cells"])
        self.assertIsNone(unknown["verb"]["group"])
        self.assertEqual(unknown["verb"]["group_status"], "unconfirmed")
        self.assertIsNone(unknown["verb"]["source"])
        incomplete = self.read("table", "aboyer")["tables"][0]["verb"]
        self.assertEqual(incomplete["auxiliary_status"], "supported")
        self.assertEqual(incomplete["auxiliaries"], [{"lemma": "avoir", "use": "intransitive; transitive"}])
        ambiguous = self.read("table", "ressortir")["tables"][0]["verb"]
        self.assertIsNone(ambiguous["group"])
        self.assertEqual(ambiguous["group_status"], "unconfirmed")
        self.assertEqual([a["lemma"] for a in ambiguous["auxiliaries"]], ["avoir", "être"])
        self.assertEqual(ambiguous["auxiliary_status"], "supported")
        coverage = json.loads((self.output / "report.json").read_text(encoding="utf-8"))["metadata_coverage"]
        self.assertEqual(coverage["verbs"], 7826)
        for field in ("group_status", "auxiliary_status", "ordinary_status"):
            self.assertEqual(sum(coverage[field].values()), 7826)
        self.assertIn("voilà", coverage["missing_lemmas"])
        self.assertEqual(coverage["excluded_source_rows"], 0)

    def test_incompatible_incomplete_and_changed_metadata_resources_fail_visibly(self):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            path = directory / "resource.sqlite3"
            shutil.copytree(self.output / "grammalecte-2.3.0", directory / "grammalecte-2.3.0")
            shutil.copytree(self.output / "lefff-syntax", directory / "lefff-syntax")
            shutil.copytree(self.output / "freedict-fra-eng-0.4.1", directory / "freedict-fra-eng-0.4.1")
            malformed = self.read("table", "aimer")["tables"][0]["verb"]
            malformed["auxiliaries"] = [{"lemma": "être"}]
            for statement, parameters in [
                ("PRAGMA user_version = 999", ()),
                ("DELETE FROM verbs WHERE lemma = ?", ("aimer",)),
                ("UPDATE verbs SET information = ? WHERE lemma = ?", (zlib.compress(json.dumps(malformed).encode()), "aimer")),
                ("UPDATE verbs SET information = ? WHERE lemma = ?", (zlib.compress(b"null"), "aimer")),
                ("UPDATE verbs SET information = ? WHERE lemma = ?", ("null", "aimer")),
                ("UPDATE verbs SET information = ? WHERE lemma = ?", (b"not zlib", "aimer")),
                ("UPDATE verbs SET source_lines = X'' WHERE lemma = ?", ("aimer",)),
                ("UPDATE models SET cells = 'null' WHERE id = (SELECT model FROM verbs WHERE lemma = ?)", ("aimer",)),
            ]:
                with self.subTest(statement=statement):
                    shutil.copyfile(self.database, path)
                    with sqlite3.connect(path) as connection:
                        connection.execute(statement, parameters)
                    connection.close()
                    result = run_cli("table", path, "aimer")
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn("Conjugation resource error:", result.stderr)
                    self.assertEqual(result.stdout, "")

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
                self.assertEqual(table["status"], "source_only" if lemma == "enfuir" else "ordinary_active")
                self.assertTrue(any(c["form"] == form and c["tense"] == tense for c in table["cells"]))
        falloir = self.read("table", "falloir")["tables"][0]["cells"]
        self.assertFalse(any(c["person"] in ("1", "2") for c in falloir))
        self.assertEqual(self.read("table", "s'enfuir")["status"], "not_found")
        self.assertEqual(self.read("query", "avons aimé")["status"], "found")

    def test_unspecified_agreement_and_tags_are_not_invented(self):
        ete = [r for r in self.read("query", "été")["matches"] if r["raw_tag"] == "K"]
        self.assertEqual([(r["raw_tag"], r["gender"], r["number"]) for r in ete], [("K", None, None)])
        unresolved = self.read("query", "voilà")["matches"]
        self.assertEqual([(r["raw_tag"], r["tense"]) for r in unresolved], [("", None)])

    def test_spelling_variants_share_a_slot_without_overwriting(self):
        table = self.read("table", "payer")["tables"][0]["cells"]
        self.assertEqual([c["form"] for c in table if c["tense"] == "P"
                          and c["person"] == "1" and c["number"] == "s"], ["paie", "paye"])

    def test_missing_forms_and_accent_distinctions(self):
        for form in ("lexique-inexistant", "ete", "x' OR 1=1 --", "  J'AI  AIME  ",
                     " JE  AVONS AIMÉ ", " JE SE LAVE ", "ELLES SE SONT ABSTENUS", " AIMÉ AVONS "):
            result = self.read("query", form)
            self.assertEqual(result["status"], "not_found")
            self.assertEqual(result["query"], form)
            self.assertEqual(result["matches"], [])

    def test_invalid_queries_are_rejected_before_resource_lookup(self):
        for entered in ("", " ", "\u00a0\u202f", "a\n", "a\x7f", "a\u0085", "a" * 513, "é" * 257):
            with self.subTest(query=entered):
                result = run_cli("query", self.output / "missing.sqlite3", entered)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("Lookup must", result.stderr)
        entered = "é" * 256
        result = self.read("query", entered)
        self.assertEqual(result["status"], "not_found")
        self.assertEqual(result["query"], entered)

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
                connection.execute("UPDATE verbs SET slots = slots - 1 WHERE lemma = 'sommer'")
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
        self.assertEqual(report["ordinary_lemmas"], 7826)
        self.assertEqual(report["lemmas"], 7826 + report["pronominal_coverage"]["identities"])
        self.assertEqual(report["database_bytes"], report["table_bytes"]
                         + report["lemma_index_bytes"] + report["reverse_index_bytes"])
        for name in ("LICENSE-LGPLLR", "provenance.json", "lefff-3.4.mlex.gz", "README.md"):
            self.assertTrue((self.output / name).is_file())

    def test_rebuild_is_reproducible_and_contains_complete_redistribution_materials(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "rebuilt"
            result = run_cli("build", output)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual((output / "conjugation.sqlite3").read_bytes(), self.database.read_bytes())
            self.assertEqual(json.loads(result.stdout), json.loads((self.output / "report.json").read_text(encoding="utf-8")))
            for name in ("conj_data.json.gz", "LICENSE-GPL3", "provenance.json", "conj.py",
                         "corrections.json", "upstream-README.txt", "README.md"):
                self.assertEqual((output / "grammalecte-2.3.0" / name).read_bytes(),
                                 (VERB_RESOURCE / name).read_bytes())
            for path in (CLI.parent.parent / "resources/lefff-syntax").iterdir():
                self.assertEqual((output / "lefff-syntax" / path.name).read_bytes(), path.read_bytes())
            for path in (CLI.parent.parent / "resources/freedict-fra-eng-0.4.1").iterdir():
                self.assertEqual((output / "freedict-fra-eng-0.4.1" / path.name).read_bytes(), path.read_bytes())
            self.assertEqual((output / "conjugation_meanings.py").read_bytes(), CLI.with_name("conjugation_meanings.py").read_bytes())
            self.assertEqual((output / "conjugation_pronominal.py").read_bytes(), CLI.with_name("conjugation_pronominal.py").read_bytes())


if __name__ == "__main__":
    unittest.main()
