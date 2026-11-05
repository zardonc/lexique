"""Independently check the fixed cache/CUDA experiment and export compact evidence."""
import argparse
from array import array
from collections import Counter
import hashlib
import json
import math
from pathlib import Path
import statistics

from measure_tts_acceleration import allocated_bytes, distribution
from measure_tts_generation import OWNED, REPO, WORKER, check_response, parse_config
from verify_tts_generation import audio_file, close, inside, pcm_file, require

RUNS = (("tts-cache-space-20261101", 8, 32),
        ("tts-cuda-diagnostic-20261101", 1, 1),
        ("tts-cuda-screen-20261101", 5, 12),
        ("tts-cuda-pad-pilot-20261101", 3, 24),
        ("tts-cuda-confirm-20261101", 21, 24))


def warmup(path):
    """Warmup has no timing record; validate its raw samples independently."""
    pcm = path.with_suffix(".f32").read_bytes()
    require(0 < len(pcm) <= 44_100 * 120 * 4 and len(pcm) % 4 == 0, "Invalid warmup PCM size")
    samples = array("f")
    samples.frombytes(pcm)
    require(all(math.isfinite(x) for x in samples) and any(samples), "Invalid warmup samples")
    audio_file(path, path.stat().st_size)


def profiles(directory):
    files = sorted(directory.glob("ort-profile*.json"))
    require(len(files) == 4, f"Expected four distinct model profiles: {directory}")
    records = []
    components = ("duration_predictor", "text_encoder", "vector_estimator", "vocoder")
    for component, path in zip(components, files, strict=True):
        events = json.loads(path.read_text(encoding="utf-8"))
        nodes = [event for event in events if event.get("cat") == "Node" and event.get("args", {}).get("provider")]
        counts = Counter(event["args"]["provider"] for event in nodes)
        require(counts["CUDAExecutionProvider"] > 0, f"Model did not execute CUDA nodes: {path}")
        operations = Counter(event["args"]["provider"] + " " + event["args"]["op_name"] for event in nodes)
        durations = Counter()
        for event in nodes:
            durations[event["args"]["provider"]] += event["dur"]
        records.append({"component": component, "file": path.name, "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                        "node_event_counts": dict(counts), "host_node_event_duration_us": dict(durations),
                        "operations": dict(operations)})
    return records


def verify_run(name, count, jobs_per_trial):
    root = inside(OWNED, name)
    summary = json.loads((root / "summary.json").read_text(encoding="utf-8"))
    require(summary["hard_execution_timeout_s"] == 60 and len(summary["trials"]) == count,
            f"Wrong timeout/trial count: {name}")
    for source, expected in summary["sources"].items():
        archive = inside(root, Path(source).name)
        require(hashlib.sha256(archive.read_bytes()).hexdigest() == expected, f"Archived source mismatch: {archive}")
    require(summary["sources"][WORKER.relative_to(REPO).as_posix()] == hashlib.sha256(WORKER.read_bytes()).hexdigest(),
            f"Product worker changed: {name}")
    measured, warmed = 0, 0
    compact = []
    for attempt in summary["trials"]:
        raw = inside(root, attempt["raw_directory"])
        record = json.loads((raw / "trial.json").read_text(encoding="utf-8"))
        require(all(attempt.get(key) == value for key, value in record.items()), f"Raw/summary trial mismatch: {raw}")
        require(attempt["status"] == "passed" and attempt["returncode"] == 0 and not attempt["timed_out"]
                and not attempt["cpu_fallback_detected"], f"Unsuccessful/CPU-fallback trial: {raw}")
        require(len(attempt["jobs"]) == len(attempt["results"]) == jobs_per_trial, f"Incomplete trial: {raw}")
        require(attempt["config"] == parse_config(attempt["settings"]["config"]), f"Config mismatch: {raw}")
        if name == "tts-cuda-confirm-20261101":
            require(attempt["child_source_sha256"] == summary["sources"]["scripts/measure_tts_acceleration.py"],
                    f"Child did not execute archived source: {raw}")
        files = set()
        for job, result in zip(attempt["jobs"], attempt["results"], strict=True):
            check_response(job, result)
            path = inside(raw, result["audio_file"])
            files.add(path.name)
            audio_file(path, result["audio_bytes"])
            pcm_file(path.with_suffix(".f32"), result)
            require(allocated_bytes(path) == result["allocated_bytes"], f"Windows allocation changed: {path}")
            require(result["generation_seed"] == job["seed"], f"Seed mismatch: {path}")
            measured += 1
        expected_warmups = {f"warmup-{i}.ogg" for i in range(attempt["config"]["workers"])}
        require({p.name for p in raw.glob("*.ogg")} == files | expected_warmups, f"Unexpected/missing Ogg files: {raw}")
        for filename in expected_warmups:
            warmup(raw / filename)
            warmed += 1
        require({p.stem for p in raw.glob("*.f32")} == {Path(p).stem for p in files | expected_warmups},
                f"Unexpected/missing PCM files: {raw}")
        item = {key: value for key, value in attempt.items() if key not in ("memory_samples", "jobs", "results")}
        for key in ("batch_ms", "process_total_ms", "active_cpu_s", "peak_private_bytes", "peak_working_set_bytes"):
            require(math.isfinite(attempt[key]) and attempt[key] > 0, f"Invalid measurement {key}: {raw}")
        require(attempt["runtime"]["version"] == "2.0.1" and attempt["runtime"]["onnxruntime_version"] == "1.28.2",
                f"Runtime version drift: {raw}")
        item["median_inference_ms"] = statistics.median(r["inference_ms"] for r in attempt["results"])
        item["over_full_scale_outputs"] = [r["id"] for r in attempt["results"] if r["peak_amplitude"] > 1]
        compact.append(item)
    return root, summary, {"run": name, "measured_outputs": measured, "warmup_outputs": warmed,
                           "trials": compact, "aggregates": summary.get("aggregates", [])}


