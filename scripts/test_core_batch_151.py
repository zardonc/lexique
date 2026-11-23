"""Check #151 at the approved resource/admission boundary."""
import csv
import json
from pathlib import Path
import tempfile
from urllib.parse import parse_qs, urlparse
import unittest

from core_batch_test_helpers import verify_archived_source_captures
import core_pilot

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-151"
MEMBERSHIP = [
    ("demeurer", "VER"), ("piloter", "VER"), ("coucou", "NOM"),
    ("punition", "NOM"), ("droguer", "VER"), ("marine", "ADJ"),
    ("beau-frère", "NOM"), ("démocratie", "NOM"), ("morale", "NOM"),
    ("construction", "NOM"), ("fringue", "NOM"), ("vainqueur", "NOM"),
    ("embaucher", "VER"), ("assommer", "VER"), ("rajouter", "VER"),
    ("concernant", "PRE"), ("gentleman", "NOM"), ("division", "NOM"),
    ("affection", "NOM"), ("tolérer", "VER"), ("siffler", "VER"),
    ("romain", "ADJ"), ("chapitre", "NOM"), ("cicatrice", "NOM"),
    ("dalle", "NOM"), ("tendance", "NOM"), ("raté", "NOM"),
    ("chantier", "NOM"), ("soûl", "ADJ"), ("foudre", "NOM"),
    ("torture", "NOM"), ("altesse", "NOM"), ("empirer", "VER"),
    ("mouillé", "ADJ"), ("agression", "NOM"), ("bousiller", "VER"),
    ("grossier", "ADJ"), ("ivrogne", "NOM"), ("procéder", "VER"),
    ("petit-fils", "NOM"), ("douzaine", "NOM"), ("insensé", "ADJ"),
    ("chanceux", "ADJ"), ("révérend", "NOM"), ("localiser", "VER"),
    ("chiotte", "NOM"), ("basque", "ADJ"), ("quoique", "CON"),
    ("confession", "NOM"), ("querelle", "NOM"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch151Tests(unittest.TestCase):
    def test_fixed_sources_bind_every_field_to_french_pos_and_exact_ipa(self):
        captures = read("source-captures.json")
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        archive = Path(captures["capture_root"])
        self.assertEqual((archive.name, archive.parent.name), ("issue-151", "author-evidence"))
        self.assertEqual(len(captures["sources"]), 52)
        raw = verify_archived_source_captures(self, captures["sources"], sources, Path)
        for capture in captures["sources"]:
            self.assertTrue(Path(capture["api_path"]).is_relative_to(archive))
            self.assertTrue(Path(capture["raw_path"]).is_relative_to(archive))
            self.assertEqual(parse_qs(urlparse(capture["fixed_url"]).query)["oldid"], [str(capture["revid"])])
        excerpts = read("source-excerpts.json")
        for record in [*excerpts["fields"], *excerpts["supporting_evidence"]]:
            line = raw[record["source_id"]][record["line_1_based"] - 1]
            column = record["column_1_based"] - 1
            self.assertEqual(line[column:column + len(record["reviewed_excerpt"])], record["reviewed_excerpt"])
            self.assertNotRegex(record["reviewed_excerpt"].lstrip(), r"^#+\*")
            self.assertNotIn("[[Fichier:", record["reviewed_excerpt"])
        by_field = {(r["sense_id"], r["field"]): r for r in excerpts["fields"]}
        self.assertEqual(len(by_field), 200)
        pos_frames = {"NOM": "nom", "VER": "verbe", "ADJ": "adjectif", "PRE": "préposition", "CON": "conjonction"}
        languages = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
        for group in read("groups.json"):
            for sense in group["senses"]:
                for name, field in sense["fields"].items():
                    record = by_field[sense["sense_id"], name]
                    self.assertEqual(field["language"], languages[name])
                    self.assertEqual(field["source_id"], record["source_id"])
                    self.assertEqual(field["source_span"]["raw_source_excerpt"], record["reviewed_excerpt"])
                    self.assertEqual(field["source_span"]["line_1_based"], record["line_1_based"])
                    preceding = raw[record["source_id"]][:record["line_1_based"]]
                    language = next(line for line in reversed(preceding) if line.startswith("== "))
                    self.assertRegex(language, r"^==\s*\{\{langue\|fr\}\}\s*==$")
                    pos = next(line for line in reversed(preceding) if line.startswith("=== {{S|"))
                    self.assertIn("{{S|" + pos_frames[group["lexique_cgram"]] + "|fr", pos)
                    if name == "ipa":
                        self.assertEqual(field["text"].strip("/"), record["ipa_variant_excerpt"])
                        offset = record["ipa_variant_column_1_based"] - 1
                        self.assertEqual(record["reviewed_excerpt"][offset:offset + len(record["ipa_variant_excerpt"])],
                                         record["ipa_variant_excerpt"])

    def test_contributor_pagination_and_credits_match_the_retained_responses(self):
        captures = read("source-captures.json")
        archive = Path(captures["capture_root"])
        names, anonymous, chains = {}, {}, {}
        for name in captures["contributor_registry"]["parts"]:
            path = Path(name)
            self.assertTrue(path.is_relative_to(archive))
            data = json.loads(path.read_bytes())
            prefix, part = path.name.removesuffix(".api.json").rsplit("-", 1)
            chains.setdefault(prefix, {})[int(part)] = (path, data)
            for page in data["query"]["pages"]:
                names.setdefault(page["title"], set()).update(p["name"] for p in page.get("contributors", []))
                if "anoncontributors" in page:
                    anonymous[page["title"]] = page["anoncontributors"]
        self.assertEqual(set(chains), {*(f"contributors-new-{n}" for n in range(1, 6)), "contributors-support"})
        for parts in chains.values():
            self.assertEqual(sorted(parts), list(range(1, len(parts) + 1)))
            for part, (_, data) in parts.items():
                continuation = data.get("continue")
                if part == len(parts):
                    self.assertFalse(continuation)
                else:
                    self.assertTrue(continuation)
                    query = parse_qs(urlparse(parts[part + 1][0].with_suffix(".request.txt").read_text(encoding="utf-8")).query)
                    for key, value in continuation.items():
                        self.assertEqual(query[key], [str(value)])
        self.assertEqual(set(names), {s["title"] for s in read("sources.json")["sources"]})
        for context in read("source-context.json")["current_content"]:
            self.assertEqual(context["current_contributors"], sorted(names[context["title"]]))
            self.assertEqual(context["current_registry_names"], len(names[context["title"]]))
            self.assertEqual(context["anonymous_contributors_count"], anonymous.get(context["title"], 0))
        self.assertFalse(captures["contributor_registry"]["selected_sentence_authorship_proved"])

    def test_existing_converter_rejects_the_pending_author_package(self):
        scratch = ROOT / "target/issue-151"
        scratch.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=scratch) as folder:
            stage = Path(folder)
            for name in (*core_pilot.MATERIALS, "input-checksums.json"):
                (stage / name).write_text("{}\n", encoding="utf-8")
            (stage / "scope-manifest.csv").write_bytes((ROOT / "resources/core-pilot-79/scope-manifest.csv").read_bytes())
            (stage / "curated-inputs.json").write_text(json.dumps({
                "schema": "lexique-core-pilot-curation-1", "content_license": "CC-BY-SA-4.0",
                "scope_denominator_groups": 4087, "scope_denominator_headwords": 3773,
                "groups": read("groups.json"), "sources": read("sources.json")["sources"],
            }, ensure_ascii=False), encoding="utf-8")
            (stage / "field-review.json").write_bytes((EVIDENCE / "field-review.json").read_bytes())
            with self.assertRaisesRegex(ValueError, "A separate independent field review is required"):
                core_pilot.load_inputs(stage)

    def test_author_proposals_and_rights_do_not_claim_independent_admission(self):
        review, rights, admission = read("field-review.json"), read("rights.json"), read("admission.json")
        self.assertEqual(review["kind"], "author_prepared_for_separate_reviewer")
        self.assertFalse(review["independent_acceptance"])
        self.assertIsNone(review["reviewer"])
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual((admission["independently_accepted_new_groups"], admission["independently_accepted_new_fields"]), (0, 0))
        self.assertEqual(admission["pending_independent_review_fields"], 200)
        self.assertEqual(admission["pending_independent_rights_fields"], 200)
        self.assertEqual(admission["missing_admitted_fields"], 200)
        self.assertEqual(rights["independent_rights_fields_accepted"], 0)
        expected = {(g["frozen_row"], name): field for g in read("groups.json")
                    for sense in g["senses"] for name, field in sense["fields"].items()}
        self.assertEqual({(r["frozen_row"], r["field"]) for r in rights["fields"]}, set(expected))
        for record in rights["fields"]:
            field = expected[record["frozen_row"], record["field"]]
            self.assertEqual(record["exact_source_span"], field["source_span"])
            self.assertEqual(record["license"], "CC-BY-SA-4.0")
            self.assertTrue(record["attribution"].strip())
            self.assertTrue(record["modification"].strip())
            self.assertEqual(record["independent_review"], "pending")
            self.assertIsNone(record["reviewer"])
        for record in review["fields"]:
            self.assertEqual(record["accepted_text_quality_status"], "pending")
            self.assertIsNone(record["reviewer"])
        for name in ("groups.json", "field-review.json", "rights.json", "sources.json", "source-context.json",
                     "source-excerpts.json", "admission.json", "reuse.json", "source-captures.json"):
            self.assertEqual((EVIDENCE / "authored" / name).read_bytes(), (EVIDENCE / name).read_bytes())
        self.assertEqual((EVIDENCE / "COPYING").read_bytes().rstrip(),
                         (ROOT / "resources/core-pilot-79/COPYING").read_bytes().rstrip())
        notices = (EVIDENCE / "NOTICES.md").read_text(encoding="utf-8")
        for source in read("sources.json")["sources"]:
            self.assertIn(source["fixed_url"], notices)
            self.assertIn(source["history_url"], notices)
        rights_text = (EVIDENCE / "selected-rights.md").read_text(encoding="utf-8")
        self.assertIn("2006-01-01", rights_text)
        self.assertIn("pending", rights_text)
        self.assertIn("CC BY-SA 4.0", rights_text)

    def test_exact_frozen_groups_have_complete_resource_records(self):
        if (EVIDENCE / "groups.json").exists():
            groups = read("groups.json")
        else:
            canonical = json.loads((ROOT / "resources/core-pilot-79/curated-inputs.json").read_text(encoding="utf-8"))
            groups = [g for g in canonical["groups"] if 3501 <= g["frozen_row"] <= 3550]
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP,
                         "The frozen #151 resource slice lacks complete batch content")
        self.assertEqual([g["frozen_row"] for g in groups], list(range(3501, 3551)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(encoding="utf-8-sig", newline="") as handle:
            frozen = list(csv.DictReader(handle))
        self.assertEqual(len(frozen), 4087)
        for group in groups:
            with self.subTest(row=group["frozen_row"]):
                row = frozen[group["frozen_row"] - 1]
                self.assertEqual((group["headword"], group["lexique_cgram"]), (row["word"], row["lexique_cgram"]))
                for frequency in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                    self.assertEqual(group[frequency], row[frequency])
                self.assertGreaterEqual(len(group["senses"]), 1)
                for sense in group["senses"]:
                    self.assertTrue(sense["use"].strip())
                    self.assertIsNone(sense["cefr"])
                    self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                    for field in sense["fields"].values():
                        self.assertTrue(field["text"].strip())
                        self.assertTrue(field["modification"].strip())


if __name__ == "__main__":
    unittest.main()
