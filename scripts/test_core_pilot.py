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
    def test_annotations_preserve_frozen_counts_and_batch_87_acceptance(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "prepared"
            report = prepare(output)
            admission = report["admission"]
            self.assertEqual((admission["accepted_groups"], admission["complete_fields"],
                              admission["annotation_exception_groups"]), (450, 1800, 2))
            batch = next(row for row in report["content_batches"] if row["issue"] == 87)
            self.assertEqual((batch["target_groups"], batch["accepted_groups"], batch["complete_fields"],
                              batch["annotation_exception_groups"], batch["scope_acceptance"]),
                             (50, 49, 196, 1, "passed"))
            coverage = json.loads((output / "coverage.json").read_text(encoding="utf-8"))
            annotations = [row for row in coverage["incomplete_groups"]
                           if row.get("annotation_exception_decision")]
            self.assertEqual([(row["frozen_row"], row["word"], row["lexique_cgram"])
                              for row in annotations], [(87, "quoi", "ADJ"), (305, "dont", "PRO:int")])
            self.assertTrue(all(set(row["incomplete_fields"]) == set(FIELD_NAMES) for row in annotations))
            self.assertEqual(annotations[1]["annotation_exception_decision"]["issue"],
                             "https://github.com/zardonc/lexique/issues/87")
            self.assertEqual((coverage["denominator_groups"], coverage["denominator_headwords"],
                              coverage["complete_groups"], coverage["remaining_unaccepted_groups"],
                              coverage["remaining_groups"]), (4087, 3773, 448, 3637, 3639))

    def test_annotations_reject_wrong_binding_approval_or_field_count(self):
        for change in ("missing", "duplicate", "row", "headword", "pos", "owner", "issue", "fields"):
            with self.subTest(change=change), tempfile.TemporaryDirectory() as temporary:
                source = Path(temporary) / "inputs"
                shutil.copytree(RESOURCE, source)
                review = json.loads((source / "field-review.json").read_text(encoding="utf-8"))
                annotations = review.setdefault("annotation_exceptions", [
                    {"frozen_row": row, "headword": word, "pos": pos, "admitted_fields": 0,
                     "decision": {"issue": f"https://github.com/zardonc/lexique/issues/{issue}",
                                  "approved_by": "project_owner", "approved_on": "2026-11-18",
                                  "decision": "Approved zero-field annotation; frozen denominator retained."}}
                    for row, word, pos, issue in ((87, "quoi", "ADJ", 82), (305, "dont", "PRO:int", 87))
                ])
                annotation = annotations[1]
                if change == "missing":
                    annotations.pop()
                elif change == "duplicate":
                    annotations[1] = annotations[0]
                elif change in ("row", "headword", "pos", "fields"):
                    key, value = {"row": ("frozen_row", 306), "headword": ("headword", "quoi"),
                                  "pos": ("pos", "PRO:rel"), "fields": ("admitted_fields", 4)}[change]
                    annotation[key] = value
                else:
                    key, value = {"owner": ("approved_by", "reviewer"),
                                  "issue": ("issue", "https://github.com/zardonc/lexique/issues/78")}[change]
                    annotation["decision"][key] = value
                (source / "field-review.json").write_text(json.dumps(review, ensure_ascii=False), encoding="utf-8")
                with self.assertRaisesRegex(ValueError, "(?i)annotation"):
                    prepare(Path(temporary) / "output", source_dir=source)
                self.assertFalse((Path(temporary) / "output/core.sqlite3").exists())

    def test_third_content_batch_keeps_all_frozen_groups_and_four_fields(self):
        expected = [
            ("sur", "PRE"), ("toi", "PRO:per"), ("ici", "ADV"),
            ("rien", "PRO:ind"), ("ma", "ADJ:pos"), ("lui", "PRO:per"),
            ("bon", "ADJ"), ("bon", "ADV"), ("bon", "NOM"), ("bon", "ONO"),
            ("là", "ADV"), ("là", "ONO"), ("suivre", "VER"),
            ("pourquoi", "ADV"), ("pourquoi", "CON"), ("parler", "NOM"),
            ("parler", "VER"), ("prendre", "VER"), ("cette", "ADJ:dem"),
            ("votre", "ADJ:pos"), ("quand", "CON"), ("son", "ADJ:pos"),
            ("son", "NOM"), ("alors", "ADV"), ("ton", "ADJ:pos"), ("ton", "NOM"),
            ("chose", "NOM"), ("par", "NOM"), ("par", "PRE"), ("croire", "VER"),
            ("aimer", "VER"), ("falloir", "VER"), ("comment", "ADV"),
            ("comment", "CON"), ("comment", "ONO"), ("très", "ADV"),
            ("ou", "CON"), ("passer", "VER"), ("penser", "VER"), ("aussi", "ADV"),
            ("jamais", "ADV"), ("même", "ADJ"), ("même", "ADV"),
            ("même", "PRO:ind"), ("attendre", "VER"), ("petit", "ADJ"),
            ("petit", "ADV"), ("petit", "NOM"), ("trouver", "VER"), ("laisser", "VER"),
        ]
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "prepared"
            report = prepare(output)
            batch = next(item for item in report["content_batches"] if item["issue"] == 83)
            self.assertEqual((batch["target_groups"], batch["new_groups"], batch["reused_pilot_groups"],
                              batch["complete_groups"], batch["accepted_groups"]),
                             (50, 50, 0, 49, 50))
            self.assertEqual(batch["approved_specialist_exception_groups"], 1)
            self.assertEqual(batch["complete_fields"], 200)
            self.assertEqual(batch["scope_acceptance"], "passed")
            golf = next(group for group in batch["groups"] if group["frozen_row"] == 128)
            self.assertEqual(golf["everyday_sense_status"], "approved_specialist_exception")
            self.assertEqual(golf["specialist_exception_decision"]["issue"],
                             "https://github.com/zardonc/lexique/issues/83")
            with closing(sqlite3.connect(output / "core.sqlite3")) as connection:
                senses = [json.loads(row[0]) for row in connection.execute(
                    "SELECT payload FROM senses WHERE frozen_row BETWEEN 101 AND 150 ORDER BY frozen_row,sense_index")]
            self.assertEqual([(sense["headword"], sense["pos"]) for sense in senses], expected)
            self.assertEqual([sense["frozen_row"] for sense in senses], list(range(101, 151)))
            self.assertTrue(all(sense[field].strip() for sense in senses for field in FIELD_NAMES.values()))

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
            self.assertEqual(content_batch["scope_acceptance"], "passed")
            unresolved = next(group for group in content_batch["groups"] if group["frozen_row"] == 87)
            self.assertEqual(unresolved["complete_fields"], 0)
            self.assertEqual(unresolved["incomplete_fields"], list(FIELD_NAMES))
            self.assertEqual(unresolved["everyday_sense_status"], "unresolved")
            self.assertEqual(report["admission"]["accepted_groups"], 450)
            self.assertEqual(report["admission"]["complete_fields"], 1800)
            self.assertEqual(report["admission"]["remaining_unaccepted_groups"], 3637)

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
            self.assertEqual(coverage["accepted_groups"], 450)
            self.assertEqual(coverage["remaining_unaccepted_groups"], 3637)
            self.assertEqual(coverage["approved_specialist_exception_groups"], 2)
            self.assertEqual(report["admission"]["everyday_sense_review"], "incomplete")
            self.assertEqual(report["admission"]["selected_scope_acceptance"], "passed")
            self.assertEqual(report["admission"]["accepted_groups"], 450)
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
            self.assertEqual(report["admission"]["complete_groups"], 448)
            self.assertEqual(report["admission"]["complete_fields"], 1800)
            self.assertEqual(report["admission"]["remaining_groups"], 3639)
            curated = json.loads((root / "first/curated-inputs.json").read_text(encoding="utf-8"))
            material_limits = " ".join(curated["material_limits"])
            admission = report["admission"]
            self.assertIn(f"{admission['complete_senses']} selected senses in {admission['accepted_groups']} frozen groups;",
                          material_limits)
            self.assertIn(f"{admission['complete_groups']} everyday groups", material_limits)
            self.assertIn(f"leaving {admission['remaining_unaccepted_groups']:,} of "
                          f"{admission['complete_groups'] + admission['remaining_groups']:,} groups unaccepted", material_limits)
            self.assertIn(f"{admission['complete_groups']} complete and {admission['remaining_groups']:,} incomplete groups",
                          material_limits)
            self.assertIn(f"All {admission['complete_fields']} fields", material_limits)
            self.assertFalse(report["learner_acceptance"]["accepted"])
            with closing(sqlite3.connect((root / "first/core.sqlite3").as_uri() + "?mode=ro", uri=True)) as connection:
                senses = [json.loads(row[0]) for row in connection.execute("SELECT payload FROM senses ORDER BY frozen_row,sense_index")]
                self.assertEqual([row["frozen_row"] for row in senses], [*range(1, 87), *range(88, 305), *range(306, 451), 483, 1911])
                self.assertEqual([row["pos"] for row in senses[:5]], ["AUX", "NOM", "VER", "AUX", "VER"])
                self.assertIn("all pronominal verbs", senses[0]["meaning_en"])
                self.assertIn("past participle", senses[3]["meaning_en"])
                self.assertIn("être", senses[3]["meaning_en"])
                self.assertEqual(dict(connection.execute("SELECT query,headword FROM aliases")),
                                 {"cœur": "coeur", "aujourd’hui": "aujourd'hui", "l’": "l'", "d’": "d'", "t’": "t'", "s’": "s'", "œil": "oeil", "OK": "ok"})
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
            coverage = json.loads((root / "first/coverage.json").read_text(encoding="utf-8"))
            self.assertEqual(len(coverage["incomplete_groups"]), 3639)
            self.assertTrue(all(set(group["incomplete_fields"]) == set(FIELD_NAMES)
                                for group in coverage["incomplete_groups"] if group["frozen_row"] not in {34, 128}))
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