def cache_evidence(summary):
    words = summary["sampled_words"]
    require(len(words) == len(set(words)) == 128, "Invalid word sample")
    for voice in ("female", "male", "both"):
        clips = [result for attempt in summary["trials"] for result in attempt["results"]
                 if voice == "both" or result["voice"] == voice]
        for field, metric in (("audio_bytes", "logical_bytes"), ("allocated_bytes", "allocated_bytes"), ("duration_s", "duration_s")):
            require(distribution([clip[field] for clip in clips]) == summary["sizes"][voice][metric], "Cache statistics mismatch")
    for row in summary["projections"]:
        for voice in ("female", "male", "both"):
            for field in ("logical_bytes", "allocated_bytes"):
                close(row[f"{voice}_{field}"], row["words"] * summary["sizes"][voice][field]["mean"] * (2 if voice == "both" else 1),
                      "Cache projection mismatch")
    return {key: value for key, value in summary.items() if key not in ("trials", "resources")}


def compare_outputs(summary):
    references = {}
    for attempt in summary["trials"]:
        key = (attempt["label"], attempt["settings"]["config"])
        if attempt["repeat"] == 0:
            references[key] = attempt
    groups = []
    for key, reference in references.items():
        expected = {result["id"]: result for result in reference["results"]}
        runs = [attempt for attempt in summary["trials"] if (attempt["label"], attempt["settings"]["config"]) == key]
        require(len(runs) == 3, "Confirmation group does not contain three repeats")
        groups.append({"label": key[0], "config": key[1], "repeat_output_count": sum(len(a["results"]) for a in runs),
                       "repeat_pcm_hash_matches": sum(r["pcm_sha256"] == expected[r["id"]]["pcm_sha256"] for a in runs for r in a["results"])})
    cpu = references[("cpu-original", "serial,1,4,8,off")]
    cpu_results = {result["id"]: result for result in cpu["results"]}
    comparisons = []
    for key, attempt in references.items():
        count_deltas, rms_deltas, exact = [], [], 0
        for result in attempt["results"]:
            reference = cpu_results[result["id"]]
            count_deltas.append(result["sample_count"] - reference["sample_count"])
            rms_deltas.append(result["rms"] / reference["rms"] - 1)
            exact += result["pcm_sha256"] == reference["pcm_sha256"]
        comparisons.append({"label": key[0], "config": key[1], "cpu_pcm_exact_matches": exact,
                            "sample_count_delta_range": [min(count_deltas), max(count_deltas)],
                            "relative_rms_delta_range": [min(rms_deltas), max(rms_deltas)]})
    parallel = []
    for label in ("cuda", "cuda-default", "cpu-original"):
        serial = next(attempt for (name, config), attempt in references.items() if name == label and config.startswith("serial,"))
        expected = {result["id"]: result["pcm_sha256"] for result in serial["results"]}
        shared = references[(label, "shared,12,2,8,off")]
        matches = sum(result["pcm_sha256"] == expected[result["id"]] for result in shared["results"])
        require(matches == 24, f"Shared-model outputs differ from serial: {label}")
        parallel.append({"label": label, "shared_pcm_outputs_matching_serial": matches})
    return {"repeat_consistency": groups, "cpu_comparisons": comparisons, "parallel_consistency": parallel,
            "human_pronunciation_and_quality_acceptance": False}


