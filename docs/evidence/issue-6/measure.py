"""Verify and measure the fixed Kokoro comparison package and its fresh audio."""

import argparse
import hashlib
import json
from pathlib import Path
import tarfile
import wave
import zipfile

import numpy as np


def main():
    root = Path(__file__).resolve().parent
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("materials", type=Path)
    parser.add_argument("package", type=Path)
    parser.add_argument("trial", type=Path)
    parser.add_argument("archive", type=Path, help="must not exist")
    args = parser.parse_args()
    measurement = args.archive.with_suffix(".measurements.json")
    for path in (args.archive, measurement, args.package / "inventory.json"):
        if path.exists():
            raise FileExistsError(f"Preserving existing evidence: {path}")

    resources = json.loads((root.parent / "issue-14/resources.json").read_text(encoding="utf-8"))
    roles = {}
    for item in resources:
        path = args.materials / item["file"]
        with path.open("rb") as source:
            digest = hashlib.file_digest(source, "sha256").hexdigest()
        if path.stat().st_size != item["bytes"] or digest != item["sha256"]:
            raise ValueError(f"changed acquisition artifact: {path.name}")
        if path.suffix in (".zip", ".whl"):
            with zipfile.ZipFile(path) as archive:
                expanded = sum(info.file_size for info in archive.infolist() if not info.is_dir())
        elif path.name.endswith(".tar.gz"):
            with tarfile.open(path) as archive:
                expanded = sum(info.size for info in archive if info.isfile())
        else:
            expanded = path.stat().st_size
        role = roles.setdefault(item["role"], {"download_bytes": 0, "expanded_bytes": 0})
        role["download_bytes"] += item["bytes"]
        role["expanded_bytes"] += expanded

    with zipfile.ZipFile(args.materials / "ff_siwis.pt") as archive:
        tensor = archive.read(next(name for name in archive.namelist() if name.endswith("/data/0")))
    with np.load(args.package / "assets/voices-v1.0.bin", allow_pickle=False) as voices:
        voice = voices["ff_siwis"]
        assert voice.dtype == np.dtype("<f4") and voice.shape == (510, 1, 256)
        assert voice.tobytes() == tensor, "ff_siwis differs from the official voice tensor"

    trial = json.loads(args.trial.read_text(encoding="utf-8"))
    common = root.parent / "issue-5/inputs.json"
    expected = json.loads(common.read_text(encoding="utf-8"))
    common_manifest = json.loads((common.parent / "resources.json").read_text(encoding="utf-8"))
    assert trial["inputs_sha256"] == common_manifest["inputs_sha256"]
    assert hashlib.sha256((args.package / "inputs.json").read_bytes()).hexdigest() == trial["inputs_sha256"]
    assert [case["input"] for case in trial["cases"]] == expected
    assert trial["status"] == "passed" and trial["offline_verified"]
    assert trial["providers"] == ["CPUExecutionProvider"] and trial["threads"] == 2
    assert trial["python"] == "3.12.10"
    packages = {name.lower().replace("_", "-"): version for name, version in trial["packages"].items()}
    expected_packages = {item["file"].split("-")[0].replace("_", "-"): item["file"].split("-")[1]
                         for item in resources if item["role"] == "wheel"}
    assert packages == expected_packages, "runtime package versions differ from the fixed wheels"
    for case in trial["cases"]:
        path = args.trial.parent / case["file"]
        assert hashlib.sha256(path.read_bytes()).hexdigest() == case["sha256"]
        with wave.open(str(path), "rb") as audio:
            assert (audio.getnchannels(), audio.getsampwidth(), audio.getframerate()) == (1, 2, 24000)
            assert audio.getnframes() > 0 and any(audio.readframes(audio.getnframes()))
        assert case["playback_seconds"] > 0 and "PlaySound completed" in case["playback"]
    for phase in ("network_denials_before", "network_denials_after"):
        assert [item["winerror"] for item in trial[phase]] == [10013, 10013]

    paths = sorted(path for path in args.package.rglob("*") if path.is_file())
    inventory = {}
    for path in paths:
        with path.open("rb") as source:
            digest = hashlib.file_digest(source, "sha256").hexdigest()
        inventory[path.relative_to(args.package).as_posix()] = {"bytes": path.stat().st_size, "sha256": digest}
    (args.package / "inventory.json").write_text(json.dumps(inventory, indent=2) + "\n", encoding="utf-8")
    paths.append(args.package / "inventory.json")
    with zipfile.ZipFile(args.archive, "x", zipfile.ZIP_DEFLATED, compresslevel=6) as archive:
        for path in paths:
            archive.write(path, path.relative_to(args.package).as_posix())
    with zipfile.ZipFile(args.archive) as archive:
        assert archive.testzip() is None
        assert set(archive.namelist()) == set(inventory) | {"inventory.json"}
        for name, item in inventory.items():
            assert hashlib.sha256(archive.read(name)).hexdigest() == item["sha256"]
    result = {
        "started_at_utc": trial["started_at_utc"], "input_version": common_manifest["input_version"],
        "inputs_sha256": trial["inputs_sha256"], "resources_manifest": "../issue-14/resources.json",
        "acquisition_by_role": roles, "acquisition_bytes": sum(item["bytes"] for item in resources),
        "expanded_acquisition_bytes": sum(role["expanded_bytes"] for role in roles.values()),
        "voice_tensor_matches_official": True, "offline_generation_and_playback": "passed",
        "quality_status": "pending human listening", "runtime_packages": trial["packages"],
        "package": {"path": str(args.archive), "zip_bytes": args.archive.stat().st_size,
                    "installed_bytes": sum(path.stat().st_size for path in paths), "file_count": len(paths),
                    "sha256": hashlib.sha256(args.archive.read_bytes()).hexdigest()},
        "total_load_seconds": trial["total_load_seconds"], "model_load_seconds": trial["load_seconds"],
        "first_case_audio_seconds": trial["cases"][0]["first_audio_seconds"],
        "total_synthesis_seconds": sum(case["synthesis_seconds"] for case in trial["cases"]),
        "total_audio_seconds": sum(case["audio_seconds"] for case in trial["cases"]),
        "peak_working_set_bytes": trial["peak_working_set_bytes"],
    }
    measurement.write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
