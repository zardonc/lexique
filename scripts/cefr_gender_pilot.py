#!/usr/bin/env python3
"""Deterministic noun-gender triage and Option 2 FLELex TT-Beacco alignment pilot."""

from __future__ import annotations

import csv
import gzip
import io
import json
import math
import re
import time
from pathlib import Path
from typing import Any

MAX_FLELEX_BYTES = 5 * 1024 * 1024
FLELEX_COLUMNS = (
    "word",
    "tag",
    "freq_A1",
    "freq_A2",
    "freq_B1",
    "freq_B2",
    "freq_C1",
    "freq_C2",
    "freq_total",
    "level",
)
VALID_CEFR_LEVELS = frozenset({"A1", "A2", "B1", "B2", "C1", "C2"})
CEFR_RANK = {"A1": 1, "A2": 2, "B1": 3, "B2": 4, "C1": 5, "C2": 6}
VALID_GENDERS = frozenset({"m", "f", "mf"})

# Exact Lexique cgram -> FLELex TreeTagger POS mapping (no cross-category fallback).
FLELEX_POS_MAP: dict[str, tuple[str, ...]] = {
    "NOM": ("NOM",),
    "VER": ("VER",),
    "AUX": ("VER",),
    "ADJ": ("ADJ",),
    "ADJ:ind": ("ADJ",),
    "ADJ:num": ("ADJ",),
    "ADJ:dem": ("ADJ",),
    "ADJ:int": ("ADJ",),
    "ADJ:pos": ("DET:POS",),
    "ART:def": ("DET:ART", "PRP:det"),
    "ART:ind": ("DET:ART", "PRP:det"),
    "PRO:per": ("PRO",),
    "PRO:ind": ("PRO",),
    "PRO:int": ("PRO",),
    "PRO:rel": ("PRO",),
    "PRO:dem": ("PRO",),
    "PRO:pos": ("PRO",),
    "ADV": ("ADV",),
    "PRE": ("PRP", "PRP:det"),
    "CON": ("KON",),
    "ONO": ("INT",),
}

# French nouns with well-known distinct-meaning m/f homographs in everyday/curriculum usage.
KNOWN_GENDER_HOMOGRAPH_NOUNS = frozenset(
    {
        "aide",
        "couple",
        "critique",
        "garde",
        "livre",
        "manche",
        "manoeuvre",
        "manœuvre",
        "merci",
        "mémoire",
        "mode",
        "moufle",
        "oeuvre",
        "œuvre",
        "ombre",
        "page",
        "parallèle",
        "pendule",
        "physique",
        "poêle",
        "poste",
        "relâche",
        "solde",
        "somme",
        "tour",
        "vague",
        "vase",
        "voile",
    }
)

# Owner-approved specialist senses in core-162-v1 that must never inherit elementary lemma CEFR.
SPECIALIST_FROZEN_ROWS = frozenset({34, 128, 3128})

UNCERTAIN_GENDER_RE = re.compile(r"\{\{\s*(?:mf\s*\?|m\s*\?\s*ou\s*f|f\s*\?\s*ou\s*m)(?:\|[^}]*)?\}\}", re.I)
EXPLICIT_GENDER_RE = re.compile(
    r"\{\{\s*(m|f|mf|masculin|féminin|masculin\s+et\s+féminin|m\s+et\s+f)(?:\|[^}]*)?\}\}"
)


class FlelexValidationError(ValueError):
    """Raised when an imported FLELex TSV fails structural or value validation."""


