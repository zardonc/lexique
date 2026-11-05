"""#24: build/run an isolated queue prototype. No desktop or learning-data access."""
import argparse
import difflib
import hashlib
import json
import os
from pathlib import Path
import platform
import statistics
import subprocess
import time

REPO = Path(__file__).resolve().parents[1]


class MeasurementTimeout(RuntimeError):
    pass


def candidate_source(source):
    replacements = [
        ("review_queue_filtered(connection, now, timezone, |_| Ok(true))",
         "review_queue_filtered(connection, now, timezone, |_| true, |_| Ok(true))"),
        ("    mut available: impl FnMut(&QueueCard) -> Result<bool, StorageError>,",
         "    mut selected: impl FnMut(&QueueCard) -> bool,\n    mut available: impl FnMut(&QueueCard) -> Result<bool, StorageError>,"),
        ("    for card in all {\n        if !available(&card)? {",
         "    for card in all {\n        if !selected(&card) {"),
        ("        } else {\n            pending.push((card, progress.card));",
         """        } else {
            let blocked_by_quota = match queue_class(state, &day, timezone)? {
                QueueClass::New => remaining_new == 0 || (!settings.new_cards_ignore_review_limit && remaining_review == 0),
                QueueClass::Interday | QueueClass::Review => remaining_review == 0,
                QueueClass::Intraday => false,
            };
            if blocked_by_quota {
                let next_day = lexique_core::scheduling::next_learning_day_at(now, timezone).map_err(StorageError::Invalid)?;
                if reason.is_none() || (reason.as_deref() != Some("clock") && Some(next_day) < available_at) {
                    reason = Some("quota".into());
                    available_at = Some(next_day);
                }
                continue;
            }
            if !available(&card)? { continue; }
            pending.push((card, progress.card));"""),
    ]
    for old, new in replacements:
        if source.count(old) != 1:
            raise ValueError("Queue source changed; review the prototype against current source")
        source = source.replace(old, new)
    return source


def pinned_cache_source(source):
    old = 'fn read_inner(database: &Path, id: i64, touch: bool) -> Result<Option<Vec<u8>>, String> {\n    let dir = directory(database)?;'
    new = '''pub struct Inspector(Dir);
impl Inspector {
    pub fn new(database: &Path) -> Result<Self, String> { Ok(Self(directory(database)?)) }
    pub fn inspect(&self, id: i64) -> Result<bool, String> { Ok(read_from(&self.0, id, false)?.is_some()) }
}
fn read_inner(database: &Path, id: i64, touch: bool) -> Result<Option<Vec<u8>>, String> {
    read_from(&directory(database)?, id, touch)
}
fn read_from(dir: &Dir, id: i64, touch: bool) -> Result<Option<Vec<u8>>, String> {'''
    if source.count(old) != 1:
        raise ValueError("Cache source changed; review the pinned directory prototype")
    return source.replace(old, new)


def run(command, output, *, env=None, timeout=60):
    with output.open("wb") as log:
        process = subprocess.Popen(command, cwd=REPO, env=env, stdout=log, stderr=subprocess.STDOUT,
                                   creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0)
        try:
            code = process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            if os.name == "nt":
                subprocess.run(["taskkill.exe", "/PID", str(process.pid), "/T", "/F"], check=True,
                               capture_output=True)
            else:
                process.kill()
            process.wait()
            raise MeasurementTimeout(f"Measurement exceeded {timeout}s: {output}") from None
    if code:
        raise RuntimeError(f"Command exited {code}: {output}")


