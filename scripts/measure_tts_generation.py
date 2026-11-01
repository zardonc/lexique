"""Measure the admitted TTS runtime in isolated Windows processes; no app data."""
import argparse
import ast
from concurrent.futures import ThreadPoolExecutor
import ctypes
from ctypes import wintypes
import hashlib
import json
import os
from pathlib import Path
import queue
import random
import re
import statistics
import subprocess
import sys
import threading
import time

REPO = Path(__file__).resolve().parents[1]
WORKER = REPO / "apps/desktop/src-tauri/speech_worker.py"
OWNED = REPO / "target/issue-24"
WORDS = ("école", "bonjour", "œil", "vacances", "aujourd’hui", "fille",
         "rouge", "beaucoup", "heureusement", "anticonstitutionnellement", "rendez-vous", "y")
SENTENCES = ("Bonjour, comment allez-vous ?", "J’apprends le français tous les jours.",
             "Aujourd’hui, il fait beau et nous allons à l’école.",
             "Les enfants ont trouvé un petit écureuil près de la rivière.",
             "Pourriez-vous m’expliquer la différence entre ces deux expressions ?",
             "À vingt-trois heures, le dernier train pour Lyon quitte la gare.")


def replace_once(text, old, new):
    if text.count(old) != 1:
        raise ValueError(f"Worker changed; review the experiment anchor: {old}")
    return text.replace(old, new)


def probe_source(source):
    """Reuse the product's resource, input, sample and encoder checks verbatim."""
    main = next(n for n in ast.parse(source).body if isinstance(n, ast.FunctionDef) and n.name == "main")
    lines = source.splitlines(True)
    header = "".join(lines[:main.lineno - 1])
    body = "".join(line[4:] for line in lines[main.body[0].lineno - 1:main.body[-1].end_lineno])
    validation, rest = body[body.index("if not isinstance(request, dict)"):].split("paths = {}", 1)
    setup, generation = ("paths = {}" + rest).split("generation = sherpa_onnx.GenerationConfig()", 1)
    setup = setup.replace("sys.argv[1]", "assets")
    setup = replace_once(setup, 'num_threads=2, provider="cpu"', 'num_threads=threads, provider=provider')
    generation = "generation = sherpa_onnx.GenerationConfig()" + generation
    generation = replace_once(generation, ", 1.0, 8", ", 1.0, steps")
    generation = replace_once(generation, 'generation.extra = {"lang": "fr"}',
                              'generation.extra = {"lang": "fr"} if seed < 0 else {"lang": "fr", "seed": str(seed)}')
    generation = replace_once(generation, 'audio = tts.generate(',
                              'inference_started = time.perf_counter()\naudio = tts.generate(')
    generation = replace_once(generation, "if audio.sample_rate != 44_100",
                              "inference_ms = (time.perf_counter() - inference_started) * 1000\nif audio.sample_rate != 44_100")
    generation = replace_once(generation, 'sys.stdout.buffer.write(encode(Path(sys.argv[2]), samples, audio.sample_rate))',
                              'encode_started = time.perf_counter()\nencoded = encode(Path(encoder), samples, audio.sample_rate)\nencode_ms = (time.perf_counter() - encode_started) * 1000')
    indent = lambda block: "\n".join("    " + line for line in block.splitlines())
    result = header + "\nimport time\n\ndef create_model(assets, threads, provider):\n" + indent(setup) + "\n    return tts\n"
    result += "\ndef generate(tts, text, voice, steps, seed, encoder):\n    started = time.perf_counter()\n    request = {\"text\": text, \"voice\": voice}\n" + indent(validation) + "\n" + indent(generation)
    result += "\n    return {\"encoded\": encoded, \"pcm\": samples.tobytes(), \"sample_count\": samples.size, \"inference_ms\": inference_ms, \"encode_ms\": encode_ms, \"generation_ms\": (time.perf_counter() - started) * 1000, \"duration_s\": samples.size / audio.sample_rate, \"peak_amplitude\": float(np.max(np.abs(samples))), \"rms\": float(np.sqrt(np.mean(samples * samples)))}\n"
    compile(result, "tts-generation-probe", "exec")
    return result


