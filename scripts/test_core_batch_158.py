"""Check #158 at the approved resource/admission boundary."""
import csv
import json
from pathlib import Path
import shutil
import tempfile
import unittest
from urllib.parse import parse_qs, urlparse

from core_batch_test_helpers import verify_archived_source_captures
from core_pilot import load_inputs, MATERIALS

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-158"
MEMBERSHIP = [('majeur', 'ADJ'), ('correctement', 'ADV'), ('dizaine', 'NOM'), ('transfert', 'NOM'), ('obscur', 'ADJ'), ('brouillard', 'NOM'), ('délai', 'NOM'), ('quitte', 'ADJ'), ('recouvrir', 'VER'), ('retomber', 'VER'), ('champignon', 'NOM'), ('contrarier', 'VER'), ('poursuite', 'NOM'), ('confus', 'ADJ'), ('peso', 'NOM'), ('grille', 'NOM'), ('spatial', 'ADJ'), ('chameau', 'NOM'), ('probable', 'ADJ'), ('crayon', 'NOM'), ('gâter', 'VER'), ('adversaire', 'NOM'), ('aimé', 'ADJ'), ('loyal', 'ADJ'), ('surgir', 'VER'), ('atroce', 'ADJ'), ('espionner', 'VER'), ('look', 'NOM'), ('g', 'NOM'), ('balai', 'NOM'), ('canne', 'NOM'), ('désagréable', 'ADJ'), ('modifier', 'VER'), ('testament', 'NOM'), ('cm', 'NOM'), ('convoquer', 'VER'), ('minimum', 'NOM'), ('pétrin', 'NOM'), ('caution', 'NOM'), ('destination', 'NOM'), ('messager', 'NOM'), ('t-shirt', 'NOM'), ('trottoir', 'NOM'), ('bombarder', 'VER'), ('processus', 'NOM'), ('stress', 'NOM'), ('pincer', 'VER'), ('boeuf', 'NOM'), ('sportif', 'ADJ'), ('condoléance', 'NOM')]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch158Tests(unittest.TestCase):
    def test_real_admission_rejects_an_author_review_without_independent_acceptance(self):
        temporary_root = ROOT / "target/issue-158"
        temporary_root.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="admission-158-", dir=temporary_root) as folder:
            staged = Path(folder)
            for name in (*MATERIALS, "input-checksums.json"):
                shutil.copy2(ROOT / "resources/core-pilot-79" / name, staged / name)
            shutil.copy2(EVIDENCE / "field-review.json", staged / "field-review.json")
            with self.assertRaisesRegex(ValueError, "A separate independent field review is required"):
                load_inputs(staged)

    def test_fixed_source_bytes_spans_frames_and_complete_contributors(self):
        captures = read("source-captures.json")
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        archive = Path(captures["capture_root"])
        def resolve(value):
            path = Path(value)
            self.assertTrue(path.is_relative_to(archive), value)
            self.assertTrue(path.is_file(), value)
            return path
        raw = verify_archived_source_captures(self, captures["sources"], sources, resolve)
        excerpts = read("source-excerpts.json")
        selected = {(r["sense_id"], r["field"]): r for r in excerpts["fields"]}
        self.assertEqual(len(selected), 200)
        for record in [*excerpts["fields"], *excerpts["supporting_evidence"]]:
            line = raw[record["source_id"]][record["line_1_based"] - 1]
            column = record["column_1_based"] - 1
            self.assertEqual(line[column:column + len(record["reviewed_excerpt"])],
                             record["reviewed_excerpt"])
            self.assertEqual(record["revid"], sources[record["source_id"]]["revid"])
            self.assertNotRegex(record["reviewed_excerpt"].lstrip(), r"^#+\*")
        pos_frames = {"NOM": "nom", "VER": "verbe", "ADJ": "adjectif", "ADV": "adverbe"}
        languages = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
        for group in read("groups.json"):
            sense = group["senses"][0]
            for name, field in sense["fields"].items():
                exact = selected[sense["sense_id"], name]
                self.assertEqual(field["source_id"], exact["source_id"])
                self.assertEqual(field["language"], languages[name])
                self.assertEqual(field["source_span"]["raw_source_excerpt"], exact["reviewed_excerpt"])
                before = raw[exact["source_id"]][:exact["line_1_based"]]
                language = next(line for line in reversed(before) if line.startswith("== "))
                self.assertRegex(language, r"^==\s*\{\{langue\|fr\}\}\s*==$")
                if name == "ipa":
                    self.assertEqual(field["text"].strip("/"), exact["ipa_variant_excerpt"])
                    column = exact["ipa_variant_column_1_based"] - 1
                    self.assertEqual(exact["reviewed_excerpt"][column:column + len(exact["ipa_variant_excerpt"])],
                                     exact["ipa_variant_excerpt"])
                else:
                    pos = next(line for line in reversed(before) if line.startswith("=== {{S|"))
                    self.assertIn("{{S|" + pos_frames[group["lexique_cgram"]] + "|fr", pos)
        names, anonymous, chains = {}, {}, {}
        for value in captures["contributor_registry"]["parts"]:
            path = resolve(value)
            data = json.loads(path.read_text(encoding="utf-8"))
            prefix, number = path.name.removesuffix(".api.json").rsplit("-", 1)
            chains.setdefault(prefix, {})[int(number)] = (path, data)
            for page in data["query"]["pages"]:
                names.setdefault(page["title"], set()).update(p["name"] for p in page.get("contributors", []))
                if "anoncontributors" in page:
                    anonymous[page["title"]] = page["anoncontributors"]
        for parts in chains.values():
            self.assertEqual(sorted(parts), list(range(1, len(parts) + 1)))
            for number, (_, data) in parts.items():
                continuation = data.get("continue")
                if number == len(parts):
                    self.assertFalse(continuation)
                else:
                    self.assertTrue(continuation)
                    request = resolve(str(parts[number + 1][0].with_suffix(".request.txt"))).read_text(encoding="utf-8")
                    query = parse_qs(urlparse(request).query)
                    for key, value in continuation.items():
                        self.assertEqual(query[key], [str(value)])
        for context in read("source-context.json")["current_content"]:
            self.assertEqual(context["current_contributors"], sorted(names[context["title"]]))
            self.assertEqual(context["current_registry_names"], len(names[context["title"]]))
            self.assertEqual(context["anonymous_contributors_count"], anonymous.get(context["title"], 0))
        self.assertFalse(captures["contributor_registry"]["selected_sentence_authorship_proved"])

    def test_measurement_spelling_and_plural_readings_retain_source_restrictions(self):
        groups = {g["frozen_row"]: g for g in read("groups.json")}
        support = read("source-excerpts.json")["supporting_evidence"]
        modern = {r["frozen_row"]: r for r in read("modern-corroboration.json")["records"]}
        for row in (3879, 3885):
            record = modern[row]
            self.assertEqual(record["independent_review"], "pending")
            self.assertIsNone(record["reviewer"])
            self.assertFalse(record["learner_text_copied"])
            self.assertTrue(Path(record["raw_path"]).is_file())
            body = Path(record["readable_path"]).read_text(encoding="utf-8")
            for exact in record["exact_excerpts"]:
                self.assertIn(exact, body)
        g = groups[3879]["senses"][0]
        self.assertEqual((groups[3879]["headword"], groups[3879]["lexique_cgram"]), ("g", "NOM"))
        self.assertEqual(g["fields"]["ipa"]["text"], "/ʒe/")
        self.assertIn("9,80665", g["fields"]["french_definition"]["text"])
        self.assertIn("gram symbol", g["excluded_uses"])
        self.assertTrue(any(r["frozen_row"] == 3879 and "{{S|nom|fr}}" in r["reviewed_excerpt"] for r in support))
        cm = groups[3885]["senses"][0]
        self.assertEqual((groups[3885]["headword"], groups[3885]["lexique_cgram"]), ("cm", "NOM"))
        self.assertEqual(cm["source_headword"], "centimètre")
        self.assertEqual(cm["fields"]["ipa"]["text"], "/sɑ̃.ti.mɛtʁ/")
        self.assertIn("not a lexical noun", next(r for r in read("source-context.json")["spelling_relations"]
                                                  if r["frozen_row"] == 3885)["construction_and_flexion_interpretation"])
        self.assertTrue(any(r["frozen_row"] == 3885 and "{{S|symbole|fr}}" in r["reviewed_excerpt"] for r in support))
        self.assertEqual(groups[3865]["senses"][0]["source_headword"], "péso")
        self.assertTrue(any(r["frozen_row"] == 3865 and "{{variante de|péso|fr}}" in r["reviewed_excerpt"] for r in support))
        self.assertEqual(groups[3898]["headword"], "boeuf")
        self.assertEqual(groups[3898]["senses"][0]["source_headword"], "bœuf")
        self.assertEqual(groups[3898]["senses"][0]["fields"]["ipa"]["text"], "/bœf/")
        self.assertTrue(any(r["frozen_row"] == 3898 and "Variante par contrainte typographique" in r["reviewed_excerpt"] for r in support))
        for field in ("french_definition", "english", "chinese"):
            self.assertIn("condoléances", groups[3900]["senses"][0]["fields"][field]["text"])
        self.assertTrue(any(r["frozen_row"] == 3900 and "{{généralement pluriel|fr}}" in r["reviewed_excerpt"] for r in support))
        sources = {s["title"]: s for s in read("sources.json")["sources"]}
        transfer = sources["transfert"]
        self.assertTrue(any("Import:CFC" in r["raw_reference"] and r["author_rights_proposal"] == "excluded_from_selected_content"
                            for r in transfer["external_reference_markers"]))
        for location in [*read("source-excerpts.json")["fields"], *support]:
            self.assertFalse(location["source_id"] == transfer["id"] and location["line_1_based"] in (38, 39))
        self.assertTrue(all(r["alias_admission"] == "not_admitted" for r in read("source-context.json")["spelling_relations"]))

    def test_author_rights_proposals_keep_every_field_pending_independent_review(self):
        rights, admission, review = read("rights.json"), read("admission.json"), read("field-review.json")
        self.assertEqual(rights["independent_review"], "pending")
        self.assertIsNone(rights["reviewer"])
        self.assertEqual(rights["independent_rights_fields_accepted"], 0)
        self.assertEqual(admission["independently_accepted_new_groups"], 0)
        self.assertEqual(admission["independently_accepted_new_fields"], 0)
        self.assertEqual(admission["missing_admitted_fields"], 200)
        self.assertEqual(admission["pending_independent_review_fields"], 200)
        self.assertEqual(admission["pending_independent_rights_fields"], 200)
        self.assertFalse(review["independent_acceptance"])
        self.assertIsNone(review["reviewer"])
        self.assertEqual((admission["scope_denominator_groups"], admission["proposed_groups"], admission["proposed_fields"]), (4087, 50, 200))
        self.assertEqual(admission["missing_authored_fields"], 0)
        self.assertEqual(admission["unresolved_field_blockers"], [])
        self.assertEqual(admission["pending_independent_review_rows"], list(range(3851, 3901)))
        self.assertEqual(len(review["fields"]), 200)
        for field in review["fields"]:
            self.assertEqual(field["accepted_text_quality_status"], "pending")
            self.assertIsNone(field["reviewer"])
        expected = {(g["senses"][0]["sense_id"], name): field
                    for g in read("groups.json")
                    for name, field in g["senses"][0]["fields"].items()}
        self.assertEqual({(r["sense_id"], r["field"]) for r in rights["fields"]}, set(expected))
        for record in rights["fields"]:
            field = expected[record["sense_id"], record["field"]]
            self.assertEqual(record["exact_source_span"], field["source_span"])
            self.assertEqual(record["source_id"], field["source_id"])
            self.assertEqual(record["modification"], field["modification"])
            self.assertEqual(record["license"], "CC-BY-SA-4.0")
            self.assertTrue(record["attribution"])
            self.assertEqual(record["independent_review"], "pending")
            self.assertIsNone(record["reviewer"])
        self.assertEqual((EVIDENCE / "COPYING").read_bytes(),
                         (ROOT / "resources/core-pilot-79/COPYING").read_bytes().rstrip() + b"\n")
        notices = (EVIDENCE / "NOTICES.md").read_text(encoding="utf-8")
        for source in read("sources.json")["sources"]:
            self.assertIn(source["fixed_url"], notices)
            self.assertIn(source["history_url"], notices)

    def test_exact_frozen_groups_have_four_scoped_candidate_fields(self):
        groups = read("groups.json") if (EVIDENCE / "groups.json").exists() else []
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        self.assertEqual([g["frozen_row"] for g in groups], list(range(3851, 3901)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            frozen = list(csv.DictReader(handle))
        self.assertEqual(len(frozen), 4087)
        frozen = frozen[3850:3900]
        for group, row in zip(groups, frozen, strict=True):
            for key in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                self.assertEqual(group[key], row[key])
            self.assertTrue(group["senses"])
            for sense in group["senses"]:
                self.assertTrue(sense["use"])
                self.assertIsNone(sense["cefr"])
                self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                for field in sense["fields"].values():
                    self.assertTrue(field["text"].strip())
                    self.assertTrue(field["source_id"])
                    self.assertEqual(field["independent_review"], "pending")


if __name__ == "__main__":
    unittest.main()
