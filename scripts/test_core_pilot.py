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
    def test_second_content_batch_keeps_frozen_targets_and_exposes_unresolved_adjective(self):
        expected = [
            ("pour", "NOM"), ("pour", "PRE"), ("des", "ART:ind"),
            ("dire", "NOM"), ("dire", "VER"), ("pouvoir", "NOM"),
            ("pouvoir", "VER"), ("vouloir", "VER"), ("mais", "CON"),
            ("me", "PRO:per"), ("nous", "PRO:per"), ("dans", "PRE"),
            ("savoir", "NOM"), ("savoir", "VER"), ("elle", "PRO:per"),
            ("du", "ART:def"), ("y", "NOM"), ("y", "PRO:per"),
            ("bien", "ADJ"), ("bien", "ADV"), ("bien", "NOM"),
            ("où", "PRO:int"), ("où", "PRO:rel"), ("t'", "PRO:per"),
            ("voir", "VER"), ("plus", "ADV"), ("plus", "NOM"),
            ("non", "ADV"), ("te", "PRO:per"), ("mon", "ADJ:pos"),
            ("au", "ART:def"), ("avec", "PRE"), ("moi", "NOM"),
            ("moi", "PRO:per"), ("si", "ADV"), ("si", "CON"),
            ("quoi", "ADJ"), ("quoi", "PRO:int"), ("quoi", "PRO:rel"),
            ("devoir", "NOM"), ("devoir", "VER"), ("s'", "CON"),
            ("s'", "PRO:per"), ("oui", "ADV"), ("ils", "PRO:per"),
            ("comme", "ADV"), ("comme", "CON"), ("se", "PRO:per"),
            ("venir", "VER"), ("sur", "ADJ"),
        ]
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "prepared"
            report = prepare(output)
            with closing(sqlite3.connect(output / "core.sqlite3")) as connection:
                batch = [json.loads(row[0]) for row in connection.execute(
                    "SELECT payload FROM senses WHERE frozen_row BETWEEN 51 AND 100 ORDER BY frozen_row,sense_index")]
            self.assertEqual([(sense["headword"], sense["pos"]) for sense in batch],
                             [group for group in expected if group != ("quoi", "ADJ")])
            self.assertTrue(all(sense[field].strip() for sense in batch for field in FIELD_NAMES.values()))
            content_batch = next(batch for batch in report["content_batches"] if batch["issue"] == 82)
            self.assertEqual((content_batch["target_groups"], content_batch["complete_groups"],
                              content_batch["complete_fields"], content_batch["reused_pilot_groups"], content_batch["new_groups"]),
                             (50, 49, 196, 2, 47))
            self.assertEqual([(group["headword"], group["pos"]) for group in content_batch["groups"]], expected)
            self.assertEqual(content_batch["scope_acceptance"], "incomplete")
            unresolved = next(group for group in content_batch["groups"] if group["frozen_row"] == 87)
            self.assertEqual(unresolved["complete_fields"], 0)
            self.assertEqual(unresolved["incomplete_fields"], list(FIELD_NAMES))
            self.assertEqual(unresolved["everyday_sense_status"], "unresolved")
            self.assertEqual(report["admission"]["accepted_groups"], 104)
            self.assertEqual(report["admission"]["complete_fields"], 416)
            self.assertEqual(report["admission"]["remaining_unaccepted_groups"], 3983)

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
                             (81, 50, 49, 200, 5, 45))
            self.assertEqual([(group["headword"], group["pos"]) for group in content_batch["groups"]], expected)

    def test_missing_duplicated_or_wrongly_bound_unresolved_fields_fail_preparation(self):
        for change in ("missing", "duplicated", "wrong_group", "wrong_field", "unexplained"):
            with self.subTest(change=change), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                source = root / "inputs"
                shutil.copytree(RESOURCE, source)
                review = json.loads((source / "field-review.json").read_text(encoding="utf-8"))
                unresolved = review["unresolved_fields"]
                if change == "missing":
                    unresolved.pop()
                elif change == "duplicated":
                    unresolved[1] = unresolved[0]
                elif change == "wrong_group":
                    unresolved[0]["frozen_row"] = 88
                elif change == "wrong_field":
                    unresolved[0]["field"] = "cefr"
                else:
                    unresolved[0]["reason"] = ""
                (source / "field-review.json").write_text(json.dumps(review, ensure_ascii=False), encoding="utf-8")
                with self.assertRaisesRegex(ValueError, "(?i)unresolved"):
                    prepare(root / "output", source_dir=source)
                self.assertFalse((root / "output/core.sqlite3").exists())

    def test_approved_ca_noun_exception_completes_acceptance_without_everyday_coverage(self):
        inputs = json.loads((RESOURCE / "curated-inputs.json").read_text(encoding="utf-8"))
        noun = next(group for group in inputs["groups"]
                    if (group["headword"], group["lexique_cgram"]) == ("ça", "NOM"))
        self.assertEqual(noun["senses"][0]["sense_id"], "ca-noun-psychoanalytic-id")
        with tempfile.TemporaryDirectory() as temporary:
            report = prepare(Path(temporary) / "prepared")
            self.assertEqual(report["content_batches"][0].get("accepted_groups"), 50,
                             "The approved specialist exception completes batch acceptance")
            self.assertEqual(report["content_batches"][0]["approved_specialist_exception_groups"], 1)
            self.assertEqual(report["content_batches"][0]["scope_acceptance"], "passed")
            self.assertEqual(report["content_batches"][0]["complete_groups"], 49,
                             "Specialist ça/NOM does not satisfy the everyday-sense requirement")
            batch_noun = next(group for group in report["content_batches"][0]["groups"]
                              if group["frozen_row"] == 34)
            self.assertEqual(batch_noun["complete_fields"], 4)
            self.assertEqual(batch_noun["everyday_sense_status"], "approved_specialist_exception")
            self.assertEqual(batch_noun["specialist_exception_decision"]["issue"],
                             "https://github.com/zardonc/lexique/issues/81")
            self.assertEqual(batch_noun["incomplete_requirements"], ["modern_everyday_sense"])
            coverage = json.loads((Path(temporary) / "prepared/coverage.json").read_text(encoding="utf-8"))
            gap = next(group for group in coverage["incomplete_groups"] if group["frozen_row"] == 34)
            self.assertEqual((gap["word"], gap["lexique_cgram"]), ("ça", "NOM"))
            self.assertEqual(gap["incomplete_fields"], [])
            self.assertEqual(gap["incomplete_requirements"], ["modern_everyday_sense"])
            self.assertIn("psychoanalytic", gap["reason"])
            self.assertEqual(coverage["accepted_groups"], 104)
            self.assertEqual(coverage["remaining_unaccepted_groups"], 3983)
            self.assertEqual(coverage["approved_specialist_exception_groups"], 1)
            self.assertEqual(report["admission"]["everyday_sense_review"], "incomplete")
            self.assertEqual(report["admission"]["selected_scope_acceptance"], "passed")
            self.assertEqual(report["admission"]["accepted_groups"], 104)
            self.assertEqual(report["first_release"], "incomplete")

    def test_missing_unapproved_or_wrongly_bound_specialist_exception_fails_admission(self):
        for change in ("missing", "unapproved", "wrong_issue", "wrong_binding", "wrong_group", "unexplained"):
            with self.subTest(change=change), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                source = root / "inputs"
                shutil.copytree(RESOURCE, source)
                review = json.loads((source / "field-review.json").read_text(encoding="utf-8"))
                usage = next(row for row in review["usage"] if row["sense_id"] == "ca-noun-psychoanalytic-id")
                if change == "missing":
                    del usage["specialist_exception_decision"]
                elif change == "unapproved":
                    usage["specialist_exception_decision"]["approved_by"] = "reviewer"
                elif change == "wrong_issue":
                    usage["specialist_exception_decision"]["issue"] = "https://github.com/zardonc/lexique/issues/78"
                elif change == "wrong_binding":
                    usage["frozen_row"] = 35
                elif change == "wrong_group":
                    pronoun = next(row for row in review["usage"] if row["frozen_row"] == 35)
                    pronoun["everyday_sense_status"] = "approved_specialist_exception"
                    pronoun["everyday_sense_reason"] = usage["everyday_sense_reason"]
                    pronoun["specialist_exception_decision"] = usage["specialist_exception_decision"]
                else:
                    del usage["specialist_exception_decision"]["decision"]
                (source / "field-review.json").write_text(json.dumps(review, ensure_ascii=False), encoding="utf-8")
                with self.assertRaisesRegex(ValueError, "specialist exception"):
                    prepare(root / "output", source_dir=source)
                self.assertFalse((root / "output/core.sqlite3").exists())

    def test_missing_invalid_or_unexplained_everyday_review_fails_admission(self):
        for change in ("missing", "invalid", "unexplained"):
            with self.subTest(change=change), tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                source = root / "inputs"
                shutil.copytree(RESOURCE, source)
                review = json.loads((source / "field-review.json").read_text(encoding="utf-8"))
                usage = next(row for row in review["usage"] if row["sense_id"] == "ca-noun-psychoanalytic-id")
                if change == "missing":
                    del usage["everyday_sense_status"]
                elif change == "invalid":
                    usage["everyday_sense_status"] = "pending"
                else:
                    del usage["everyday_sense_reason"]
                (source / "field-review.json").write_text(json.dumps(review, ensure_ascii=False), encoding="utf-8")
                with self.assertRaisesRegex(ValueError, "everyday-sense"):
                    prepare(root / "output", source_dir=source)
                self.assertFalse((root / "output/core.sqlite3").exists())

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
            self.assertEqual(report["admission"]["complete_groups"], 103)
            self.assertEqual(report["admission"]["complete_fields"], 416)
            self.assertEqual(report["admission"]["remaining_groups"], 3984)
            self.assertFalse(report["learner_acceptance"]["accepted"])
            connection = sqlite3.connect((root / "first/core.sqlite3").as_uri() + "?mode=ro", uri=True)
            with connection:
                senses = [json.loads(row[0]) for row in connection.execute("SELECT payload FROM senses ORDER BY frozen_row,sense_index")]
                self.assertEqual([row["frozen_row"] for row in senses], [*range(1, 87), *range(88, 101), 360, 422, 423, 483, 1911])
                self.assertEqual([row["pos"] for row in senses[:5]], ["AUX", "NOM", "VER", "AUX", "VER"])
                self.assertIn("all pronominal verbs", senses[0]["meaning_en"])
                self.assertIn("past participle", senses[3]["meaning_en"])
                self.assertIn("être", senses[3]["meaning_en"])
                self.assertEqual(dict(connection.execute("SELECT query,headword FROM aliases")),
                                 {"cœur": "coeur", "aujourd’hui": "aujourd'hui", "l’": "l'", "d’": "d'", "t’": "t'", "s’": "s'"})
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
            self.assertEqual(len(coverage["incomplete_groups"]), 3984)
            self.assertTrue(all(set(group["incomplete_fields"]) == set(FIELD_NAMES)
                                for group in coverage["incomplete_groups"] if group["frozen_row"] != 34))
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