def parse_config(value):
    mode, workers, threads, steps, spin = value.split(",")
    workers, threads, steps = int(workers), int(threads), int(steps)
    if mode not in ("direct", "serial", "shared", "process", "fresh") or not 1 <= workers <= 16 or not 1 <= threads <= 16 or not 1 <= steps <= 16 or spin not in ("on", "off", "bounded"):
        raise ValueError(f"Invalid experiment configuration: {value}")
    if mode == "process" and workers > 8:
        raise ValueError("Independent model processes are bounded to eight")
    if mode in ("direct", "serial", "fresh") and workers != 1:
        raise ValueError("Serial experiments have one worker")
    if mode == "fresh" and (threads, steps, spin) != (2, 8, "on"):
        raise ValueError("Fresh baseline must use the unchanged product worker")
    return {"name": value.replace(",", "-"), "mode": mode, "workers": workers,
            "threads": threads, "steps": steps, "spin": spin}


def new_output(path):
    path = path.resolve()
    owned = OWNED.resolve()
    if path == owned or not path.is_relative_to(owned):
        raise ValueError("Output must be a new child of target/issue-24")
    path.mkdir(parents=True, exist_ok=False)
    return path


class Memory(ctypes.Structure):
    _fields_ = [("cb", wintypes.DWORD), ("faults", wintypes.DWORD)] + [
        (name, ctypes.c_size_t) for name in ("peak_ws", "ws", "peak_paged", "paged",
                                           "peak_nonpaged", "nonpaged", "pagefile", "peak_pagefile", "private")]


class Monitor:
    """Sample simultaneous working sets and private commit of owned processes."""
    def __init__(self):
        self.kernel = ctypes.WinDLL("kernel32", use_last_error=True)
        self.psapi = ctypes.WinDLL("psapi", use_last_error=True)
        self.kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
        self.kernel.OpenProcess.restype = wintypes.HANDLE
        self.kernel.CloseHandle.argtypes = [wintypes.HANDLE]
        self.kernel.GetProcessTimes.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
        self.kernel.GetSystemTimes.argtypes = [ctypes.POINTER(wintypes.FILETIME)] * 3
        self.psapi.GetProcessMemoryInfo.argtypes = [wintypes.HANDLE, ctypes.POINTER(Memory), wintypes.DWORD]
        self.handles, self.samples, self.error = {}, [], None
        self.guard, self.stop = threading.Lock(), threading.Event()
        self.started = time.perf_counter()
        self.add(os.getpid())
        self.thread = threading.Thread(target=self.loop, daemon=True)
        self.thread.start()

    def add(self, pid):
        handle = self.kernel.OpenProcess(0x410, False, pid)
        if not handle:
            raise ctypes.WinError(ctypes.get_last_error())
        with self.guard:
            self.handles[pid] = handle

    def sample(self):
        ws, private, cpu = 0, 0, 0
        with self.guard:
            for handle in self.handles.values():
                info = Memory()
                info.cb = ctypes.sizeof(info)
                if self.psapi.GetProcessMemoryInfo(handle, ctypes.byref(info), info.cb):
                    ws += info.ws
                    private += info.private
                created, ended, kernel, user = (wintypes.FILETIME() for _ in range(4))
                if not self.kernel.GetProcessTimes(handle, ctypes.byref(created), ctypes.byref(ended), ctypes.byref(kernel), ctypes.byref(user)):
                    raise ctypes.WinError(ctypes.get_last_error())
                cpu += ((kernel.dwHighDateTime << 32) + kernel.dwLowDateTime +
                        (user.dwHighDateTime << 32) + user.dwLowDateTime) / 10_000_000
            idle, system_kernel, system_user = (wintypes.FILETIME() for _ in range(3))
            if not self.kernel.GetSystemTimes(ctypes.byref(idle), ctypes.byref(system_kernel), ctypes.byref(system_user)):
                raise ctypes.WinError(ctypes.get_last_error())
            seconds = lambda value: ((value.dwHighDateTime << 32) + value.dwLowDateTime) / 10_000_000
            total = seconds(system_kernel) + seconds(system_user)
            result = {"elapsed_ms": (time.perf_counter() - self.started) * 1000,
                      "working_set_bytes": ws, "private_bytes": private, "cpu_s": cpu,
                      "system_total_cpu_s": total, "system_busy_cpu_s": total - seconds(idle)}
            self.samples.append(result)
            return result

    def loop(self):
        while not self.stop.wait(0.05):
            try:
                self.sample()
            except Exception as error:
                self.error = error
                return

    def close(self):
        self.stop.set()
        self.thread.join()
        self.sample()
        with self.guard:
            for handle in self.handles.values():
                self.kernel.CloseHandle(handle)
        if self.error:
            raise self.error


