"""Offline conjugation resource CLI: LGPL-LR forms and GPL-3.0 metadata."""

import argparse
from collections import Counter, defaultdict
from contextlib import closing
import gzip
import hashlib
import itertools
import json
from pathlib import Path
import re
import shutil
import sqlite3
import struct
import unicodedata
import zlib

from conjugation_generation import SIMPLE, SUBJECTS, generate, source_form
from conjugation_metadata import RESOURCE as VERB_RESOURCE, load_metadata, metadata_report, verb_metadata
from conjugation_syntax import RESOURCE as SYNTAX_RESOURCE
from conjugation_meanings import RESOURCE as MEANING_RESOURCE, load_meanings, verb_meanings
from conjugation_pronominal import generate_pronominal, pronominal_metadata
from conjugation_coloring import coloring, coloring_report

RESOURCE = Path(__file__).resolve().parents[1] / "resources/lefff-3.4"
PROVENANCE = json.loads((RESOURCE / "provenance.json").read_text(encoding="utf-8"))
SOURCE = RESOURCE / "lefff-3.4.mlex.gz"
TENSES = {
    "P": "indicatif présent", "I": "indicatif imparfait", "J": "indicatif passé simple",
    "F": "indicatif futur simple", "C": "conditionnel présent", "S": "subjonctif présent",
    "T": "subjonctif imparfait", "Y": "impératif présent", "W": "infinitif présent",
    "G": "participe présent", "K": "participe passé",
    "PC": "passé composé", "PQP": "plus-que-parfait", "PA": "passé antérieur",
    "FA": "futur antérieur", "CP": "conditionnel passé", "SP": "subjonctif passé",
    "SQP": "subjonctif plus-que-parfait", "YP": "impératif passé", "WP": "infinitif passé",
    "KP": "participe composé", "GP": "gérondif présent", "GPP": "gérondif passé",
}
COLUMNS = "source_line, form, lemma, raw_tag, tense, person, number, gender, expressions"
APPLICATION_ID = 0x4C58434A
FORMAT_VERSION = 6
CONVERTERS = ("conjugation_sample.py", "conjugation_metadata.py", "conjugation_generation.py",
              "conjugation_syntax.py", "conjugation_meanings.py", "conjugation_pronominal.py",
              "conjugation_runtime.py", "conjugation_coloring.py", "conjugation_endings.json")


def construction_forms(data):
    return {auxiliary: {tense: {pn or "": source_form(data, auxiliary, tense, ":" + pn if pn else ":P")
                               for pn in ([None] if tense in ("W", "G") else SUBJECTS)}
                        for tense in ("P", "I", "J", "F", "C", "S", "T", "Y", "W", "G")}
            for auxiliary in ("avoir", "être")}


def store_models(connection, rows, verbs, meanings, data):
    simple = defaultdict(list)
    counts = Counter(row[2] for row in rows)
    for row in rows:
        if "pronominal" not in verbs[row[2]] and (row[4] in SIMPLE or row[4] in ("W", None)):
            simple[row[2]].append(row)
    models = {}
    stems = set()
    for lemma, info in verbs.items():
        base = info.get("pronominal", {}).get("base", lemma)
        model = None
        anchors = b""
        if base == lemma:
            pattern = []
            for row in simple[lemma]:
                common = 0
                for left, right in zip(lemma, row[1]):
                    if left != right:
                        break
                    common += 1
                pattern.append([len(lemma) - common, row[1][common:], *row[3:8]])
            encoded = json.dumps(pattern, ensure_ascii=False, separators=(",", ":"))
            if encoded not in models:
                models[encoded] = len(models)
                connection.execute("INSERT INTO models VALUES (?,?)", (models[encoded], encoded))
            model = models[encoded]
            anchors = struct.pack("<" + "I" * len(simple[lemma]), *(row[0] for row in simple[lemma]))
            for cut in {cell[0] for cell in pattern}:
                stems.add((lemma[:len(lemma) - cut], model, cut, lemma))
        connection.execute("INSERT INTO verbs VALUES (?,?,?,?,?,?,?,?)",
                           (lemma, base, model, anchors,
                            zlib.compress(json.dumps(info, ensure_ascii=False, separators=(",", ":")).encode("utf-8")),
                            json.dumps(meanings[lemma], ensure_ascii=False, separators=(",", ":")),
                            int(base in data["h_aspire"]), counts[lemma]))
    connection.executemany("INSERT INTO stems VALUES (?,?,?,?)", sorted(stems))
    return len(models), len(stems)


