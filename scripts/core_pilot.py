"""Prepare the independently reviewed D2 core content and its real notices."""

import argparse
import csv
import hashlib
import json
from pathlib import Path
import shutil
import sqlite3
import tempfile
import unicodedata
from urllib.parse import urlsplit


ROOT = Path(__file__).resolve().parents[1]
RESOURCE = ROOT / "resources/core-pilot-79"
ROWS = (*range(1, 87), *range(88, 151), *range(201, 251), 360, *range(401, 451), 483, 1911)
FIELDS = {"french_definition": "definition_fr", "ipa": "ipa", "english": "meaning_en", "chinese": "meaning_zh"}
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
MATERIALS = (
    "COPYING", "curated-inputs.json", "field-review.json", "MEMBERSHIP-NOTICES.txt",
    "scope-manifest.csv", "selected-source-context.json", "selected-source-excerpts.json",
    "LEXIQUE-UPSTREAM-README.txt", "lexique-provenance.json", "README.md", "NOTICES.md",
)


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"Duplicate JSON field: {key}")
        result[key] = value
    return result


def read_json(path):
    try:
        return json.loads(path.read_text(encoding="utf-8"), object_pairs_hook=unique_object,
                          parse_constant=lambda value: (_ for _ in ()).throw(ValueError(f"Invalid JSON number: {value}")))
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError(f"Invalid or missing curated material {path.name}: {error}") from error


def text(value, context, maximum=20_000):
    if not isinstance(value, str) or not value.strip() or len(value) > maximum or any(unicodedata.category(char) == "Cc" for char in value):
        raise ValueError(f"Invalid {context}: require 1–{maximum} characters without controls")
    return value


def web_url(value, context):
    text(value, context)
    parsed = urlsplit(value)
    if parsed.scheme not in ("http", "https") or not parsed.hostname or any(char.isspace() for char in value):
        raise ValueError(f"Invalid HTTP/HTTPS {context}")
    return value


