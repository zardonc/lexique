"""Check #154 at the approved resource/admission boundary."""
import csv
import json
from html import unescape
from pathlib import Path
import re
import shutil
import tempfile
import unittest

import core_pilot
from core_batch_test_helpers import verify_archived_source_captures, verify_archived_contributor_captures

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-154"
MEMBERSHIP = [
    ("requin", "NOM"), ("tordre", "VER"), ("vital", "ADJ"),
    ("autopsie", "NOM"), ("déplaire", "VER"), ("grotte", "NOM"),
    ("désespérer", "VER"), ("boum", "NOM"), ("trinquer", "VER"),
    ("chacune", "PRO:ind"), ("couvert", "NOM"), ("amer", "ADJ"),
    ("calculer", "VER"), ("supermarché", "NOM"), ("tué", "ADJ"),
    ("sot", "ADJ"), ("allée", "NOM"), ("balayer", "VER"),
    ("zoo", "NOM"), ("priorité", "NOM"), ("routine", "NOM"),
    ("vulgaire", "ADJ"), ("assaut", "NOM"), ("engagement", "NOM"),
    ("classer", "VER"), ("gratter", "VER"), ("rituel", "NOM"),
    ("vaisselle", "NOM"), ("repentir", "VER"), ("aspirer", "VER"),
    ("cognac", "NOM"), ("citron", "NOM"), ("honnêtement", "ADV"),
    ("louper", "VER"), ("musulman", "NOM"), ("tatouage", "NOM"),
    ("indépendant", "ADJ"), ("jadis", "ADV"), ("cadet", "NOM"),
    ("association", "NOM"), ("discipline", "NOM"), ("favori", "ADJ"),
    ("province", "NOM"), ("fraise", "NOM"), ("serré", "ADJ"),
    ("juillet", "NOM"), ("refermer", "VER"), ("saoul", "ADJ"),
    ("trancher", "VER"), ("acide", "NOM"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch154Tests(unittest.TestCase):
    def test_scraping_erases_a_mark_on_the_surface(self):
        group = next(g for g in read("groups.json") if g["frozen_row"] == 3676)
        fields = group["senses"][0]["fields"]
        self.assertEqual(fields["french_definition"]["text"],
                         "Racler une surface pour la nettoyer, effacer quelque chose qui s’y trouve ou la polir (verbe transitif).")
        self.assertIn("erase something on it", fields["english"]["text"])

    def test_muslim_adherence_does_not_require_practising(self):
        group = next(g for g in read("groups.json") if g["frozen_row"] == 3685)
        field = group["senses"][0]["fields"]["french_definition"]
        self.assertEqual(field["text"],
                         "Adepte de l’islam (nom masculin ; féminin : musulmane).")
        self.assertIn("Adepte de la", field["source_span"]["raw_source_excerpt"])

    def test_exact_frozen_groups_have_complete_resource_records(self):
        if (EVIDENCE / "groups.json").exists():
            groups = read("groups.json")
        else:
            canonical = json.loads((ROOT / "resources/core-pilot-79/curated-inputs.json").read_text(encoding="utf-8"))
            groups = [g for g in canonical["groups"] if 3651 <= g["frozen_row"] <= 3700]
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP,
                         "The frozen #154 resource slice lacks complete batch content")
        self.assertEqual([g["frozen_row"] for g in groups], list(range(3651, 3701)))
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

    def test_independent_review_binds_all_accepted_fields_and_uses(self):
        rights, admission, review = read("rights.json"), read("admission.json"), read("field-review.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual((admission["independently_accepted_new_groups"], admission["independently_accepted_new_fields"]), (50, 200))
        self.assertEqual(admission["pending_independent_review_fields"], 0)
        self.assertEqual(admission["pending_independent_rights_fields"], 0)
        self.assertEqual(admission["missing_admitted_fields"], 200)
        self.assertEqual(admission["missing_authored_fields"], 0)
        self.assertEqual(admission["unresolved_field_blockers"], [])
        self.assertEqual(admission["pending_independent_review_rows"], [])
        self.assertTrue(review["independent_acceptance"])
        self.assertEqual(review["kind"], "separate_independent_field_review")
        self.assertTrue(review["reviewer"])
        self.assertEqual(rights["independent_rights_fields_accepted"], 200)
        self.assertEqual(rights["final_combined_package_status"], "coordinator_owned_pending")
        expected = {(g["senses"][0]["sense_id"], name): field for g in read("groups.json")
                    for name, field in g["senses"][0]["fields"].items()}
        self.assertEqual({(f["sense_id"], f["field"]) for f in rights["fields"]}, set(expected))
        for field in rights["fields"]:
            self.assertEqual(field["exact_source_span"], expected[field["sense_id"], field["field"]]["source_span"])
            self.assertEqual(field["license"], "CC-BY-SA-4.0")
            self.assertTrue(field["attribution"].strip())
            self.assertTrue(field["modification"].strip())
            self.assertEqual(field["independent_review"], "passed")
            self.assertEqual(field["reviewer"], review["reviewer"])
            self.assertEqual(field["accepted_text"], expected[field["sense_id"], field["field"]]["text"])
            self.assertEqual(field["rights_eligibility_status"], "passed_at_selected_text_boundary")
            self.assertTrue(field["review_reason"].strip())
        authored = {(g["senses"][0]["sense_id"], name): field
                    for g in read("authored/groups.json")
                    for name, field in g["senses"][0]["fields"].items()}
        self.assertEqual(len(review["fields"]), 200)
        self.assertEqual({(f["sense_id"], f["field"]) for f in review["fields"]}, set(expected))
        for field in review["fields"]:
            key = field["sense_id"], field["field"]
            self.assertEqual(field["accepted_text_quality_status"], "passed")
            self.assertEqual(field["reviewer"], review["reviewer"])
            self.assertEqual(field["proposed_text"], authored[key]["text"])
            self.assertEqual(field["accepted_text"], expected[key]["text"])
            self.assertEqual(field["accepted_source"], field["exact_source"])
            self.assertTrue(field["review_reason"].strip())
            self.assertTrue(field["rights_review_reason"].strip())
        uses = {g["senses"][0]["sense_id"]: g["senses"][0] for g in read("groups.json")}
        self.assertEqual(len(review["usage"]), 50)
        self.assertEqual(len(review["spelling_relations"]), 50)
        for usage in review["usage"]:
            self.assertEqual(usage["accepted_usage"], uses[usage["sense_id"]]["use"])
            self.assertEqual(usage["accepted_usage_quality_status"], "passed")
            self.assertEqual(usage["everyday_sense_status"], "passed")
            self.assertEqual(usage["reviewer"], review["reviewer"])
            self.assertTrue(usage["review_reason"].strip())
        for spelling in review["spelling_relations"]:
            self.assertEqual(spelling["independent_review"], "passed")
            self.assertEqual(spelling["alias_admission"], "not_admitted")
        replacements = read("accepted-replacements.json")
        self.assertEqual([(r["frozen_row"], r["field"]) for r in replacements],
                         [(3676, "french_definition"), (3685, "french_definition")])
        self.assertEqual((admission["accepted_unchanged_fields"], admission["accepted_changed_fields"]), (198, 2))

    def test_author_review_cannot_enter_the_existing_admission_pipeline(self):
        with tempfile.TemporaryDirectory(prefix="core-batch-154-") as folder:
            source_dir = Path(folder)
            for name in (*core_pilot.MATERIALS, "input-checksums.json"):
                shutil.copyfile(core_pilot.RESOURCE / name, source_dir / name)
            shutil.copyfile(EVIDENCE / "authored/field-review.json", source_dir / "field-review.json")
            with self.assertRaisesRegex(ValueError, "A separate independent field review is required"):
                core_pilot.load_inputs(source_dir)

    def test_fixed_spans_contributors_and_notices_accompany_all_fields(self):
        captures = read("source-captures.json")
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        archive = Path(captures["capture_root"])
        self.assertEqual((archive.name, archive.parent.name), ("issue-154", "author-evidence"))
        self.assertEqual(len(captures["sources"]), 54)
        for capture in captures["sources"]:
            for key in ("api_path", "raw_path"):
                self.assertTrue(Path(capture[key]).is_relative_to(archive))
        lines = verify_archived_source_captures(self, captures["sources"], sources, Path)
        excerpts = read("source-excerpts.json")
        self.assertEqual(len(excerpts["fields"]), 200)
        for exact in [*excerpts["fields"], *excerpts["supporting_evidence"]]:
            line = lines[exact["source_id"]][exact["line_1_based"] - 1]
            start = exact["column_1_based"] - 1
            self.assertEqual(line[start:start + len(exact["reviewed_excerpt"])], exact["reviewed_excerpt"])
            self.assertNotRegex(exact["reviewed_excerpt"].lstrip(), r"^#+\*")
            self.assertEqual(exact["revid"], sources[exact["source_id"]]["revid"])
        for decision in [*read("field-review.json")["fields"], *read("field-review.json")["usage"]]:
            for exact in [decision["exact_source"], *decision["supporting_sources"]]:
                line = lines[exact["source_id"]][exact["line_1_based"] - 1]
                column = exact["column_1_based"] - 1
                self.assertEqual(line[column:column + len(exact["reviewed_excerpt"])], exact["reviewed_excerpt"])
        by_field = {(r["sense_id"], r["field"]): r for r in excerpts["fields"]}
        pos_frames = {"NOM": "nom", "VER": "verbe", "ADJ": "adjectif", "ADV": "adverbe", "PRO:ind": "pronom indéfini"}
        languages = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
        for group in read("groups.json"):
            sense = group["senses"][0]
            for name, field in sense["fields"].items():
                exact = by_field[sense["sense_id"], name]
                self.assertEqual(field["source_id"], exact["source_id"])
                self.assertEqual(field["source_span"]["raw_source_excerpt"], exact["reviewed_excerpt"])
                self.assertEqual(field["language"], languages[name])
                preceding = lines[exact["source_id"]][:exact["line_1_based"]]
                language = next(line for line in reversed(preceding) if line.startswith("== "))
                self.assertRegex(language, r"^==\s*\{\{langue\|fr\}\}\s*==$")
                if name == "ipa":
                    self.assertEqual(field["text"].strip("/"), exact["ipa_variant_excerpt"])
                    column = exact["ipa_variant_column_1_based"] - 1
                    self.assertEqual(exact["reviewed_excerpt"][column:column + len(exact["ipa_variant_excerpt"])], exact["ipa_variant_excerpt"])
                else:
                    pos = next(line for line in reversed(preceding) if line.startswith("=== {{S|"))
                    expected = "verbe" if group["frozen_row"] == 3665 else pos_frames[group["lexique_cgram"]]
                    self.assertIn("{{S|" + expected + "|fr", pos)
        def resolve(value):
            path = Path(value)
            self.assertTrue(path.is_relative_to(archive))
            return path
        names, anonymous, chains = verify_archived_contributor_captures(
            self, captures["contributor_registry"]["parts"], resolve)
        self.assertEqual(set(names), {s["title"] for s in sources.values()})
        for context in read("source-context.json")["current_content"]:
            self.assertEqual(context["current_contributors"], sorted(names[context["title"]]))
            self.assertEqual(context["anonymous_contributors_count"], anonymous.get(context["title"], 0))
        self.assertFalse(captures["contributor_registry"]["selected_sentence_authorship_proved"])
        notices = (EVIDENCE / "NOTICES.md").read_text(encoding="utf-8")
        for source in sources.values():
            self.assertIn(source["fixed_url"], notices)
            self.assertIn(source["history_url"], notices)
        self.assertEqual((EVIDENCE / "COPYING").read_bytes().rstrip(), (core_pilot.RESOURCE / "COPYING").read_bytes().rstrip())

    def test_inflections_register_and_excluded_rights_are_evidenced(self):
        groups = {g["frozen_row"]: g for g in read("groups.json")}
        support = read("source-excerpts.json")["supporting_evidence"]
        modern = {r["frozen_row"]: r for r in read("modern-corroboration.json")["records"]}
        for row in (3665, 3666):
            self.assertEqual(modern[row]["independent_review"], "passed")
            self.assertTrue(Path(modern[row]["raw_path"]).is_file())
            body = Path(modern[row]["readable_path"]).read_text(encoding="utf-8")
            for excerpt in modern[row]["exact_excerpts"]:
                self.assertIn(excerpt, body)
            self.assertEqual(modern[row]["learner_text_copied"], False)
        self.assertEqual(modern[3665]["publisher"], "Office québécois de la langue française")
        self.assertEqual(modern[3666]["publisher"], "Le Robert")
        chacune = groups[3660]["senses"][0]
        self.assertEqual(groups[3660]["lexique_cgram"], "PRO:ind")
        self.assertEqual(chacune["source_headword"], "chacun")
        self.assertEqual(chacune["fields"]["ipa"]["text"], "/ʃa.kyn/")
        self.assertIn("féminine singulière", chacune["fields"]["french_definition"]["text"])
        self.assertTrue(any(r["frozen_row"] == 3660 and "Féminin (singulier)" in r["reviewed_excerpt"] for r in support))
        tue = groups[3665]["senses"][0]
        self.assertEqual(groups[3665]["lexique_cgram"], "ADJ")
        self.assertIn("adjectif participe", tue["fields"]["french_definition"]["text"])
        self.assertTrue(any(r["frozen_row"] == 3665 and "Participe passé masculin singulier" in r["reviewed_excerpt"] for r in support))
        self.assertIn("littéraire", groups[3666]["senses"][0]["fields"]["french_definition"]["text"])
        repentir = groups[3679]["senses"][0]
        self.assertEqual(repentir["source_headword"], "se repentir")
        self.assertEqual(repentir["fields"]["ipa"]["text"], "/ʁə.pɑ̃.tiʁ/")
        for name in ("french_definition", "english", "chinese"):
            self.assertIn("se repentir", repentir["fields"][name]["text"])
        self.assertEqual(groups[3658]["senses"][0]["fields"]["ipa"]["text"], "/bum/")
        self.assertIn("nom masculin", groups[3658]["senses"][0]["fields"]["french_definition"]["text"])
        self.assertEqual(groups[3698]["headword"], "saoul")
        self.assertEqual(groups[3698]["senses"][0]["fields"]["ipa"]["text"], "/su/")
        self.assertIn("Au sens courant", groups[3700]["senses"][0]["fields"]["french_definition"]["text"])
        sources = {s["title"]: s for s in read("sources.json")["sources"]}
        amer = sources["amer"]
        for location in [*read("source-excerpts.json")["fields"], *support]:
            self.assertFalse(location["source_id"] == amer["id"] and location["line_1_based"] == 234)
        self.assertTrue(all(r["alias_admission"] == "not_admitted" for r in read("source-context.json")["spelling_relations"]))

    def test_selected_original_expressions_are_fixed_and_separate_from_quality(self):
        originals = read("original-expression-review.json")
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        self.assertEqual(originals["unresolved_sources"], [])
        self.assertEqual({r["source_id"] for r in originals["sources"]}, set(sources))
        self.assertEqual(originals["matched_original_sources"], 32)
        for record in originals["sources"]:
            self.assertEqual(record["fixed_revision"], sources[record["source_id"]]["revid"])
            self.assertEqual(record["selected_original_expression_decision"], "passed")
            self.assertTrue(record["review_reason"].strip())
            original = record["original_comparison"]
            if original:
                text = Path(original["plain_text_path"]).read_text(encoding="utf-8")
                self.assertIn("8e édition (1935)", text)
                start = original["plain_text_column_1_based"] - 1
                self.assertEqual(text[start:start + len(original["original_expression"])], original["original_expression"])
                raw = Path(original["raw_path"]).read_text(encoding="utf-8")
                plain = re.sub(r"\s+", " ", unescape(re.sub(r"<[^>]+>", " ", raw)))
                self.assertIn(re.sub(r"\s+", " ", original["original_expression"]), plain)
        for modern in read("modern-corroboration.json")["records"]:
            raw = Path(modern["raw_path"]).read_text(encoding="utf-8")
            plain = re.sub(r"\s+", " ", unescape(re.sub(r"<[^>]+>", " ", raw)))
            for excerpt in modern["exact_excerpts"]:
                self.assertIn(re.sub(r"\s+", " ", excerpt), plain)
            extra = modern.get("additional_independent_lexical_corroboration")
            if extra:
                raw = Path(extra["raw_path"]).read_text(encoding="utf-8")
                data = json.JSONDecoder().raw_decode(raw.split("window.__PINIA__=", 1)[1])[0]
                response = data["payload"]["responses"]["entry:tué::adjectif"]
                self.assertEqual(response["header"]["pos"], "adjectif")
                dictionary = next(d for d in response["content"] if d["id"] == "tlfi")
                plain = re.sub(r"\s+", " ", unescape(re.sub(r"<[^>]+>", "", "".join(dictionary["content"]))))
                for excerpt in extra["exact_excerpts"]:
                    self.assertIn(excerpt, Path(extra["readable_path"]).read_text(encoding="utf-8"))
                    self.assertIn(re.sub(r"\s+", " ", excerpt), plain, "TLFi excerpt absent from retained response content")
                self.assertFalse(extra["learner_text_copied"])
        tattoo = [r for r in read("field-review.json")["fields"] if r["frozen_row"] == 3686 and r["field"] != "ipa"]
        self.assertEqual(len(tattoo), 3)
        for field in tattoo:
            self.assertTrue(any(s["line_1_based"] == 10 and "[[substance]]s [[coloré]]es sous la [[peau]]" in s["reviewed_excerpt"]
                                for s in field["supporting_sources"]))


if __name__ == "__main__":
    unittest.main()