def source_rows(source):
    archive = Path(source).read_bytes()
    if hashlib.sha256(archive).hexdigest() != PROVENANCE["archive_sha256"]:
        raise ValueError("Source archive SHA-256 differs from pinned Lefff")
    raw = gzip.decompress(archive)
    if hashlib.sha256(raw).hexdigest() != PROVENANCE["source_sha256"]:
        raise ValueError("Uncompressed source SHA-256 differs from pinned Lefff")
    rows = []
    for ordinal, line in enumerate(raw.decode("utf-8").splitlines(), 1):
        fields = line.split("\t")
        if len(fields) != 4:
            raise ValueError(f"Invalid Lefff row at line {ordinal}")
        form, category, lemma, tag = fields
        if category != "v":
            continue
        if not form or not lemma or (tag and not re.fullmatch(r"[PFIJCYSTKGW]+[123]*[mf]*[sp]*", tag)):
            raise ValueError(f"Unsupported verb record at line {ordinal}: {tag}")
        # Expand only explicit dimensions; unspecified agreement stays unspecified.
        dimensions = [[c for c in tag if c in alphabet] or [None]
                      for alphabet in ("PFIJCYSTKGW", "123", "sp", "mf")]
        for tense, person, number, gender in itertools.product(*dimensions):
            rows.append((ordinal, form, lemma, tag, tense, person, number, gender))
    return rows, len(raw)