def compact(value):
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"), allow_nan=False)


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_inputs(source_dir):
    for name in (*MATERIALS, "input-checksums.json"):
        if not (source_dir / name).is_file():
            raise ValueError(f"Missing curated material: {name}")
    curated = read_json(source_dir / "curated-inputs.json")
    reviewed = read_json(source_dir / "field-review.json")
    with (source_dir / "scope-manifest.csv").open(encoding="utf-8-sig", newline="") as handle:
        reader = csv.DictReader(handle)
        if reader.fieldnames != ["word", "lexique_cgram", "subtitle_lemma_per_million", "subtitle_pos_per_million"]:
            raise ValueError("Invalid frozen scope columns")
        scope = list(reader)
    if len(scope) != 4087 or len({row["word"] for row in scope}) != 3773 or len({(row["word"], row["lexique_cgram"]) for row in scope}) != 4087:
        raise ValueError("Invalid frozen scope denominator or identities")
    if curated.get("schema") != "lexique-core-pilot-curation-1" or curated.get("content_license") != "CC-BY-SA-4.0":
        raise ValueError("Invalid curated schema or content licence")
    if curated.get("scope_denominator_groups") != 4087 or curated.get("scope_denominator_headwords") != 3773:
        raise ValueError("Curated scope denominator changed")
    if reviewed.get("kind") != "separate_independent_field_review" or not reviewed.get("independence"):
        raise ValueError("A separate independent field review is required")
    groups = curated.get("groups", [])
    if [group.get("frozen_row") for group in groups] != list(ROWS):
        raise ValueError("Admitted frozen scope rows changed")
    sources = {source["id"]: source for source in curated["sources"]}
    if len(sources) != len(curated["sources"]):
        raise ValueError("Duplicate field source identity")
    for source in sources.values():
        if source.get("content_license") != "CC-BY-SA-4.0" or source.get("selected_text_review") != "passed_for_selected_spans":
            raise ValueError("Unreviewed field source or incompatible licence")
        for name in ("fixed_url", "history_url", "license_url"):
            web_url(source[name], f"field source {name}")
        text(source["attribution"], "field source attribution")
        if not isinstance(source["revid"], int) or isinstance(source["revid"], bool) or source["revid"] <= 0:
            raise ValueError("Invalid fixed field source version")
    field_reviews = {(field["sense_id"], field["field"]): field for field in reviewed["fields"]}
    usage_reviews = {usage["sense_id"]: usage for usage in reviewed["usage"]}
    if len(field_reviews) != len(ROWS) * 4 or len(usage_reviews) != len(ROWS):
        raise ValueError("Independent field review is incomplete or duplicated")
    selected = []
    scope_gaps = {}
    seen = set()
    for group in groups:
        original = scope[group["frozen_row"] - 1]
        if (group["headword"], group["lexique_cgram"], group["subtitle_lemma_per_million"], group["subtitle_pos_per_million"]) != tuple(original.values()):
            raise ValueError("A selected field is bound to the wrong frozen scope group")
        if unicodedata.normalize("NFC", group["headword"]) != group["headword"]:
            raise ValueError("Frozen scope spelling must remain NFC")
        if not group.get("senses"):
            raise ValueError("An admitted group is missing its complete four-field sense")
        for index, sense in enumerate(group["senses"], 1):
            identity = sense["sense_id"]
            if identity in seen:
                raise ValueError("Duplicate selected sense identity")
            seen.add(identity)
            usage = usage_reviews[identity]
            if sense.get("independent_review") != "passed_after_exact_accepted_replacements" or usage["accepted_usage_quality_status"] != "passed":
                raise ValueError("A selected sense is not independently reviewed")
            if (sense["use"], sense["use_language"], sense["source_headword"]) != (usage["accepted_usage"], usage["language"], usage["source_headword"]):
                raise ValueError("Selected usage differs from the independently reviewed wording")
            status = usage.get("everyday_sense_status")
            if status not in ("passed", "unresolved", "approved_specialist_exception"):
                raise ValueError("An explicit everyday-sense review is required")
            if status != "passed":
                scope_gaps[group["frozen_row"]] = {
                    "everyday_sense_status": status,
                    "incomplete_requirements": ["modern_everyday_sense"],
                    "reason": text(usage.get("everyday_sense_reason"), "non-everyday-sense reason"),
                }
            if status == "approved_specialist_exception":
                approved_issue = {
                    (34, "ça", "NOM", "ca-noun-psychoanalytic-id"): 81,
                    (128, "par", "NOM", "par-noun-golf-standard-strokes"): 83,
                }.get((group["frozen_row"], group["headword"], group["lexique_cgram"], identity))
                if approved_issue is None or (
                    usage.get("frozen_row"), usage.get("headword"), usage.get("pos")
                ) != (group["frozen_row"], group["headword"], group["lexique_cgram"]):
                    raise ValueError("The specialist exception applies only to the approved frozen noun and sense")
                decision = usage.get("specialist_exception_decision")
                if not isinstance(decision, dict) or decision.get("issue") != f"https://github.com/zardonc/lexique/issues/{approved_issue}" or decision.get("approved_by") != "project_owner":
                    raise ValueError("The specialist exception requires the project owner's matching issue approval")
                text(decision.get("approved_on"), "specialist exception approval date", 32)
                text(decision.get("decision"), "specialist exception decision")
                scope_gaps[group["frozen_row"]]["specialist_exception_decision"] = decision
            if sense.get("cefr") is not None or set(sense.get("fields", {})) != set(FIELDS):
                raise ValueError("A selected sense requires exactly four complete fields and no inferred CEFR")
            payload = {"sense_id": text(identity, "sense identity", 128), "frozen_row": group["frozen_row"],
                       "headword": text(group["headword"], "frozen headword", 200),
                       "source_headword": text(sense["source_headword"], "source headword", 200),
                       "pos": group["lexique_cgram"], "usage": text(sense["use"], "selected usage")}
            provenance = {}
            for name, target in FIELDS.items():
                field = sense["fields"][name]
                review = field_reviews[identity, name]
                source = sources[field["source_id"]]
                span = field["source_span"]
                if field.get("independent_review") != "passed" or review["accepted_text_quality_status"] != "passed" or review["rights_eligibility_status"] != "passed_at_selected_text_boundary":
                    raise ValueError("A selected field is not independently reviewed or rights-eligible")
                if field.get("content_license") != "CC-BY-SA-4.0" or field.get("language") != LANGUAGES[name]:
                    raise ValueError("Incorrect field language or licence")
                if (field["text"], group["frozen_row"], group["headword"], group["lexique_cgram"]) != (review["accepted_text"], review["frozen_row"], review["headword"], review["pos"]):
                    raise ValueError("Selected field differs from its independently reviewed text or frozen scope")
                exact = review["exact_source"]
                if (source["revid"], source["fixed_url"], field["source_id"], span["line_1_based"], span["raw_source_excerpt"]) != (exact["revid"], exact["url"], exact["source_id"], exact["line_1_based"], exact["reviewed_excerpt"]):
                    raise ValueError("Selected field source span differs from the independent review")
                payload[target] = text(field["text"], f"selected {name} field", 200 if name == "ipa" else 20_000)
                edition = "Wiktionnaire" if source["edition"] == "fr" else "Wiktionary"
                provenance[target] = {
                    "title": f"{edition} — {source['title']}",
                    "version": f"revision {source['revid']} ({source['timestamp']})",
                    "attribution": f"{source['attribution']}; author history: {source['history_url']}; recorded adaptations by Lexique contributors",
                    "source_url": source["fixed_url"], "license": "CC BY-SA 4.0", "license_url": source["license_url"],
                    "span": f"Fixed wikitext line {span['line_1_based']}; exact selected excerpt in selected-source-excerpts.json",
                    "modification": text(field["modification"], "field modification notice"),
                }
            payload["field_sources"] = provenance
            selected.append((group["frozen_row"], index, group["headword"], payload))
    if len(selected) != len(ROWS) or len(seen) != len(ROWS):
        raise ValueError("Admitted selected-sense scope changed")
    unresolved = reviewed.get("unresolved_fields", [])
    if len(unresolved) != 4:
        raise ValueError("Frozen row 87 requires four explicit unresolved fields")
    for field in unresolved:
        row = field["frozen_row"]
        if not isinstance(row, int) or row != 87 or (field["headword"], field["pos"]) != (scope[row - 1]["word"], scope[row - 1]["lexique_cgram"]) or field["field"] not in FIELDS:
            raise ValueError("Unresolved field is bound to an admitted or incorrect frozen group")
        gap = scope_gaps.setdefault(row, {
            "everyday_sense_status": "unresolved", "incomplete_fields": [],
            "incomplete_requirements": ["same_pos_source_binding", "modern_everyday_sense"],
            "reason": text(field["reason"], "unresolved field reason"),
        })
        if field["field"] in gap["incomplete_fields"]:
            raise ValueError("Duplicate unresolved field")
        gap["incomplete_fields"].append(field["field"])
    relations = curated["spelling_relations"]
    expected_relations = reviewed["spelling_relations"]
    if len(relations) != 8 or [{key: row[key] for key in expected_relations[0]} for row in relations] != expected_relations:
        raise ValueError("Spelling relation differs from the independently reviewed evidence")
    aliases = {row["source_headword"]: row["frozen_headword"] for row in relations}
    if aliases != {"cœur": "coeur", "aujourd’hui": "aujourd'hui", "l’": "l'", "d’": "d'", "t’": "t'", "s’": "s'"} or any(
        relation["source_span"]["raw_source_excerpt"] != f"#REDIRECT [[{relation['source_headword']}]]"
        for relation in relations if relation["relationship"] == "explicit hard redirect"
    ):
        raise ValueError("Unsupported spelling relation or incomplete redirect evidence")
    checksums = read_json(source_dir / "input-checksums.json")
    if set(checksums) != set(MATERIALS):
        raise ValueError("Incomplete curated-input checksum manifest")
    for name, expected in checksums.items():
        if digest(source_dir / name) != expected:
            raise ValueError(f"Curated material changed after review: {name}")
    return curated, scope, selected, aliases, scope_gaps


