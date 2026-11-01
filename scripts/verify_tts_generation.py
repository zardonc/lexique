"""Verify the model-only experiment's raw evidence and optionally export it."""
import argparse
from array import array
import hashlib
import html
import json
import math
from pathlib import Path
import statistics
import sys
import tempfile

from measure_tts_generation import OWNED, WORKER, new_output, parse_config, validate_job

RUNS = (
    ("tts-pilot-20261101", 12, 3, 1),
    ("tts-screen-a-20261101", 12, 10, 1),
    ("tts-screen-b-20261101", 12, 18, 1),
    ("tts-confirm-words-20261101", 24, 7, 3),
    ("tts-latency-20261101", 12, 4, 3),
    ("tts-direct-control-20261101", 12, 4, 3),
    ("tts-sentences-pilot-20261101", 6, 4, 1),
    ("tts-confirm-sentences-20261101", 12, 4, 3),
    ("tts-stress-20261101", 48, 3, 3),
    ("tts-current-seed-20261101", 24, 4, 3),
    ("tts-steps-20261101", 18, 3, 3),
)


def require(condition, context):
    if not condition:
        raise ValueError(context)


def inside(root, relative):
    path = (root / relative).resolve()
    require(path.is_relative_to(root.resolve()) and path != root.resolve(),
            f"Evidence path escapes its directory: {relative}")
    return path


def close(actual, expected, context):
    require(math.isfinite(actual) and math.isclose(actual, expected, rel_tol=1e-6, abs_tol=1e-6), context)


def audio_file(path, size):
    audio = path.read_bytes()
    require(len(audio) == size and 0 < size <= 10_000_000 and audio.startswith(b"OggS"),
            f"Invalid Ogg size/header: {path}")
    head = audio.find(b"OpusHead")
    require(head >= 0 and len(audio) >= head + 19 and audio[head + 8:head + 10] == b"\x01\x01"
            and int.from_bytes(audio[head + 12:head + 16], "little") == 44_100,
            f"Invalid mono Opus input metadata: {path}")


def pcm_file(path, result):
    pcm = path.read_bytes()
    require(hashlib.sha256(pcm).hexdigest() == result["pcm_sha256"], f"PCM hash mismatch: {path}")
    require(len(pcm) == result["sample_count"] * 4 and 0 < len(pcm) <= 44_100 * 120 * 4,
            f"PCM sample count mismatch: {path}")
    samples = array("f")
    samples.frombytes(pcm)
    if sys.byteorder != "little":
        samples.byteswap()
    require(all(math.isfinite(value) for value in samples) and any(samples), f"Invalid PCM values: {path}")
    close(len(samples) / 44_100, result["duration_s"], f"PCM duration mismatch: {path}")
    close(max(map(abs, samples)), result["peak_amplitude"], f"PCM peak mismatch: {path}")
    close(math.sqrt(sum(value * value for value in samples) / len(samples)), result["rms"],
          f"PCM RMS mismatch: {path}")
    return samples