def build(output, source, metadata_source=VERB_RESOURCE, syntax_source=SYNTAX_RESOURCE, meanings_source=MEANING_RESOURCE,
          equivalence_baseline=None):
    english, english_provenance = load_meanings(meanings_source)
    rows, source_bytes = source_rows(source)
    linguistic_data, corrections, supplemental_provenance = load_metadata(metadata_source, syntax_source)
    verbs = {lemma: verb_metadata(lemma, linguistic_data, corrections) for lemma in sorted({r[2] for r in rows})}
    source_slots = len(rows)
    rows, block_coverage = generate(rows, verbs, linguistic_data)
    ordinary_verbs = dict(verbs)
    pronominal_rows, pronominal_verbs, pronominal_coverage = generate_pronominal(rows, verbs, linguistic_data)
    rows.extend(pronominal_rows)
    verbs.update(pronominal_verbs)
    coloring_coverage = coloring_report(rows, verbs)
    meanings = {lemma: verb_meanings(lemma, verb, english) for lemma, verb in verbs.items()}
    output = Path(output)
    output.mkdir(parents=True, exist_ok=False)
    database = output / "conjugation.sqlite3"
    with closing(sqlite3.connect(database)) as connection:
        connection.executescript(f"""
            PRAGMA application_id = {APPLICATION_ID};
            PRAGMA user_version = {FORMAT_VERSION};
            CREATE TABLE metadata (provenance TEXT NOT NULL, slots INTEGER NOT NULL,
                                   supplement TEXT NOT NULL, verbs INTEGER NOT NULL, english TEXT NOT NULL,
                                   construction TEXT NOT NULL);
            CREATE TABLE models (id INTEGER PRIMARY KEY, cells TEXT NOT NULL);
            CREATE TABLE verbs (lemma TEXT PRIMARY KEY COLLATE BINARY, base TEXT NOT NULL COLLATE BINARY,
                                model INTEGER, source_lines BLOB NOT NULL, information BLOB NOT NULL,
                                meanings TEXT NOT NULL, h_aspire INTEGER NOT NULL, slots INTEGER NOT NULL) WITHOUT ROWID;
            CREATE TABLE stems (stem TEXT NOT NULL COLLATE BINARY, model INTEGER NOT NULL, cut INTEGER NOT NULL,
                                lemma TEXT NOT NULL COLLATE BINARY, PRIMARY KEY(stem, model, cut, lemma)) WITHOUT ROWID;
        """)
        model_count, stem_count = store_models(connection, rows, verbs, meanings, linguistic_data)
        connection.execute("INSERT INTO metadata VALUES (?,?,?,?,?,?)",
                           (json.dumps(PROVENANCE, ensure_ascii=False), len(rows),
                            json.dumps(supplemental_provenance, ensure_ascii=False), len(verbs),
                            json.dumps(english_provenance, ensure_ascii=False),
                            json.dumps(construction_forms(linguistic_data), ensure_ascii=False, separators=(",", ":"))))
        connection.commit()
        table_bytes = database.stat().st_size
        connection.execute("CREATE INDEX verbs_base ON verbs(base)")
        connection.commit()
        lemma_bytes = database.stat().st_size - table_bytes
        reverse_bytes = connection.execute("SELECT sum(pgsize) FROM dbstat WHERE name = 'stems'").fetchone()[0]
        table_bytes -= reverse_bytes
    for name in (SOURCE.name, "LICENSE-LGPLLR", "provenance.json", "README.md"):
        shutil.copyfile(RESOURCE / name, output / name)
    supplement = output / "grammalecte-2.3.0"
    supplement.mkdir()
    for name in (*supplemental_provenance["grammalecte"]["files"], "provenance.json"):
        shutil.copyfile(Path(metadata_source) / name, supplement / name)
    syntax = output / "lefff-syntax"
    syntax.mkdir()
    for name in (*supplemental_provenance["lefff_syntax"]["files"], "provenance.json"):
        shutil.copyfile(Path(syntax_source) / name, syntax / name)
    english_source = output / "freedict-fra-eng-0.4.1"
    english_source.mkdir()
    for name in (*english_provenance["files"], "provenance.json"):
        shutil.copyfile(Path(meanings_source) / name, english_source / name)
    for name in CONVERTERS:
        shutil.copyfile(Path(__file__).with_name(name), output / name)
    shutil.copyfile(RESOURCE.parents[1] / "LICENSE", output / "LICENSE-MIT")
    report = {
        "sqlite_sha256": hashlib.sha256(database.read_bytes()).hexdigest(),
        "format": FORMAT_VERSION, "models": model_count, "stems": stem_count,
        "source_bytes": source_bytes, "source_gzip_bytes": Path(source).stat().st_size,
        "source_verb_rows": len({r[0] for r in rows}), "slots": len(rows), "source_slots": source_slots, "block_coverage": block_coverage,
        "lemmas": len({r[2] for r in rows}), "ordinary_lemmas": len(ordinary_verbs), "pronominal_coverage": pronominal_coverage,
        "coloring_coverage": coloring_coverage,
        "forms": len({r[1] for r in rows}),
        "table_bytes": table_bytes, "lemma_index_bytes": lemma_bytes,
        "reverse_index_bytes": reverse_bytes, "database_bytes": database.stat().st_size,
        "database_gzip_bytes": len(gzip.compress(database.read_bytes(), mtime=0)),
        "slots_by_tense": dict(sorted(Counter(r[4] or "unspecified" for r in rows).items())),
        "unspecified_finite_slots": sum(r[4] is not None and r[4] in "PFIJCYST" and (r[5] is None or r[6] is None)
                                        for r in rows),
        "data_license": "LGPL-LR", "modification": PROVENANCE["modified"],
        "metadata_license": supplemental_provenance["grammalecte"]["license"],
        "syntax_provenance": supplemental_provenance["lefff_syntax"],
        "metadata_source_bytes": supplemental_provenance["grammalecte"]["source_bytes"],
        "metadata_gzip_bytes": (Path(metadata_source) / "conj_data.json.gz").stat().st_size,
        "metadata_coverage": metadata_report(ordinary_verbs),
        "english_provenance": english_provenance,
        "english_coverage": {"verbs": len(verbs), "available": sum(m["status"] == "available" for m in meanings.values()),
                             "unavailable": [lemma for lemma, m in meanings.items() if m["status"] == "unavailable"],
                             "senses": sum(len(m["senses"]) for m in meanings.values())},
    }
    if equivalence_baseline is not None:
        report["equivalence"] = verify_equivalence(database, equivalence_baseline)
    (output / "report.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n",
                                        encoding="utf-8")
    return report


