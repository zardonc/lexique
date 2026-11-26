"""Check #160 at the approved resource/admission boundary."""
import csv
import json
from pathlib import Path
import shutil
import unittest

from core_batch_test_helpers import (
    assert_core_load_inputs_rejected,
    verify_archived_contributor_captures,
    verify_archived_source_captures,
)
from core_pilot import load_inputs, MATERIALS, RESOURCE

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-160"
MEMBERSHIP = [
    ('perle', 'NOM'),
    ('refroidir', 'VER'),
    ('rex', 'NOM'),
    ('terrasse', 'NOM'),
    ('ménager', 'VER'),
    ('attrape', 'NOM'),
    ('roux', 'ADJ'),
    ('égarer', 'VER'),
    ('balcon', 'NOM'),
    ('effectuer', 'VER'),
    ('chimique', 'ADJ'),
    ('porteur', 'NOM'),
    ('sensé', 'ADJ'),
    ('tourmenter', 'VER'),
    ('insulte', 'NOM'),
    ('jet', 'NOM'),
    ('tracer', 'VER'),
    ('brûlant', 'ADJ'),
    ('coupure', 'NOM'),
    ('horloge', 'NOM'),
    ('sir', 'NOM'),
    ('caché', 'ADJ'),
    ('resto', 'NOM'),
    ('coiffure', 'NOM'),
    ('coutume', 'NOM'),
    ('fiche', 'NOM'),
    ('pharmacie', 'NOM'),
    ('égoïste', 'ADJ'),
    ('complot', 'NOM'),
    ('habillé', 'ADJ'),
    ('référence', 'NOM'),
    ('errer', 'VER'),
    ('situer', 'VER'),
    ('gangster', 'NOM'),
    ('pirate', 'NOM'),
    ('dépression', 'NOM'),
    ('officiellement', 'ADV'),
    ('pou', 'NOM'),
    ('privilège', 'NOM'),
    ('souhait', 'NOM'),
    ('barque', 'NOM'),
    ('député', 'NOM'),
    ('cafard', 'NOM'),
    ('critiquer', 'VER'),
    ('soie', 'NOM'),
    ('invention', 'NOM'),
    ('parrain', 'NOM'),
    ('revivre', 'VER'),
    ('tracasser', 'VER'),
    ('radar', 'NOM'),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch160Tests(unittest.TestCase):
    def test_owner_annotation_preserves_rex_identity_and_four_missing_fields(self):
        approval, review = read("owner-approval.json"), read("field-review.json")
        self.assertEqual((approval["frozen_row"], approval["headword"], approval["lexique_cgram"]),
                         (3953, "rex", "NOM"))
        self.assertEqual(approval["exact_reply"],
                         "批准 rex/NOM 为零字段注记（推荐）：保留第 3953 行及词频，四字段继续计为缺失。")
        group = next(g for g in read("groups.json") if g["frozen_row"] == 3953)
        self.assertEqual((group["subtitle_lemma_per_million"], group["subtitle_pos_per_million"]),
                         ("10.57", "10.57"))
        self.assertEqual(group["senses"], [])
        self.assertEqual((group["admitted_fields"], group["missing_fields"]), (0, 4))
        self.assertFalse(any(f["frozen_row"] == 3953 for f in review["fields"]))
        self.assertEqual([g["frozen_row"] for g in review["unresolved_fields"]], [3953] * 4)
        self.assertEqual(read("content-blockers.json")["blocked_groups"], 0)

    def test_rejected_rex_proposal_remains_preserved_after_owner_annotation(self):
        rex = next(g for g in read("groups.json") if g["frozen_row"] == 3953)
        self.assertEqual((rex["headword"], rex["lexique_cgram"]), ("rex", "NOM"))
        self.assertEqual(rex["senses"], [])
        blockers = read("content-blockers.json")
        self.assertEqual((blockers["blocked_groups"], blockers["blocked_fields"]), (0, 0))
        self.assertEqual(blockers["approved_annotation_exceptions"], 1)
        self.assertEqual(blockers["missing_annotation_fields"], 4)
        self.assertEqual(blockers["approved_specialist_exceptions"], 0)
        rejected = read("rejected-proposals.json")
        self.assertEqual(len(rejected["fields"]), 4)
        self.assertEqual(rejected["groups"][0], read("authored/groups.json")[2])
        self.assertEqual(len(blockers["blockers"][0]["owner_alternatives"]), 3)

    def test_converter_rejects_author_proposals_without_independent_review(self):
        def replace_review(stage):
            shutil.copyfile(EVIDENCE / "authored/field-review.json", stage / "field-review.json")

        assert_core_load_inputs_rejected(
            self, RESOURCE, MATERIALS, load_inputs, replace_review,
            "A separate independent field review is required", "lexique-160-admission-"
        )

    def test_exact_frozen_groups_retain_accepted_content_or_explicit_gaps(self):
        if (EVIDENCE / "groups.json").exists():
            groups = read("groups.json")
        else:
            canonical = json.loads((ROOT / "resources/core-pilot-79/curated-inputs.json").read_text(encoding="utf-8"))
            groups = [g for g in canonical["groups"] if 3951 <= g["frozen_row"] <= 4000]
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP,
                         "The frozen #160 resource slice lacks complete batch content")
        self.assertEqual([g["frozen_row"] for g in groups], list(range(3951, 4001)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(encoding="utf-8-sig", newline="") as handle:
            frozen = list(csv.DictReader(handle))
        self.assertEqual(len(frozen), 4087)
        for group in groups:
            with self.subTest(row=group["frozen_row"]):
                row = frozen[group["frozen_row"] - 1]
                self.assertEqual((group["headword"], group["lexique_cgram"]), (row["word"], row["lexique_cgram"]))
                for frequency in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                    self.assertEqual(group[frequency], row[frequency])
                if group["frozen_row"] == 3953:
                    self.assertEqual(group["senses"], [])
                    self.assertEqual((group["admitted_fields"], group["missing_fields"]), (0, 4))
                    self.assertEqual(group["status"], "approved_annotation_exception")
                    continue
                self.assertEqual(len(group["senses"]), 1)
                sense = group["senses"][0]
                self.assertTrue(sense["use"].strip())
                self.assertIsNone(sense["cefr"])
                self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                for field in sense["fields"].values():
                    self.assertTrue(field["text"].strip())
                    self.assertTrue(field["modification"].strip())

    def test_accepted_fields_have_independent_source_specific_rights_and_quality(self):
        rights, review, admission = read("rights.json"), read("field-review.json"), read("admission.json")
        self.assertEqual(rights["proposed_fields"], 200)
        self.assertEqual(rights["independent_rights_fields_accepted"], 196)
        self.assertEqual(len(rights["fields"]), 196)
        self.assertTrue(review["independent_acceptance"])
        self.assertTrue(review["reviewer"])
        self.assertEqual((len(review["fields"]), len(review["usage"])), (196, 49))
        self.assertEqual(admission["missing_admitted_fields"], 4)
        self.assertEqual(admission["independently_accepted_new_groups"], 49)
        self.assertEqual(admission["independently_accepted_new_fields"], 196)
        self.assertEqual(admission["pending_independent_review_fields"], 0)
        self.assertEqual(admission["pending_independent_rights_fields"], 0)
        self.assertEqual((admission["newly_admitted_fields"], admission["canonical_new_fields"]), (0, 0))
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual(admission["annotation_exception_groups"], 1)
        self.assertEqual(admission["specialist_exception_groups"], 0)
        expected = {(sense["sense_id"], name): field for group in read("groups.json")
                    for sense in group["senses"] for name, field in sense["fields"].items()}
        self.assertEqual({(r["sense_id"], r["field"]) for r in rights["fields"]}, set(expected))
        for record in rights["fields"]:
            field = expected[record["sense_id"], record["field"]]
            self.assertEqual(record["source_id"], field["source_id"])
            self.assertEqual(record["exact_source_span"], field["source_span"])
            self.assertTrue(record["attribution"].strip())
            self.assertTrue(record["modification"].strip())
            self.assertEqual(record["independent_review"], "passed")
            self.assertTrue(record["reviewer"])
            self.assertEqual(record["accepted_text"], field["text"])
            self.assertTrue(record["selected_text_rights_reason"])
        proposed = {(s["sense_id"], k): f for g in read("authored/groups.json")
                    for s in g["senses"] for k, f in s["fields"].items()}
        for record in review["fields"]:
            key = record["sense_id"], record["field"]
            self.assertEqual(record["accepted_text"], expected[key]["text"])
            self.assertEqual(record["proposed_text"], proposed[key]["text"])
            self.assertEqual(record["accepted_text_quality_status"], "passed")
            self.assertTrue(record["independence"])
            self.assertTrue(record["review_reason"])
        self.assertEqual((EVIDENCE / "COPYING").read_bytes(), (RESOURCE / "COPYING").read_bytes().rstrip() + b"\n")

    def test_fixed_source_bytes_support_every_field_and_french_pos(self):
        captures = read("source-captures.json")
        sources = {source["id"]: source for source in read("sources.json")["sources"]}
        self.assertEqual(len(sources), 50)
        self.assertTrue(Path(captures["capture_root"]).is_absolute())
        lines = verify_archived_source_captures(self, captures["sources"], sources, Path)
        excerpts = read("source-excerpts.json")
        fields = {(record["sense_id"], record["field"]): record for record in excerpts["fields"]}
        expected_pos = {"NOM": "nom", "VER": "verbe", "ADJ": "adjectif", "ADV": "adverbe"}
        for group in read("groups.json"):
            if not group["senses"]:
                continue
            sense = group["senses"][0]
            for name, field in sense["fields"].items():
                with self.subTest(row=group["frozen_row"], field=name):
                    record = fields[sense["sense_id"], name]
                    self.assertEqual(record["source_id"], field["source_id"])
                    self.assertEqual(record["revid"], sources[field["source_id"]]["revid"])
                    self.assertEqual(record["reviewed_excerpt"],
                                     lines[field["source_id"]][record["line_1_based"] - 1])
                    self.assertEqual(field["source_span"]["raw_source_excerpt"], record["reviewed_excerpt"])
                    self.assertEqual(field["independent_review"], "passed")
                    self.assertEqual(field["language"], {"french_definition": "fr", "ipa": "und",
                                                         "english": "en", "chinese": "zh"}[name])
                    before = lines[field["source_id"]][:record["line_1_based"]]
                    language = next(line for line in reversed(before) if line.startswith("== "))
                    pos = next(line for line in reversed(before) if line.startswith("=== {{S|")
                               and not any(tag in line for tag in ("étymologie", "prononciation", "références")))
                    self.assertEqual(language, "== {{langue|fr}} ==")
                    self.assertIn("{{S|" + expected_pos[group["lexique_cgram"]] + "|fr", pos)
                    if name == "ipa":
                        self.assertIn(field["text"].strip("/"), record["reviewed_excerpt"])
                    else:
                        self.assertTrue(record["reviewed_excerpt"].startswith("# "))
            self.assertIsNone(sense["cefr"])
            self.assertEqual(sense["resource_admission"], "independently_accepted_parent_admission_pending")
        self.assertEqual(len(fields), 196)
        for record in excerpts["supporting_evidence"]:
            self.assertFalse(record["reviewed_excerpt"].lstrip().startswith("#*"))
            raw = lines[record["source_id"]][record["line_1_based"] - 1]
            start = record["column_1_based"] - 1
            self.assertEqual(raw[start:start + len(record["reviewed_excerpt"])], record["reviewed_excerpt"])

    def test_contributor_credit_matches_complete_retained_pagination(self):
        captures = read("source-captures.json")
        names, anonymous, _ = verify_archived_contributor_captures(
            self, captures["contributor_registry"]["parts"], Path)
        for context in read("source-context.json")["current_content"]:
            self.assertEqual(set(context["current_contributors"]), names[context["title"]])
            self.assertEqual(context["current_registry_names"], len(names[context["title"]]))
            self.assertEqual(context["anonymous_contributors_count"], anonymous[context["title"]])
            self.assertTrue(context["history_url"].endswith("&action=history"))

    def test_sir_binding_and_unselected_import_have_separate_evidence(self):
        context = read("source-context.json")
        relation = next(r for r in context["spelling_relations"] if r["frozen_row"] == 3971)
        self.assertEqual((relation["frozen_headword"], relation["source_headword"]), ("sir", "Sir"))
        self.assertEqual(relation["relation"], "attested_french_capitalized_title")
        self.assertEqual(relation["alias_admission"], "not_admitted")
        self.assertEqual(relation["independent_review"], "passed")
        spelling = relation["external_spelling_evidence"]
        self.assertIn('"sir"', Path(spelling["raw_path"]).read_text(encoding="utf-8"))
        self.assertEqual(spelling["field_content_reused"], False)
        coupure = next(c for c in context["current_content"] if c["title"] == "coupure")
        cfc = next(r for r in coupure["external_reference_markers"] if "Import:CFC" in r["raw_reference"])
        self.assertEqual(cfc["author_rights_proposal"], "excluded_from_selected_content")
        self.assertEqual(cfc["line_1_based"], 37)
        self.assertEqual({r["line_1_based"] for r in coupure["selected_locations"]}, {9, 33})
        self.assertEqual(read("content-blockers.json")["blocked_groups"], 0)

    def test_accepted_constructions_and_original_expression_scope_remain_readable(self):
        senses = {g["headword"]: g["senses"][0] for g in read("groups.json") if g["senses"]}
        self.assertEqual(senses["sir"]["fields"]["ipa"]["text"], "/sœʁ/")
        self.assertIn("given name", senses["sir"]["fields"]["english"]["text"])
        self.assertIn("普通敬称", senses["sir"]["fields"]["chinese"]["text"])
        self.assertIn("territorial legislature", senses["député"]["fields"]["english"]["text"])
        self.assertIn("省、地区", senses["député"]["fields"]["chinese"]["text"])
        self.assertIn("hyperbolically", senses["revivre"]["fields"]["english"]["text"])
        self.assertEqual(senses["balcon"]["fields"]["chinese"]["text"],
                         "阳台：从建筑外墙凸出、四周设有栏杆的平台。")
        replacements = read("review-corrections.json")["replacements"]
        self.assertEqual(len(replacements), 12)
        self.assertEqual({r["headword"] for r in replacements}, {"balcon", "sir", "député", "revivre"})
        for replacement in replacements:
            self.assertNotEqual(replacement["proposed_text"], replacement["accepted_text"])
            self.assertTrue(replacement["reason"])


if __name__ == '__main__':
    unittest.main()