def prepare(destination, source_dir=RESOURCE, project_license=ROOT / "LICENSE"):
    destination, source_dir = Path(destination), Path(source_dir)
    curated, scope, selected, aliases, scope_gaps = load_inputs(source_dir)
    if source_dir.resolve() == destination.resolve() or source_dir.resolve().is_relative_to(destination.resolve()):
        raise ValueError("Prepared output must not overwrite curated inputs")
    project_license = Path(project_license)
    if not project_license.is_file():
        raise ValueError("Missing separate MIT project/converter licence")
    version = text(curated["content_version"], "content version", 200)
    source = {"title": "Lexique FR–EN–ZH", "version": version,
              "attribution": "Wiktionnaire/Wiktionary contributors; selected adaptations by Lexique contributors; Lexique 3.83 membership by Boris New and Christophe Pallier",
              "license": "CC BY-SA 4.0", "source_url": "https://github.com/zardonc/lexique/tree/main/resources/core-pilot-79",
              "license_url": "https://creativecommons.org/licenses/by-sa/4.0/legalcode.en"}
    metadata = {"format_version": "1", "content_version": version, "source": compact(source),
                "scope_denominator_groups": "4087", "scope_denominator_headwords": "3773"}
    selected_rows = {row[0] for row in selected}
    complete_rows = selected_rows - scope_gaps.keys()
    specialist_rows = {row for row, gap in scope_gaps.items() if gap["everyday_sense_status"] == "approved_specialist_exception"}
    accepted_rows = complete_rows | specialist_rows
    acceptance = {"accepted_groups": len(accepted_rows), "approved_specialist_exception_groups": len(specialist_rows),
                  "remaining_unaccepted_groups": 4087 - len(accepted_rows)}
    if (curated.get("complete_groups"), curated.get("remaining_groups")) != (len(complete_rows), 4087 - len(complete_rows)):
        raise ValueError("Curated everyday-sense coverage differs from its review")
    if any(curated.get(key) != value for key, value in acceptance.items()):
        raise ValueError("Curated scope acceptance differs from its reviewed specialist exception")
    coverage = {"denominator_groups": 4087, "denominator_headwords": 3773, "complete_groups": len(complete_rows),
                "complete_fields": len(selected) * 4, "remaining_groups": 4087 - len(complete_rows), **acceptance,
                "incomplete_groups": [{"frozen_row": index, **group,
                                       "incomplete_fields": [] if index in selected_rows else list(FIELDS),
                                       **scope_gaps.get(index, {})}
                                      for index, group in enumerate(scope, 1) if index not in complete_rows]}
    destination.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="lexique-core-", dir=destination.parent) as temporary:
        staged = Path(temporary)
        database = staged / "core.sqlite3"
        connection = sqlite3.connect(database)
        try:
            connection.executescript("""
                PRAGMA page_size=4096;
                CREATE TABLE metadata(key TEXT PRIMARY KEY,value TEXT NOT NULL) WITHOUT ROWID;
                CREATE TABLE senses(frozen_row INTEGER NOT NULL,sense_index INTEGER NOT NULL,headword TEXT NOT NULL,payload TEXT NOT NULL,PRIMARY KEY(frozen_row,sense_index)) WITHOUT ROWID;
                CREATE INDEX senses_by_headword ON senses(headword,frozen_row,sense_index);
                CREATE TABLE aliases(query TEXT PRIMARY KEY,headword TEXT NOT NULL) WITHOUT ROWID;
            """)
            connection.executemany("INSERT INTO metadata VALUES(?,?)", sorted(metadata.items()))
            connection.executemany("INSERT INTO senses VALUES(?,?,?,?)", [(row, index, word, compact(payload)) for row, index, word, payload in selected])
            connection.executemany("INSERT INTO aliases VALUES(?,?)", sorted(aliases.items()))
            connection.commit()
            connection.execute("VACUUM")
        finally:
            connection.close()
        connection = sqlite3.connect(database.as_uri() + "?mode=ro", uri=True)
        try:
            if connection.execute("PRAGMA quick_check").fetchone()[0] != "ok":
                raise ValueError("Produced core failed SQLite integrity check")
            produced = list(connection.execute("SELECT frozen_row,sense_index,headword,payload FROM senses ORDER BY frozen_row,sense_index"))
            if produced != [(row, index, word, compact(payload)) for row, index, word, payload in selected] or dict(connection.execute("SELECT query,headword FROM aliases")) != aliases:
                raise ValueError("Produced core disagrees with reviewed field bindings")
            if dict(connection.execute("SELECT key,value FROM metadata")) != metadata:
                raise ValueError("Produced core metadata differs from its actual version/source")
        finally:
            connection.close()
        for name in (*MATERIALS, "input-checksums.json"):
            shutil.copyfile(source_dir / name, staged / name)
        shutil.copyfile(Path(__file__), staged / "core_pilot.py")
        shutil.copyfile(project_license, staged / "LICENSE-MIT")
        (staged / "coverage.json").write_text(json.dumps(coverage, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")
        inputs = {file.name: digest(file) for file in staged.iterdir() if file.name != "core.sqlite3"}
        content_batches = []
        for batch in curated["content_batches"]:
            if batch["issue"] == 79:
                continue
            first, last = batch["frozen_first_row"], batch["frozen_last_row"]
            batch_rows = set(range(first, last + 1))
            content_batches.append({
                "issue": batch["issue"], "target_groups": len(batch_rows),
                "reused_pilot_groups": batch["reused_groups"], "new_groups": batch["new_groups"],
                "complete_groups": len(complete_rows & batch_rows),
                "accepted_groups": len(accepted_rows & batch_rows),
                "approved_specialist_exception_groups": len(specialist_rows & batch_rows),
                "scope_acceptance": "passed" if batch_rows <= accepted_rows else "incomplete",
                "complete_fields": sum(4 for row in selected if row[0] in batch_rows),
                "groups": [{"frozen_row": row, "headword": scope[row - 1]["word"],
                            "pos": scope[row - 1]["lexique_cgram"], "complete_fields": 4 if row in selected_rows else 0,
                            **scope_gaps.get(row, {})}
                           for row in range(first, last + 1)],
            })
        report = {"format_version": 1, "content_version": version, "sqlite_sha256": digest(database),
                  "sqlite_bytes": database.stat().st_size, "inputs": inputs,
                  "admission": {"independent_field_review": "passed", "produced_resource_self_check": "passed",
                                "everyday_sense_review": "incomplete" if scope_gaps else "passed",
                                "selected_scope_acceptance": "passed" if not ((scope_gaps.keys() & selected_rows) - specialist_rows) else "incomplete",
                                **acceptance,
                                "selected_source_rights_and_materials": "passed", "complete_groups": len(complete_rows),
                                "complete_senses": len(selected), "complete_fields": len(selected) * 4, "remaining_groups": 4087 - len(complete_rows)},
                  "learner_acceptance": {"status": "not_run", "accepted": False}, "first_release": "incomplete",
                  "content_batches": content_batches,
                  "limits": {"query_utf8_bytes": 512, "matching_senses": 200, "response_bytes": 4 * 1024 * 1024,
                             "resource_bytes": 16_000_000, "field_unicode_scalars": 20_000},
                  "measurements": {"maximum_sense_json_bytes": max(len(compact(row[3]).encode()) for row in selected),
                                   "maximum_field_unicode_scalars": max(len(row[3][name]) for row in selected for name in ("definition_fr", "meaning_en", "meaning_zh")),
                                   "read_only_sqlite_bytes": database.stat().st_size}}
        (staged / "report.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")
        destination.mkdir(parents=True, exist_ok=True)
        for file in staged.iterdir():
            existing = destination / file.name
            if not existing.is_file() or digest(file) != digest(existing):
                file.replace(existing)
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=("prepare",))
    parser.add_argument("destination", type=Path)
    parser.add_argument("--source-dir", type=Path, default=RESOURCE)
    parser.add_argument("--project-license", type=Path, default=ROOT / "LICENSE")
    args = parser.parse_args()
    report = prepare(args.destination, args.source_dir, args.project_license)
    print(json.dumps({"sqlite_sha256": report["sqlite_sha256"], "sqlite_bytes": report["sqlite_bytes"], "admission": report["admission"]}))


if __name__ == "__main__":
    main()
