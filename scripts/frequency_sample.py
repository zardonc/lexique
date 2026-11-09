"""Build the fixed Lexique 3.83 read-only word-frequency resource."""

import argparse
import csv
import gzip
import hashlib
import json
import math
import shutil
import sqlite3
import time
import unicodedata
from collections import defaultdict
from contextlib import closing
from decimal import Decimal, InvalidOperation
from pathlib import Path


def read_value(raw: str | None) -> tuple[Decimal | None, str | None]:
    if raw is None or not raw.strip():
        return None, "missing"
    try:
        value = Decimal(raw)
        if value.is_finite() and value >= 0 and math.isfinite(float(value)):
            return value, None
    except InvalidOperation:
        pass
    return None, "invalid"


def build_database(source: Path, database: Path) -> dict:
    if database.exists():
        raise FileExistsError(f"Frequency output already exists: {database}")
    groups = defaultdict(set)
    forms = set()
    base_lemmas = set()
    rows = 0
    with gzip.open(source, "rt", encoding="utf-8-sig", newline="") as stream:
        reader = csv.DictReader(stream, delimiter="\t")
        required = {"ortho", "lemme", "cgram", "freqlemfilms2"}
        if not required <= set(reader.fieldnames or []):
            raise ValueError("Frequency source is missing required columns")
        for row in reader:
            rows += 1
            if not all(row.get(field) and row[field].strip() for field in ("ortho", "lemme")):
                raise ValueError(f"Missing word or lemma at source row {rows}")
            lemma = unicodedata.normalize("NFC", row["lemme"])
            part = row["cgram"] or ""
            groups[(lemma, part)].add(read_value(row["freqlemfilms2"]))
            forms.add((unicodedata.normalize("NFC", row["ortho"]).lower(), lemma))
            if row.get("islem") == "1":
                base_lemmas.add(lemma)
    if not rows:
        raise ValueError("Frequency source has no rows")
    lemmas = defaultdict(list)
    for (lemma, part), values in sorted(groups.items()):
        errors = {reason for _, reason in values if reason is not None}
        reason = "invalid" if "invalid" in errors else "missing" if errors else "conflict" if len(values) != 1 else None
        value = next(iter(values))[0] if reason is None else None
        if reason is None and not part.strip():
            reason = "missing_pos"
        lemmas[lemma].append({"part_of_speech": part, "value": value, "reason": reason})
    with closing(sqlite3.connect(database)) as connection, connection:
        connection.execute("PRAGMA foreign_keys = ON")
        connection.execute("CREATE TABLE metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL)")
        connection.executemany("INSERT INTO metadata VALUES (?, ?)",
                               [("format_version", "1"), ("version", "3.83"), ("metric", "freqlemfilms2")])
        connection.execute(
            "CREATE TABLE lemmas (lemma TEXT PRIMARY KEY, search_key TEXT NOT NULL, "
            "per_million REAL, reason TEXT, parts TEXT NOT NULL)"
        )
        connection.execute("CREATE INDEX lemma_search ON lemmas(search_key)")
        connection.execute(
            "CREATE TABLE forms (key TEXT NOT NULL, lemma TEXT NOT NULL REFERENCES lemmas(lemma), "
            "PRIMARY KEY (key, lemma)) WITHOUT ROWID"
        )
        for lemma, parts in lemmas.items():
            reason = next((part["reason"] for part in parts if part["reason"] is not None), None)
            value = float(sum((part["value"] for part in parts), Decimal(0))) if reason is None else None
            if value is not None and not math.isfinite(value):
                value, reason = None, "invalid"
            detail = [{"part_of_speech": part["part_of_speech"] if part["part_of_speech"].strip() else None,
                       "per_million": float(part["value"]) if part["value"] is not None else None,
                       "reason": part["reason"]} for part in parts]
            connection.execute(
                "INSERT INTO lemmas VALUES (?, ?, ?, ?, ?)",
                (lemma, lemma.lower(), value, reason,
                 json.dumps(detail, ensure_ascii=False, allow_nan=False, separators=(",", ":"))),
            )
        connection.executemany("INSERT INTO forms VALUES (?, ?)", sorted(forms))
        unknown = connection.execute("SELECT COUNT(*) FROM lemmas WHERE reason IS NOT NULL").fetchone()[0]
        zero = connection.execute("SELECT COUNT(*) FROM lemmas WHERE per_million = 0").fetchone()[0]
        ambiguous = connection.execute(
            "SELECT COUNT(*) FROM (SELECT key FROM forms GROUP BY key HAVING COUNT(*) > 1)"
        ).fetchone()[0]
    return {"rows": rows, "lemmas": len(lemmas), "lemma_pos_groups": len(groups),
            "surface_keys": len({key for key, _ in forms}), "surface_links": len(forms),
            "ambiguous_surface_keys": ambiguous, "lemmas_without_islem": len(lemmas.keys() - base_lemmas),
            "unknown_lemmas": unknown, "zero_frequency_lemmas": zero,
            "missing_part_of_speech_groups": sum(not part["part_of_speech"].strip() for parts in lemmas.values() for part in parts),
            "missing_groups": sum(part["reason"] == "missing" for parts in lemmas.values() for part in parts),
            "invalid_groups": sum(part["reason"] == "invalid" for parts in lemmas.values() for part in parts),
            "conflicting_groups": sum(part["reason"] == "conflict" for parts in lemmas.values() for part in parts)}


def prepare(output: Path) -> dict:
    root = Path(__file__).resolve().parent.parent
    resource = root / "resources" / "lexique-3.83"
    sources = [(name, resource / name) for name in (
        "Lexique383.tsv.gz", "COPYING", "UPSTREAM-README.txt", "README.md", "provenance.json")]
    sources += [("frequency_sample.py", Path(__file__)), ("LICENSE-MIT", root / "LICENSE")]
    inputs = {name: hashlib.sha256(path.read_bytes()).hexdigest() for name, path in sources}
    provenance = json.loads((resource / "provenance.json").read_text(encoding="utf-8"))
    if inputs["Lexique383.tsv.gz"] != provenance["archive_sha256"]:
        raise ValueError("Fixed Lexique 3.83 source archive checksum mismatch")
    database = output / "frequency.sqlite3"
    report_path = output / "report.json"
    if output.exists():
        report = json.loads(report_path.read_text(encoding="utf-8"))
        if report["inputs"] != inputs:
            raise ValueError("Frequency preparation inputs changed; use a new output directory")
        if hashlib.sha256(database.read_bytes()).hexdigest() != report["sqlite_sha256"]:
            raise ValueError(f"Prepared frequency database checksum mismatch: {database}")
        for name, _ in sources:
            if hashlib.sha256((output / name).read_bytes()).hexdigest() != inputs[name]:
                raise ValueError(f"Prepared frequency source or notice changed: {name}")
        return report
    output.mkdir(parents=True, exist_ok=False)
    started = time.perf_counter()
    report = build_database(resource / "Lexique383.tsv.gz", database)
    for name, source in sources:
        shutil.copyfile(source, output / name)
    report.update({"source_bytes": provenance["source_bytes"], "source_sha256": provenance["source_sha256"],
                   "sqlite_bytes": database.stat().st_size,
                   "sqlite_sha256": hashlib.sha256(database.read_bytes()).hexdigest(),
                   "conversion_seconds": time.perf_counter() - started, "inputs": inputs})
    report_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    return report


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    print(json.dumps(prepare(parser.parse_args().output), ensure_ascii=False, indent=2))
