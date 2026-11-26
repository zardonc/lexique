"""Check #162 reviewed content and preserved author proposals at the resource boundary."""
import csv
import json
from pathlib import Path
import unittest

from core_batch_test_helpers import (
    assert_core_load_inputs_rejected,
    verify_archived_contributor_captures,
    verify_archived_source_captures,
)
from core_pilot import load_inputs, MATERIALS, RESOURCE

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-162"
MEMBERSHIP = [
    ("environnement", "NOM"), ("litre", "NOM"), ("certitude", "NOM"),
    ("israélien", "ADJ"), ("compassion", "NOM"), ("séparation", "NOM"),
    ("ais", "NOM"), ("diviser", "VER"), ("flûte", "NOM"),
    ("protester", "VER"), ("base-ball", "NOM"), ("décharger", "VER"),
    ("fréquence", "NOM"), ("morgue", "NOM"), ("nièce", "NOM"),
    ("patienter", "VER"), ("pelle", "NOM"), ("revue", "NOM"),
    ("brisé", "ADJ"), ("coton", "NOM"), ("fusée", "NOM"),
    ("stable", "ADJ"), ("débuter", "VER"), ("quelconque", "ADJ"),
    ("serrure", "NOM"), ("spécialité", "NOM"), ("amiral", "NOM"),
    ("désigner", "VER"), ("inquiétude", "NOM"), ("académie", "NOM"),
    ("comique", "NOM"), ("caillou", "NOM"), ("tout-puissant", "ADJ"),
    ("paye", "NOM"), ("artistique", "ADJ"), ("explorer", "VER"),
    ("poignet", "NOM"),
]


def read(name):
    return json.loads((EVIDENCE / "authored" / name).read_text(encoding="utf-8"))