def verify():
    runs, outputs = [], []
    worker_hash = hashlib.sha256(WORKER.read_bytes()).hexdigest()
    for name, job_count, config_count, repeats in RUNS:
        root = inside(OWNED, name)
        summary = json.loads(inside(root, "summary.json").read_text(encoding="utf-8"))
        require(summary["worker_sha256"] == worker_hash and summary["hard_execution_timeout_s"] == 60,
                f"Worker identity/execution limit changed: {name}")
        jobs = summary["jobs"]
        require(len(jobs) == job_count and [job["id"] for job in jobs] == list(range(job_count)),
                f"Incomplete/duplicate jobs: {name}")
        for job in jobs:
            validate_job(job)
        policy = "current_random" if name == "tts-current-seed-20261101" else "fixed_per_request"
        require(summary.get("seed_policy", policy) == policy
                and all(job["seed"] == (-1 if policy == "current_random" else 42 + job["id"]) for job in jobs),
                f"Wrong seed policy: {name}")
        require(len(summary["aggregates"]) == config_count
                and len(summary["trials"]) == config_count * repeats, f"Incomplete matrix: {name}")
        configurations = []
        for aggregate in summary["aggregates"]:
            config = aggregate["config"]
            require(config == parse_config(",".join(str(config[key]) for key in ("mode", "workers", "threads", "steps", "spin"))),
                    f"Invalid configuration: {name}")
            selected = [trial for trial in summary["trials"] if trial["config"] == config]
            require(sorted(trial["repeat"] for trial in selected) == list(range(repeats)),
                    f"Incomplete repeats: {name}/{config['name']}")
            for trial in selected:
                trial_root = inside(root, trial["raw_directory"])
                require(trial_root.name == f"r{trial['repeat']}-{config['name']}", f"Wrong trial identity: {trial_root}")
                raw = json.loads(inside(trial_root, "trial.json").read_text(encoding="utf-8"))
                require({key: value for key, value in raw.items() if key != "memory_samples"} == trial,
                        f"Summary differs from raw trial: {trial_root}")
                require(len(trial["results"]) == job_count, f"Missing results: {trial_root}")
                for job, result in zip(jobs, trial["results"]):
                    require((result["id"], result["voice"], result["seed"]) == (job["id"], job["voice"], job["seed"]),
                            f"Wrong response identity: {trial_root}")
                    audio_path = inside(trial_root, result["audio_file"])
                    audio_file(audio_path, result["audio_bytes"])
                    if config["mode"] == "fresh":
                        require(result.get("generation_seed", -1) == -1, "Product baseline must keep its RNG")
                        continue
                    require(result.get("generation_seed", result["seed"]) == job["seed"], "Wrong model RNG")
                    pcm_file(inside(trial_root, Path(result["audio_file"]).with_suffix(".f32")), result)
                    outputs.append({"key": (job["text"], job["voice"], job["seed"], config["steps"]),
                                    "hash": result["pcm_sha256"], "serial": config["mode"] in ("serial", "direct"),
                                    "sample_count": result["sample_count"]})
                close(trial["jobs_per_s"], job_count * 1000 / trial["batch_ms"], f"Wrong throughput: {trial_root}")
                for key, sample_key in (("peak_private_bytes", "private_bytes"), ("peak_working_set_bytes", "working_set_bytes")):
                    require(trial[key] == max(sample[sample_key] for sample in raw["memory_samples"]),
                            f"Wrong sampled memory maximum: {trial_root}")
            close(aggregate["median_batch_ms"], statistics.median(trial["batch_ms"] for trial in selected), "Wrong batch median")
            close(aggregate["median_process_ms"], statistics.median(trial["process_total_ms"] for trial in selected), "Wrong process median")
            require(aggregate["peak_private_bytes"] == max(trial["peak_private_bytes"] for trial in selected), "Wrong memory aggregate")
            metrics = {"config": config, "batch_ms": [round(trial["batch_ms"], 3) for trial in selected],
                       "median_batch_ms": round(aggregate["median_batch_ms"], 3),
                       "median_process_total_ms": round(aggregate["median_process_ms"], 3),
                       "median_initialization_ms": round(statistics.median(trial["initialization_ms"] for trial in selected), 3),
                       "median_active_cpu_s": round(statistics.median(trial["active_cpu_s"] for trial in selected), 3),
                       "peak_private_mib": round(aggregate["peak_private_bytes"] / 1_048_576, 3),
                       "peak_working_set_mib": round(max(trial["peak_working_set_bytes"] for trial in selected) / 1_048_576, 3),
                       "other_process_cpu_percent": [round(trial["other_process_cpu_percent"], 3)
                                                     if "other_process_cpu_percent" in trial else None for trial in selected]}
            for key in ("inference_ms", "service_ms", "completion_ms"):
                values = [result[key] for trial in selected for result in trial["results"] if key in result]
                metrics["median_" + key] = round(statistics.median(values), 3) if values else None
            peaks = [result["peak_amplitude"] for trial in selected for result in trial["results"] if "peak_amplitude" in result]
            metrics["max_peak_amplitude"] = max(peaks) if peaks else None
            metrics["outputs_above_full_scale"] = sum(peak > 1 for peak in peaks) if peaks else None
            configurations.append(metrics)
        require(len({config["config"]["name"] for config in configurations}) == config_count, f"Duplicate configuration: {name}")
        runs.append({"raw_directory": root.relative_to(OWNED.parent.parent).as_posix(), "jobs": jobs,
                     "worker_sha256": worker_hash, "measured_script_sha256": summary["script_sha256"],
                     "probe_sha256": hashlib.sha256(inside(root, "probe.py").read_bytes()).hexdigest(),
                     "seed_policy": summary.get("seed_policy", "fixed_per_request"), "configurations": configurations})
    serial = {output["key"]: output["hash"] for output in outputs if output["serial"] and output["key"][2] >= 0}
    hashes, lengths, serial_count, random_count = {}, {}, 0, 0
    for output in outputs:
        key = output["key"]
        if key[2] < 0:
            random_count += 1
            continue
        require(hashes.setdefault(key, output["hash"]) == output["hash"], f"Fixed-seed PCM changed: {key}")
        require(lengths.setdefault(key[:3], output["sample_count"]) == output["sample_count"], f"Step-count duration changed: {key}")
        if key in serial:
            require(serial[key] == output["hash"], f"Concurrent PCM differs from serial reference: {key}")
            serial_count += 1
    runtime_root = inside(OWNED, "tts-runtime-check-v2-20261101")
    runtime = json.loads(inside(runtime_root, "results.json").read_text(encoding="utf-8"))
    require(runtime["worker_sha256"] == worker_hash and runtime["hard_execution_timeout_s"] == 60,
            "Runtime checks used a different worker or execution limit")
    require(runtime["script_sha256"] == hashlib.sha256(inside(runtime_root, "source.py").read_bytes()).hexdigest(),
            "Runtime check source hash mismatch")
    inputs, gpu = runtime["checks"]["inputs"], runtime["checks"]["gpu"]
    require({check["case"] for check in inputs["checks"]} == {"empty_text", "long_text", "control_character", "non_string_text",
                                                             "invalid_voice", "path_like_id", "model_checksum"}
            and len(inputs["checks"]) == 7 and all(check["passed"] is True for check in inputs["checks"])
            and inputs["no_audio_outputs"] is True and inputs["returncode"] == 0, "Runtime input checks failed")
    require(gpu["diagnostic"] == inside(runtime_root, "gpu/stderr.log").read_bytes().decode("utf-8")
            and gpu["cpu_fallback_detected"] is True and gpu["gpu_execution_verified"] is False
            and gpu["returncode"] == 0 and gpu["model_loaded"] is True and gpu["sample_rate"] == 44_100
            and gpu["speakers"] == 10
            and gpu["available_provider_diagnostics"] == ["CPUExecutionProvider, "],
            "CPU fallback evidence mismatch")
    runtime["raw_directory"] = runtime_root.relative_to(OWNED.parent.parent).as_posix()
    return {"date": "2026-11-01", "scope": "model-only; production unchanged", "hard_execution_timeout_s": 60,
            "runtime_checks": runtime,
            "verification": {"trials": sum(len(run["configurations"]) * RUNS[i][3] for i, run in enumerate(runs)),
                             "ogg_outputs": sum(RUNS[i][1] * RUNS[i][2] * RUNS[i][3] for i in range(len(RUNS))),
                             "pcm_outputs": len(outputs), "fixed_seed_pcm_outputs": len(outputs) - random_count,
                             "pcm_outputs_matching_serial_reference": serial_count, "random_seed_pcm_outputs": random_count,
                             "all_fixed_seed_repeats_equal": True, "step_variant_sample_counts_equal": True}, "runs": runs}