def prepare(output):
    """Reuse verified build output without replacing existing resources."""
    output = Path(output)
    if not output.exists():
        return build(output, SOURCE)
    read_resource(output / "conjugation.sqlite3")
    report = json.loads((output / "report.json").read_text(encoding="utf-8"))
    if hashlib.sha256((output / "conjugation.sqlite3").read_bytes()).hexdigest() != report["sqlite_sha256"]:
        raise ValueError("Generated conjugation resource checksum mismatch")
    for name in (SOURCE.name, "LICENSE-LGPLLR", "provenance.json", "README.md"):
        if (output / name).read_bytes() != (RESOURCE / name).read_bytes():
            raise ValueError(f"Generated conjugation resource differs from source: {name}")
    for name in CONVERTERS:
        if (output / name).read_bytes() != Path(__file__).with_name(name).read_bytes():
            raise ValueError(f"Generated conjugation converter differs from source; rebuild in a new directory: {name}")
    return report


def cell_order(row, columns):
    return tuple((row[column] is not None, row[column]) for column in columns)


def verify_equivalence(database, baseline):
    """Compare runtime tables with the preserved materialized resource."""
    from conjugation_runtime import load_models, table_cells

    baseline = Path(baseline).resolve()
    original_report = json.loads(baseline.with_name("report.json").read_text(encoding="utf-8"))
    if hashlib.sha256(baseline.read_bytes()).hexdigest() != original_report["sqlite_sha256"]:
        raise ValueError("Equivalence baseline checksum mismatch")
    with closing(sqlite3.connect(Path(database).resolve().as_uri() + "?mode=ro", uri=True)) as current, \
            closing(sqlite3.connect(baseline.as_uri() + "?mode=ro", uri=True)) as original:
        if (original.execute("PRAGMA application_id").fetchone()[0] != APPLICATION_ID
                or original.execute("PRAGMA user_version").fetchone()[0] != 5):
            raise ValueError("Equivalence baseline must be the materialized format-5 resource")
        models = load_models(current)
        construction = json.loads(current.execute("SELECT construction FROM metadata").fetchone()[0])
        expected_lemmas = [row[0] for row in original.execute("SELECT lemma FROM verbs ORDER BY lemma")]
        if [row[0] for row in current.execute("SELECT lemma FROM verbs ORDER BY lemma")] != expected_lemmas:
            raise ValueError("Conjugation identity mismatch")
        cache = {}
        count = 0
        for headword, information, meanings in original.execute("SELECT lemma, information, meanings FROM verbs ORDER BY lemma"):
            stored = current.execute("SELECT information, meanings, slots FROM verbs WHERE lemma=?", (headword,)).fetchone()
            if json.loads(zlib.decompress(stored[0])) != json.loads(information) or json.loads(stored[1]) != json.loads(meanings):
                raise ValueError(f"Conjugation metadata equivalence mismatch: {headword}")
            expected = original.execute(
                f"SELECT {COLUMNS} FROM slots WHERE lemma=? ORDER BY tense, person, number, gender, source_line",
                (headword,)).fetchall()
            actual = sorted(table_cells(current, headword, models, construction, cache),
                            key=lambda row: cell_order(row, (4, 5, 6, 7, 0)))
            if len(actual) != len(expected) or len(actual) != stored[2]:
                raise ValueError(f"Conjugation cell count equivalence mismatch: {headword}")
            for ordinal, (left, right) in enumerate(zip(expected, actual)):
                if tuple(left[:8]) != tuple(right[:8]) or json.loads(left[8]) != json.loads(right[8]):
                    raise ValueError(f"Conjugation cell equivalence mismatch: {headword}, cell {ordinal}, {left[:8]} != {right[:8]}")
            count += len(actual)
        if count != original_report["slots"]:
            raise ValueError("Conjugation full-corpus equivalence count mismatch")
        return {"status": "equal", "verbs": len(expected_lemmas), "cells": count,
                "baseline_sha256": original_report["sqlite_sha256"]}


