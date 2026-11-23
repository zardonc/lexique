"""Check #152 at the approved resource/admission boundary."""
import csv
import json
from pathlib import Path
import shutil
import tempfile
import unittest
from unittest.mock import patch
from urllib.parse import parse_qs, urlparse

import core_pilot
from core_batch_test_helpers import verify_archived_source_captures
from core_pilot import load_inputs, MATERIALS

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-152"
MEMBERSHIP = [
    ("assumer", "VER"), ("européen", "ADJ"), ("noix", "NOM"),
    ("relier", "VER"), ("boy", "NOM"), ("décès", "NOM"),
    ("désastre", "NOM"), ("équipement", "NOM"), ("mobile", "NOM"),
    ("emparer", "VER"), ("précisément", "ADV"), ("évader", "VER"),
    ("flotter", "VER"), ("moustache", "NOM"), ("aspect", "NOM"),
    ("distinguer", "VER"), ("poster", "VER"), ("procurer", "VER"),
    ("flèche", "NOM"), ("jupe", "NOM"), ("possession", "NOM"),
    ("soviétique", "ADJ"), ("ancêtre", "NOM"), ("casino", "NOM"),
    ("rempli", "ADJ"), ("cuisse", "NOM"), ("doué", "ADJ"),
    ("détenir", "VER"), ("biscuit", "NOM"), ("branler", "VER"),
    ("leader", "NOM"), ("découper", "VER"), ("financier", "ADJ"),
    ("matinée", "NOM"), ("précédent", "ADJ"), ("tabasser", "VER"),
    ("futé", "ADJ"), ("panneau", "NOM"), ("coma", "NOM"),
    ("homo", "ADJ"), ("mis", "ADJ"), ("août", "NOM"),
    ("anonyme", "ADJ"), ("marteau", "NOM"), ("mont", "NOM"),
    ("visiteur", "NOM"), ("infini", "ADJ"), ("activer", "VER"),
    ("introduire", "VER"), ("recette", "NOM"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch152Tests(unittest.TestCase):
    def test_real_admission_rejects_an_author_review_without_independent_acceptance(self):
        temporary_root = ROOT / "target/issue-152"
        temporary_root.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="admission-152-", dir=temporary_root) as folder:
            staged = Path(folder)
            for name in (*MATERIALS, "input-checksums.json"):
                shutil.copy2(ROOT / "resources/core-pilot-79" / name, staged / name)
            shutil.copy2(EVIDENCE / "authored/field-review.json", staged / "field-review.json")
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

    def test_pronominal_verbs_and_adjectival_component_keep_frozen_identity(self):
        groups = {g["frozen_row"]: g for g in read("groups.json")}
        for row, construction in ((3560, "s’emparer"), (3562, "s’évader")):
            sense = groups[row]["senses"][0]
            for name in ("french_definition", "english", "chinese"):
                self.assertIn(construction, sense["fields"][name]["text"])
        mis = groups[3591]
        self.assertEqual((mis["headword"], mis["lexique_cgram"]), ("mis", "ADJ"))
        self.assertEqual(mis["senses"][0]["source_headword"], "bien mis")
        self.assertEqual(mis["senses"][0]["fields"]["ipa"]["text"], "/mi/")
        for name in ("french_definition", "english", "chinese"):
            self.assertIn("bien mis", mis["senses"][0]["fields"][name]["text"])
        relation = next(r for r in read("source-context.json")["spelling_relations"] if r["frozen_row"] == 3591)
        self.assertEqual(relation["relation_role"], "source_binding_only")
        self.assertEqual(relation["alias_admission"], "not_admitted")
        homo = groups[3590]["senses"][0]
        self.assertIn("invariable", homo["fields"]["french_definition"]["text"])
        self.assertIn("familière", homo["fields"]["french_definition"]["text"])

    def test_author_rights_proposals_keep_every_field_pending_independent_review(self):
        rights, admission, review = (read("authored/" + name) for name in
                                    ("rights.json", "admission.json", "field-review.json"))
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
                    for g in read("authored/groups.json")
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
        self.assertEqual([g["frozen_row"] for g in groups], list(range(3551, 3601)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            frozen = list(csv.DictReader(handle))[3550:3600]
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
                    self.assertEqual(field["independent_review"], "passed")

    def test_modern_death_and_coma_retain_the_correct_defining_features(self):
        groups = {g["frozen_row"]: g for g in read("groups.json")}
        expected = {
            3556: {"french_definition": "Mort d’une personne ; nom masculin.",
                   "english": "A person’s death; masculine noun.",
                   "chinese": "一个人的死亡；阳性名词。"},
            3589: {"french_definition": "État de profonde inconscience, avec perte de la sensibilité et des mouvements volontaires ; nom masculin.",
                   "english": "Coma: a state of deep unconsciousness with loss of sensation and voluntary movement; masculine noun.",
                   "chinese": "昏迷：深度失去意识，并丧失感觉和自主运动的状态；阳性名词。"},
        }
        for row, texts in expected.items():
            for name, text in texts.items():
                with self.subTest(row=row, field=name):
                    self.assertEqual(groups[row]["senses"][0]["fields"][name]["text"], text)
        replacements = read("accepted-replacements.json")["fields"]
        self.assertEqual({(r["frozen_row"], r["field"]) for r in replacements},
                         {(row, name) for row, texts in expected.items() for name in texts})

    def test_selected_littre_original_is_distinct_from_the_wiki_licence(self):
        source = next(s for s in read("rights.json")["sources"] if s["title"] == "décès")
        layer = next(l for l in source["original_historical_dictionary_layers"]
                     if l["original_authors"] == "Émile Littré")
        self.assertEqual(layer["conservative_expiry"], "1952-01-01")
        self.assertIn("Mort naturelle d'une personne", layer["original_expression_evidence"]["original_expression"])
        self.assertEqual(source["contribution_and_adaptation_license"], "CC-BY-SA-4.0")

    def test_each_accepted_field_use_and_rights_record_agrees_with_the_shard(self):
        review = read("field-review.json")
        self.assertEqual(review["kind"], "separate_independent_field_review")
        self.assertTrue(review["independent_acceptance"])
        self.assertEqual(review["unresolved_fields"], [])
        fields = {(r["sense_id"], r["field"]): r for r in review["fields"]}
        rights = {(r["sense_id"], r["field"]): r for r in read("rights.json")["fields"]}
        usages = {r["sense_id"]: r for r in review["usage"]}
        self.assertEqual((len(fields), len(rights), len(usages)), (200, 200, 50))
        for group in read("groups.json"):
            sense = group["senses"][0]
            usage = usages[sense["sense_id"]]
            self.assertEqual(usage["accepted_usage"], sense["use"])
            self.assertEqual(usage["accepted_usage_quality_status"], "passed")
            self.assertTrue(usage["review_reason"])
            for name, field in sense["fields"].items():
                r, legal = fields[sense["sense_id"], name], rights[sense["sense_id"], name]
                self.assertEqual(r["accepted_text"], field["text"])
                self.assertEqual(legal["accepted_text"], field["text"])
                self.assertEqual(r["accepted_source"]["reviewed_excerpt"],
                                 field["source_span"]["raw_source_excerpt"])
                self.assertEqual(legal["exact_source_span"], field["source_span"])
                self.assertEqual(legal["modification"], field["modification"])
                self.assertEqual(r["accepted_text_quality_status"], "passed")
                self.assertEqual(r["rights_eligibility_status"], "passed_at_selected_text_boundary")
                self.assertEqual(legal["independent_review"], "passed")
                self.assertEqual(r["reviewer"], review["reviewer"])
                self.assertTrue(r["review_reason"])
                self.assertTrue(r["rights_review_reason"])
        admission = read("admission.json")
        self.assertEqual(admission["independently_accepted_new_fields"], 200)
        self.assertEqual(admission["newly_admitted_fields"], 0)
        self.assertEqual(admission["pending_independent_rights_fields"], 0)

    def test_reviewed_shard_passes_the_existing_resource_admission_seam(self):
        temporary_root = ROOT / "target/issue-152-review"
        temporary_root.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="resource-152-", dir=temporary_root) as folder:
            staged = Path(folder)
            for name in (*MATERIALS, "input-checksums.json"):
                shutil.copy2(ROOT / "resources/core-pilot-79" / name, staged / name)
            curated = json.loads((staged / "curated-inputs.json").read_text(encoding="utf-8"))
            reviewed = json.loads((staged / "field-review.json").read_text(encoding="utf-8"))
            curated["groups"] = read("groups.json")
            curated["sources"] = read("sources.json")["sources"]
            shard = read("field-review.json")
            reviewed["fields"], reviewed["usage"] = shard["fields"], shard["usage"]
            # Retain the canonical annotation/alias evidence; evaluate only this shard's rows.
            for name, value in (("curated-inputs.json", curated), ("field-review.json", reviewed)):
                (staged / name).write_text(json.dumps(value, ensure_ascii=False), encoding="utf-8")
            (staged / "input-checksums.json").write_text(
                json.dumps({name: core_pilot.digest(staged / name) for name in MATERIALS}),
                encoding="utf-8")
            with patch.object(core_pilot, "ROWS", list(range(3551, 3601))):
                result = load_inputs(staged)
            selected = result[2]
            self.assertEqual([r[0] for r in selected], list(range(3551, 3601)))
            for group, (_, _, headword, payload) in zip(curated["groups"], selected, strict=True):
                self.assertEqual(headword, group["headword"])
                for name, field in group["senses"][0]["fields"].items():
                    target = core_pilot.FIELDS[name]
                    self.assertEqual(payload[target], field["text"])
                    self.assertEqual(payload["field_sources"][target]["modification"], field["modification"])


if __name__ == "__main__":
    unittest.main()