def validate_flelex_beacco_tsv(
    raw_bytes: bytes, *, max_bytes: int = MAX_FLELEX_BYTES
) -> list[dict[str, Any]]:
    """Validate a user-supplied FleLex_TT_Beacco.tsv byte payload (CRLF or LF)."""
    if not raw_bytes:
        raise FlelexValidationError("FLELex file is empty")
    if len(raw_bytes) > max_bytes:
        raise FlelexValidationError(
            f"FLELex file exceeds maximum size ({len(raw_bytes)} > {max_bytes} bytes)"
        )
    try:
        text = raw_bytes.decode("utf-8")
    except UnicodeDecodeError as exc:
        raise FlelexValidationError(f"FLELex file must be valid UTF-8: {exc}") from exc

    stream = io.StringIO(text, newline="")
    reader = csv.reader(stream, delimiter="\t")
    try:
        header = next(reader)
    except StopIteration as exc:
        raise FlelexValidationError("FLELex file has no header row") from exc

    if tuple(header) != FLELEX_COLUMNS:
        raise FlelexValidationError(
            f"Invalid FLELex header: expected 10-column FleLex_TT_Beacco header {list(FLELEX_COLUMNS)}, got {header}"
        )

    rows: list[dict[str, Any]] = []
    seen_keys: set[tuple[str, str]] = set()
    for line_no, cols in enumerate(reader, start=2):
        if not cols or (len(cols) == 1 and cols[0] == ""):
            continue
        if len(cols) != 10:
            raise FlelexValidationError(
                f"Line {line_no}: expected 10 tab-separated columns, got {len(cols)}"
            )
        word, tag = cols[0], cols[1]
        if (
            not word
            or not tag
            or word != word.strip()
            or tag != tag.strip()
            or any(ord(ch) < 32 for ch in word + tag)
        ):
            raise FlelexValidationError(f"Line {line_no}: invalid word or POS tag")
        key = (word, tag)
        if key in seen_keys:
            raise FlelexValidationError(f"Line {line_no}: duplicate (word, tag) key {key!r}")
        seen_keys.add(key)

        freqs: list[float] = []
        for idx, raw_num in enumerate(cols[2:9]):
            try:
                val = float(raw_num)
            except ValueError as exc:
                raise FlelexValidationError(
                    f"Line {line_no}: non-numeric frequency in {FLELEX_COLUMNS[2 + idx]}"
                ) from exc
            if not math.isfinite(val) or val < 0.0:
                raise FlelexValidationError(
                    f"Line {line_no}: frequency must be finite and non-negative in {FLELEX_COLUMNS[2 + idx]}"
                )
            freqs.append(val)

        level = cols[9]
        if level not in VALID_CEFR_LEVELS:
            raise FlelexValidationError(f"Line {line_no}: invalid CEFR level {level!r}")

        rows.append(
            {
                "word": word,
                "tag": tag,
                "freq_total": freqs[6],
                "level": level,
            }
        )

    if not rows:
        raise FlelexValidationError("FLELex file contains zero data rows")
    return rows


def extract_wiktionary_gender_markers(
    ipa_excerpt: str, def_excerpt: str
) -> tuple[str | None, bool, str | None]:
    """Return (gender_code, is_uncertain_or_multi, matched_field) from fixed Wiktionary spans."""
    for field_name, snippet in (("ipa", ipa_excerpt), ("french_definition", def_excerpt)):
        if UNCERTAIN_GENDER_RE.search(snippet):
            return None, True, field_name
        found: set[str] = set()
        for raw_tag in EXPLICIT_GENDER_RE.findall(snippet):
            norm = re.sub(r"\s+", " ", raw_tag.strip().lower())
            if norm in ("m", "masculin"):
                found.add("m")
            elif norm in ("f", "féminin"):
                found.add("f")
            else:
                found.add("mf")
        if found:
            if found == {"m"}:
                return "m", False, field_name
            if found == {"f"}:
                return "f", False, field_name
            if found == {"mf"}:
                return "mf", False, field_name
            # Both {{m}} and {{f}} or {{m}} + {{mf}} appeared on the same span
            return "mf", True, field_name
    return None, False, None


