"""Compare the native lookup with complete tables from the preserved format-5 resource."""

from contextlib import closing
import json
from pathlib import Path
import sqlite3
import subprocess
import sys
import time

sys.path.insert(0, str(Path(__file__).resolve().parents[3] / "scripts"))
from conjugation_generation import query_base


baseline, compact, executable, output = map(Path, sys.argv[1:])
output.mkdir(parents=True, exist_ok=False)
report = json.loads(compact.with_name("report.json").read_text(encoding="utf-8"))
columns = "source_line, form, lemma, raw_tag, tense, person, number, gender, expressions"
probes = [
    "sommes", "suis", "paie", "abstenues", "été", "avons aimé", "aie aimé", "je suis allée",
    "en ayant aimé", "que nous ayons aimé", "j’ai pu", "aie occis", "j’habite", "je hais",
    "elles se sont abstenues", "elles se sont parlé", "elles se sont lavées", "elle s’est arrogé",
    "nous nous sommes écrites", "on s’est abstenues", "va-t’en", "je m’en suis allé",
    "je me hais", "je m’habille", "il s’agit", "on s’entraide", "lave-toi", "en se lavant",
    "je avons aimé", "en ayant vraiment aimé", "j’ai parvenu", "je suis avenu", "j’hais",
    "je habite", "elles se sont abstenus", "elles se sont parlées", "je se lave", "je m’agis",
    "il s’est abstenues", "s’étant agis", "s’en découdre", "SOMMES", "ete", "' OR 1=1 --",
]
with closing(sqlite3.connect(baseline.resolve().as_uri() + "?mode=ro", uri=True)) as connection:
    connection.row_factory = sqlite3.Row
    assert connection.execute("PRAGMA user_version").fetchone()[0] == 5
    lemmas = [row[0] for row in connection.execute("SELECT lemma FROM verbs ORDER BY lemma")]
    queries = []
    for lemma in lemmas:
        queries.append(connection.execute(
            "SELECT form FROM slots WHERE lemma=? ORDER BY form != ?, source_line LIMIT 1",
            (lemma, lemma)).fetchone()[0])
    queries = list(dict.fromkeys([*queries, *probes]))
    covered = set()

    def expected_lookup(query):
        matches = connection.execute(f"SELECT lemma, form, expressions FROM slots WHERE form IN (?,?)",
                                     (query, query_base(query))).fetchall()
        identities = sorted({row[0] for row in matches if row[1] == query or query in json.loads(row[2])})
        tables = []
        for lemma in identities:
            verb, meanings = connection.execute("SELECT information, meanings FROM verbs WHERE lemma=?",
                                                (lemma,)).fetchone()
            cells = []
            for row in connection.execute(
                    f"SELECT {columns} FROM slots WHERE lemma=? ORDER BY tense, person, number, gender, source_line",
                    (lemma,)):
                cell = dict(row)
                cell["expressions"] = json.loads(cell["expressions"])
                cells.append(cell)
            tables.append({"lemma": lemma, "verb": json.loads(verb), "meanings": json.loads(meanings), "cells": cells})
        return {"query": query, "tables": tables}

    assert [table["lemma"] for table in expected_lookup("sommes")["tables"]] == ["sommer", "être"]
    assert expected_lookup("je avons aimé")["tables"] == []
    batches = []
    for offset in range(0, len(queries), 400):
        path = output / f"expected-{offset // 400:03}.jsonl"
        with path.open("x", encoding="utf-8") as stream:
            for query in queries[offset:offset + 400]:
                expected = expected_lookup(query)
                covered.update(table["lemma"] for table in expected["tables"])
                stream.write(json.dumps(expected, ensure_ascii=False, separators=(",", ":")) + "\n")
        started = time.perf_counter()
        result = subprocess.run([str(executable.resolve()), str(compact.resolve()), report["sqlite_sha256"], str(path.resolve())],
                                capture_output=True, text=True, encoding="utf-8", timeout=60,
                                creationflags=subprocess.CREATE_NO_WINDOW if sys.platform == "win32" else 0)
        if result.returncode:
            raise AssertionError(result.stdout + result.stderr)
        batches.append({"cases": min(400, len(queries) - offset), "milliseconds": round((time.perf_counter() - started) * 1000),
                        "result": result.stdout.strip()})
        print(batches[-1], flush=True)
    assert covered == set(lemmas), "Every baseline identity must be exercised through native lookup"
    cells = connection.execute("SELECT count(*) FROM slots").fetchone()[0]
    assert cells == report["slots"]
    result = {"status": "equal", "identities": len(covered), "cells": cells, "lookups": len(queries),
              "resource_sha256": report["sqlite_sha256"], "batch_timeout_seconds": 60, "batches": batches}
    (output / "report.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