def namespace():
    space = {"__name__": "tts_generation_probe"}
    exec(probe_source(WORKER.read_text(encoding="utf-8")), space)
    return space


def provider(config, output):
    if config["spin"] == "on":
        return "cpu"
    settings = "SessionConfig.session.intra_op.allow_spinning=0\nSessionConfig.session.inter_op.allow_spinning=0\n"
    if config["spin"] == "bounded":
        settings = "".join(f"SessionConfig.session.{kind}.{key}={value}\n" for kind in ("intra_op", "inter_op") for key, value in (("allow_spinning", 1), ("spin_duration_us", 1000), ("spin_backoff_max", 8)))
    path = output / "cpu.config"
    path.write_text(settings, encoding="utf-8")
    return "cpu:" + str(path)


def finish(space, model, job, config, resources, output):
    validate_job(job)
    result = space["generate"](model, job["text"], job["voice"], config["steps"], job["seed"], str(resources / "opusenc.dll"))
    audio, pcm = result.pop("encoded"), result.pop("pcm")
    name = f"{job['id']}.ogg"
    (output / name).write_bytes(audio)
    (output / f"{job['id']}.f32").write_bytes(pcm)
    return {"id": job["id"], "voice": job["voice"], "seed": job["seed"], "generation_seed": job["seed"], "audio_file": name,
            "audio_bytes": len(audio), "pcm_sha256": hashlib.sha256(pcm).hexdigest(), **result}


def model_worker(resources, output, config):
    started = time.perf_counter()
    space = namespace()
    model = space["create_model"](str(resources / "assets"), config["threads"], provider(config, output))
    print(json.dumps({"ready": True, "initialization_ms": (time.perf_counter() - started) * 1000}), flush=True)
    for line in sys.stdin:
        job = json.loads(line)
        print(json.dumps(finish(space, model, job, config, resources, output)), flush=True)


def check_response(job, result):
    if (result.get("id"), result.get("voice"), result.get("seed")) != (job["id"], job["voice"], job["seed"]):
        raise ValueError("TTS response identity mismatch")
    if not 0 < result.get("audio_bytes", 0) <= 10_000_000:
        raise ValueError("Invalid TTS output size")


def validate_job(job):
    if not isinstance(job, dict) or set(job) != {"id", "text", "voice", "seed"}:
        raise ValueError("Invalid experiment request")
    identity = job["id"]
    if not (type(identity) is int and 0 <= identity <= 1000) and not (isinstance(identity, str) and re.fullmatch(r"warmup-(?:[0-9]|1[0-5])", identity)):
        raise ValueError("Invalid experiment request identity")
    if type(job["seed"]) is not int or not -1 <= job["seed"] <= 2_147_483_647:
        raise ValueError("Invalid experiment seed")