def infer_use_gender_hint(use_text: str) -> str | None:
    """Read curated-inputs `use` note only as a consistency check (never as sole provenance)."""
    lowered = use_text.lower()
    has_either = any(
        phrase in lowered
        for phrase in (
            "either grammatical gender",
            "common-gender",
            "masculine or feminine",
            "masculine/feminine",
            "usable for women and men",
            "male or female",
        )
    )
    if has_either:
        return "mf"
    for raw_clause in lowered.split(";"):
        clause = raw_clause.strip()
        if clause.startswith(("not ", "distinct ", "excluding ", "unlike ", "rather than ")):
            continue
        has_m = bool(re.search(r"\bmasculine\b", clause))
        has_f = bool(re.search(r"\bfeminine\b", clause))
        if has_m and not has_f:
            return "m"
        if has_f and not has_m:
            return "f"
        if has_m and has_f:
            return "mf"
    return None


def load_lexique_noun_genders(lexique_gz_path: Path) -> dict[str, Any]:
    """Load Lexique 3.83 NOM genders keyed by lemma and same-stem plural forms."""
    attested_nom_lemmas: set[str] = set()
    lemma_genders: dict[str, set[str]] = {}
    same_stem_genders: dict[str, set[str]] = {}
    ortho_genders: dict[str, set[str]] = {}
    distinct_lemma_entries: dict[str, int] = {}
    with gzip.open(lexique_gz_path, "rt", encoding="utf-8") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        for row in reader:
            if row["cgram"] != "NOM":
                continue
            lem = row["lemme"]
            ortho = row["ortho"]
            attested_nom_lemmas.add(lem)
            genre = row["genre"].strip()
            if genre in ("m", "f"):
                ortho_genders.setdefault(ortho, set()).add(genre)
                if ortho in (lem, f"{lem}s", f"{lem}x"):
                    same_stem_genders.setdefault(lem, set()).add(genre)
                if row["islem"] == "1":
                    lemma_genders.setdefault(lem, set()).add(genre)
                    distinct_lemma_entries[lem] = (
                        distinct_lemma_entries.get(lem, 0) + 1
                    )
    resolved_lemma: dict[str, set[str]] = {}
    for lem in attested_nom_lemmas:
        gset = set(lemma_genders.get(lem, set())) or set(same_stem_genders.get(lem, set()))
        if gset:
            resolved_lemma[lem] = gset
    blank_genre_nouns = attested_nom_lemmas - set(resolved_lemma.keys())
    homograph_lemmas = {
        lemma
        for lemma, gset in resolved_lemma.items()
        if len(gset) > 1 or distinct_lemma_entries.get(lemma, 0) > 1
    }
    return {
        "lemma": resolved_lemma,
        "ortho": ortho_genders,
        "blank_genre": blank_genre_nouns,
        "homographs": {k: set() for k in homograph_lemmas},
    }


