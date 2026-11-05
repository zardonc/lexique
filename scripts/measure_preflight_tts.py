"""#24 V02: isolated resident CPU model with a reserved interactive executor."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import json
from pathlib import Path
import statistics
import subprocess
import sys
import threading
import time

from measure_tts_generation import Monitor, WORDS, WORKER, finish, namespace, new_output, parse_config, provider


def trial(resources, output, background, spin, seed):
    config = parse_config(f"shared,{max(1, background + 1)},2,8,{spin}")
    monitor = Monitor()
    started = time.perf_counter()
    space = namespace()
    model = space["create_model"](str(resources / "assets"), 2, provider(config, output))
    initialization_ms = (time.perf_counter() - started) * 1000
    jobs = [{"id": i, "text": word, "voice": "female" if i % 2 == 0 else "male",
             "seed": -1 if seed == "random" else 42 + i} for i, word in enumerate(WORDS)]
    warm_started = time.perf_counter()
    finish(space, model, {"id": "warmup-0", "text": "bonjour", "voice": "female", "seed": 42}, config, resources, output)
    warmup_ms = (time.perf_counter() - warm_started) * 1000
    entered = threading.Barrier(background + 1) if background else None
    before = monitor.sample()
    batch_started = time.perf_counter()

    def execute(job, barrier=False):
        if barrier:
            entered.wait(timeout=5)
        service = time.perf_counter()
        result = finish(space, model, job, config, resources, output)
        return {**result, "service_ms": (time.perf_counter() - service) * 1000,
                "completion_ms": (time.perf_counter() - batch_started) * 1000}

    try:
        with ThreadPoolExecutor(max_workers=max(1, background)) as batch, ThreadPoolExecutor(max_workers=1) as interactive:
            pending = [batch.submit(execute, job, i < background) for i, job in enumerate(jobs[1:])] if background else []
            if entered:
                entered.wait(timeout=5)
            clicked = time.perf_counter()
            p0 = interactive.submit(execute, jobs[0]).result()
            p0["click_to_complete_ms"] = (time.perf_counter() - clicked) * 1000
            results = [p0, *([future.result() for future in pending] if background else [execute(job) for job in jobs[1:]])]
        after = monitor.sample()
        batch_ms = (time.perf_counter() - batch_started) * 1000
    finally:
        monitor.close()
    return {"background": background, "spin": spin, "seed": seed, "initialization_ms": initialization_ms,
            "warmup_ms": warmup_ms, "p0": p0, "results": results, "batch_ms": batch_ms,
            "cpu_s": after["cpu_s"] - before["cpu_s"],
            "peak_private_bytes": max(s["private_bytes"] for s in monitor.samples),
            "peak_working_set_bytes": max(s["working_set_bytes"] for s in monitor.samples),
            "samples": monitor.samples}


def main():
    if len(sys.argv) > 1 and sys.argv[1] == "--trial":
        resources, output = map(Path, sys.argv[2:4])
        print(json.dumps(trial(resources, output, int(sys.argv[4]), sys.argv[5], sys.argv[6])))
        return
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("resources", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    output = new_output(args.output)
    evidence = {"worker_sha256": hashlib.sha256(WORKER.read_bytes()).hexdigest(),
                "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                "hard_timeout_s": 60, "trials": []}
    for seed, spin in (("fixed", "on"), ("fixed", "off"), ("random", "off")):
        for repeat in range(3):
            for background in (0, 1, 2, 4):
                directory = output / f"{seed}-{spin}-{repeat}-{background}"
                directory.mkdir()
                command = [str(args.resources / "runtime/python.exe"), "-I", "-B", "-X", "utf8",
                           str(Path(__file__).resolve()), "--trial", str(args.resources.resolve()),
                           str(directory), str(background), spin, seed]
                # -I omits the script directory; import the repository helper explicitly.
                command[5:6] = ["-c", f"import sys,runpy; sys.path.insert(0,{str(Path(__file__).parent)!r}); runpy.run_path({str(Path(__file__).resolve())!r},run_name='__main__')"]
                started = time.perf_counter()
                with (directory / "stdout.json").open("wb") as stdout, (directory / "stderr.log").open("wb") as stderr:
                    child = subprocess.Popen(command, stdout=stdout, stderr=stderr, creationflags=subprocess.CREATE_NO_WINDOW)
                    try:
                        code = child.wait(timeout=60)
                    except subprocess.TimeoutExpired:
                        subprocess.run(["taskkill.exe", "/PID", str(child.pid), "/T", "/F"], check=True, capture_output=True)
                        child.wait()
                        raise RuntimeError(f"Hard 60-second timeout: {directory}") from None
                if code:
                    raise RuntimeError(f"Trial exited {code}; see {directory / 'stderr.log'}")
                result = json.loads((directory / "stdout.json").read_text(encoding="utf-8"))
                result.update(repeat=repeat, directory=directory.name, process_ms=(time.perf_counter() - started) * 1000)
                evidence["trials"].append(result)
                (output / "results.json").write_text(json.dumps(evidence, indent=2), encoding="utf-8")
                print(f"{directory.name}: P0 {result['p0']['click_to_complete_ms']:.1f} ms; batch {result['batch_ms']:.1f} ms", flush=True)
    for seed, spin in (("fixed", "on"), ("fixed", "off"), ("random", "off")):
        selected = [t for t in evidence["trials"] if (t["seed"], t["spin"]) == (seed, spin)]
        baseline = statistics.median(t["p0"]["click_to_complete_ms"] for t in selected if t["background"] == 0)
        for background in (1, 2, 4):
            trials = [t for t in selected if t["background"] == background]
            latencies = [t["p0"]["click_to_complete_ms"] for t in trials]
            accepted = statistics.median(latencies) <= baseline * 1.5 and max(latencies) <= 3000 and all(
                t["peak_private_bytes"] <= 1.5 * 1024**3 and t["peak_working_set_bytes"] <= 1024**3 for t in trials)
            print(f"accept {seed}/{spin}/{background}: {accepted}; P0 median {statistics.median(latencies):.1f} / baseline {baseline:.1f}")


if __name__ == "__main__":
    main()
