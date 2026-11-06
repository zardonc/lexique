"""Offline conjugation resource CLI: LGPL-LR forms and GPL-3.0 metadata."""

import argparse
from collections import Counter
from contextlib import closing
import gzip
import hashlib
import itertools
import json
from pathlib import Path
import re
import shutil
import sqlite3

from conjugation_generation import generate, query_base
from conjugation_metadata import RESOURCE as VERB_RESOURCE, load_metadata, metadata_report, verb_metadata
from conjugation_syntax import RESOURCE as SYNTAX_RESOURCE

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


def build(output, source, metadata_source=VERB_RESOURCE, syntax_source=SYNTAX_RESOURCE):
    rows, source_bytes = source_rows(source)
    linguistic_data, corrections, supplemental_provenance = load_metadata(metadata_source, syntax_source)
    verbs = {lemma: verb_metadata(lemma, linguistic_data, corrections) for lemma in sorted({r[2] for r in rows})}
    source_slots = len(rows)
    rows, block_coverage = generate(rows, verbs, linguistic_data)
    output = Path(output)
    output.mkdir(parents=True, exist_ok=False)
    database = output / "conjugation.sqlite3"
    with closing(sqlite3.connect(database)) as connection:
        connection.executescript(f"""
            PRAGMA application_id = {APPLICATION_ID};
            PRAGMA user_version = 3;
            CREATE TABLE metadata (provenance TEXT NOT NULL, slots INTEGER NOT NULL,
                                   supplement TEXT NOT NULL, verbs INTEGER NOT NULL);
            CREATE TABLE verbs (lemma TEXT PRIMARY KEY, information TEXT NOT NULL);
            CREATE TABLE slots (
                source_line INTEGER NOT NULL, form TEXT NOT NULL COLLATE BINARY,
                lemma TEXT NOT NULL COLLATE BINARY, raw_tag TEXT NOT NULL,
                tense TEXT, person TEXT, number TEXT, gender TEXT, expressions TEXT NOT NULL
            );
        """)
        connection.executemany("INSERT INTO slots VALUES (?,?,?,?,?,?,?,?,?)", rows)
        connection.executemany("INSERT INTO verbs VALUES (?,?)",
                               [(lemma, json.dumps(info, ensure_ascii=False)) for lemma, info in verbs.items()])
        connection.execute("INSERT INTO metadata VALUES (?,?,?,?)",
                           (json.dumps(PROVENANCE, ensure_ascii=False), len(rows),
                            json.dumps(supplemental_provenance, ensure_ascii=False), len(verbs)))
        connection.commit()
        table_bytes = database.stat().st_size
        connection.execute("CREATE INDEX slots_lemma ON slots(lemma)")
        connection.commit()
        lemma_bytes = database.stat().st_size - table_bytes
        before_reverse = database.stat().st_size
        connection.execute("CREATE INDEX slots_form ON slots(form)")
        connection.commit()
        reverse_bytes = database.stat().st_size - before_reverse
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
    for name in ("conjugation_sample.py", "conjugation_metadata.py", "conjugation_generation.py", "conjugation_syntax.py"):
        shutil.copyfile(Path(__file__).with_name(name), output / name)
    shutil.copyfile(RESOURCE.parents[1] / "LICENSE", output / "LICENSE-MIT")
    report = {
        "sqlite_sha256": hashlib.sha256(database.read_bytes()).hexdigest(),
        "source_bytes": source_bytes, "source_gzip_bytes": Path(source).stat().st_size,
        "source_verb_rows": len({r[0] for r in rows}), "slots": len(rows), "source_slots": source_slots, "block_coverage": block_coverage,
        "lemmas": len({r[2] for r in rows}), "forms": len({r[1] for r in rows}),
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
        "metadata_coverage": metadata_report(verbs),
    }
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
    for name in ("conjugation_sample.py", "conjugation_metadata.py", "conjugation_generation.py", "conjugation_syntax.py"):
        if (output / name).read_bytes() != Path(__file__).with_name(name).read_bytes():
            raise ValueError(f"Generated conjugation converter differs from source; rebuild in a new directory: {name}")
    return report


