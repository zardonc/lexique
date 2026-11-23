"""Check #153 at the approved resource/admission boundary."""
import csv
import json
from pathlib import Path
import shutil
import tempfile
import unittest
from urllib.parse import parse_qs, urlparse

from core_pilot import MATERIALS, load_inputs
from core_batch_test_helpers import verify_archived_source_captures

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-153"
MEMBERSHIP = [
    ("aborder", "VER"), ("cinquante", "ADJ:num"), ("formule", "NOM"),
    ("racine", "NOM"), ("destiner", "VER"), ("frotter", "VER"),
    ("fâché", "ADJ"), ("industrie", "NOM"), ("chômage", "NOM"),
    ("spirituel", "ADJ"), ("économiser", "VER"), ("by", "NOM"),
    ("vôtres", "PRO:pos"), ("coq", "NOM"), ("plaquer", "VER"),
    ("fasciste", "ADJ"), ("civilisation", "NOM"), ("humble", "ADJ"),
    ("mourant", "ADJ"), ("récent", "ADJ"), ("duc", "NOM"),
    ("désespéré", "ADJ"), ("pris", "ADJ"), ("baignoire", "NOM"),
    ("bourgeois", "ADJ"), ("rétablir", "VER"), ("ambiance", "NOM"),
    ("baisse", "NOM"), ("héritage", "NOM"), ("exposition", "NOM"),
    ("flatter", "VER"), ("prophète", "NOM"), ("manipuler", "VER"),
    ("mannequin", "NOM"), ("manuel", "NOM"), ("option", "NOM"),
    ("certificat", "NOM"), ("défaire", "VER"), ("régiment", "NOM"),
    ("vice", "NOM"), ("papillon", "NOM"), ("baraque", "NOM"),
    ("énormément", "ADV"), ("facteur", "NOM"), ("nid", "NOM"),
    ("volé", "ADJ"), ("cheville", "NOM"), ("corriger", "VER"),
    ("deuil", "NOM"), ("défi", "NOM"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch153Tests(unittest.TestCase):
    def test_exact_frozen_groups_have_complete_resource_records(self):
        if (EVIDENCE / "groups.json").exists():
            groups = read("groups.json")
        else:
            canonical = json.loads((ROOT / "resources/core-pilot-79/curated-inputs.json").read_text(encoding="utf-8"))
            groups = [g for g in canonical["groups"] if 3601 <= g["frozen_row"] <= 3650]
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP,
                         "The frozen #153 resource slice lacks complete batch content")
        self.assertEqual([g["frozen_row"] for g in groups], list(range(3601, 3651)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(encoding="utf-8-sig", newline="") as handle:
            frozen = list(csv.DictReader(handle))
        self.assertEqual(len(frozen), 4087)
        for group in groups:
            with self.subTest(row=group["frozen_row"]):
                row = frozen[group["frozen_row"] - 1]
                self.assertEqual((group["headword"], group["lexique_cgram"]), (row["word"], row["lexique_cgram"]))
                for frequency in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                    self.assertEqual(group[frequency], row[frequency])
                self.assertEqual(len(group["senses"]), 1)
                sense = group["senses"][0]
                self.assertTrue(sense["use"].strip())
                self.assertIsNone(sense["cefr"])
                self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                for field in sense["fields"].values():
                    self.assertTrue(field["text"].strip())
                    self.assertTrue(field["modification"].strip())

    def test_author_proposals_keep_independent_admission_closed(self):
        admission, review, rights = read("admission.json"), read("field-review.json"), read("rights.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual((admission["independently_accepted_new_groups"], admission["independently_accepted_new_fields"]), (0, 0))
        self.assertEqual(admission["pending_independent_review_fields"], 200)
        self.assertEqual(admission["pending_independent_rights_fields"], 200)
        self.assertEqual(admission["missing_admitted_fields"], 200)
        self.assertEqual(admission["author_unresolved_content_rows"], [3612])
        self.assertEqual(admission["annotation_exception_groups"], 0)
        self.assertEqual(admission["specialist_exception_groups"], 0)
        self.assertFalse(review["independent_acceptance"])
        self.assertIsNone(review["reviewer"])
        self.assertEqual(rights["independent_rights_fields_accepted"], 0)
        expected = {(g["senses"][0]["sense_id"], name): field for g in read("groups.json")
                    for name, field in g["senses"][0]["fields"].items()}
        self.assertEqual(len(review["fields"]), 200)
        self.assertEqual({(f["sense_id"], f["field"]) for f in rights["fields"]}, set(expected))
        for field in rights["fields"]:
            proposed = expected[field["sense_id"], field["field"]]
            self.assertEqual(field["exact_source_span"], proposed["source_span"])
            self.assertEqual(field["license"], "CC-BY-SA-4.0")
            self.assertTrue(field["attribution"])
            self.assertTrue(field["modification"])
            self.assertEqual(field["independent_review"], "pending")
            self.assertIsNone(field["reviewer"])
        for field in review["fields"]:
            self.assertEqual(field["accepted_text_quality_status"], "pending")
            self.assertIsNone(field["reviewer"])
        temporary_root = ROOT / "target/issue-153"
        temporary_root.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=temporary_root) as directory:
            candidate = Path(directory)
            for name in (*MATERIALS, "input-checksums.json"):
                shutil.copy2(ROOT / "resources/core-pilot-79" / name, candidate / name)
            curated_path = candidate / "curated-inputs.json"
            curated = json.loads(curated_path.read_text(encoding="utf-8"))
            curated["groups"].extend(read("groups.json"))
            curated_path.write_text(json.dumps(curated), encoding="utf-8")
            (candidate / "field-review.json").write_text(json.dumps(review), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "separate independent field review is required"):
                load_inputs(candidate)

    def test_fixed_captures_support_every_field_at_its_actual_language_and_use(self):
        captures = read("source-captures.json")
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        archive = Path(captures["capture_root"])
        self.assertEqual((archive.name, archive.parent.name), ("issue-153", "author-evidence"))
        for capture in captures["sources"]:
            self.assertTrue(Path(capture["api_path"]).is_relative_to(archive))
            self.assertTrue(Path(capture["raw_path"]).is_relative_to(archive))
        raw = verify_archived_source_captures(self, captures["sources"], sources, Path)
        with tempfile.TemporaryDirectory(dir=ROOT / "target/issue-153") as directory:
            corrupt_path = Path(directory) / "corrupt.wiki"
            first = captures["sources"][0]
            corrupt_path.write_bytes(Path(first["raw_path"]).read_bytes() + b"\ncorrupt")
            corrupted = dict(first, raw_path=str(corrupt_path))
            with self.assertRaises(AssertionError):
                verify_archived_source_captures(self, [corrupted], sources, Path)
        excerpts = read("source-excerpts.json")
        self.assertEqual(len(excerpts["fields"]), 200)
        for record in [*excerpts["fields"], *excerpts["supporting_evidence"]]:
            line = raw[record["source_id"]][record["line_1_based"] - 1]
            column = record["column_1_based"] - 1
            self.assertEqual(line[column:column + len(record["reviewed_excerpt"])], record["reviewed_excerpt"])
            self.assertNotRegex(record["reviewed_excerpt"].lstrip(), r"^#+\*")
            self.assertEqual(record["revid"], sources[record["source_id"]]["revid"])
        exact_fields = {(f["sense_id"], f["field"]): f for f in excerpts["fields"]}
        pos_frames = {"NOM": "nom", "VER": "verbe", "ADJ": "adjectif", "ADV": "adverbe",
                      "ADJ:num": "adjectif numéral", "PRO:pos": "pronom possessif"}
        for group in read("groups.json"):
            sense = group["senses"][0]
            for name, field in sense["fields"].items():
                exact = exact_fields[sense["sense_id"], name]
                self.assertEqual(field["source_id"], exact["source_id"])
                self.assertEqual(field["source_span"]["raw_source_excerpt"], exact["reviewed_excerpt"])
                self.assertEqual(field["language"], {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}[name])
                preceding = raw[exact["source_id"]][:exact["line_1_based"]]
                language = next(line for line in reversed(preceding) if line.startswith("== "))
                self.assertRegex(language, r"^==\s*\{\{langue\|fr\}\}\s*==$")
                if name == "ipa":
                    self.assertEqual(field["text"].strip("/"), exact["ipa_variant_excerpt"])
                    column = exact["ipa_variant_column_1_based"] - 1
                    self.assertEqual(exact["reviewed_excerpt"][column:column + len(exact["ipa_variant_excerpt"])], exact["ipa_variant_excerpt"])
                else:
                    pos = next(line for line in reversed(preceding) if line.startswith("=== {{S|"))
                    expected = "verbe" if group["frozen_row"] == 3646 else pos_frames[group["lexique_cgram"]]
                    self.assertIn("{{S|" + expected + "|fr", pos)

    def test_complete_contributors_and_notices_accompany_proposals(self):
        captures = read("source-captures.json")
        names, anonymous, chains = {}, {}, {}
        for name in captures["contributor_registry"]["parts"]:
            path = Path(name)
            data = json.loads(path.read_bytes())
            prefix, part = path.name.removesuffix(".api.json").rsplit("-", 1)
            chains.setdefault(prefix, {})[int(part)] = (path, data)
            for page in data["query"]["pages"]:
                names.setdefault(page["title"], set()).update(p["name"] for p in page.get("contributors", []))
                if "anoncontributors" in page:
                    anonymous[page["title"]] = page["anoncontributors"]
        for parts in chains.values():
            self.assertEqual(sorted(parts), list(range(1, len(parts) + 1)))
            for part, (path, data) in parts.items():
                continuation = data.get("continue")
                if part == len(parts):
                    self.assertFalse(continuation)
                else:
                    query = parse_qs(urlparse(parts[part + 1][0].with_suffix(".request.txt").read_text(encoding="utf-8")).query)
                    for key, value in continuation.items():
                        self.assertEqual(query[key], [str(value)])
        sources = read("sources.json")["sources"]
        self.assertEqual(set(names), {s["title"] for s in sources})
        for context in read("source-context.json")["current_content"]:
            self.assertEqual(context["current_contributors"], sorted(names[context["title"]]))
            self.assertEqual(context["anonymous_contributors_count"], anonymous.get(context["title"], 0))
        self.assertFalse(captures["contributor_registry"]["selected_sentence_authorship_proved"])
        notices = (EVIDENCE / "NOTICES.md").read_text(encoding="utf-8")
        for source in sources:
            self.assertIn(source["fixed_url"], notices)
            self.assertIn(source["history_url"], notices)
        self.assertEqual((EVIDENCE / "COPYING").read_bytes().rstrip(), (ROOT / "resources/core-pilot-79/COPYING").read_bytes().rstrip())

    def test_difficult_groups_keep_their_exact_construction_and_pending_choice(self):
        groups = {g["frozen_row"]: g for g in read("groups.json")}
        pronoun = groups[3613]["senses"][0]
        self.assertEqual(pronoun["source_headword"], "les vôtres")
        self.assertEqual(pronoun["fields"]["ipa"]["text"], "/votʁ/")
        self.assertIn("polite singular", pronoun["fields"]["english"]["text"])
        support = read("source-excerpts.json")["supporting_evidence"]
        self.assertTrue(any(r["sense_id"] == pronoun["sense_id"] and r["reviewed_excerpt"] == "|mp=les vôtres" for r in support))
        stolen = groups[3646]["senses"][0]
        self.assertEqual(groups[3646]["lexique_cgram"], "ADJ")
        self.assertTrue(any(r["sense_id"] == stolen["sense_id"] and "Participe passé masculin singulier" in r["reviewed_excerpt"] for r in support))
        self.assertIn("Stolen", stolen["fields"]["english"]["text"])
        blocked = groups[3612]["senses"][0]
        self.assertEqual((groups[3612]["headword"], groups[3612]["lexique_cgram"]), ("by", "NOM"))
        self.assertEqual(blocked["fields"]["ipa"]["text"], "/bi/")
        self.assertIn("everyday status not established", blocked["fields"]["english"]["text"])
        usage = next(u for u in read("field-review.json")["usage"] if u["frozen_row"] == 3612)
        self.assertEqual(usage["everyday_sense_status"], "unresolved_modern_everyday_use")
        relations = read("source-context.json")["spelling_relations"]
        self.assertEqual(len(relations), 50)
        self.assertTrue(all(r["alias_admission"] == "not_admitted" and r["independent_review"] == "pending" for r in relations))


if __name__ == "__main__":
    unittest.main()
