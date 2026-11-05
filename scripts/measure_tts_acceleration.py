"""Verify word-cache footprint and isolated CUDA inference; never change app data."""
import argparse
import ctypes
from ctypes import wintypes
import hashlib
import json
import os
from pathlib import Path
import random
import statistics
import subprocess
import sys
import time
import unicodedata

# The embedded Python's _pth deliberately excludes the script directory.
sys.path.insert(0, str(Path(__file__).resolve().parent))
from dictionary_sample import Dictionary
from measure_tts_generation import (OWNED, REPO, WORDS, WORKER, check_response,
                                    namespace, new_output, parse_config, trial, validate_job)
from verify_tts_generation import audio_file, pcm_file, require


class FileStandardInfo(ctypes.Structure):
    _fields_ = [("allocation", ctypes.c_longlong), ("length", ctypes.c_longlong),
                ("links", wintypes.DWORD), ("deleting", ctypes.c_ubyte), ("directory", ctypes.c_ubyte)]


def allocated_bytes(path):
    """Ask Windows for actual stream allocation; fail rather than infer clusters."""
    import msvcrt
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    query = kernel.GetFileInformationByHandleEx
    query.argtypes = [wintypes.HANDLE, ctypes.c_int, ctypes.c_void_p, wintypes.DWORD]
    query.restype = wintypes.BOOL
    info = FileStandardInfo()
    with path.open("rb") as stream:
        if not query(msvcrt.get_osfhandle(stream.fileno()), 1, ctypes.byref(info), ctypes.sizeof(info)):
            raise ctypes.WinError(ctypes.get_last_error())
        require(info.length == os.fstat(stream.fileno()).st_size and info.allocation >= 0,
                f"Invalid Windows allocation result: {path}")
    return info.allocation