def triage_core_noun_genders(
    curated_inputs: dict[str, Any], lexique_info: dict[str, Any]
) -> dict[str, Any]:
    """Deterministically triage all 4,062 core-162-v1 groups for sense-bound NOM gender."""
    lemma_map: dict[str, set[str]] = lexique_info["lemma"]
    ortho_map: dict[str, set[str]] = lexique_info["ortho"]
    blank_genre: set[str] = set(lexique_info.get("blank_genre", set()))
    lex_homographs: set[str] = set(lexique_info["homographs"].keys())

    auto_accepted: list[dict[str, Any]] = []
    residuals: list[dict[str, Any]] = []
    non_noun_count = 0
    auto_mode_counts: dict[str, int] = {}

    for group in curated_inputs["groups"]:
        pos = group["lexique_cgram"]
        if pos != "NOM":
            non_noun_count += len(group["senses"])
            continue

        hw = group["headword"]
        for sense in group["senses"]:
            shw = sense["source_headword"]
            ipa_span = sense["fields"]["ipa"]["source_span"]
            def_span = sense["fields"]["french_definition"]["source_span"]
            wikt_gender, wikt_uncertain, matched_field = extract_wiktionary_gender_markers(
                ipa_span["raw_source_excerpt"], def_span["raw_source_excerpt"]
            )
            use_hint = infer_use_gender_hint(sense["use"])

            lex_set = lemma_map.get(hw) or lemma_map.get(shw) or ortho_map.get(hw) or set()
            lex_code = (
                "m"
                if lex_set == {"m"}
                else "f"
                if lex_set == {"f"}
                else "mf"
                if lex_set == {"m", "f"}
                else None
            )
            has_blank_lex_genre = hw in blank_genre or shw in blank_genre
            is_homograph = (
                hw in KNOWN_GENDER_HOMOGRAPH_NOUNS
                or shw in KNOWN_GENDER_HOMOGRAPH_NOUNS
                or hw in lex_homographs
                or shw in lex_homographs
                or len(group["senses"]) > 1
                or (has_blank_lex_genre and wikt_gender in ("m", "f"))
            )

            base_record = {
                "frozen_row": group["frozen_row"],
                "headword": hw,
                "source_headword": shw,
                "sense_id": sense["sense_id"],
                "wikt_gender": wikt_gender,
                "wikt_matched_field": matched_field,
                "lexique_gender": lex_code,
                "lexique_blank_dual_signal": has_blank_lex_genre,
                "use_hint": use_hint,
                "use": sense["use"],
                "definition_fr": sense["fields"]["french_definition"]["text"],
                "ipa_span": ipa_span,
                "def_span": def_span,
                "source_id": sense["fields"][matched_field or "ipa"]["source_id"],
            }

            if wikt_uncertain:
                residuals.append(
                    {**base_record, "queue": "residual_missing_or_uncertain_span"}
                )
            elif wikt_gender is None:
                # Trimmed IPA span: auto-accept ONLY when non-homograph Lexique single m/f
                # agrees with the independently reviewed `use` field in core-162-v1.
                if (
                    not is_homograph
                    and not has_blank_lex_genre
                    and lex_code in ("m", "f")
                    and use_hint == lex_code
                ):
                    auto_mode_counts["reviewed_use_plus_lexique"] = (
                        auto_mode_counts.get("reviewed_use_plus_lexique", 0) + 1
                    )
                    auto_accepted.append(
                        {
                            **base_record,
                            "accepted_gender": lex_code,
                            "auto_mode": "reviewed_use_plus_lexique",
                            "queue": "auto_accepted",
                        }
                    )
                else:
                    residuals.append(
                        {**base_record, "queue": "residual_missing_or_uncertain_span"}
                    )
            elif is_homograph:
                residuals.append({**base_record, "queue": "residual_homograph"})
            elif (lex_code is not None and lex_code != wikt_gender) or (
                use_hint is not None and use_hint != wikt_gender
            ):
                residuals.append({**base_record, "queue": "residual_conflict"})
            elif wikt_gender in ("m", "f"):
                auto_mode_counts["wikt_span_agreement"] = (
                    auto_mode_counts.get("wikt_span_agreement", 0) + 1
                )
                auto_accepted.append(
                    {
                        **base_record,
                        "accepted_gender": wikt_gender,
                        "auto_mode": "wikt_span_agreement",
                        "queue": "auto_accepted",
                    }
                )
            else:
                # Wiktionary says {{mf}}: review epicene vs single-gender sense scope
                residuals.append({**base_record, "queue": "residual_conflict"})

    queue_counts: dict[str, int] = {}
    for item in residuals:
        queue_counts[item["queue"]] = queue_counts.get(item["queue"], 0) + 1

    return {
        "total_admitted_groups": len(curated_inputs["groups"]),
        "non_noun_senses_null": non_noun_count,
        "total_nom_senses": len(auto_accepted) + len(residuals),
        "auto_accepted_count": len(auto_accepted),
        "auto_accepted_by_mode": auto_mode_counts,
        "residual_total_count": len(residuals),
        "residual_queue_counts": queue_counts,
        "auto_accepted": auto_accepted,
        "residuals": residuals,
    }