def reviewed(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch162Tests(unittest.TestCase):
    def test_owner_approved_ais_annotation_preserves_zero_fields_and_frozen_identity(self):
        admission = reviewed("admission.json")
        self.assertEqual((admission["independently_accepted_new_groups"],
                          admission["independently_accepted_new_fields"]), (36, 144))
        self.assertEqual(admission["missing_admitted_fields"], 4)
        self.assertEqual((admission["pending_independent_review_fields"],
                          admission["pending_independent_rights_fields"]), (0, 0))
        self.assertEqual((admission["annotation_exception_groups"],
                          admission["specialist_exception_groups"]), (1, 0))
        blocker = reviewed("content-blockers.json")["groups"][0]
        self.assertEqual((blocker["frozen_row"], blocker["headword"], blocker["lexique_cgram"]),
                         (4057, "ais", "NOM"))
        self.assertTrue(blocker["owner_exception_approved"])
        self.assertEqual(set(blocker["pending_fields"]),
                         {"french_definition", "ipa", "english", "chinese"})
        rejected = reviewed("rejected-proposals.json")["groups"][0]
        self.assertEqual(rejected["proposal"]["headword"], "ais")
        self.assertEqual(len(rejected["field_decisions"]), 4)
        group = reviewed("groups.json")[6]
        self.assertEqual(group["senses"], [])
        self.assertEqual((group["admitted_fields"], group["missing_fields"]), (0, 4))
        self.assertEqual((group["subtitle_lemma_per_million"], group["subtitle_pos_per_million"]),
                         ("10.14", "10.14"))
        approval = reviewed("owner-approval.json")
        self.assertEqual(approval["approved_by"], "project_owner")
        self.assertEqual(approval["approved_on"], "2026-11-26")
        self.assertEqual(approval["exact_reply"],
                         "批准 ais/NOM 为零字段注记（推荐）：保留第 4057 行身份与词频，四字段继续计为缺失。")
        annotations = reviewed("field-review.json")["annotation_exceptions"]
        self.assertEqual([(item["frozen_row"], item["headword"], item["pos"]) for item in annotations],
                         [(4057, "ais", "NOM")])
        for gap in reviewed("field-review.json")["unresolved_fields"]:
            self.assertEqual(gap["frozen_row"], 4057)
            self.assertEqual(gap["annotation_exception_decision"], group["owner_approval"])

    def test_reviewed_fields_match_final_texts_fixed_spans_and_rights_decisions(self):
        groups = reviewed("groups.json")
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        for group, original in zip(groups, read("groups.json"), strict=True):
            for key in ("frozen_row", "headword", "lexique_cgram", "subtitle_lemma_per_million",
                        "subtitle_pos_per_million"):
                self.assertEqual(group[key], original[key])
        selected = {(s["sense_id"], name): field for g in groups for s in g["senses"]
                    for name, field in s["fields"].items()}
        self.assertEqual(len(selected), 144)
        review = reviewed("field-review.json")
        self.assertTrue(review["independence"])
        self.assertTrue(review["independent_acceptance"])
        self.assertEqual(len(review["fields"]), 144)
        rights = {(f["sense_id"], f["field"]): f for f in reviewed("rights.json")["fields"]}
        captures = {c["source_id"]: c for c in reviewed("source-captures.json")["sources"]}
        raw = {sid: Path(c["raw_path"]).read_text(encoding="utf-8").splitlines()
               for sid, c in captures.items()}
        for decision in review["fields"]:
            key = decision["sense_id"], decision["field"]
            right = rights[key]
            self.assertTrue(decision["independence"])
            self.assertTrue(decision["review_reason"])
            self.assertTrue(right["selected_text_rights_reason"])
            self.assertEqual(right["independent_review"], "passed_at_selected_text_boundary")
            self.assertIn(key, selected)
            field = selected[key]
            self.assertEqual(field["language"],
                             {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}[key[1]])
            self.assertEqual(field["text"], decision["accepted_text"])
            self.assertEqual(field["text"], right["accepted_text"])
            self.assertEqual(field["independent_review"], "passed")
            self.assertEqual(decision["accepted_text_quality_status"], "passed")
            self.assertEqual(decision["rights_eligibility_status"], "passed_at_selected_text_boundary")
            self.assertEqual(decision["exact_source"]["revid"], captures[field["source_id"]]["revid"])
            span = field["source_span"]
            self.assertEqual(span, right["exact_source_span"])
            self.assertEqual(span["raw_source_excerpt"], decision["exact_source"]["reviewed_excerpt"])
            line = raw[field["source_id"]][span["line_1_based"] - 1]
            column = span["column_1_based"] - 1
            self.assertEqual(line[column:column + len(span["raw_source_excerpt"])], span["raw_source_excerpt"])
            before = raw[field["source_id"]][:span["line_1_based"]]
            language = next(line for line in reversed(before) if line.startswith("== "))
            self.assertIn("{{langue|fr}}", language)
            if key[1] != "ipa":
                pos = next(line for line in reversed(before) if line.startswith("=== {{S|"))
                self.assertIn("{{S|" + {"NOM": "nom", "ADJ": "adjectif", "VER": "verbe"}[decision["pos"]] + "|fr", pos)
        self.assertEqual(len(review["usage"]), 36)
        self.assertEqual(sum(u["everyday_sense_status"] == "passed" for u in review["usage"]), 36)
        self.assertTrue(all(r["independent_review"] == "passed" for r in review["spelling_relations"]))

    def test_replacements_keep_modern_use_and_exclude_other_language_imports(self):
        groups = {g["frozen_row"]: g for g in reviewed("groups.json")}
        replacements = reviewed("accepted-replacements.json")["fields"]
        self.assertEqual(len(replacements), 9)
        self.assertEqual({f["frozen_row"] for f in replacements}, {4061, 4071, 4077})
        baseball = groups[4061]["senses"][0]["fields"]
        self.assertIn("normalement neuf manches", baseball["french_definition"]["text"])
        self.assertIn("extra innings if tied", baseball["english"]["text"])
        self.assertIn("平局时延长", baseball["chinese"]["text"])
        rocket = groups[4071]["senses"][0]["fields"]
        self.assertIn("baguette", rocket["french_definition"]["text"])
        self.assertIn("attached to a stick", rocket["english"]["text"])
        self.assertNotIn("手掷", rocket["chinese"]["text"])
        admiral = groups[4077]["senses"][0]["fields"]
        for name in ("french_definition", "english", "chinese"):
            self.assertEqual(admiral[name]["source_span"]["line_1_based"], 35)
        self.assertIn("En France", admiral["french_definition"]["text"])
        self.assertNotIn("supreme commander", admiral["english"]["text"])
        explorer = next(s for s in reviewed("sources.json")["sources"] if s["title"] == "explorer")
        self.assertEqual(explorer["original_historical_dictionary_layers"], [])
        marker = next(m for m in explorer["external_reference_markers"] if m["line_1_based"] == 112)
        self.assertEqual(marker["independent_boundary_decision"],
                         "excluded_English_noun_section_not_French_verb_original")
        records = reviewed("modern-corroboration.json")["sources"]
        rule = next(r for r in records if r["id"] == "baseball-rule")
        self.assertTrue(Path(rule["raw_path"]).is_file())
        self.assertIn(rule["exact_excerpt"], Path(rule["text_path"]).read_text(encoding="utf-8"))

    def test_author_package_retains_pending_reviews_and_one_exact_use_blocker(self):
        admission, review, rights = read("admission.json"), read("field-review.json"), read("rights.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (37, 148))
        self.assertEqual((admission["independently_accepted_new_groups"],
                          admission["independently_accepted_new_fields"]), (0, 0))
        self.assertEqual((admission["pending_independent_review_fields"],
                          admission["pending_independent_rights_fields"],
                          admission["missing_admitted_fields"]), (148, 148, 148))
        self.assertEqual((admission["ordinary_everyday_candidate_groups"],
                          admission["blocked_research_candidate_groups"]), (36, 1))
        self.assertEqual((admission["annotation_exception_groups"],
                          admission["specialist_exception_groups"]), (0, 0))
        self.assertFalse(review["independent_acceptance"])
        self.assertIsNone(review["reviewer"])
        self.assertEqual(rights["independent_rights_fields_accepted"], 0)
        blocker = read("content-blockers.json")["groups"]
        self.assertEqual([(b["frozen_row"], b["headword"], b["lexique_cgram"]) for b in blocker],
                         [(4057, "ais", "NOM")])
        self.assertEqual(admission["unresolved_field_blockers"], blocker)
        sense = read("groups.json")[6]["senses"][0]
        self.assertEqual(sense["author_everyday_eligibility"], "unresolved")
        self.assertIn("vieilli", sense["fields"]["french_definition"]["source_span"]["raw_source_excerpt"])
        self.assertIn("vieilli", sense["fields"]["french_definition"]["text"])
        self.assertIn("old-fashioned", sense["fields"]["english"]["text"])
        self.assertIn("旧用", sense["fields"]["chinese"]["text"])

    def test_archived_bytes_bind_fields_frames_and_complete_contributor_credits(self):
        captures = read("source-captures.json")
        archive = Path(captures["capture_root"])
        self.assertEqual((archive.name, archive.parent.name), ("issue-162", "author-evidence"))
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        raw = verify_archived_source_captures(self, captures["sources"], sources, Path)
        self.assertEqual(len(raw), 38)
        for capture in captures["sources"]:
            for key in ("api_path", "raw_path"):
                self.assertTrue(Path(capture[key]).is_relative_to(archive))
        excerpts = read("source-excerpts.json")
        by_field = {(f["sense_id"], f["field"]): f for f in excerpts["fields"]}
        self.assertEqual(len(by_field), 148)
        for record in [*excerpts["fields"], *excerpts["supporting_evidence"]]:
            line = raw[record["source_id"]][record["line_1_based"] - 1]
            column = record["column_1_based"] - 1
            self.assertEqual(line[column:column + len(record["reviewed_excerpt"])], record["reviewed_excerpt"])
            self.assertNotRegex(record["reviewed_excerpt"].lstrip(), r"^#+\*")
            self.assertEqual(record["revid"], sources[record["source_id"]]["revid"])
        expected = {}
        languages = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
        pos_names = {"NOM": "nom", "ADJ": "adjectif", "VER": "verbe"}
        for group in read("groups.json"):
            sense = group["senses"][0]
            for name, field in sense["fields"].items():
                exact = by_field[sense["sense_id"], name]
                expected[sense["sense_id"], name] = field
                self.assertEqual(field["language"], languages[name])
                self.assertEqual(field["source_id"], exact["source_id"])
                self.assertEqual(field["source_span"]["raw_source_excerpt"], exact["reviewed_excerpt"])
                self.assertEqual(field["source_span"]["line_1_based"], exact["line_1_based"])
                before = raw[exact["source_id"]][:exact["line_1_based"]]
                self.assertRegex(next(line for line in reversed(before) if line.startswith("== ")),
                                 r"^==\s*\{\{langue\|fr\}\}\s*==$")
                if name == "ipa":
                    self.assertEqual(field["text"].strip("/"), exact["ipa_variant_excerpt"])
                    column = exact["ipa_variant_column_1_based"] - 1
                    self.assertEqual(exact["reviewed_excerpt"][column:column + len(exact["ipa_variant_excerpt"])],
                                     exact["ipa_variant_excerpt"])
                else:
                    pos = next(line for line in reversed(before) if line.startswith("=== {{S|"))
                    self.assertIn("{{S|" + pos_names[group["lexique_cgram"]] + "|fr", pos)
        for records in (read("rights.json")["fields"], read("field-review.json")["fields"]):
            self.assertEqual({(f["sense_id"], f["field"]) for f in records}, set(expected))
            for record in records:
                field = expected[record["sense_id"], record["field"]]
                self.assertIsNone(record["reviewer"])
                if "exact_source_span" in record:
                    self.assertEqual(record["exact_source_span"], field["source_span"])
                    self.assertEqual(record["license"], "CC-BY-SA-4.0")
                    self.assertEqual(record["independent_review"], "pending")
                    self.assertTrue(record["attribution"])
                    self.assertEqual(record["modification"], field["modification"])
                else:
                    self.assertEqual(record["proposed_text"], field["text"])
                    self.assertEqual(record["accepted_text_quality_status"], "pending")
        names, anonymous, _ = verify_archived_contributor_captures(
            self, captures["contributor_registry"]["parts"], Path
        )
        self.assertEqual(set(names), {s["title"] for s in sources.values()})
        for context in read("source-context.json")["current_content"]:
            self.assertEqual(context["current_contributors"], sorted(names[context["title"]]))
            self.assertEqual(context["current_registry_names"], len(names[context["title"]]))
            self.assertEqual(context["anonymous_contributors_count"], anonymous.get(context["title"], 0))
        self.assertFalse(captures["contributor_registry"]["selected_sentence_authorship_proved"])

    def test_converter_rejects_author_review_as_independent_acceptance(self):
        def replace_review(stage):
            (stage / "field-review.json").write_bytes((EVIDENCE / "authored/field-review.json").read_bytes())

        assert_core_load_inputs_rejected(
            self, RESOURCE, MATERIALS, load_inputs, replace_review,
            "A separate independent field review is required", "lexique-162-admission-"
        )

    def test_spelling_and_grammar_restrictions_preserve_exact_groups(self):
        groups = {g["frozen_row"]: g for g in read("groups.json")}
        baseball = groups[4061]
        sense = baseball["senses"][0]
        self.assertEqual(baseball["headword"], "base-ball")
        self.assertEqual(sense["source_headword"], "baseball")
        relations = read("field-review.json")["spelling_relations"]
        relation = next(r for r in relations if r["frozen_row"] == 4061)
        self.assertEqual(relation["relation"], "attested_french_orthographic_variant")
        self.assertEqual(relation["alias_admission"], "not_admitted")
        self.assertTrue(any("[[baseball]]" in r["reviewed_excerpt"] for r in relation["proposed_evidence"]))
        debuter = groups[4073]["senses"][0]["fields"]
        self.assertIn("sans complément d’objet direct", debuter["french_definition"]["text"])
        self.assertIn("rather than a direct object", debuter["english"]["text"])
        self.assertIn("不采用直接宾语", debuter["chinese"]["text"])
        self.assertIn("fr-accord-rég", groups[4069]["senses"][0]["fields"]["ipa"]["source_span"]["raw_source_excerpt"])
        self.assertEqual(groups[4081]["senses"][0]["fields"]["french_definition"]["source_span"]["line_1_based"], 106)

    def test_notices_legal_materials_and_excluded_import_are_retained(self):
        self.assertEqual((EVIDENCE / "COPYING").read_bytes().rstrip(), (RESOURCE / "COPYING").read_bytes().rstrip())
        notices = (EVIDENCE / "NOTICES.md").read_text(encoding="utf-8")
        for source in read("sources.json")["sources"]:
            self.assertIn(source["fixed_url"], notices)
            self.assertIn(source["history_url"], notices)
        for authority in read("rights.json")["authority_captures"]:
            self.assertTrue(Path(authority["path"]).is_file())
            self.assertEqual(Path(authority["path"]).read_bytes(), Path(authority["reuse_origin"]).read_bytes())
        fusée = next(s for s in read("rights.json")["sources"] if s["title"] == "fusée")
        cfc = next(r for r in fusée["reference_markers"] if "Import:CFC" in r["raw_reference"])
        self.assertEqual(cfc["line_1_based"], 56)
        self.assertEqual(cfc["author_rights_proposal"], "excluded_from_selected_content")
        self.assertEqual({r["line_1_based"] for r in fusée["selected_locations"]}, {11, 16})

    def test_each_frozen_group_retains_four_sourced_proposed_fields(self):
        for group in read("groups.json"):
            with self.subTest(row=group["frozen_row"]):
                self.assertEqual(len(group["senses"]), 1)
                sense = group["senses"][0]
                self.assertTrue(sense["use"].strip())
                self.assertIsNone(sense["cefr"])
                self.assertEqual(sense["independent_review"], "pending")
                self.assertEqual(sense["resource_admission"], "pending_separate_independent_review")
                self.assertEqual(set(sense["fields"]),
                                 {"french_definition", "ipa", "english", "chinese"})
                for field in sense["fields"].values():
                    self.assertTrue(field["text"].strip())
                    self.assertTrue(field["source_span"]["raw_source_excerpt"].strip())
                    self.assertTrue(field["modification"].strip())
                    self.assertEqual(field["independent_review"], "pending")

    def test_frozen_membership_order_and_frequency_strings(self):
        groups = read("groups.json")
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        self.assertEqual([g["frozen_row"] for g in groups], list(range(4051, 4088)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            frozen = list(csv.DictReader(handle))
        self.assertEqual(len(frozen), 4087)
        for group, row in zip(groups, frozen[4050:4087], strict=True):
            self.assertEqual((group["headword"], group["lexique_cgram"]),
                             (row["word"], row["lexique_cgram"]))
            for key in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                self.assertEqual(group[key], row[key])


if __name__ == "__main__":
    unittest.main()