def dependencies():
    root = REPO / ".tooling/issue-24/gpu-cuda13-20261101"
    download = json.loads((root / "download.json").read_text(encoding="utf-8"))
    digest = "206f56c80cca702ca564fb21e48462f4210a13f941de32c336a4c6debb1e9efb"
    archive = root / "archive.tar.bz2"
    require(download["publisher_sha256"] == digest and hashlib.sha256(archive.read_bytes()).hexdigest() == digest
            and archive.stat().st_size == download["publisher_bytes"], "GPU archive digest/size mismatch")
    nvidia = json.loads((root / "nvidia-runtime.json").read_text(encoding="utf-8"))
    for package in nvidia["packages"]:
        wheel = inside(root / "wheels", package["filename"])
        require(hashlib.sha256(wheel.read_bytes()).hexdigest() == package["digests"]["sha256"]
                and wheel.stat().st_size == package["size"], f"NVIDIA wheel mismatch: {wheel}")
    for entry in nvidia["files"]:
        path = inside(root / "nvidia-runtime", entry["path"])
        require(path.stat().st_size == entry["bytes"] and hashlib.sha256(path.read_bytes()).hexdigest() == entry["sha256"],
                f"NVIDIA extracted file mismatch: {path}")
    replacements = json.loads((root / "native-replacements.json").read_text(encoding="utf-8"))
    for entry in replacements:
        path = inside(root / "stage/runtime/Lib/site-packages/sherpa_onnx/lib", entry["file"])
        require(path.stat().st_size == entry["bytes"] and hashlib.sha256(path.read_bytes()).hexdigest() == entry["sha256"],
                f"GPU native replacement mismatch: {path}")
    native = sum(entry["bytes"] for entry in replacements if "tensorrt" not in entry["file"])
    original = REPO / ".tooling/issue-15-opus/speech-stage/runtime/Lib/site-packages/sherpa_onnx/lib"
    cpu = sum(path.stat().st_size for path in original.glob("*.dll"))
    return {"sherpa_archive": download, "nvidia_packages": nvidia["packages"], "native_replacements": replacements,
            "nvidia_wheel_bytes": nvidia["wheel_bytes"], "nvidia_extracted_bytes": nvidia["extracted_bytes"],
            "nvidia_dll_bytes": nvidia["dll_bytes"], "cuda_native_dll_bytes_excluding_tensorrt": native,
            "cpu_native_dll_bytes": cpu, "experimental_dll_net_increase_bytes": native + nvidia["dll_bytes"] - cpu,
            "is_minimized_product_distribution": False}


def fallback_rejection():
    root = inside(OWNED, "tts-cuda-unavailable-20261101")
    summary = json.loads((root / "summary.json").read_text(encoding="utf-8"))
    require(len(summary["trials"]) == 1, "Missing CPU-only CUDA rejection trial")
    attempt = summary["trials"][0]
    raw = inside(root, attempt["raw_directory"])
    require(attempt["status"] == "failed" and attempt["cpu_fallback_detected"] and not attempt["timed_out"]
            and attempt["returncode"] == 0, "CPU fallback was not rejected")
    diagnostic = (raw / "stderr.log").read_text(encoding="utf-8", errors="replace")
    require("Fallback to cpu" in diagnostic, "Missing explicit CPU fallback diagnostic")
    result = json.loads((raw / "stdout.json").read_text(encoding="utf-8"))
    audio = result["results"][0]
    audio_file(inside(raw, audio["audio_file"]), audio["audio_bytes"])
    pcm_file(inside(raw, Path(audio["audio_file"]).with_suffix(".f32")), audio)
    return {"cpu_only_runtime_cuda_request_rejected": True, "valid_audio_is_not_gpu_execution_proof": True,
            "expected_parent_cli_exit_code": 1, "native_child_exit_code": 0, "run": root.name}


def verify():
    runs, data, roots = [], {}, {}
    for name, count, jobs in RUNS:
        root, summary, record = verify_run(name, count, jobs)
        runs.append(record)
        data[name], roots[name] = summary, root
    diagnostic = data["tts-cuda-diagnostic-20261101"]["trials"][0]
    placement = profiles(inside(roots["tts-cuda-diagnostic-20261101"], diagnostic["raw_directory"]))
    confirmation = data["tts-cuda-confirm-20261101"]
    for aggregate in confirmation["aggregates"]:
        selected = [trial for trial in confirmation["trials"] if trial["label"] == aggregate["label"]
                    and trial["settings"] == aggregate["settings"]]
        require(len(selected) == aggregate["passed"] == 3 and aggregate["failed"] == 0, "Invalid aggregate group")
        close(aggregate["median_batch_ms"], statistics.median(t["batch_ms"] for t in selected), "Batch aggregate mismatch")
        close(aggregate["median_process_ms"], statistics.median(t["process_total_ms"] for t in selected), "Process aggregate mismatch")
    return {"date": "2026-11-01", "scope": "standalone Windows x64 experiments; product/cache policy unchanged",
            "verified_measured_outputs": sum(run["measured_outputs"] for run in runs),
            "verified_warmup_outputs": sum(run["warmup_outputs"] for run in runs),
            "cache": cache_evidence(data["tts-cache-space-20261101"]), "cuda_placement": placement,
            "output_consistency": compare_outputs(confirmation), "dependencies": dependencies(),
            "failure_checks": fallback_rejection(), "runs": runs}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--export", type=Path)
    args = parser.parse_args()
    evidence = verify()
    if args.export:
        destination = args.export.resolve()
        require(destination.is_relative_to(REPO / "docs/evidence/tts-generation") or destination.is_relative_to(OWNED),
                "Export must stay in the experiment evidence directories")
        with destination.open("x", encoding="utf-8") as stream:
            stream.write(json.dumps(evidence, ensure_ascii=False, indent=2) + "\n")
    print(json.dumps({"verified_measured_outputs": evidence["verified_measured_outputs"],
                      "verified_warmup_outputs": evidence["verified_warmup_outputs"],
                      "output_consistency": evidence["output_consistency"]}, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
