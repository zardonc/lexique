"""Check #156 at the approved resource/admission boundary."""
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
EVIDENCE = ROOT / "docs/evidence/issue-156"
MEMBERSHIP = [
    ("caravane", "NOM"),
    ("comté", "NOM"),
    ("fascinant", "ADJ"),
    ("survie", "NOM"),
    ("symptôme", "NOM"),
    ("frappe", "NOM"),
    ("intellectuel", "ADJ"),
    ("redescendre", "VER"),
    ("damer", "VER"),
    ("antenne", "NOM"),
    ("descente", "NOM"),
    ("potion", "NOM"),
    ("répugnant", "ADJ"),
    ("collège", "NOM"),
    ("culpabilité", "NOM"),
    ("tribu", "NOM"),
    ("évoquer", "VER"),
    ("mafia", "NOM"),
    ("orgueil", "NOM"),
    ("ramper", "VER"),
    ("fermeture", "NOM"),
    ("mouchoir", "NOM"),
    ("obséder", "VER"),
    ("gaspiller", "VER"),
    ("bail", "NOM"),
    ("précédemment", "ADV"),
    ("troupeau", "NOM"),
    ("calcul", "NOM"),
    ("coquin", "NOM"),
    ("évidence", "NOM"),
    ("plaider", "VER"),
    ("scanner", "NOM"),
    ("ya", "NOM"),
    ("honorable", "ADJ"),
    ("ressusciter", "VER"),
    ("télégramme", "NOM"),
    ("majorité", "NOM"),
    ("enfiler", "VER"),
    ("glacé", "ADJ"),
    ("insupportable", "ADJ"),
    ("graine", "NOM"),
    ("piqûre", "NOM"),
    ("tâcher", "VER"),
    ("tonnerre", "NOM"),
    ("valet", "NOM"),
    ("actuellement", "ADV"),
    ("cheminée", "NOM"),
    ("maquillage", "NOM"),
    ("dérouler", "VER"),
    ("hanter", "VER"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch156Tests(unittest.TestCase):
    def test_difficult_bindings_preserve_pos_register_spelling_and_unresolved_use(self):
        groups = {g["frozen_row"]: g for g in read("groups.json")}
        scanner = groups[3782]["senses"][0]
        self.assertEqual(scanner["fields"]["ipa"]["text"], "/ska.nɛʁ/")
        self.assertIn("medical", scanner["use"].lower())
        self.assertIn("France", groups[3764]["senses"][0]["fields"]["english"]["text"])
        self.assertIn("familièrement", groups[3779]["senses"][0]["fields"]["french_definition"]["text"].lower())
        self.assertIn("tâcher de", groups[3793]["senses"][0]["fields"]["english"]["text"])
        self.assertIn("aspirated h", groups[3800]["senses"][0]["use"])
        ya = groups[3783]
        self.assertEqual((ya["headword"], ya["lexique_cgram"]), ("ya", "NOM"))
        self.assertEqual(ya["senses"][0]["fields"]["french_definition"]["text"], "En argot, couteau.")
        self.assertEqual(read("admission.json")["author_unresolved_content_rows"], [3783])
        usage = next(u for u in read("field-review.json")["usage"] if u["frozen_row"] == 3783)
        self.assertEqual(usage["author_everyday_use_assessment"], "unresolved_modern_everyday_slang_use")
        self.assertEqual({r["frozen_row"] for r in read("field-review.json")["unresolved_fields"]}, {3783})
        self.assertEqual(len(read("field-review.json")["unresolved_fields"]), 4)
        for relation in read("source-context.json")["spelling_relations"]:
            self.assertEqual(relation["relation_role"], "source_binding_only")
            self.assertEqual(relation["alias_admission"], "not_admitted")

    def test_real_admission_rejects_an_author_review_without_independent_acceptance(self):
        temporary_root = ROOT / "target/issue-156"
        temporary_root.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="admission-156-", dir=temporary_root) as folder:
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

    def test_exact_frozen_groups_have_four_scoped_candidate_fields(self):
        groups = read("groups.json")
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        self.assertEqual([g["frozen_row"] for g in groups], list(range(3751, 3801)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            frozen = list(csv.DictReader(handle))[3750:3800]
        for group, row in zip(groups, frozen, strict=True):
            for key in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                self.assertEqual(group[key], row[key])
            self.assertTrue(group["senses"])
            for sense in group["senses"]:
                self.assertTrue(sense["use"])
                self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                for field in sense["fields"].values():
                    self.assertTrue(field["text"].strip())
                    self.assertTrue(field["source_id"])
                    self.assertEqual(field["independent_review"], "pending")


if __name__ == "__main__":
    unittest.main()
