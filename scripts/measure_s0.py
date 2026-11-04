"""Build and run #25 on new target/issue-25 fixtures; never open a user database."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import sqlite3
import subprocess
import sys
import time

from s0_baseline import NOW, REPO, SEED, operations, oracle, seed_rows, summarize
from s0_faults import files


def replace_once(source, before, after):
    if source.count(before) != 1:
        raise ValueError(f"Current source changed; inspect observation anchor: {before[:80]}")
    return source.replace(before, after)


def new_directory(path):
    path = path.resolve()
    owner = (REPO / "target/issue-25").resolve()
    if not path.is_relative_to(owner) or path == owner:
        raise ValueError("Output must be a new child of target/issue-25")
    path.mkdir(parents=True, exist_ok=False)
    (path / ".s0-owned").write_text("isolated synthetic S0 data\n", encoding="utf8")
    return path


def execute(command, log, *, env=None, timeout=60, request=None):
    """Compilation may have a separate budget; every execution is hard <=60s."""
    started = time.perf_counter()
    with log.open("wb") as stdout, log.with_suffix(".stderr").open("wb") as stderr:
        child = subprocess.Popen(command, cwd=REPO, env=env, stdin=subprocess.PIPE,
            stdout=stdout, stderr=stderr, creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0)
        try:
            child.communicate(None if request is None else json.dumps(request).encode(), timeout=timeout)
        except subprocess.TimeoutExpired:
            if os.name == "nt":
                subprocess.run(["taskkill.exe", "/PID", str(child.pid), "/T", "/F"], capture_output=True, check=True)
            else:
                child.kill()
            child.wait()
            return {"status": "timeout", "ms": timeout * 1000, "error": "hard execution limit"}
    return {"status": "passed" if child.returncode == 0 else "failed", "exit_code": child.returncode,
        "ms": (time.perf_counter() - started) * 1000}


def sources(root):
    """Copy the current blocking body; change only clock injection and measurement."""
    files = {name: (REPO / path).read_text(encoding="utf8") for name, path in {
        "vocabulary.rs": "apps/desktop/src-tauri/src/vocabulary.rs",
        "speech_tasks.rs": "apps/desktop/src-tauri/src/speech_tasks.rs",
        "speech_cache.rs": "apps/desktop/src-tauri/src/speech_cache.rs",
        "review.rs": "crates/storage/src/review.rs"}.items()}
    vocabulary = files["vocabulary.rs"]
    opening = vocabulary.split("pub(crate) fn open(", 1)[1].split("\nfn first_translation", 1)[0]
    opening = "pub(crate) fn open(" + opening
    opening = replace_once(opening, "    let connection =\n        Connection::open_with_flags(&path.0, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE)",
        '    let connection = crate::metrics::measure("database_open", || Connection::open_with_flags(&path.0, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE))')
    opening = replace_once(opening, "    lexique_storage::initialize(&connection).map_err(|error| error.to_string())?;",
        '    crate::metrics::attach(&connection);\n    crate::metrics::measure("database_initialize", || lexique_storage::initialize(&connection)).map_err(|error| error.to_string())?;')
    inspection = vocabulary.split("type AudioInspection =", 1)[1].split("#[tauri::command]", 1)[0]
    body = vocabulary.split("pub async fn review_queue(", 1)[1].split("tauri::async_runtime::spawn_blocking(move || {", 1)[1].split("\n    })\n    .await", 1)[0]
    body = replace_once(body, "        let now = std::time::SystemTime::now()\n            .duration_since(std::time::UNIX_EPOCH)\n            .map_err(|e| e.to_string())?\n            .as_secs();", "        let now = u64::try_from(supplied_now).map_err(|e| e.to_string())?;")
    body = body.replace("lexique_storage::review_queue_filtered(", "crate::review::review_queue_filtered(")
    files["vocabulary.rs"] = "use std::{path::PathBuf,time::Duration};\nuse rusqlite::Connection;\npub struct DatabasePath(pub PathBuf);\n" + opening + "\ntype AudioInspection =" + inspection + "\n#[derive(serde::Serialize)]\npub struct QueueView { pub queue: crate::review::ReviewQueue, pub record: Option<lexique_storage::VocabularyRecord>, pub audio_pending: bool, pub audio_error: Option<String>, pub audio_vocabulary_id: Option<i64> }\npub fn queue(path: &DatabasePath, timezone: String, skipped: Vec<i64>, supplied_now: i64) -> Result<QueueView,String> {\n" + body + "\n}\n"
    tasks = files["speech_tasks.rs"]
    files["speech_tasks.rs"] = "use rusqlite::Connection;\nuse lexique_storage as storage;\nuse crate::speech_cache;\n" + "pub fn cached_asset(" + tasks.split("pub fn cached_asset(", 1)[1].split("\ntype Reply =", 1)[0]
    files["speech_tasks.rs"] = files["speech_tasks.rs"].replace("storage::content_hash(&bytes)", 'crate::metrics::measure("file_hash", || storage::content_hash(&bytes))').replace("storage::content_hash(bytes)", 'crate::metrics::measure("file_hash", || storage::content_hash(bytes))')
    cache = files["speech_cache.rs"]
    cache = replace_once(cache, "    let file = match dir.open_with(name, &options) {",
        '    let file = match crate::metrics::measure("file_open", || dir.open_with(name, &options)) {')
    cache = replace_once(cache, "    (&file)\n        .take(10_000_001)\n        .read_to_end(&mut bytes)\n        .map_err(|e| e.to_string())?;",
        '    crate::metrics::measure("file_read", || (&file).take(10_000_001).read_to_end(&mut bytes)).map_err(|e| e.to_string())?;\n    crate::metrics::record("file_bytes", 1, bytes.len() as u64, 0.0);')
    files["speech_cache.rs"] = cache
    cache = replace_once(cache, '        file.write_all(bytes).map_err(|e| e.to_string())?;',
        '        crate::metrics::measure("publication_tmp_write", || file.write_all(bytes)).map_err(|e| e.to_string())?;')
    cache = replace_once(cache, '        file.sync_all().map_err(|e| e.to_string())?;',
        '        crate::metrics::measure("publication_tmp_sync", || file.sync_all()).map_err(|e| e.to_string())?;')
    cache = replace_once(cache, '        return publish_new(&dir, &temporary, name)',
        '        return crate::metrics::measure("publication_rename", || publish_new(&dir, &temporary, name))')
    cache = replace_once(cache, '            return dir\n                .rename(&temporary, &dir, name)',
        '            return crate::metrics::measure("publication_rename", || dir.rename(&temporary, &dir, name))')
    files["speech_cache.rs"] = cache
    review = files["review.rs"].replace("serde_json::from_str(", "crate::metrics::parse(")
    review = replace_once(review, "    old.sort_unstable();", '    crate::metrics::measure("sort_review", || old.sort_unstable());')
    review = replace_once(review, "    ordered.sort_by_key(|(not_minute, position, _)| (*not_minute, *position));", '    crate::metrics::measure("sort_position", || ordered.sort_by_key(|(not_minute, position, _)| (*not_minute, *position)));')
    files["review.rs"] = review
    files.update(worker_sources())
    for name, source in files.items():
        (root / name).write_text(source, encoding="utf8")
    return files


def worker_sources():
    """Observe the current Actor; replace only its UI event sink with a counter."""
    actor = (REPO / "apps/desktop/src-tauri/src/speech_tasks.rs").read_text(encoding="utf8")
    actor = actor.replace("//!", "//")
    actor = replace_once(actor, "use tauri::Emitter;", "use crate::tauri;")
    for name in ("speech_worker", "speech_protocol", "speech_resident"):
        actor = actor.replace(f'include_str!("../{name}.py")', f'include_str!(concat!(env!("S0_SOURCE"), "/{name}.py"))')
    actor = replace_once(actor, '("speech_worker",', '("_s0", include_str!(concat!(env!("S0_SOURCE"), "/phase_log.py"))),\n                    ("speech_worker",')
    anchor = "            storage::claim_speech(&mut self.connection, now()?, 2).map_err(|e| e.to_string())?\n        {"
    actor = replace_once(actor, anchor, anchor + '\n            crate::metrics::claimed(&task.semantic_key);')
    actor = replace_once(actor, "                if header.id == 0 {", '                if header.id == 0 {\n                    if let Some(start) = self.starting { crate::metrics::record("worker_start_to_ready", 1, 0, start.elapsed().as_secs_f64()*1000.0); }')
    actor = actor.replace("storage::prepare_speech_publication(", "crate::metrics::prepare_publication(")
    actor = actor.replace("storage::complete_speech_publication(", "crate::metrics::complete_publication(")
    actor = replace_once(actor, "                            let bytes = inspector\n                                .read_asset(&running.task.semantic_key, &hash)?",
        '                            let bytes = crate::metrics::measure("publication_readback", || inspector.read_asset(&running.task.semantic_key, &hash))?')
    resident = "import _s0\n" + (REPO / "apps/desktop/src-tauri/speech_resident.py").read_text(encoding="utf8")
    resident = replace_once(resident, 'hashlib.file_digest(file, "sha256")', '_s0.measure("resource_hash", lambda: hashlib.file_digest(file, "sha256"))')
    resident = replace_once(resident, "    model = sherpa_onnx.OfflineTts(config)", '    model = _s0.measure("model_load", lambda: sherpa_onnx.OfflineTts(config))')
    resident = replace_once(resident, '    audio = model.generate("i grec" if text == "y" else text, generation)',
        '    audio = _s0.measure("inference", lambda: model.generate("i grec" if text == "y" else text, generation))')
    resident = replace_once(resident, '    return encode(Path(encoder), samples, audio.sample_rate)',
        '    return _s0.measure("encoding", lambda: encode(Path(encoder), samples, audio.sample_rate))')
    protocol = "import _s0\n" + (REPO / "apps/desktop/src-tauri/speech_protocol.py").read_text(encoding="utf8")
    protocol = replace_once(protocol, "    def execute(value):", '    def execute(value):\n        _s0.enter(value["id"])')
    protocol = replace_once(protocol, "            slots.acquire()", '            _s0.queued(value["id"])\n            _s0.measure("slot_wait", slots.acquire)')
    return {"speech_actor.rs": actor, "speech_resident.py": resident, "speech_protocol.py": protocol,
        "speech_worker.py": (REPO / "apps/desktop/src-tauri/speech_worker.py").read_text(encoding="utf8"),
        "phase_log.py": '''import json, os, threading, time
_lock = threading.Lock()
_local = threading.local()
_queued = {}
def record(stage, ms):
    with _lock:
        with open(os.environ['S0_PHASE_LOG'], 'a', encoding='utf8') as file:
            file.write(json.dumps({'id':getattr(_local,'id',0),'stage':stage,'ms':ms})+'\\n')
def measure(stage, action):
    start=time.perf_counter()
    try: return action()
    finally: record(stage, (time.perf_counter()-start)*1000)
def queued(identity):
    with _lock: _queued[identity]=time.perf_counter()
def enter(identity):
    _local.id=identity
    with _lock: start=_queued.pop(identity)
    record('worker_queue_to_execute', (time.perf_counter()-start)*1000)
'''}


def build(root):
    generated = sources(root)
    environment = os.environ.copy()
    environment["PATH"] = str(Path(sys.executable).parent) + os.pathsep + environment["PATH"]
    environment["S0_SOURCE"], environment["S0_REPO"] = str(root), str(REPO)
    command = ["cargo", "build", "--locked", "--release", "-p", "lexique-desktop", "--lib", "--message-format=json"]
    result = execute(command, root / "cargo.jsonl", env=environment, timeout=1200)
    cargo_seconds = result["ms"] / 1000
    if result["status"] != "passed":
        raise RuntimeError(f"Compilation failed: {root / 'cargo.stderr'}")
    artifacts, native_paths = {}, set()
    for line in (root / "cargo.jsonl").read_text(encoding="utf8").splitlines():
        if not line.startswith("{"):
            continue
        message = json.loads(line)
        if message.get("reason") == "build-script-executed":
            native_paths.update(message["linked_paths"])
        if message.get("reason") == "compiler-artifact" and message["profile"]["opt_level"] == "3":
            for filename in message["filenames"]:
                if filename.endswith(".rlib"):
                    artifacts[message["target"]["name"]] = filename
    binary = root / "baseline_probe.exe"
    command = ["rustc", "--edition=2024", "-C", "opt-level=3", "-L", f"dependency={REPO / 'target/release/deps'}",
        str(REPO / "docs/evidence/issue-25/baseline_probe.rs"), "-o", str(binary)]
    for name in ("lexique_core", "lexique_storage", "rusqlite", "serde", "serde_json", "cap_std", "cap_fs_ext", "cap_primitives"):
        command.extend(["--extern", f"{name}={artifacts[name]}"])
    for path in sorted(native_paths):
        command.extend(["-L", path])
    result = execute(command, root / "rustc.log", env=environment, timeout=1200)
    if result["status"] != "passed":
        raise RuntimeError(f"Probe compilation failed: {root / 'rustc.stderr'}")
    metadata = {"seed": SEED, "now": NOW, "platform": platform.platform(), "python": sys.version,
        "head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip(),
        "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(), "cargo_lock_sha256": hashlib.sha256((REPO / "Cargo.lock").read_bytes()).hexdigest(),
        "original_source_sha256": {name: hashlib.sha256((REPO / path).read_bytes()).hexdigest() for name, path in {
            "vocabulary": "apps/desktop/src-tauri/src/vocabulary.rs", "review": "crates/storage/src/review.rs", "cache": "apps/desktop/src-tauri/src/speech_cache.rs", "worker": "apps/desktop/src-tauri/speech_worker.py"}.items()},
        "generated_sha256": {name: hashlib.sha256(value.encode()).hexdigest() for name, value in generated.items()},
        "execution_timeout_s": 60, "compile_seconds": cargo_seconds + result["ms"] / 1000,
        "cargo_compile_seconds": cargo_seconds, "probe_compile_seconds": result["ms"] / 1000,
        "cache": "fresh files then OS-warm; fresh probe process per request; physical cold and app-resident cache unclaimed"}
    (root / "manifest.json").write_text(json.dumps(metadata, indent=2), encoding="utf8")


def probe(binary, root, mode="queue", **arguments):
    request = {"root": str(root), "mode": mode, "now": NOW, "timezone": "UTC", "skipped": [], **arguments}
    log = root / f"probe-{mode}-{time.time_ns()}.json"
    result = execute([str(binary)], log, request=request)
    if result["status"] != "passed":
        return result
    return {**json.loads(log.read_text(encoding="utf8")), "process_ms": result["ms"]}


def fixture(binary, root, words, scenario, missing):
    initialized = probe(binary, root, "init")
    if initialized.get("status") in ("failed", "timeout"):
        raise RuntimeError("Fixture initialization failed")
    database = root / "Lexique-data/data/lexique.sqlite3"
    with sqlite3.connect(database) as connection:
        seed_rows(connection, words, scenario)
    cache = root / "Lexique-data/temp/speech"
    cache.mkdir(parents=True)
    health = {}
    for word in range(1, words + 1):
        absent = missing == 100 or missing == 50 and word % 2 == 0
        health[word] = "missing" if absent else "healthy"
        if not absent:
            (cache / f"{word}.ogg").write_bytes(b"OggS" + b"x" * 8188)
    (root / "fixture.json").write_text(json.dumps({"words": words, "cards": words * 2, "scenario": scenario,
        "missing_percent": missing, "seed": SEED, "audio": "H: Ogg header fixture, not decodable or playback evidence", **initialized}, indent=2), encoding="utf8")
    return database, health


def queue_case(binary, root, words, scenario, missing, attempts):
    database, health = fixture(binary, root, words, scenario, missing)
    rows = []
    for iteration in range(attempts + 1):
        with sqlite3.connect(database) as connection:
            expected = oracle(connection, NOW, "UTC", health)
        row = probe(binary, root)
        if row["status"] == "passed":
            actual = row["result"]["queue"]
            row["oracle_equal"] = ([card["id"] for card in actual["cards"]] == expected["ids"]
                and actual["reason"] == expected["reason"] and actual["available_at"] == expected["available_at"])
            if not row["oracle_equal"]:
                row["status"] = "failed"
                row["expected"] = expected
        row["iteration"] = iteration
        row["oracle"] = expected
        rows.append(row)
        (root / "samples.json").write_text(json.dumps(rows, indent=2), encoding="utf8")
    report = {"words": words, "scenario": scenario, "missing_percent": missing, "first": rows[0],
        "steady": summarize(rows[1:]) if attempts >= 100 else {"attempts": attempts,
            "failures": sum(row["status"] != "passed" for row in rows[1:]), "percentiles": "not claimed"}}
    (root / "summary.json").write_text(json.dumps(report, indent=2), encoding="utf8")
    return report


def regressions(binary, root, require_contract=False):
    intraday = new_directory(root / "zero-intraday")
    database, health = fixture(binary, intraday, 1, "intraday", 0)
    with sqlite3.connect(database) as connection:
        connection.execute("UPDATE review_settings SET new_limit=0,review_limit=0")
        expected = oracle(connection, NOW, "UTC", health)
    actual = probe(binary, intraday)
    ids = [card["id"] for card in actual["result"]["queue"]["cards"]]
    assert expected["ids"] == [1, 2]
    hot = new_directory(root / "hot-write")
    fixture(binary, hot, 100, "new", 0)
    probe(binary, hot)
    refresh = probe(binary, hot)
    begin = refresh["stages"].get("sql_BEGIN_other", {}).get("calls", 0)
    lock = probe(binary, hot, "lock")
    fault_manifest = files(root / "fault-files")
    report = {"Q04": {"status": "passed" if ids == expected["ids"] else "reproduced", "owner": 26, "expected": expected["ids"], "actual": ids, "sample": actual},
        "Q10": {"status": "passed" if begin == 0 else "reproduced", "owner": 26, "hot_begin_calls": begin, "sample": refresh,
            "note": "Hot queue takes an immediate transaction and per-admitted-card position reads; existing positions need not be rewritten"},
        "shared_writer": {"status": "passed" if lock["status"] == "passed" else "reproduced", "owner": 30, "sample": lock}, "fault_files": fault_manifest,
        "future_contracts": "Window/selection/new GC tests are not executed here; owners run them before implementation"}
    report["contract_passed"] = ids == expected["ids"] and begin == 0 and lock["status"] == "passed"
    (root / "results.json").write_text(json.dumps(report, indent=2), encoding="utf8")
    if not require_contract:
        assert ids == [1] and begin > 0 and lock["status"] == "failed" and "locked" in lock["error"], \
            "Current gap changed; inspect and rebaseline rather than retaining a stale reproduction"
    return report


def differential(binary, root):
    database, health = fixture(binary, root, 40, "mixed", 0)
    log, last_rating, zone = [], None, "UTC"
    for step in operations():
        word, value = step["word"], step["value"]
        kind = step["kind"]
        outcome = None
        if kind in ("pause", "resume"):
            outcome = probe(binary, root, "pause", word=word, paused=kind == "pause")
        elif kind == "rate":
            with sqlite3.connect(database) as connection:
                expected = oracle(connection, NOW, zone, health)
                candidates = [identity for identity in expected["ids"] if identity % 2 == 1]
                if candidates:
                    identity = candidates[value % len(candidates)]
                    row = connection.execute("SELECT revision FROM review_progress WHERE card_id=?", [identity]).fetchone()
                    operation = f"s0-rate-{step['step']}"
                    outcome = probe(binary, root, "rate", card_id=identity, revision=row[0] if row else 0, operation=operation, timezone=zone)
                    if outcome["status"] == "passed":
                        last_rating = operation
        elif kind == "undo" and last_rating:
            outcome = probe(binary, root, "undo", operation=f"s0-undo-{step['step']}", rating_operation=last_rating)
            last_rating = None
        elif kind == "limits":
            with sqlite3.connect(database) as connection:
                connection.execute("UPDATE review_settings SET new_limit=?,review_limit=?", [value + 1, value + 3])
        elif kind == "timezone":
            zone = ("UTC", "Europe/Paris", "America/Vancouver")[value % 3]
        elif kind == "audio":
            audio = root / f"Lexique-data/temp/speech/{word}.ogg"
            if audio.exists():
                audio.unlink()
                health[word] = "missing"
            else:
                audio.write_bytes(b"OggS" + b"x" * 8188)
                health[word] = "healthy"
        # Each request reopens a real database; no cached/projection source can supply expected values.
        with sqlite3.connect(database) as connection:
            expected = oracle(connection, NOW, zone, health)
        sample = probe(binary, root, timezone=zone)
        actual = sample.get("result", {}).get("queue", {})
        equal = [card["id"] for card in actual.get("cards", [])] == expected["ids"]
        equal = equal and actual.get("reason") == expected["reason"] and actual.get("available_at") == expected["available_at"]
        log.append({**step, "outcome": outcome, "oracle_equal": equal, "sample": sample, "expected": expected})
        (root / "operations.json").write_text(json.dumps(log, indent=2), encoding="utf8")
        if not equal or outcome and outcome["status"] != "passed":
            raise AssertionError(f"Differential failed at fixed step {step['step']}; raw checkpoint retained")
    return {"steps": len(log), "seed": SEED, "oracle_equal": True}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument("--build-only", action="store_true")
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--words", type=int, choices=(0, 100, 1000, 10000, 50000), default=100)
    parser.add_argument("--scenario", choices=("new", "mixed", "backlog", "intraday", "future"), default="new")
    parser.add_argument("--missing", type=int, choices=(0, 50, 100), default=0)
    parser.add_argument("--attempts", type=int, default=100)
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--matrix", action="store_true", help="Sequential scale/state/missing matrix, checkpoint every case")
    parser.add_argument("--regressions", action="store_true")
    parser.add_argument("--require-contract", action="store_true", help="Explicit future-contract red command; never part of default suite")
    parser.add_argument("--operations", action="store_true")
    parser.add_argument("--worker-phases", type=Path, help="Admitted speech resources for the current Actor observer")
    args = parser.parse_args()
    if args.attempts < 1:
        raise ValueError("At least one attempt required")
    root = new_directory(args.output)
    if args.build_only:
        build(root)
        return
    if not args.binary:
        raise ValueError("Precompile with --build-only and pass --binary")
    if args.regressions:
        report = regressions(args.binary.resolve(), root, args.require_contract)
        print(json.dumps({name: report[name]["status"] for name in ("Q04", "Q10", "shared_writer")}))
        sys.exit(int(args.require_contract and not report["contract_passed"]))
    if args.worker_phases:
        fixture(args.binary.resolve(), root, 0, "new", 100)
        environment = os.environ.copy()
        environment["S0_PHASE_LOG"] = str(root / "worker-phases.jsonl")
        result = execute([str(args.binary.resolve())], root / "actor.json", env=environment,
            request={"root": str(root), "mode": "worker", "now": NOW, "timezone": "UTC",
                "resources": str(args.worker_phases.resolve())})
        if result["status"] == "passed":
            result["actor"] = json.loads((root / "actor.json").read_text(encoding="utf8"))
            result["status"] = result["actor"]["status"]
            stages = result["actor"]["stages"]
            for stage in ("publication_intent", "publication_tmp_write", "publication_tmp_sync",
                    "publication_rename", "publication_readback", "publication_db_complete", "committed_enqueue_to_claim"):
                if stages.get(stage, {}).get("calls") != 6:
                    raise AssertionError(f"Missing six actual Actor samples: {stage}")
            events = [json.loads(line) for line in (root / "worker-phases.jsonl").read_text(encoding="utf8").splitlines()]
            for stage, count in (("model_load", 1), ("resource_hash", 7), ("inference", 6), ("encoding", 6), ("worker_queue_to_execute", 6)):
                if sum(event["stage"] == stage for event in events) != count:
                    raise AssertionError(f"Missing actual resident samples: {stage}")
        print(json.dumps(result))
        if result["status"] != "passed":
            sys.exit(1)
        return
    if args.operations:
        print(json.dumps(differential(args.binary.resolve(), root)))
        return
    if args.matrix:
        reports = []
        for words in (100, 1000, 10000):
            for scenario, missing in (("new", 0), ("new", 50), ("new", 100), ("mixed", 0), ("backlog", 50), ("future", 0)):
                case = new_directory(root / f"{words}-{scenario}-{missing}")
                report = queue_case(args.binary.resolve(), case, words, scenario, missing, args.attempts)
                reports.append({"case": case.name, "first_ms": report["first"]["ms"],
                    "first_status": report["first"]["status"], "steady": report["steady"]})
                (root / "results.json").write_text(json.dumps(reports, indent=2), encoding="utf8")
                print(json.dumps(reports[-1]), flush=True)
        sys.exit(int(any(report["first_status"] != "passed" or report["steady"]["failures"] for report in reports)))
    if args.prepare_only:
        fixture(args.binary.resolve(), root, args.words, args.scenario, args.missing)
        print("pressure input prepared; execution not claimed")
    else:
        report = queue_case(args.binary.resolve(), root, args.words, args.scenario, args.missing, args.attempts)
        print(json.dumps({k: v for k, v in report.items() if k != "first"}))
        if report["steady"].get("failures", 0) or report["first"]["status"] != "passed":
            sys.exit(1)


if __name__ == "__main__":
    main()
