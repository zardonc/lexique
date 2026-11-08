"""Measure the actual standalone package and distinct acquisition artifacts."""

import argparse
import hashlib
import json
from pathlib import Path
import zipfile


def main():
    root = Path(__file__).resolve().parent
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("downloads", type=Path)
    parser.add_argument("package", type=Path)
    parser.add_argument("report", type=Path, help="report from this packaged runtime's trial")
    parser.add_argument("archive", type=Path, help="must not exist")
    args = parser.parse_args()
    package = args.package
    resources = json.loads((root / "resources.json").read_text(encoding="utf-8"))
    trial = json.loads(args.report.read_text(encoding="utf-8"))
    trial_date = trial["date"]
    hardware = {"cpu": trial["cpu"], "logical_processors": trial["logical_cpus"],
                "os": trial["platform"], "python": trial["python"]}
    quality = json.loads((root / "samples-tom-v2/quality.json").read_text(encoding="utf-8"))
    inventory = json.loads((package / "inventory.json").read_text(encoding="utf-8"))
    for name, item in inventory.items():
        path = package / name
        if path.stat().st_size != item["bytes"] or hashlib.sha256(path.read_bytes()).hexdigest() != item["sha256"]:
            raise ValueError(f"changed packaged file: {name}")
    archive_path = args.archive
    measurement_path = archive_path.with_suffix(".measurements.json")
    for path in (archive_path, measurement_path):
        if path.exists():
            raise FileExistsError(f"Preserving measured package: {path}")
    paths = sorted(path for path in package.rglob("*") if path.is_file())
    if {path.relative_to(package).as_posix() for path in paths} != set(inventory) | {"inventory.json"}:
        raise ValueError("package file inventory mismatch")
    with zipfile.ZipFile(archive_path, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for path in paths:
            archive.write(path, path.relative_to(package).as_posix())
    with zipfile.ZipFile(archive_path) as archive:
        if archive.testzip() is not None:
            raise ValueError("ZIP CRC validation failed")
        if set(archive.namelist()) != {path.relative_to(package).as_posix() for path in paths}:
            raise ValueError("ZIP file inventory mismatch")
        for name, item in inventory.items():
            if hashlib.sha256(archive.read(name)).hexdigest() != item["sha256"]:
                raise ValueError(f"ZIP member checksum mismatch: {name}")
    raw_archives = []
    for item in resources["downloads"] + resources["preparation_downloads"]:
        raw = item["bytes"]
        if item["file"].endswith((".whl", ".zip")):
            with zipfile.ZipFile(args.downloads / item["file"]) as archive:
                raw = sum(member.file_size for member in archive.infolist())
        raw_archives.append({"file": item["file"], "download_bytes": item["bytes"], "raw_bytes": raw})
    measurements = {
        "date": trial_date, "target": "Windows x64", "hardware": hardware,
        "methods": {
            "hardware": "Trial process: platform.processor, os.cpu_count and platform.platform; exact CPU model, physical cores and RAM recorded separately in README using Get-CimInstance",
            "times": "Python perf_counter; total_load includes Python package imports, frontend init and model init, excludes process startup and asset checksum preflight; first PCM is from input frontend start; generation and synchronous playback timed separately",
            "memory": "Windows GetProcessMemoryInfo PeakWorkingSetSize for the full process",
            "raw": "Sum ZIP/wheel uncompressed member sizes plus original ONNX/config/model card and the unopened VC installer; preserves upstream duplicate members",
            "download": "Sum distinct pinned artifact file bytes; no estimate of HTTP/TLS traffic",
            "installed": "Sum actual package file logical sizes, including evidence, inventory and the retained VC prerequisite installer; excludes filesystem allocation slack",
            "compressed": "Actual whole-package ZIP_DEFLATED level 9, CRC, exact member set and SHA-256 against every inventory entry",
        },
        "runtime_acquisition_bytes": sum(item["bytes"] for item in resources["downloads"] + resources["source_files"]),
        "preparation_additional_download_bytes": sum(item["bytes"] for item in resources["preparation_downloads"]),
        "complete_from_source_download_bytes": sum(item["bytes"] for item in resources["downloads"] + resources["source_files"] + resources["preparation_downloads"]),
        "raw_runtime_acquisition_bytes": sum(item["raw_bytes"] for item in raw_archives[:len(resources["downloads"])]) + sum(item["bytes"] for item in resources["source_files"]),
        "raw_complete_from_source_bytes": sum(item["raw_bytes"] for item in raw_archives) + sum(item["bytes"] for item in resources["source_files"]),
        "package": {"path": archive_path.as_posix(), "files": len(paths),
                    "installed_bytes": sum(path.stat().st_size for path in paths), "zip_bytes": archive_path.stat().st_size,
                    "sha256": hashlib.sha256(archive_path.read_bytes()).hexdigest(), "inventory_verified": True},
        "installed_components": {name: sum(path.stat().st_size for path in (package / name).rglob("*") if path.is_file())
                                 for name in ("runtime", "assets", "prerequisites", "docs", "scripts")},
        "archives": raw_archives,
        "performance": {
            "raw_report": args.report.as_posix(),
            "measured_package": package.as_posix(),
            "total_load_seconds": trial["total_load_seconds"],
            "model_load_seconds": trial["model_load_seconds"],
            "first_input_to_pcm_seconds": trial["cases"][0]["first_audio_seconds"],
            "generation_seconds": sum(case["generation_seconds"] for case in trial["cases"]),
            "audio_seconds": sum(case["audio_seconds"] for case in trial["cases"]),
            "peak_process_memory_bytes": trial["peak_process_memory_bytes"],
        },
        "quality": {"record": "samples-tom-v2/quality.json", "verdict": quality["verdict"],
                    "feedback": quality["feedback"]},
        "limitations": ["OS caches were not cleared; these are fresh processes, not certified cold starts",
                        "The listening verdict applies to the saved replacement samples; package-run audio was not separately reviewed",
                        "VC installer was retained without execution; clean-machine dependencies are untested",
                        "Sandbox harness and existing host tools are recorded separately, not shipped as speech resources",
                        "Other-platform execution and full binary redistribution verification remain untested"],
    }
    measurement_path.write_text(json.dumps(measurements, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"package": measurements["package"], "raw_runtime_acquisition_bytes": measurements["raw_runtime_acquisition_bytes"],
                      "raw_complete_from_source_bytes": measurements["raw_complete_from_source_bytes"],
                      "measurements": measurement_path.as_posix()}))


if __name__ == "__main__":
    main()