def trial(resources, output, config, jobs):
    monitor = Monitor()
    children, streams, pool = [], [], None
    initialized = time.perf_counter()
    try:
        if config["mode"] == "process":
            for index in range(config["workers"]):
                child_dir = output / f"model-{index}"
                child_dir.mkdir()
                diagnostic = (child_dir / "stderr.log").open("w", encoding="utf-8")
                streams.append(diagnostic)
                child = subprocess.Popen([sys.executable, "-I", "-B", "-X", "utf8", str(Path(__file__).resolve()), "--model", str(resources), str(child_dir), json.dumps(config)],
                                         stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=diagnostic,
                                         text=True, encoding="utf-8", creationflags=subprocess.CREATE_NO_WINDOW)
                children.append(child)
                monitor.add(child.pid)
            for child in children:
                if not json.loads(child.stdout.readline()).get("ready"):
                    raise ValueError("TTS worker failed initialization")
            available = queue.Queue()
            for index, child in enumerate(children):
                available.put((index, child))
            def execute(job):
                validate_job(job)
                index, child = available.get()
                try:
                    child.stdin.write(json.dumps(job, ensure_ascii=False) + "\n")
                    child.stdin.flush()
                    result = json.loads(child.stdout.readline())
                    result["audio_file"] = f"model-{index}/" + result["audio_file"]
                    return result
                finally:
                    available.put((index, child))
        elif config["mode"] == "fresh":
            source = WORKER.read_text(encoding="utf-8")
            def execute(job):
                started = time.perf_counter()
                child = subprocess.Popen([sys.executable, "-I", "-B", "-X", "utf8", "-c", source, str(resources / "assets"), str(resources / "opusenc.dll")],
                                         stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                         creationflags=subprocess.CREATE_NO_WINDOW)
                monitor.add(child.pid)
                audio, diagnostic = child.communicate(json.dumps({"text": job["text"], "voice": job["voice"]}).encode("utf-8"))
                if child.returncode or not audio.startswith(b"OggS") or len(audio) > 10_000_000:
                    raise ValueError(f"Product worker failed: {diagnostic.decode(errors='replace')[:2048]}")
                (output / f"{job['id']}.ogg").write_bytes(audio)
                return {"id": job["id"], "voice": job["voice"], "seed": job["seed"], "generation_seed": -1, "audio_file": f"{job['id']}.ogg", "audio_bytes": len(audio), "generation_ms": (time.perf_counter() - started) * 1000}
        else:
            space = namespace()
            model = space["create_model"](str(resources / "assets"), config["threads"], provider(config, output))
            def execute(job):
                return finish(space, model, job, config, resources, output)
        initialization_ms = (time.perf_counter() - initialized) * 1000
        memory_ready = monitor.sample()
        if config["mode"] != "direct":
            pool = ThreadPoolExecutor(max_workers=config["workers"])
        if config["mode"] != "fresh":
            warmups = [{"id": f"warmup-{i}", "text": "bonjour", "voice": "female", "seed": -1 if jobs[0]["seed"] < 0 else 42} for i in range(config["workers"])]
            list(pool.map(execute, warmups) if pool else map(execute, warmups))
        before = monitor.sample()
        batch_started = time.perf_counter()
        def measured(job):
            service_started = time.perf_counter()
            result = execute(job)
            check_response(job, result)
            result["service_ms"] = (time.perf_counter() - service_started) * 1000
            result["completion_ms"] = (time.perf_counter() - batch_started) * 1000
            return result
        results = list(pool.map(measured, jobs) if pool else map(measured, jobs))
        batch_ms = (time.perf_counter() - batch_started) * 1000
        after = monitor.sample()
        active_cpu_s = after["cpu_s"] - before["cpu_s"]
        total_cpu_s = after["system_total_cpu_s"] - before["system_total_cpu_s"]
        busy_cpu_s = after["system_busy_cpu_s"] - before["system_busy_cpu_s"]
    finally:
        try:
            if pool:
                pool.shutdown(wait=True)
            exit_errors = []
            for child in children:
                try:
                    child.stdin.close()
                except BrokenPipeError:
                    pass
            for child in children:
                try:
                    code = child.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    if child.poll() is None:
                        subprocess.run(["taskkill.exe", "/PID", str(child.pid), "/T", "/F"], check=True, capture_output=True)
                    child.wait(timeout=5)
                    exit_errors.append("TTS worker failed to exit")
                    continue
                if code != 0:
                    exit_errors.append(f"TTS worker {child.pid} exited with failure; see model stderr.log")
            if exit_errors:
                raise RuntimeError("; ".join(exit_errors))
        finally:
            for stream in streams:
                stream.close()
            monitor.close()
    return {"config": config, "initialization_ms": initialization_ms, "memory_ready": memory_ready,
            "batch_ms": batch_ms, "jobs_per_s": len(jobs) * 1000 / batch_ms, "active_cpu_s": active_cpu_s,
            "system_cpu_percent": busy_cpu_s * 100 / total_cpu_s,
            "other_process_cpu_percent": max(0, busy_cpu_s - active_cpu_s) * 100 / total_cpu_s,
            "peak_working_set_bytes": max(s["working_set_bytes"] for s in monitor.samples),
            "peak_private_bytes": max(s["private_bytes"] for s in monitor.samples), "results": results,
            "memory_samples": monitor.samples}