def read_resource(database, query=None, lemma=None):
    uri = Path(database).resolve().as_uri() + "?mode=ro"
    with closing(sqlite3.connect(uri, uri=True)) as connection:
        connection.row_factory = sqlite3.Row
        connection.execute("PRAGMA trusted_schema = OFF")
        if (connection.execute("PRAGMA application_id").fetchone()[0] != APPLICATION_ID
                or connection.execute("PRAGMA user_version").fetchone()[0] != 3):
            raise ValueError("Unsupported conjugation resource format")
        if [r[0] for r in connection.execute("PRAGMA integrity_check")] != ["ok"]:
            raise ValueError("Conjugation resource failed integrity check")
        metadata = connection.execute("SELECT provenance, slots, supplement, verbs FROM metadata").fetchall()
        if (len(metadata) != 1 or json.loads(metadata[0][0]) != PROVENANCE
                or connection.execute("SELECT count(*) FROM slots").fetchone()[0] != metadata[0][1]
                or connection.execute("SELECT count(*) FROM verbs").fetchone()[0] != metadata[0][3]
                or connection.execute("SELECT count(DISTINCT lemma) FROM slots").fetchone()[0] != metadata[0][3]
                or connection.execute("SELECT lemma FROM slots EXCEPT SELECT lemma FROM verbs").fetchone()):
            raise ValueError("Conjugation resource metadata/count mismatch")
        linguistic_data, corrections, supplemental_provenance = load_metadata(
            Path(database).resolve().parent / "grammalecte-2.3.0", Path(database).resolve().parent / "lefff-syntax")
        if json.loads(metadata[0][2]) != supplemental_provenance:
            raise ValueError("Conjugation supplemental provenance mismatch")
        matches = [] if query is None else [dict(r) for r in connection.execute(
            f"SELECT {COLUMNS} FROM slots WHERE form IN (?, ?) ORDER BY lemma, source_line, tense, person",
            (query, query_base(query)))]
        for cell in matches:
            cell["expressions"] = json.loads(cell["expressions"])
        matches = [cell for cell in matches if cell["form"] == query or query in cell["expressions"]]
        lemmas = [lemma] if lemma is not None else sorted({r["lemma"] for r in matches})
        tables = []
        for headword in lemmas:
            cells = [dict(r) for r in connection.execute(
                f"SELECT {COLUMNS} FROM slots WHERE lemma = ? ORDER BY tense, person, number, gender, source_line",
                (headword,))]
            for cell in cells:
                cell["expressions"] = json.loads(cell["expressions"])
                cell["matched"] = query is not None and (cell["form"] == query or query in cell["expressions"])
            information = connection.execute("SELECT information FROM verbs WHERE lemma = ?", (headword,)).fetchone()
            verb = json.loads(information[0]) if information else None
            if information is not None and (not isinstance(verb, dict) or {k: v for k, v in verb.items() if k != "blocks"} != verb_metadata(headword, linguistic_data, corrections)):
                raise ValueError("Conjugation verb metadata differs from pinned sources")
            table_status = "ordinary_active" if verb and verb["ordinary_status"] == "supported" else "source_only"
            tables.append({"lemma": headword, "status": table_status if cells else "not_found",
                           "verb": verb, "cells": cells})
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
            result = build(args.output, args.source, args.metadata_source, args.syntax_source)
        else:
            value = args.form if args.command == "query" else args.lemma
            if not value or any(ord(c) < 32 for c in value):
                raise ValueError("Lookup must be nonempty and contain no control characters")
            result = read_resource(args.database, query=value if args.command == "query" else None,
                                   lemma=value if args.command == "table" else None)
    except (OSError, ValueError, sqlite3.Error, EOFError) as error:
        parser.exit(1, f"Conjugation resource error: {error}\n")
    print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