def build_spelling_Lookup(
    spelling_relations: list[dict[str, Any]],
) -> dict[tuple[str, str | None], list[str]]:
    """Map (frozen_headword, lexique_cgram | None) -> verified target spellings."""
    out: dict[tuple[str, str | None], list[str]] = {}
    for rel in spelling_relations:
        key = (rel["frozen_headword"], rel.get("lexique_cgram"))
        target = rel["source_headword"]
        out.setdefault(key, [])
        if target not in out[key]:
            out[key].append(target)
    return out


def candidate_flelex_words(
    headword: str,
    source_headword: str,
    pos: str,
    relation_map: dict[tuple[str, str | None], list[str]],
) -> list[str]:
    """Generate deterministic orthographic variants (apostrophes and œ/æ ligatures only)."""
    seeds = [headword]
    if source_headword not in seeds:
        seeds.append(source_headword)
    for rel_key in ((headword, pos), (headword, None)):
        for rel_word in relation_map.get(rel_key, ()):
            if rel_word not in seeds:
                seeds.append(rel_word)

    candidates: list[str] = []
    for word in seeds:
        for variant in (
            word,
            word.replace("’", "'"),
            word.replace("'", "’"),
            word.replace("œ", "oe").replace("Œ", "Oe"),
            word.replace("æ", "ae").replace("Æ", "Ae"),
        ):
            if variant not in candidates:
                candidates.append(variant)
    return candidates