def self_check():
    probe_source(WORKER.read_text(encoding="utf-8"))
    for value in ("serial,2,2,8,on", "shared,17,2,8,on", "process,9,1,8,off", "serial,1,0,8,on", "fresh,1,1,8,on", "serial,1,2,8,cuda"):
        try:
            parse_config(value)
        except ValueError:
            continue
        raise AssertionError(f"Invalid configuration accepted: {value}")
    try:
        new_output(REPO)
    except ValueError:
        pass
    else:
        raise AssertionError("Output outside the experiment boundary was accepted")
    try:
        check_response({"id": 1, "voice": "female", "seed": 42}, {"id": 2, "voice": "female", "seed": 42, "audio_bytes": 100})
    except ValueError:
        pass
    else:
        raise AssertionError("Wrong response identity was accepted")
    try:
        validate_job({"id": "../../outside", "text": "bonjour", "voice": "female", "seed": 42})
    except ValueError:
        pass
    else:
        raise AssertionError("Path-like request identity was accepted")
    print("Probe compilation, invalid configuration, output boundary and response identity checks passed")


def runtime_probe(resources, output, kind):
    space = namespace()
    if kind == "gpu":
        model = space["create_model"](str(resources / "assets"), 2, "cuda")
        return {"requested_provider": "cuda", "model_loaded": True,
                "speakers": model.num_speakers, "sample_rate": model.sample_rate,
                "runtime": {key: getattr(space["sherpa_onnx"], key)
                            for key in ("version", "git_sha1", "onnxruntime_version")}}
    if kind != "inputs":
        raise ValueError("Unknown runtime check")
    checks = []
    def rejects(label, expected, call):
        try:
            call()
        except ValueError as error:
            if str(error) != expected:
                raise AssertionError(f"{label}: unexpected error: {error}") from error
            checks.append({"case": label, "error": str(error), "passed": True})
            return
        raise AssertionError(f"{label}: invalid input accepted")
    config = parse_config("shared,12,2,8,off")
    for label, text, voice, expected in (
        ("empty_text", "", "female", "Invalid headword"),
        ("long_text", "a" * 201, "female", "Invalid headword"),
        ("control_character", "bon\njour", "female", "Invalid headword"),
        ("non_string_text", 123, "female", "Invalid headword"),
        ("invalid_voice", "bonjour", "unknown", "Invalid voice"),
    ):
        job = {"id": 0, "text": text, "voice": voice, "seed": 42}
        rejects(label, expected, lambda: finish(space, None, job, config, resources, output))
    job = {"id": "../../escape", "text": "bonjour", "voice": "female", "seed": 42}
    rejects("path_like_id", "Invalid experiment request identity",
            lambda: finish(space, None, job, config, resources, output))
    fixture = output / "bad-assets"
    fixture.mkdir()
    first_name = next(iter(space["ASSETS"].values()))[0]
    (fixture / first_name).write_bytes(b"intentionally invalid experiment model fixture")
    rejects("model_checksum", f"Model checksum mismatch: {first_name}",
            lambda: space["create_model"](str(fixture), 2, "cpu"))
    if list(output.glob("*.ogg")) or list(output.glob("*.f32")):
        raise AssertionError("Rejected requests left audio files")
    return {"checks": checks, "no_audio_outputs": True}


