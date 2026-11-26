"""Check #151 at the approved resource/admission boundary."""
import csv
import json
from pathlib import Path
from urllib.parse import parse_qs, urlparse
import unittest

from core_batch_test_helpers import (
    assert_core_load_inputs_rejected,
    verify_archived_contributor_captures,
    verify_archived_source_captures,
)
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
    def test_construction_noun_includes_completed_buildings(self):
        group = next(g for g in read("groups.json") if g["frozen_row"] == 3510)
        fields = group["senses"][0]["fields"]
        self.assertEqual(fields["english"]["text"], "A building or structure; the edifice resulting from construction (feminine noun).")
        self.assertEqual(fields["chinese"]["text"], "建筑物；建造而成的房屋或构筑物（阴性名词）。")

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
        def resolve(value):
            path = Path(value)
            self.assertTrue(path.is_relative_to(archive))
            return path
        names, anonymous, chains = verify_archived_contributor_captures(
            self, captures["contributor_registry"]["parts"], resolve)
        self.assertEqual(set(chains), {*(f"contributors-new-{n}" for n in range(1, 6)), "contributors-support"})
        self.assertEqual(set(names), {s["title"] for s in read("sources.json")["sources"]})
        for context in read("source-context.json")["current_content"]:
            self.assertEqual(context["current_contributors"], sorted(names[context["title"]]))
            self.assertEqual(context["current_registry_names"], len(names[context["title"]]))
            self.assertEqual(context["anonymous_contributors_count"], anonymous.get(context["title"], 0))
        self.assertFalse(captures["contributor_registry"]["selected_sentence_authorship_proved"])

    def test_existing_converter_rejects_the_pending_author_package(self):
        def mutate(stage):
            (stage / "curated-inputs.json").write_text(json.dumps({
                "schema": "lexique-core-pilot-curation-1", "content_license": "CC-BY-SA-4.0",
                "scope_denominator_groups": 4087, "scope_denominator_headwords": 3773,
                "groups": read("authored/groups.json"), "sources": read("authored/sources.json")["sources"],
            }, ensure_ascii=False), encoding="utf-8")
            (stage / "field-review.json").write_bytes((EVIDENCE / "authored/field-review.json").read_bytes())

        assert_core_load_inputs_rejected(
            self, core_pilot.RESOURCE, core_pilot.MATERIALS, core_pilot.load_inputs,
            mutate, "A separate independent field review is required", "admission-151-",
        )

    def test_independent_review_binds_accepted_fields_uses_and_rights(self):
        review, rights, admission = read("field-review.json"), read("rights.json"), read("admission.json")
        self.assertEqual(review["kind"], "separate_independent_field_review")
        self.assertTrue(review["independent_acceptance"])
        self.assertTrue(review["reviewer"])
        self.assertEqual(review["unresolved_fields"], [])
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual((admission["independently_accepted_new_groups"], admission["independently_accepted_new_fields"]), (50, 200))
        self.assertEqual(admission["pending_independent_review_fields"], 0)
        self.assertEqual(admission["pending_independent_rights_fields"], 0)
        self.assertEqual(admission["missing_admitted_fields"], 200)
        self.assertEqual(rights["independent_rights_fields_accepted"], 200)
        expected = {(g["frozen_row"], name): field for g in read("groups.json")
                    for sense in g["senses"] for name, field in sense["fields"].items()}
        self.assertEqual({(r["frozen_row"], r["field"]) for r in rights["fields"]}, set(expected))
        for record in rights["fields"]:
            field = expected[record["frozen_row"], record["field"]]
            self.assertEqual(record["exact_source_span"], field["source_span"])
            self.assertEqual(record["license"], "CC-BY-SA-4.0")
            self.assertTrue(record["attribution"].strip())
            self.assertTrue(record["modification"].strip())
            self.assertEqual(record["independent_review"], "passed")
            self.assertEqual(record["rights_eligibility_status"], "passed_at_selected_text_boundary")
            self.assertEqual(record["accepted_text"], field["text"])
            self.assertTrue(record["reviewer"])
        for record in review["fields"]:
            field = expected[record["frozen_row"], record["field"]]
            self.assertEqual(record["accepted_text_quality_status"], "passed")
            self.assertEqual(record["accepted_text"], field["text"])
            self.assertEqual(record["rights_eligibility_status"], field["rights_eligibility_status"])
            self.assertTrue(record["review_reason"])
            self.assertTrue(record["rights_review_reason"])
            self.assertTrue(record["reviewer"])
        uses = {g["senses"][0]["sense_id"]: g["senses"][0]["use"] for g in read("groups.json")}
        self.assertEqual(len(review["usage"]), 50)
        for record in review["usage"]:
            self.assertEqual(record["accepted_usage"], uses[record["sense_id"]])
            self.assertEqual(record["accepted_usage_quality_status"], "passed")
            self.assertEqual(record["everyday_sense_status"], "passed")
        author = read("authored/field-review.json")
        self.assertFalse(author["independent_acceptance"])
        proposed = {(g["frozen_row"], name): field for g in read("authored/groups.json")
                    for sense in g["senses"] for name, field in sense["fields"].items()}
        for record in review["fields"]:
            self.assertEqual(record["proposed_text"], proposed[record["frozen_row"], record["field"]]["text"])
        changes = read("accepted-replacements.json")
        self.assertEqual({(r["frozen_row"], r["field"]) for r in changes}, {(3510, "english"), (3510, "chinese")})
        self.assertEqual((EVIDENCE / "COPYING").read_bytes().rstrip(),
                         (ROOT / "resources/core-pilot-79/COPYING").read_bytes().rstrip())
        notices = (EVIDENCE / "NOTICES.md").read_text(encoding="utf-8")
        for source in read("sources.json")["sources"]:
            self.assertIn(source["fixed_url"], notices)
            self.assertIn(source["history_url"], notices)
        rights_text = (EVIDENCE / "selected-rights.md").read_text(encoding="utf-8")
        self.assertIn("2006-01-01", rights_text)
        self.assertIn("200 selected fields", rights_text)
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