def read_resource(database, query=None, lemma=None):
    from conjugation_runtime import candidate_lemmas, load_models, matches_query, normalize_query, table_cells

    normalized = normalize_query(query) if query is not None else None
    uri = Path(database).resolve().as_uri() + "?mode=ro"
    with closing(sqlite3.connect(uri, uri=True)) as connection:
        connection.row_factory = sqlite3.Row
        connection.execute("PRAGMA trusted_schema = OFF")
        if (connection.execute("PRAGMA application_id").fetchone()[0] != APPLICATION_ID
                or connection.execute("PRAGMA user_version").fetchone()[0] != FORMAT_VERSION):
            raise ValueError("Unsupported conjugation resource format")
        if [r[0] for r in connection.execute("PRAGMA integrity_check")] != ["ok"]:
            raise ValueError("Conjugation resource failed integrity check")
        metadata = connection.execute("SELECT provenance, slots, supplement, verbs, english, construction FROM metadata").fetchall()
        if (len(metadata) != 1 or json.loads(metadata[0][0]) != PROVENANCE
                or connection.execute("SELECT sum(slots) FROM verbs").fetchone()[0] != metadata[0][1]
                or connection.execute("SELECT count(*) FROM verbs").fetchone()[0] != metadata[0][3]
                or connection.execute("SELECT base FROM verbs EXCEPT SELECT lemma FROM verbs").fetchone()):
            raise ValueError("Conjugation resource metadata/count mismatch")
        linguistic_data, corrections, supplemental_provenance = load_metadata(
            Path(database).resolve().parent / "grammalecte-2.3.0", Path(database).resolve().parent / "lefff-syntax")
        if json.loads(metadata[0][2]) != supplemental_provenance:
            raise ValueError("Conjugation supplemental provenance mismatch")
        english, english_provenance = load_meanings(Path(database).resolve().parent / "freedict-fra-eng-0.4.1")
        if json.loads(metadata[0][4]) != english_provenance:
            raise ValueError("Conjugation English meaning provenance mismatch")
        models = load_models(connection)
        construction = json.loads(metadata[0][5])
        if construction != construction_forms(linguistic_data):
            raise ValueError("Conjugation auxiliary forms differ from pinned source")
        information_by_lemma = {}
        for headword, information, meaning in connection.execute("SELECT lemma, information, meanings FROM verbs"):
            if not isinstance(information, bytes):
                raise ValueError("Invalid compressed conjugation verb metadata")
            info = json.loads(zlib.decompress(information))
            if not isinstance(info, dict) or info.get("ordinary_status") not in ("supported", "not_applicable", "unconfirmed"):
                raise ValueError("Invalid conjugation verb metadata")
            if "pronominal" in info and (not isinstance(info["pronominal"], dict)
                                        or not isinstance(info["pronominal"].get("base"), str)
                                        or not isinstance(info["pronominal"].get("with_en"), bool)):
                raise ValueError("Invalid pronominal metadata")
            meanings = json.loads(meaning)
            if meanings != verb_meanings(headword, info, english):
                raise ValueError("Conjugation English meanings differ from pinned source")
            information_by_lemma[headword] = (info, meanings)
        cache = {}
        generated = {headword: table_cells(connection, headword, models, construction, cache)
                     for headword in ([] if query is None else candidate_lemmas(connection, normalized, models))}
        matched = sorted((row for rows in generated.values() for row in rows
                          if matches_query(row[1], json.loads(row[8]), normalized)),
                         key=lambda row: cell_order(row, (2, 0, 4, 5)))
        matches = [dict(zip(COLUMNS.split(", "), row)) for row in matched]
        for cell in matches:
            cell["expressions"] = json.loads(cell["expressions"])
        lemmas = [lemma] if lemma is not None else sorted({r["lemma"] for r in matches})
        tables = []
        for headword in lemmas:
            rows = generated.get(headword)
            if rows is None:
                rows = table_cells(connection, headword, models, construction, cache)
            cells = [dict(zip(COLUMNS.split(", "), row)) for row in sorted(rows, key=lambda row: cell_order(row, (4, 5, 6, 7, 0)))]
            for cell in cells:
                cell["expressions"] = json.loads(cell["expressions"])
                cell["matched"] = query is not None and matches_query(cell["form"], cell["expressions"], normalized)
            information = information_by_lemma.get(headword)
            verb = information[0] if information else None
            base = verb.get("pronominal", {}).get("base", headword) if isinstance(verb, dict) else headword
            expected = verb_metadata(base, linguistic_data, corrections)
            if isinstance(verb, dict) and "pronominal" in verb:
                expected = pronominal_metadata(base, expected, linguistic_data, verb["pronominal"]["with_en"])
            if information is not None and (not isinstance(verb, dict) or {k: v for k, v in verb.items() if k != "blocks"} != expected):
                raise ValueError("Conjugation verb metadata differs from pinned sources")
            for cell in cells:
                cell["coloring"] = coloring(cell, verb)
            table_status = "pronominal" if verb and "pronominal" in verb else "ordinary_active" if verb and verb["ordinary_status"] == "supported" else "source_only"
            tables.append({"lemma": headword, "status": table_status if cells else "not_found",
                           "verb": verb, "meanings": information[1] if information else {"status": "unavailable", "senses": []}, "cells": cells})
        for cell in matches:
            cell["coloring"] = coloring(cell, information_by_lemma[cell["lemma"]][0])
        return {"status": "found" if matches or any(t["cells"] for t in tables) else "not_found",
                "query": query, "matches": matches, "tables": tables,
                "tense_labels": TENSES, "coverage": "ordinary/v1; see per-verb blocks and resource-wide block_coverage"}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    builder = commands.add_parser("build")
    builder.add_argument("output", type=Path)
    builder.add_argument("--source", type=Path, default=SOURCE)
    builder.add_argument("--metadata-source", type=Path, default=VERB_RESOURCE)
    builder.add_argument("--syntax-source", type=Path, default=SYNTAX_RESOURCE)
    builder.add_argument("--meanings-source", type=Path, default=MEANING_RESOURCE)
    builder.add_argument("--equivalence-baseline", type=Path,
                         help="Compare every generated cell with a preserved format-5 database and report")
    commands.add_parser("prepare").add_argument("output", type=Path)
    for command, argument in (("query", "form"), ("table", "lemma")):
        reader = commands.add_parser(command)
        reader.add_argument("database", type=Path)
        reader.add_argument(argument)
    args = parser.parse_args()
    try:
        if args.command == "prepare":
            result = prepare(args.output)
        elif args.command == "build":
            result = build(args.output, args.source, args.metadata_source, args.syntax_source, args.meanings_source,
                           args.equivalence_baseline)
        else:
            value = args.form if args.command == "query" else args.lemma
            if not value.strip() or len(value.encode("utf-8")) > 512 or any(unicodedata.category(c) == "Cc" for c in value):
                raise ValueError("Lookup must contain 1–512 UTF-8 bytes without control characters")
            result = read_resource(args.database, query=value if args.command == "query" else None,
                                   lemma=value if args.command == "table" else None)
    except (OSError, ValueError, sqlite3.Error, EOFError, zlib.error) as error:
        parser.exit(1, f"Conjugation resource error: {error}\n")
    print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
