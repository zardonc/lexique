#!/usr/bin/env python3
"""Unit tests for Option 2 FLELex validation/alignment and core noun-gender triage."""

from __future__ import annotations

import gzip
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from cefr_gender_pilot import (  # noqa: E402
    FLELEX_COLUMNS,
    FlelexValidationError,
    align_flelex_to_core,
    assert_zero_nc_levels_in_tracked_payload,
    extract_wiktionary_gender_markers,
    load_lexique_noun_genders,
    recover_full_header_line,
    resolve_notebook_headword_level,
    triage_core_noun_genders,
    validate_flelex_beacco_tsv,
    verify_pilot_decisions,
)


def make_tsv(rows: list[tuple[str, ...]], *, line_ending: str = "\r\n", header: tuple[str, ...] = FLELEX_COLUMNS) -> bytes:
    lines = ["\t".join(header)] + ["\t".join(r) for r in rows]
    return (line_ending.join(lines) + line_ending).encode("utf-8")


class CefrGenderPilotTests(unittest.TestCase):
    def test_validate_flelex_accepts_crlf_and_lf(self) -> None:
        sample = [
            ("maison", "NOM", "120.5", "80.0", "40.0", "20.0", "10.0", "5.0", "275.5", "A1"),
            ("cœur", "NOM", "10.0", "25.0", "15.0", "5.0", "0.0", "0.0", "55.0", "A2"),
        ]
        crlf_rows = validate_flelex_beacco_tsv(make_tsv(sample, line_ending="\r\n"))
        lf_rows = validate_flelex_beacco_tsv(make_tsv(sample, line_ending="\n"))
        self.assertEqual(len(crlf_rows), 2)
        self.assertEqual(crlf_rows, lf_rows)
        self.assertEqual(crlf_rows[0]["level"], "A1")

    def test_validate_flelex_rejects_wrong_header_and_corrupt_rows(self) -> None:
        # 8-column FleLex_TT header without level must fail clearly
        wrong_header = ("word", "tag", "freq_A1", "freq_A2", "freq_B1", "freq_B2", "freq_C1", "freq_total")
        with self.assertRaisesRegex(FlelexValidationError, "Invalid FLELex header"):
            validate_flelex_beacco_tsv(
                make_tsv([("maison", "NOM", "1", "1", "1", "1", "1", "5")], header=wrong_header)
            )

        # Invalid CEFR level
        with self.assertRaisesRegex(FlelexValidationError, "invalid CEFR level"):
            validate_flelex_beacco_tsv(
                make_tsv([("maison", "NOM", "1", "0", "0", "0", "0", "0", "1", "B3")])
            )

        # Negative frequency
        with self.assertRaisesRegex(FlelexValidationError, "non-negative"):
            validate_flelex_beacco_tsv(
                make_tsv([("maison", "NOM", "-0.1", "0", "0", "0", "0", "0", "0", "A1")])
            )

        # Duplicate (word, tag)
        with self.assertRaisesRegex(FlelexValidationError, "duplicate"):
            validate_flelex_beacco_tsv(
                make_tsv(
                    [
                        ("maison", "NOM", "1", "0", "0", "0", "0", "0", "1", "A1"),
                        ("maison", "NOM", "2", "0", "0", "0", "0", "0", "2", "A2"),
                    ]
                )
            )

        # Size limit
        with self.assertRaisesRegex(FlelexValidationError, "exceeds maximum size"):
            validate_flelex_beacco_tsv(b"x" * 128, max_bytes=64)

    def test_wiktionary_gender_extraction_handles_regular_epicene_and_uncertain(self) -> None:
        self.assertEqual(
            extract_wiktionary_gender_markers("'''maison''' {{pron|me.zɔ̃|fr}} {{f}}", "# Déf"),
            ("f", False, "ipa"),
        )
        self.assertEqual(
            extract_wiktionary_gender_markers("'''enfant''' {{pron|ɑ̃.fɑ̃|fr}} {{mf}}", "# Déf"),
            ("mf", False, "ipa"),
        )
        self.assertEqual(
            extract_wiktionary_gender_markers("'''gens''' {{pron|ʒɑ̃|fr}}, {{mf ?|fr}}", "# Déf"),
            (None, True, "ipa"),
        )
        self.assertEqual(
            extract_wiktionary_gender_markers("{{pron|pʁɔ.fɛ.sœʁ|fr}}", "# Personne qui enseigne."),
            (None, False, None),
        )

    def test_triage_routes_auto_homograph_conflict_missing_and_non_noun(self) -> None:
        curated = {
            "groups": [
                {
                    "frozen_row": 252,
                    "headword": "maison",
                    "lexique_cgram": "NOM",
                    "senses": [
                        {
                            "sense_id": "s-maison",
                            "source_headword": "maison",
                            "use": "Feminine noun: a building used as a dwelling.",
                            "fields": {
                                "ipa": {"source_id": "src1", "source_span": {"line_1_based": 9, "raw_source_excerpt": "'''maison''' {{pron|me.zɔ̃|fr}} {{f}}"}},
                                "french_definition": {"text": "Bâtiment.", "source_id": "src1", "source_span": {"line_1_based": 10, "raw_source_excerpt": "# Bâtiment."}},
                            },
                        }
                    ],
                },
                {
                    "frozen_row": 551,
                    "headword": "livre",
                    "lexique_cgram": "NOM",
                    "senses": [
                        {
                            "sense_id": "s-livre",
                            "source_headword": "livre",
                            "use": "Masculine noun: physical book.",
                            "fields": {
                                "ipa": {"source_id": "src2", "source_span": {"line_1_based": 11, "raw_source_excerpt": "'''livre''' {{pron|livʁ|fr}} {{m}}"}},
                                "french_definition": {"text": "Assemblage.", "source_id": "src2", "source_span": {"line_1_based": 12, "raw_source_excerpt": "# Assemblage."}},
                            },
                        }
                    ],
                },
                {
                    "frozen_row": 669,
                    "headword": "flic",
                    "lexique_cgram": "NOM",
                    "senses": [
                        {
                            "sense_id": "s-flic",
                            "source_headword": "flic",
                            "use": "Noun; selected masculine use for a police officer, in slang.",
                            "fields": {
                                "ipa": {"source_id": "src3", "source_span": {"line_1_based": 10, "raw_source_excerpt": "'''flic''' {{pron|flik|fr}} {{mf}}"}},
                                "french_definition": {"text": "Policier.", "source_id": "src3", "source_span": {"line_1_based": 11, "raw_source_excerpt": "# Policier."}},
                            },
                        }
                    ],
                },
                {
                    "frozen_row": 253,
                    "headword": "gens",
                    "lexique_cgram": "NOM",
                    "senses": [
                        {
                            "sense_id": "s-gens",
                            "source_headword": "gens",
                            "use": "Plural-only noun.",
                            "fields": {
                                "ipa": {"source_id": "src4", "source_span": {"line_1_based": 10, "raw_source_excerpt": "'''gens''' {{pron|ʒɑ̃|fr}}, {{mf ?|fr}}"}},
                                "french_definition": {"text": "Personnes.", "source_id": "src4", "source_span": {"line_1_based": 11, "raw_source_excerpt": "# Personnes."}},
                            },
                        }
                    ],
                },
                {
                    "frozen_row": 1,
                    "headword": "être",
                    "lexique_cgram": "AUX",
                    "senses": [
                        {
                            "sense_id": "s-etre",
                            "source_headword": "être",
                            "use": "Auxiliary.",
                            "fields": {
                                "ipa": {"source_id": "src5", "source_span": {"line_1_based": 8, "raw_source_excerpt": "'''être''' {{pron|ɛtʁ|fr}}"}},
                                "french_definition": {"text": "Auxiliaire.", "source_id": "src5", "source_span": {"line_1_based": 53, "raw_source_excerpt": "# Auxiliaire."}},
                            },
                        }
                    ],
                },
            ]
        }
        lex_info = {
            "lemma": {"maison": {"f"}, "livre": {"m", "f"}, "flic": {"m"}},
            "ortho": {},
            "homographs": {"livre": set()},
        }
        res = triage_core_noun_genders(curated, lex_info)
        self.assertEqual(res["non_noun_senses_null"], 1)
        self.assertEqual(res["auto_accepted_count"], 1)
        self.assertEqual(res["auto_accepted"][0]["headword"], "maison")
        queues = {r["headword"]: r["queue"] for r in res["residuals"]}
        self.assertEqual(queues["livre"], "residual_homograph")
        self.assertEqual(queues["flic"], "residual_conflict")
        self.assertEqual(queues["gens"], "residual_missing_or_uncertain_span")

    def test_flelex_alignment_strict_pos_aliases_specialist_and_notebook_rule(self) -> None:
        curated = {
            "groups": [
                {"frozen_row": 1, "headword": "être", "lexique_cgram": "AUX", "senses": [{"sense_id": "s-etre", "source_headword": "être", "use": "Aux"}]},
                {"frozen_row": 13, "headword": "la", "lexique_cgram": "ART:def", "senses": [{"sense_id": "s-la-art", "source_headword": "la", "use": "Definite article"}]},
                {"frozen_row": 128, "headword": "par", "lexique_cgram": "NOM", "senses": [{"sense_id": "s-par-golf", "source_headword": "par", "use": "Golf strokes"}]},
                {"frozen_row": 129, "headword": "par", "lexique_cgram": "PRE", "senses": [{"sense_id": "s-par-pre", "source_headword": "par", "use": "Preposition"}]},
                {"frozen_row": 483, "headword": "coeur", "lexique_cgram": "NOM", "senses": [{"sense_id": "s-coeur", "source_headword": "cœur", "use": "Heart"}]},
                {"frozen_row": 899, "headword": "ferme", "lexique_cgram": "ADJ", "senses": [{"sense_id": "s-ferme-adj", "source_headword": "ferme", "use": "Firm"}]},
                {"frozen_row": 900, "headword": "ferme", "lexique_cgram": "NOM", "senses": [{"sense_id": "s-ferme-nom", "source_headword": "ferme", "use": "Farm", "excluded_uses": ["Slang sense"]}]},
                {"frozen_row": 150, "headword": "quelqu'un", "lexique_cgram": "PRO:ind", "senses": [{"sense_id": "s-quelquun", "source_headword": "quelqu’un", "use": "Someone"}]},
            ]
        }
        relations = [{"frozen_headword": "quelqu'un", "source_headword": "quelqu’un"}]
        flelex = [
            {"word": "être", "tag": "NOM", "freq_total": 10.0, "level": "B1"},
            {"word": "être", "tag": "VER", "freq_total": 9000.0, "level": "A1"},
            {"word": "la", "tag": "PRO", "freq_total": 500.0, "level": "A1"},
            {"word": "la", "tag": "NOM", "freq_total": 2.0, "level": "C2"},
            {"word": "par", "tag": "NOM", "freq_total": 5.0, "level": "B2"},
            {"word": "par", "tag": "PRP", "freq_total": 3000.0, "level": "A1"},
            {"word": "coeur", "tag": "NOM", "freq_total": 200.0, "level": "A1"},
            {"word": "ferme", "tag": "ADJ", "freq_total": 40.0, "level": "B1"},
            {"word": "ferme", "tag": "NOM", "freq_total": 80.0, "level": "A2"},
            {"word": "quelqu'un", "tag": "PRO", "freq_total": 150.0, "level": "A1"},
        ]
        aligned = align_flelex_to_core(curated, relations, flelex)
        matched = aligned["matched_by_sense_id"]
        self.assertEqual(matched["s-etre"]["level"], "A1")
        self.assertEqual(matched["s-quelquun"]["level"], "A1")
        self.assertNotIn("s-la-art", matched)  # Must not borrow PRO or musical NOM=C2
        self.assertNotIn("s-par-golf", matched)  # Specialist row 128 suppressed
        self.assertEqual(matched["s-par-pre"]["level"], "A1")
        self.assertEqual(matched["s-coeur"]["level"], "A1")

        # Notebook Q8 rule: core record with both ferme/ADJ (B1) + ferme/NOM (A2) shows foundational A2
        self.assertEqual(
            resolve_notebook_headword_level("ferme", ["s-ferme-adj", "s-ferme-nom"], matched, flelex),
            "A2",
        )
        # Non-core MDX/CSV record for "ferme" has 2 distinct FLELex levels (A2, B1) -> None (ambiguous)
        self.assertIsNone(resolve_notebook_headword_level("ferme", None, matched, flelex))
        # Non-core MDX/CSV record for "coeur" has 1 unambiguous FLELex level -> A1
        self.assertEqual(resolve_notebook_headword_level("coeur", None, matched, flelex), "A1")

    def test_load_lexique_recovers_plural_gender_and_isolates_pure_blank_lemmas(self) -> None:
        header = "ortho\tlemme\tcgram\tgenre\tnombre\tislem\n"
        rows = [
            "maison\tmaison\tNOM\t\ts\t1\n",
            "maisons\tmaison\tNOM\tf\tp\t0\n",
            "docteur\tdocteur\tNOM\tm\ts\t1\n",
            "doctoresse\tdocteur\tNOM\tf\ts\t0\n",
            "livre\tlivre\tNOM\t\t\t1\n",
            "livres\tlivre\tNOM\t\tp\t0\n",
        ]
        with tempfile.TemporaryDirectory() as tmp:
            gz_path = Path(tmp) / "Lexique383.tsv.gz"
            with gzip.open(gz_path, "wt", encoding="utf-8") as handle:
                handle.write(header + "".join(rows))
            info = load_lexique_noun_genders(gz_path)

        self.assertEqual(info["lemma"]["maison"], {"f"})
        self.assertNotIn("maison", info["blank_genre"])
        self.assertEqual(info["lemma"]["docteur"], {"m"})
        self.assertNotIn("docteur", info["homographs"])
        self.assertIn("livre", info["blank_genre"])

    def test_recover_full_header_line_and_pilot_verifier(self) -> None:
        wiki = "== {{langue|fr}} ==\n=== {{S|nom|fr}} ===\n'''professeur''' {{pron|pʁɔ.fɛ.sœʁ|fr}} {{mf}}\n"
        recovered = recover_full_header_line(
            wiki,
            line_1_based=3,
            column_1_based=18,
            raw_source_excerpt="{{pron|pʁɔ.fɛ.sœʁ|fr}}",
        )
        self.assertEqual(recovered, "'''professeur''' {{pron|pʁɔ.fɛ.sœʁ|fr}} {{mf}}")
        self.assertIsNone(
            recover_full_header_line(
                wiki,
                line_1_based=3,
                column_1_based=10,
                raw_source_excerpt="{{pron|pʁɔ.fɛ.sœʁ|fr}}",
            )
        )

        curated = {
            "groups": [
                {
                    "frozen_row": 669,
                    "headword": "flic",
                    "lexique_cgram": "NOM",
                    "senses": [
                        {
                            "sense_id": "s-flic",
                            "use": "Noun; selected masculine use for a police officer.",
                            "fields": {
                                "ipa": {"source_span": {"raw_source_excerpt": "'''flic''' {{pron|flik|fr}} {{mf}}"}},
                                "french_definition": {"source_span": {"raw_source_excerpt": "# Policier."}},
                            },
                        }
                    ],
                },
                {
                    "frozen_row": 128,
                    "headword": "par",
                    "lexique_cgram": "NOM",
                    "senses": [
                        {
                            "sense_id": "s-par-golf",
                            "use": "Masculine noun: golf strokes.",
                            "fields": {
                                "ipa": {"source_span": {"raw_source_excerpt": "'''par''' {{pron|paʁ|fr}} {{m}}"}},
                                "french_definition": {"source_span": {"raw_source_excerpt": "# Golf."}},
                            },
                        }
                    ],
                },
            ]
        }
        problems = verify_pilot_decisions(
            [
                {"sense_id": "s-flic", "final_gender": "m", "cefr_overlay_eligible": True},
                {"sense_id": "s-par-golf", "final_gender": "m", "cefr_overlay_eligible": False},
            ],
            curated,
            {"s-par-golf"},
        )
        self.assertEqual(problems, [])

        with self.assertRaisesRegex(ValueError, "Tracked payload must not contain"):
            assert_zero_nc_levels_in_tracked_payload({"sense_id": "s-flic", "cefr_level": "B1"})


if __name__ == "__main__":
    unittest.main()