def align_flelex_to_core(
    curated_inputs: dict[str, Any],
    spelling_relations: list[dict[str, Any]],
    flelex_rows: list[dict[str, Any]],
    *,
    extra_excluded_sense_ids: set[str] | None = None,
) -> dict[str, Any]:
    """Join validated FLELex rows against core-162-v1 senses with strict POS & exclusions."""
    flelex_index: dict[tuple[str, str], str] = {
        (row["word"], row["tag"]): row["level"] for row in flelex_rows
    }
    relation_map = build_spelling_Lookup(spelling_relations)
    excluded_ids = set(extra_excluded_sense_ids or set())

    # Always exclude the 3 owner-approved specialist senses.
    for group in curated_inputs["groups"]:
        if group["frozen_row"] in SPECIALIST_FROZEN_ROWS:
            for sense in group["senses"]:
                excluded_ids.add(sense["sense_id"])

    matched_senses: list[dict[str, Any]] = []
    suppressed_specialist_or_excluded: list[dict[str, Any]] = []
    ambiguous_tag_conflicts: list[dict[str, Any]] = []
    unmatched_senses: list[dict[str, Any]] = []
    polysemy_candidates: list[dict[str, Any]] = []

    level_counts: dict[str, int] = {}
    alias_recovered = 0

    for group in curated_inputs["groups"]:
        pos = group["lexique_cgram"]
        hw = group["headword"]
        target_tags = FLELEX_POS_MAP[pos]

        for sense in group["senses"]:
            sid = sense["sense_id"]
            shw = sense["source_headword"]
            cands = candidate_flelex_words(hw, shw, pos, relation_map)

            found_levels: list[tuple[str, str, str, bool]] = []
            for idx, cand_word in enumerate(cands):
                tag_hits = [
                    (cand_word, tag, flelex_index[(cand_word, tag)], idx > 0)
                    for tag in target_tags
                    if (cand_word, tag) in flelex_index
                ]
                if tag_hits:
                    found_levels = tag_hits
                    break

            if not found_levels:
                unmatched_senses.append(
                    {"frozen_row": group["frozen_row"], "headword": hw, "pos": pos, "sense_id": sid}
                )
                continue

            distinct_levels = {item[2] for item in found_levels}
            if len(distinct_levels) > 1:
                ambiguous_tag_conflicts.append(
                    {
                        "frozen_row": group["frozen_row"],
                        "headword": hw,
                        "pos": pos,
                        "sense_id": sid,
                        "candidates": found_levels,
                    }
                )
                continue

            matched_word, matched_tag, level, used_alias = found_levels[0]
            if sid in excluded_ids:
                suppressed_specialist_or_excluded.append(
                    {
                        "frozen_row": group["frozen_row"],
                        "headword": hw,
                        "pos": pos,
                        "sense_id": sid,
                        "raw_flelex_level_suppressed": True,
                        "reason": "specialist_or_excluded_sense",
                    }
                )
                continue

            if used_alias:
                alias_recovered += 1
            level_counts[level] = level_counts.get(level, 0) + 1

            entry = {
                "frozen_row": group["frozen_row"],
                "headword": hw,
                "pos": pos,
                "sense_id": sid,
                "matched_word": matched_word,
                "matched_tag": matched_tag,
                "used_alias": used_alias,
                "level": level,
            }
            matched_senses.append(entry)

            # Flag narrow/figurative/slang/domain SELECTED senses on A1/A2 words for polysemy check
            # (Do NOT match words mentioned inside `excluded_uses`, which state what was excluded!)
            use_lower = sense["use"].lower()
            if level in ("A1", "A2") and any(
                kw in use_lower
                for kw in (
                    "slang",
                    "vulgar",
                    "specialist",
                    "heraldry",
                    "golf",
                    "psychoanaly",
                    "self-regard",
                    "figurative only",
                    "archaic",
                    "old-fashioned",
                )
            ):
                polysemy_candidates.append(
                    {
                        "frozen_row": group["frozen_row"],
                        "headword": hw,
                        "pos": pos,
                        "sense_id": sid,
                        "level": level,
                        "use": sense["use"],
                    }
                )

    return {
        "total_admitted_senses": sum(len(g["senses"]) for g in curated_inputs["groups"]),
        "matched_count": len(matched_senses),
        "alias_recovered_count": alias_recovered,
        "suppressed_excluded_count": len(suppressed_specialist_or_excluded),
        "ambiguous_tag_conflict_count": len(ambiguous_tag_conflicts),
        "unmatched_count": len(unmatched_senses),
        "level_distribution": {
            lvl: level_counts.get(lvl, 0) for lvl in ("A1", "A2", "B1", "B2", "C1", "C2")
        },
        "polysemy_flag_count": len(polysemy_candidates),
        "polysemy_candidates": polysemy_candidates,
        "suppressed": suppressed_specialist_or_excluded,
        "ambiguous_tag_conflicts": ambiguous_tag_conflicts,
        "matched_by_sense_id": {m["sense_id"]: m for m in matched_senses},
        "excluded_sense_ids": sorted(excluded_ids),
    }


def recover_full_header_line(
    raw_wiki_text: str,
    *,
    line_1_based: int,
    column_1_based: int | None,
    raw_source_excerpt: str,
) -> str | None:
    """Verify exact (line, column) slice playback on a fixed Wiktionary page and return full line."""
    lines = raw_wiki_text.splitlines()
    if line_1_based < 1 or line_1_based > len(lines):
        return None
    full_line = lines[line_1_based - 1]
    if column_1_based is not None and column_1_based >= 1:
        start = column_1_based - 1
        if full_line[start : start + len(raw_source_excerpt)] != raw_source_excerpt:
            return None
        return full_line
    if raw_source_excerpt not in full_line:
        return None
    return full_line


def resolve_notebook_headword_level(
    headword: str,
    saved_core_sense_ids: list[str] | None,
    matched_by_sense_id: dict[str, dict[str, Any]],
    flelex_rows: list[dict[str, Any]],
) -> str | None:
    """Apply Q8 Notebook row CEFR display rule (foundational core level or unambiguous FLELex level)."""
    if saved_core_sense_ids:
        levels = [
            matched_by_sense_id[sid]["level"]
            for sid in saved_core_sense_ids
            if sid in matched_by_sense_id
        ]
        if not levels:
            return None
        return min(levels, key=lambda lvl: CEFR_RANK[lvl])

    # Non-core (MDX / external CSV) record: display only if FLELex has an unambiguous level.
    direct_levels = {row["level"] for row in flelex_rows if row["word"] == headword}
    if len(direct_levels) == 1:
        return next(iter(direct_levels))
    return None