def summarize(root):
    cases = []
    for path in sorted(root.glob("queue-*.jsonl")):
        samples = [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines()]
        _, count, scenario, cache, size = path.stem.split("-")
        for mode in sorted({row["mode"] for row in samples}):
            selected = [row for row in samples if row["mode"] == mode]
            warm = [row["total_ms"] for row in selected if row["iteration"] > 0]
            for row in selected:
                queue = row.pop("queue")
                row["cards"] = len(queue["cards"])
                row["reason"] = queue["reason"]
                row["available_at"] = queue["available_at"]
                row["queue_sha256"] = hashlib.sha256(json.dumps(queue, sort_keys=True).encode()).hexdigest()
            cases.append({"count":int(count),"scenario":scenario,"cache":cache,"audio_size":int(size),"mode":mode,
                          "first_ms":selected[0]["total_ms"],"repeat_median_ms":statistics.median(warm),
                          "repeat_max_ms":max(warm),"samples":selected})
    batches = [row for path in sorted(root.glob("batch-*.jsonl")) for line in path.read_text().splitlines()
               if "event" not in (row := json.loads(line))]
    summary = {"environment":json.loads((root / "environment.json").read_text()), "queue":cases,"batch":batches}
    (root / "summary.json").write_text(json.dumps(summary, indent=2), encoding="utf-8")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument("--build-only", action="store_true")
    parser.add_argument("--unmodified", action="store_true", help="negative self-check: queue probe assertion must fail")
    parser.add_argument("--pin", action="store_true", help="compare request-scoped pinned directory, with identical queue semantics")
    parser.add_argument("--batch-only", action="store_true")
    args = parser.parse_args()
    root = args.output.resolve()
    owned = (REPO / "target" / "issue-24").resolve()
    if not root.is_relative_to(owned) or root == owned:
        raise ValueError("Output must be a new child of target/issue-24")
    root.mkdir(parents=True, exist_ok=False)
    source = (REPO / "crates/storage/src/review.rs").read_text(encoding="utf-8")
    alternative = candidate_source(source)
    if args.unmodified:
        # Keep the extra selection seam so the unchanged ordering compiles.
        alternative = alternative.replace("        if !selected(&card) {", "        if !selected(&card) || !available(&card)? {")
    (root / "review.rs").write_text(alternative, encoding="utf-8")
    (root / "candidate.patch").write_text("".join(difflib.unified_diff(source.splitlines(True), alternative.splitlines(True), fromfile="baseline/review.rs", tofile="prototype/review.rs")), encoding="utf-8")
    cache_source_text = (REPO / "apps/desktop/src-tauri/src/speech_cache.rs").read_text(encoding="utf-8")
    pinned = pinned_cache_source(cache_source_text)
    (root / "speech_cache.rs").write_text(pinned, encoding="utf-8")
    (root / "pinned-cache.patch").write_text("".join(difflib.unified_diff(cache_source_text.splitlines(True), pinned.splitlines(True), fromfile="baseline/speech_cache.rs", tofile="prototype/speech_cache.rs")), encoding="utf-8")
    env = os.environ.copy()
    env["LEXIQUE_PERF_CANDIDATE_DIR"] = str(root)
    run(["cargo", "build", "--locked", "--release", "-p", "lexique-desktop", "--lib", "--message-format=json"], root / "build.log", env=env, timeout=1200)
    artifacts = {}
    native_paths = set()
    for line in (root / "build.log").read_text(encoding="utf-8").splitlines():
        if not line.startswith("{"):
            continue
        message = json.loads(line)
        if message.get("reason") == "build-script-executed":
            native_paths.update(message["linked_paths"])
        if message.get("reason") == "compiler-artifact" and message["profile"]["opt_level"] == "3":
            for filename in message["filenames"]:
                if filename.endswith(".rlib"):
                    artifacts[message["target"]["name"]] = filename
    # Compile the evidence harness using Cargo's exact existing artifacts, without
    # adding a workspace target, manifest, dependency, or changing the lockfile.
    harness = (REPO / "docs/evidence/issue-24/review_performance.rs").read_text(encoding="utf-8")
    cache_source = (REPO / "apps/desktop/src-tauri/src/speech_cache.rs").as_posix()
    harness = harness.replace('"../../../apps/desktop/src-tauri/src/speech_cache.rs"', f'"{cache_source}"')
    (root / "harness.rs").write_text(harness, encoding="utf-8")
    binary = root / "review_performance.exe"
    command = ["rustc", "--edition=2024", "-C", "opt-level=3", "-L", f"dependency={REPO / 'target/release/deps'}", str(root / "harness.rs"), "-o", str(binary)]
    for name in ("lexique_core", "lexique_storage", "rusqlite", "serde", "serde_json", "cap_std", "cap_fs_ext", "cap_primitives"):
        command += ["--extern", f"{name}={artifacts[name]}"]
    for path in sorted(native_paths):
        command += ["-L", path]
    run(command, root / "harness-build.log", env=env, timeout=1200)
    metadata = {"utc":time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()), "platform":platform.platform(),
                "source_sha256":hashlib.sha256(source.encode()).hexdigest(), "candidate_sha256":hashlib.sha256(alternative.encode()).hexdigest(),
                "cache_source_sha256":hashlib.sha256(cache_source_text.encode()).hexdigest(), "pinned_cache_sha256":hashlib.sha256(pinned.encode()).hexdigest(),
                "baseline_commit":subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=REPO, text=True).strip(),
                "cache_note":"Fixtures just written; first pass is not cold disk. Later passes are OS-warm. Logical payload only; no OS cache eviction."}
    (root / "environment.json").write_text(json.dumps(metadata, indent=2), encoding="utf-8")
    if args.build_only:
        return
    if args.unmodified:
        run([str(binary), str(root / "negative"), "10", "future", "valid", "8192"], root / "negative.log")
        raise AssertionError("Unmodified prototype unexpectedly passed")
    cases = [(n, scenario, cache, size) for n in (100, 1000, 10000)
             for scenario, cache, size in (("due", "valid", 8192), ("sparse", "valid", 8192),
                                           ("future", "valid", 8192), ("quota", "valid", 8192),
                                           ("due", "missing", 8192), ("due", "mixed", 8192))]
    cases += [(1000, "paused", "valid", 8192), (1000, "skipped", "valid", 8192),
              (1000, "sparse", "valid", 65536)]
    if args.pin:
        cases = [(n, "due", "valid", 8192) for n in (100, 1000, 10000)]
        cases += [(1000, "due", "mixed", 8192), (1000, "due", "missing", 8192), (1000, "sparse", "valid", 65536)]
    if args.batch_only:
        cases = []
    for count, scenario, cache, size in cases:
        name = f"queue-{count}-{scenario}-{cache}-{size}"
        with (root / f"{name}.jsonl").open("wb") as combined:
            for iteration in range(3 if args.pin else 6):
                output = root / f"{name}-{iteration}.log"
                run([str(binary), str(root / name), str(count), scenario, cache, str(size), str(iteration), "pin" if args.pin else "selection"], output)
                combined.write(output.read_bytes())
                combined.flush()
        print(name, flush=True)
    for count in (() if args.pin else (100, 1000, 10000)):
        for chunk in (1, 100, 500):
            for repeat in range(3):
                name = f"batch-{count}-{chunk}-{repeat}"
                output = root / f"{name}.jsonl"
                try:
                    run([str(binary), str(root / name), str(count), "batch", str(chunk)], output)
                except MeasurementTimeout:
                    with output.open("a", encoding="utf-8") as log:
                        log.write(json.dumps({"count":count,"chunk":chunk,"execution_timeout_s":60,"total_ms":None}) + "\n")
                    print(f"{name}: timed out; remaining repeats skipped", flush=True)
                    break
                print(name, flush=True)
    summarize(root)


if __name__ == "__main__":
    main()