def distribution(values):
    ordered = sorted(values)
    require(bool(ordered), "Empty audio-size sample")
    return {"count": len(values), "sum": sum(values), "mean": statistics.mean(values),
            "median": statistics.median(values), "min": ordered[0], "max": ordered[-1],
            "p95": ordered[max(0, (95 * len(values) + 99) // 100 - 1)]}


def cuda_options(values):
    allowed = {"cudnn_conv1d_pad_to_nc1d": {"0", "1"}, "cudnn_conv_use_max_workspace": {"0", "1"},
               "cudnn_conv_algo_search": {"DEFAULT", "HEURISTIC", "EXHAUSTIVE"}, "use_tf32": {"0", "1"}}
    options = {}
    for value in values:
        key, setting = value.split("=", 1)
        require(key in allowed and setting in allowed[key] and key not in options, "Invalid CUDA option")
        options[key] = setting
    return options


def dll_directories(values):
    paths = [Path(value).resolve() for value in values]
    require(all(path.is_relative_to(REPO / ".tooling/issue-24") and path.is_dir() for path in paths),
            "DLL directory must be in the isolated experiment")
    return [str(path) for path in paths]


def provider_config(output, requested, profiling, options):
    require(requested in ("cpu", "cuda"), "Unsupported experiment provider")
    require(requested == "cuda" or not options, "CUDA options require CUDA")
    cuda_options([f"{key}={value}" for key, value in options.items()])
    text = "SessionConfig.session.intra_op.allow_spinning=0\nSessionConfig.session.inter_op.allow_spinning=0\n"
    text += "".join(f"{key}={value}\n" for key, value in options.items())
    if profiling:
        text += f"LogSeverityLevel=0\nProfilingFilePrefix={output / 'ort-profile'}\n"
    path = output / "provider.config"
    path.write_text(text, encoding="utf-8")
    return requested + ":" + str(path)


def child(resources, output, settings, jobs):
    require(output != OWNED.resolve() and output.is_relative_to(OWNED.resolve()) and output.is_dir(),
            "Invalid experiment output boundary")
    config = parse_config(settings["config"])
    require(config["mode"] in ("serial", "shared", "direct") and config["spin"] == "off",
            "This experiment requires one model and spinning off")
    require(isinstance(jobs, list) and 1 <= len(jobs) <= 256, "Invalid request count")
    for job in jobs:
        validate_job(job)
    require(len({job["id"] for job in jobs}) == len(jobs), "Duplicate request identities")
    handles = []
    try:
        for value in dll_directories(settings["dll_dirs"]):
            handles.append(os.add_dll_directory(value))
        result = trial(resources, output, config, jobs,
                       provider_config(output, settings["provider"], settings["profile"], settings.get("cuda_options", {})))
        space = namespace()
        result["runtime"] = {key: getattr(space["sherpa_onnx"], key)
                             for key in ("version", "git_sha1", "onnxruntime_version")}
        result["requested_provider"] = settings["provider"]
        result["child_source_sha256"] = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
        return result
    finally:
        for handle in handles:
            handle.close()


def run_one(resources, output, settings, jobs):
    output.mkdir()
    env = os.environ.copy()
    (output / "temp").mkdir()
    env["TEMP"] = env["TMP"] = str(output / "temp")
    # Scope legacy LoadLibrary searches to this child, as well as add_dll_directory.
    if settings["dll_dirs"]:
        env["PATH"] = os.pathsep.join(dll_directories(settings["dll_dirs"]) + [env.get("PATH", "")])
    started = time.perf_counter()
    process = subprocess.Popen([str(resources / "runtime/python.exe"), "-I", "-B", "-X", "utf8",
                                str(Path(__file__).resolve()), "--child", str(resources), str(output),
                                json.dumps(settings)], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE, env=env, creationflags=subprocess.CREATE_NO_WINDOW)
    timed_out = False
    try:
        data, diagnostic = process.communicate(json.dumps(jobs, ensure_ascii=False).encode("utf-8"), timeout=60)
    except subprocess.TimeoutExpired:
        timed_out = True
        if process.poll() is None:
            subprocess.run(["taskkill.exe", "/PID", str(process.pid), "/T", "/F"],
                           check=True, capture_output=True, timeout=5)
        data, diagnostic = process.communicate(timeout=5)
    (output / "stdout.json").write_bytes(data)
    (output / "stderr.log").write_bytes(diagnostic)
    attempt = {"settings": settings, "jobs": jobs, "returncode": process.returncode,
               "hard_execution_timeout_s": 60, "timed_out": timed_out,
               "process_total_ms": (time.perf_counter() - started) * 1000,
               "raw_directory": output.name, "cpu_fallback_detected": "Fallback to cpu" in diagnostic.decode(errors="replace")}
    if process.returncode == 0 and not timed_out and not (attempt["cpu_fallback_detected"] and settings["provider"] == "cuda"):
        result = json.loads(data)
        require(len(result["results"]) == len(jobs), f"Incomplete results: {output}")
        for job, audio in zip(jobs, result["results"], strict=True):
            check_response(job, audio)
            audio_file(output / audio["audio_file"], audio["audio_bytes"])
            pcm_file(output / Path(audio["audio_file"]).with_suffix(".f32"), audio)
            audio["allocated_bytes"] = allocated_bytes(output / audio["audio_file"])
        attempt.update(status="passed", **result)
    else:
        attempt["status"] = "timeout" if timed_out else "failed"
    (output / "trial.json").write_text(json.dumps(attempt, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"{output.name}: {attempt['status']}" + (f", batch={attempt['batch_ms']:.1f} ms" if "batch_ms" in attempt else ""), flush=True)
    return attempt


def archive(output, resources):
    paths = [Path(__file__).resolve(), REPO / "scripts/measure_tts_generation.py", WORKER,
             REPO / "scripts/verify_tts_generation.py", REPO / "scripts/dictionary_sample.py"]
    hashes = {}
    for path in paths:
        data = path.read_bytes()
        (output / path.name).write_bytes(data)
        hashes[path.relative_to(REPO).as_posix()] = hashlib.sha256(data).hexdigest()
    return {"sources": hashes, "resources": str(resources), "hard_execution_timeout_s": 60,
            "audio_encoding": "44.1 kHz mono, 48 kbit/s Ogg Opus", "trials": []}


def save(output, evidence):
    (output / "summary.json").write_text(json.dumps(evidence, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def cache_sample(args):
    resources = args.resources.resolve()
    output = new_output(args.output)
    evidence = archive(output, resources)
    with Dictionary(args.dictionary) as dictionary:
        keys = sorted({unicodedata.normalize("NFC", row[0]) for row in
                       dictionary.connection.execute("SELECT DISTINCT key FROM entries")})
    eligible = [key for key in keys if key and len(key) <= 200 and not any(c.isspace() or ord(c) < 32 for c in key)]
    require(1 <= args.words <= min(256, len(eligible)), "Invalid dictionary sample size")
    words = random.Random(20261101).sample(eligible, args.words)
    evidence.update(dictionary=str(args.dictionary.resolve()), distinct_nfc_keys=len(keys),
                    eligible_single_word_keys=len(eligible), sampling_seed=20261101, sampled_words=words,
                    scope="public dictionary; uniform sample, not frequency weighted; proper nouns included")
    settings = {"config": "shared,12,2,8,off", "provider": "cpu", "profile": False, "dll_dirs": []}
    for voice in ("female", "male"):
        jobs = [{"id": i, "text": word, "voice": voice, "seed": 42 + i} for i, word in enumerate(words)]
        for start in range(0, len(jobs), 32):
            result = run_one(resources, output / f"{voice}-{start:03d}", settings, jobs[start:start + 32])
            evidence["trials"].append(result)
            save(output, evidence)
            require(result["status"] == "passed", f"Cache-size generation failed: {result['raw_directory']}")
    sizes = {}
    for voice in ("female", "male", "both"):
        clips = [audio for attempt in evidence["trials"] for audio in attempt["results"]
                 if voice == "both" or audio["voice"] == voice]
        sizes[voice] = {"logical_bytes": distribution([a["audio_bytes"] for a in clips]),
                        "allocated_bytes": distribution([a["allocated_bytes"] for a in clips]),
                        "duration_s": distribution([a["duration_s"] for a in clips])}
    evidence["sizes"] = sizes
    evidence["projections"] = [{"words": count, "female_logical_bytes": count * sizes["female"]["logical_bytes"]["mean"],
                                "female_allocated_bytes": count * sizes["female"]["allocated_bytes"]["mean"],
                                "male_logical_bytes": count * sizes["male"]["logical_bytes"]["mean"],
                                "male_allocated_bytes": count * sizes["male"]["allocated_bytes"]["mean"],
                                "both_logical_bytes": count * sizes["both"]["logical_bytes"]["mean"] * 2,
                                "both_allocated_bytes": count * sizes["both"]["allocated_bytes"]["mean"] * 2}
                               for count in (1000, 5000, 10000, 30000)]
    save(output, evidence)
    print(json.dumps({"sizes": sizes, "projections": evidence["projections"]}, ensure_ascii=False, indent=2))


def benchmark(args):
    require(1 <= args.repeats <= 8 and 1 <= args.cycles <= 4, "Invalid repeats/cycles")
    options = cuda_options(args.cuda_options)
    require(args.provider == "cuda" or not options, "CUDA options require CUDA")
    directories = dll_directories(args.dll_dirs)
    require(len(set(args.configs)) == len(args.configs), "Duplicate configurations")
    require(not args.cpu_control or args.provider == "cuda", "CPU comparison requires a CUDA primary")
    require(not args.compare_default or bool(options), "Default comparison requires explicit CUDA tuning")
    resources = args.resources.resolve()
    output = new_output(args.output)
    evidence = archive(output, resources)
    jobs = [{"id": i, "text": word, "voice": "female", "seed": 42 + i}
            for i, word in enumerate(WORDS * args.cycles)]
    if args.profile:
        require(args.repeats == 1 and not args.cpu_control and not args.compare_default,
                "Profiling is a separate single diagnostic run")
        jobs = jobs[:1]
    cases = []
    def add(label, stage, config, requested, dlls, tuning):
        parsed = parse_config(config)
        require(parsed["mode"] in ("serial", "shared", "direct") and parsed["spin"] == "off",
                "This experiment requires one model and spinning off")
        cases.append({"label": label, "resources": stage, "config": parsed,
                      "settings": {"config": config, "provider": requested, "profile": args.profile,
                                   "dll_dirs": dlls, "cuda_options": tuning}})
    for config in args.configs:
        add(args.provider, resources, config, args.provider, directories, options)
        if args.compare_default:
            add("cuda-default", resources, config, "cuda", directories, {})
    if args.cpu_control:
        cpu = args.cpu_control.resolve()
        require((cpu / "runtime/python.exe").is_file(), "Missing CPU control runtime")
        for config in ("serial,1,4,8,off", "shared,12,2,8,off"):
            add("cpu-original", cpu, config, "cpu", [], {})
        add("cpu-gpu-build", resources, "shared,12,2,8,off", "cpu", directories, {})
    rng = random.Random(20261101)
    for repeat in range(args.repeats):
        order = list(cases)
        rng.shuffle(order)
        for case in order:
            result = run_one(case["resources"], output / f"r{repeat}-{case['label']}-{case['config']['name']}",
                             case["settings"], jobs)
            result.update(repeat=repeat, label=case["label"], resources=str(case["resources"]))
            evidence["trials"].append(result)
            save(output, evidence)
    evidence["aggregates"] = []
    for case in cases:
        runs = [attempt for attempt in evidence["trials"]
                if attempt["settings"] == case["settings"] and attempt["label"] == case["label"]]
        passed = [attempt for attempt in runs if attempt["status"] == "passed"]
        aggregate = {"label": case["label"], "settings": case["settings"],
                     "passed": len(passed), "failed": len(runs) - len(passed)}
        if passed:
            aggregate.update(median_batch_ms=statistics.median(a["batch_ms"] for a in passed),
                             median_process_ms=statistics.median(a["process_total_ms"] for a in passed),
                             median_inference_ms=statistics.median(audio["inference_ms"] for a in passed for audio in a["results"]),
                             peak_private_bytes=max(a["peak_private_bytes"] for a in passed),
                             peak_working_set_bytes=max(a["peak_working_set_bytes"] for a in passed))
        evidence["aggregates"].append(aggregate)
    save(output, evidence)
    print(json.dumps(evidence["aggregates"], indent=2))
    require(all(a["status"] == "passed" for a in evidence["trials"]), f"Some trials failed; see {output}")


def main():
    if sys.argv[1:] == ["--self-check"]:
        require(distribution([4, 1, 2, 3])["median"] == 2.5, "Statistics check failed")
        require(distribution([1] * 19 + [100])["p95"] == 1, "Percentile check failed")
        require(ctypes.sizeof(FileStandardInfo) == 24, "Windows structure layout changed")
        require(cuda_options(["cudnn_conv1d_pad_to_nc1d=1"]) == {"cudnn_conv1d_pad_to_nc1d": "1"}, "CUDA option check failed")
        for value in ("unknown=1", "use_tf32=2", "use_tf32=1\nLogSeverityLevel=0"):
            try:
                cuda_options([value])
            except ValueError:
                pass
            else:
                raise AssertionError("Unsafe CUDA option accepted")
        print("statistics and Windows structure checks passed")
        return
    require(os.name == "nt", "Windows experiment only")
    if sys.argv[1:2] == ["--child"] and len(sys.argv) == 5:
        print(json.dumps(child(Path(sys.argv[2]).resolve(), Path(sys.argv[3]).resolve(),
                               json.loads(sys.argv[4]), json.load(sys.stdin)), ensure_ascii=False))
        return
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    cache = sub.add_parser("cache")
    bench = sub.add_parser("bench")
    for command in (cache, bench):
        command.add_argument("resources", type=Path)
        command.add_argument("output", type=Path)
    cache.add_argument("--dictionary", type=Path, default=REPO / "target/desktop-dictionary-sample/dictionary.sqlite3")
    cache.add_argument("--words", type=int, default=128)
    bench.add_argument("--provider", choices=("cpu", "cuda"), required=True)
    bench.add_argument("--configs", nargs="+", required=True)
    bench.add_argument("--repeats", type=int, default=3)
    bench.add_argument("--cycles", type=int, default=2)
    bench.add_argument("--dll-dirs", nargs="*", type=Path, default=[])
    bench.add_argument("--profile", action="store_true")
    bench.add_argument("--cuda-options", nargs="*", default=[])
    bench.add_argument("--cpu-control", type=Path)
    bench.add_argument("--compare-default", action="store_true")
    args = parser.parse_args()
    for relative in ("runtime/python.exe", "assets/voice.bin", "opusenc.dll"):
        require((args.resources / relative).is_file(), f"Missing runtime file: {relative}")
    (cache_sample if args.command == "cache" else benchmark)(args)


if __name__ == "__main__":
    main()