def verify_pilot_decisions(
    decisions: list[dict[str, Any]],
    curated_inputs: dict[str, Any],
    excluded_sense_ids: set[str],
) -> list[str]:
    """Verify Stage 2 pilot gender & CEFR eligibility contracts without writing NC levels."""
    problems: list[str] = []
    sense_lookup: dict[str, tuple[dict[str, Any], dict[str, Any]]] = {}
    for group in curated_inputs["groups"]:
        for sense in group["senses"]:
            sense_lookup[sense["sense_id"]] = (group, sense)

    for dec in decisions:
        sid = dec["sense_id"]
        if sid not in sense_lookup:
            problems.append(f"Unknown sense_id: {sid}")
            continue
        group, sense = sense_lookup[sid]
        pos = group["lexique_cgram"]
        hw = group["headword"]
        gender = dec["final_gender"]

        if pos != "NOM":
            if gender is not None:
                problems.append(f"{sid} ({hw}/{pos}): non-noun must have final_gender=None, got {gender!r}")
        elif hw == "gens":
            if gender is not None:
                problems.append(f"{sid} (gens/NOM): plural-agreement exception must stay final_gender=None")
        else:
            if gender not in ("m", "f", "mf", None):
                problems.append(f"{sid}: invalid final_gender {gender!r}")
            elif gender is not None:
                excerpt = dec.get("verified_wiktionary_line") or sense["fields"]["ipa"]["source_span"]["raw_source_excerpt"]
                def_excerpt = sense["fields"]["french_definition"]["source_span"]["raw_source_excerpt"]
                wikt_g, wikt_uncertain, _ = extract_wiktionary_gender_markers(excerpt, def_excerpt)
                use_hint = infer_use_gender_hint(sense["use"])
                if wikt_uncertain:
                    problems.append(f"{sid}: uncertain Wiktionary marker cannot admit gender {gender!r}")
                elif wikt_g == gender:
                    pass
                elif wikt_g == "mf" and gender in ("m", "f") and use_hint == gender:
                    # Sense-scoped single gender on an epicene headword (e.g. flic/NOM selected masculine sense)
                    pass
                elif wikt_g is None and use_hint == gender and dec.get("span_capture_needed") is True:
                    # Trimmed IPA excerpt where reviewed `use` states the sense gender, flagged for line capture
                    pass
                else:
                    problems.append(
                        f"{sid}: gender {gender!r} lacks verified source-span playback (wikt_g={wikt_g!r}, use_hint={use_hint!r})"
                    )

        if sid in excluded_sense_ids and dec.get("cefr_overlay_eligible") is not False:
            problems.append(f"{sid}: excluded specialist/polysemy sense must have cefr_overlay_eligible=False")

    return problems


def assert_zero_nc_levels_in_tracked_payload(payload: Any) -> None:
    """Raise ValueError if any tracked dict/list contains FLELex A1-C2 level assignments."""
    if isinstance(payload, dict):
        for key, value in payload.items():
            if key in ("level", "cefr_level", "level_distribution") and value is not None:
                raise ValueError(f"Tracked payload must not contain FLELex level field {key!r}")
            assert_zero_nc_levels_in_tracked_payload(value)
    elif isinstance(payload, list):
        for item in payload:
            assert_zero_nc_levels_in_tracked_payload(item)
    elif isinstance(payload, str) and payload in VALID_CEFR_LEVELS:
        raise ValueError(f"Tracked payload must not contain bare CEFR level value {payload!r}")