def runtime_checks(resources, destination):
    if os.name != "nt" or not (resources / "runtime/python.exe").is_file():
        raise ValueError("Windows and the admitted runtime are required")
    output = new_output(destination)
    evidence = {"hard_execution_timeout_s": 60, "worker_sha256": hashlib.sha256(WORKER.read_bytes()).hexdigest(),
                "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(), "checks": {}}
    (output / "source.py").write_bytes(Path(__file__).read_bytes())
    for kind in ("inputs", "gpu"):
        child_dir = output / kind
        child_dir.mkdir()
        process = subprocess.Popen([str(resources / "runtime/python.exe"), "-I", "-B", "-X", "utf8",
                                    str(Path(__file__).resolve()), "--runtime-probe", str(resources), str(child_dir), kind],
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, creationflags=subprocess.CREATE_NO_WINDOW)
        try:
            data, diagnostic = process.communicate(timeout=60)
        except subprocess.TimeoutExpired:
            if process.poll() is None:
                subprocess.run(["taskkill.exe", "/PID", str(process.pid), "/T", "/F"], check=True, capture_output=True)
            process.communicate(timeout=5)
            raise RuntimeError(f"Runtime check exceeded hard 60s: {child_dir}") from None
        (child_dir / "stdout.json").write_bytes(data)
        (child_dir / "stderr.log").write_bytes(diagnostic)
        if process.returncode:
            raise RuntimeError(f"Runtime check failed: {diagnostic.decode(errors='replace')[:4096]}")
        result = json.loads(data)
        result["returncode"] = process.returncode
        if kind == "gpu":
            message = diagnostic.decode("utf-8", errors="replace")
            result.update(diagnostic=message,
                          available_provider_diagnostics=re.findall(r"Available\s+providers:\s*(.*?)\.\s*Fallback to cpu", message),
                          cpu_fallback_detected="Fallback to cpu" in message, gpu_execution_verified=False)
        evidence["checks"][kind] = result
    (output / "results.json").write_text(json.dumps(evidence, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(evidence, ensure_ascii=False))


def main():
    if sys.argv[1:] == ["--self-check"]:
        self_check()
        return
    if sys.argv[1:2] == ["--runtime-checks"] and len(sys.argv) == 4:
        runtime_checks(Path(sys.argv[2]).resolve(), Path(sys.argv[3]))
        return
    if sys.argv[1:2] == ["--runtime-probe"] and len(sys.argv) == 5:
        resources, output = Path(sys.argv[2]).resolve(), Path(sys.argv[3]).resolve()
        if output == OWNED.resolve() or not output.is_relative_to(OWNED.resolve()) or not output.is_dir():
            raise ValueError("Invalid runtime check output boundary")
        print(json.dumps(runtime_probe(resources, output, sys.argv[4]), ensure_ascii=False))
        return
    if sys.argv[1:2] in (["--trial"], ["--model"]):
        resources, output = Path(sys.argv[2]).resolve(), Path(sys.argv[3]).resolve()
        if output == OWNED.resolve() or not output.is_relative_to(OWNED.resolve()) or not output.is_dir():
            raise ValueError("Invalid private experiment output boundary")
        config = json.loads(sys.argv[4])
        if config != parse_config(",".join(str(config[key]) for key in ("mode", "workers", "threads", "steps", "spin"))):
            raise ValueError("Invalid private experiment configuration")
        if sys.argv[1] == "--model":
            model_worker(resources, output, config)
        else:
            jobs = json.load(sys.stdin)
            if not isinstance(jobs, list) or not 1 <= len(jobs) <= 256:
                raise ValueError("Invalid experiment request count")
            for job in jobs:
                validate_job(job)
            if len({job["id"] for job in jobs}) != len(jobs):
                raise ValueError("Duplicate experiment request identity")
            print(json.dumps(trial(resources, output, config, jobs), ensure_ascii=False))
        return
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("resources", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--configs", nargs="+", required=True, help="mode,workers,threads,steps,on|off|bounded (CPU only)")
    parser.add_argument("--corpus", choices=("words", "sentences", "mixed"), default="words")
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--cycles", type=int, default=1)
    parser.add_argument("--both-voices", action="store_true")
    parser.add_argument("--random-seed", action="store_true", help="use the current product RNG path, seed=-1")
    args = parser.parse_args()
    if os.name != "nt" or not 1 <= args.repeats <= 8 or not 1 <= args.cycles <= 8:
        raise ValueError("Windows only; bounded repeats and cycles required")
    configs = [parse_config(value) for value in args.configs]
    resources = args.resources.resolve()
    for path in (resources / "runtime/python.exe", resources / "assets/voice.bin", resources / "opusenc.dll"):
        if not path.is_file():
            raise ValueError(f"Missing admitted runtime input: {path}")
    output = new_output(args.output)
    (output / "temp").mkdir()
    env = os.environ.copy()
    env["TEMP"] = env["TMP"] = str(output / "temp")
    texts = WORDS if args.corpus == "words" else SENTENCES if args.corpus == "sentences" else WORDS + SENTENCES
    jobs = [{"id": i, "text": text, "voice": "male" if args.both_voices and i % 2 else "female", "seed": -1 if args.random_seed else 42 + i} for i, text in enumerate(texts * args.cycles)]
    evidence = {"worker_sha256": hashlib.sha256(WORKER.read_bytes()).hexdigest(),
                "script_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
                "hard_execution_timeout_s": 60, "corpus": args.corpus,
                "seed_policy": "current_random" if args.random_seed else "fixed_per_request", "jobs": jobs, "trials": []}
    (output / "probe.py").write_text(probe_source(WORKER.read_text(encoding="utf-8")), encoding="utf-8")
    rng = random.Random(20261101)
    for repeat in range(args.repeats):
        order = list(configs)
        rng.shuffle(order)
        for config in order:
            trial_dir = output / f"r{repeat}-{config['name']}"
            trial_dir.mkdir()
            started = time.perf_counter()
            process = subprocess.Popen([str(resources / "runtime/python.exe"), "-I", "-B", "-X", "utf8", str(Path(__file__).resolve()), "--trial", str(resources), str(trial_dir), json.dumps(config)],
                                       stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                       env=env, creationflags=subprocess.CREATE_NO_WINDOW)
            try:
                data, diagnostic = process.communicate(json.dumps(jobs, ensure_ascii=False).encode("utf-8"), timeout=60)
            except subprocess.TimeoutExpired:
                if process.poll() is None:
                    subprocess.run(["taskkill.exe", "/PID", str(process.pid), "/T", "/F"], check=True, capture_output=True)
                process.communicate(timeout=5)
                raise RuntimeError(f"TTS experiment exceeded hard 60s: {trial_dir}") from None
            (trial_dir / "stderr.log").write_bytes(diagnostic)
            if process.returncode:
                raise RuntimeError(f"TTS experiment failed: {diagnostic.decode(errors='replace')[:4096]}")
            result = json.loads(data)
            result.update(repeat=repeat, process_total_ms=(time.perf_counter() - started) * 1000,
                          raw_directory=trial_dir.relative_to(output).as_posix())
            (trial_dir / "trial.json").write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding="utf-8")
            del result["memory_samples"]
            evidence["trials"].append(result)
            (output / "summary.json").write_text(json.dumps(evidence, ensure_ascii=False, indent=2), encoding="utf-8")
            print(f"r{repeat} {config['name']}: batch={result['batch_ms']:.1f} ms, {result['jobs_per_s']:.2f} jobs/s, peak private={result['peak_private_bytes']/1048576:.1f} MiB", flush=True)
    evidence["aggregates"] = [{"config": config,
        "median_batch_ms": statistics.median(t["batch_ms"] for t in evidence["trials"] if t["config"] == config),
        "median_process_ms": statistics.median(t["process_total_ms"] for t in evidence["trials"] if t["config"] == config),
        "median_inference_ms": statistics.median(r["inference_ms"] for t in evidence["trials"] if t["config"] == config for r in t["results"]) if config["mode"] != "fresh" else None,
        "peak_private_bytes": max(t["peak_private_bytes"] for t in evidence["trials"] if t["config"] == config)} for config in configs]
    (output / "summary.json").write_text(json.dumps(evidence, ensure_ascii=False, indent=2), encoding="utf-8")


if __name__ == "__main__":
    main()
