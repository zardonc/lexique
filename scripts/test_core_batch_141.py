"""Check ticket #141 at the approved resource/admission seam."""
import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-141"
MEMBERSHIP = [
    ("absent", "ADJ"), ("douceur", "NOM"), ("massacrer", "VER"),
    ("échantillon", "NOM"), ("recommander", "VER"), ("défoncer", "VER"),
    ("vendeur", "NOM"), ("couvent", "NOM"), ("solitaire", "ADJ"),
    ("ôter", "VER"), ("mineur", "NOM"), ("légal", "ADJ"),
    ("came", "NOM"), ("masse", "NOM"), ("splendide", "ADJ"),
    ("dialogue", "NOM"), ("missile", "NOM"), ("bien-aimé", "NOM"),
    ("dater", "VER"), ("marquis", "NOM"), ("spécialiste", "NOM"),
    ("baleine", "NOM"), ("exceptionnel", "ADJ"), ("habitant", "NOM"),
    ("langage", "NOM"), ("savon", "NOM"), ("polonais", "ADJ"),
    ("tissu", "NOM"), ("do", "NOM"), ("brigade", "NOM"),
    ("poumon", "NOM"), ("tente", "NOM"), ("villa", "NOM"),
    ("adjudant", "NOM"), ("séjour", "NOM"), ("tournage", "NOM"),
    ("las", "ADJ"), ("ski", "NOM"), ("motif", "NOM"),
    ("beau-père", "NOM"), ("dispute", "NOM"), ("gonfler", "VER"),
    ("issue", "NOM"), ("amateur", "NOM"), ("investir", "VER"),
    ("pâle", "ADJ"), ("accoucher", "VER"), ("basket", "NOM"),
    ("pâte", "NOM"), ("séduire", "VER"),
]
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch141Tests(unittest.TestCase):
    def test_exact_frozen_groups_have_complete_bound_candidate_content(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(encoding="utf-8-sig") as handle:
            frozen = list(csv.DictReader(handle))[3000:3050]
        self.assertEqual([(r["word"], r["lexique_cgram"]) for r in frozen], MEMBERSHIP)
        groups = read("groups.json")
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        self.assertEqual([g["frozen_row"] for g in groups], list(range(3001, 3051)))
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        for group, row in zip(groups, frozen, strict=True):
            self.assertEqual(group["subtitle_lemma_per_million"], row["subtitle_lemma_per_million"])
            self.assertEqual(group["subtitle_pos_per_million"], row["subtitle_pos_per_million"])
            self.assertGreaterEqual(len(group["senses"]), 1)
            for sense in group["senses"]:
                self.assertTrue(sense["use"])
                self.assertEqual(sense["source_headword"], group["headword"])
                self.assertEqual(set(sense["fields"]), set(LANGUAGES))
                for name, field in sense["fields"].items():
                    self.assertEqual(field["language"], LANGUAGES[name])
                    self.assertTrue(field["text"].strip())
                    self.assertIn(field["source_id"], sources)
                    self.assertTrue(field["source_span"]["raw_source_excerpt"])
                    self.assertTrue(field["modification"])

    def test_author_fields_have_rights_and_remain_pending_separate_review(self):
        admission = read("admission.json")
        review = read("field-review.json")
        rights = read("rights.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual(admission["proposed_fields"], 200)
        self.assertEqual(admission["independently_accepted_new_fields"], 0)
        self.assertEqual(admission["missing_admitted_fields"], 200)
        self.assertFalse(review["independent_acceptance"])
        self.assertIsNone(review["reviewer"])
        fields = {(r["sense_id"], r["field"]): r for r in review["fields"]}
        legal = {(r["sense_id"], r["field"]): r for r in rights["fields"]}
        self.assertEqual(len(fields), 200)
        self.assertEqual(set(fields), set(legal))
        self.assertEqual(len(review["usage"]), 50)
        self.assertEqual(len(review["spelling_relations"]), 50)
        for group in read("groups.json"):
            for sense in group["senses"]:
                self.assertEqual(sense["independent_review"], "pending")
                for name, field in sense["fields"].items():
                    key = sense["sense_id"], name
                    self.assertEqual(fields[key]["proposed_text"], field["text"])
                    self.assertEqual(fields[key]["accepted_text_quality_status"], "pending")
                    self.assertIsNone(fields[key]["reviewer"])
                    self.assertEqual(legal[key]["source_id"], field["source_id"])
                    self.assertEqual(legal[key]["license"], "CC-BY-SA-4.0")
                    self.assertEqual(legal[key]["independent_review"], "pending")
                    self.assertTrue(legal[key]["attribution"])
                    self.assertTrue(legal[key]["author_rights_basis"])
                    self.assertIsNone(legal[key]["reviewer"])
        self.assertEqual(read("authored/groups.json"), read("groups.json"))
        self.assertEqual(read("authored/field-review.json"), review)
        self.assertEqual(read("authored/rights.json"), rights)
        self.assertEqual((EVIDENCE / "COPYING").read_bytes().rstrip(),
                         (ROOT / "resources/core-pilot-79/COPYING").read_bytes().rstrip())
        self.assertTrue((EVIDENCE / "NOTICES.md").is_file())


if __name__ == "__main__":
    unittest.main()