def listening(output):
    root = OWNED / "tts-steps-20261101"
    summary = json.loads((root / "summary.json").read_text(encoding="utf-8"))
    directory = new_output(output)
    rows, files = [], []
    for identity in (0, 1, 12, 13):
        job = summary["jobs"][identity]
        label = html.escape(f"{job['text']} · {'女声' if job['voice'] == 'female' else '男声'} · seed {job['seed']}")
        cells = []
        for steps in (8, 4, 2):
            trial = next(trial for trial in summary["trials"] if trial["repeat"] == 0 and trial["config"]["steps"] == steps)
            result = trial["results"][identity]
            trial_root = inside(root, trial["raw_directory"])
            pcm_path = inside(trial_root, Path(result["audio_file"]).with_suffix(".f32"))
            pcm_file(pcm_path, result)
            name = f"sample-{identity}-steps-{steps}.ogg"
            audio_path = inside(trial_root, result["audio_file"])
            audio = audio_path.read_bytes()
            audio_file(audio_path, result["audio_bytes"])
            (directory / name).write_bytes(audio)
            files.append({"file": name, "text": job["text"], "voice": job["voice"], "seed": job["seed"], "steps": steps,
                          "source_pcm_sha256": result["pcm_sha256"], "ogg_sha256": hashlib.sha256(audio).hexdigest(),
                          "source_peak_amplitude": result["peak_amplitude"]})
            cells.append(f'<td><audio controls preload="metadata" src="{name}" aria-label="{label}，{steps} 步"></audio><br>{result["duration_s"]:.2f} 秒</td>')
        rows.append(f'<tr><th scope="row">{label}</th>{"".join(cells)}</tr>')
    (directory / "manifest.json").write_text(json.dumps(files, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    page = '''<!doctype html><html lang="zh-CN"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>TTS 合成步数试听</title><style>body{font:16px/1.7 system-ui,sans-serif;margin:32px;color:#23231f;background:#f5f5ef}main{max-width:1250px;margin:auto}table{border-collapse:collapse;width:100%;background:white}th,td{padding:16px;border:1px solid #d7d7ce;text-align:left}audio{max-width:100%;width:250px}.scroll{overflow:auto}p{max-width:900px}</style>
<main><h1>8 / 4 / 2 步合成试听</h1><p>Supertonic 3 机器合成音频，法语女声 / 男声、正常速度。每行使用相同文本、声线和固定种子，使用实验实际生成的 48 kbit/s Ogg Opus 音频。</p>
<p>当前推荐保留 8 步。4 / 2 步仅是更快的实验候选，部分原始 PCM 峰值超过满幅，存在削波或失真风险；发音准确、音量稳定和自然度尚待人工试听确认。试听样本不代表完整词汇或所有设备的音质验收。</p>
<div class="scroll"><table><caption>按行比较同一文本的三个版本</caption><thead><tr><th scope="col">文本 / 声线 / 种子</th><th scope="col">8 步</th><th scope="col">4 步</th><th scope="col">2 步</th></tr></thead><tbody>'''
    (directory / "listen.html").write_text(page + "".join(rows) + "</tbody></table></div></main></html>\n", encoding="utf-8")


def self_check():
    for path in ("../escape", "../../escape"):
        try:
            inside(OWNED, path)
        except ValueError:
            continue
        raise AssertionError("Escaping evidence path accepted")
    try:
        close(float("nan"), 1, "Expected rejection")
    except ValueError:
        pass
    else:
        raise AssertionError("Non-finite metric accepted")
    require(inside(OWNED, "fixture/trial.json") == (OWNED / "fixture/trial.json").resolve(), "Valid path rejected")
    OWNED.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="tts-verify-", dir=OWNED) as temporary:
        root = Path(temporary).resolve()
        require(root.is_relative_to(OWNED.resolve()), "Invalid self-check fixture directory")
        path = root / "sample.f32"
        pcm = array("f", (0.25, -0.5))
        if sys.byteorder != "little":
            pcm.byteswap()
        path.write_bytes(pcm.tobytes())
        result = {"pcm_sha256": hashlib.sha256(path.read_bytes()).hexdigest(), "sample_count": 2,
                  "duration_s": 2 / 44_100, "peak_amplitude": 0.5, "rms": math.sqrt(0.15625)}
        pcm_file(path, result)
        path.write_bytes(b"corrupt")
        try:
            pcm_file(path, result)
        except ValueError as error:
            require("PCM hash mismatch" in str(error), "Wrong corrupt PCM failure")
        else:
            raise AssertionError("Corrupt PCM accepted")
        path = root / "sample.ogg"
        path.write_bytes(b"OggS" + b"invalid codec")
        try:
            audio_file(path, path.stat().st_size)
        except ValueError as error:
            require("Invalid mono Opus" in str(error), "Wrong invalid codec failure")
        else:
            raise AssertionError("Invalid Opus metadata accepted")
    print("Evidence path, non-finite metric, corrupt PCM and invalid Opus checks passed")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-check", action="store_true")
    parser.add_argument("--summary", type=Path, help="write a compact, verified result JSON to this explicit path")
    parser.add_argument("--listen", type=Path, help="create a new listening directory below target/issue-24")
    args = parser.parse_args()
    if args.self_check:
        self_check()
        return
    result = verify()
    if args.summary:
        args.summary.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    if args.listen:
        listening(args.listen)
    print(json.dumps(result["verification"], ensure_ascii=False))


if __name__ == "__main__":
    main()
