"""Deliver pinned upstream sources and this probe's build and conversion materials."""

import argparse
from datetime import date
import json
from pathlib import Path
import shutil
import sys
import tarfile
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parent))
from bundle import digest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("downloads", type=Path, help="source companion downloads")
    parser.add_argument("model", type=Path, help="original ONNX, HF config and model card")
    parser.add_argument("output", type=Path, help="directory, ZIP and measurements must not exist")
    args = parser.parse_args()
    evidence = Path(__file__).resolve().parent
    resources = json.loads((evidence / "resources.json").read_text(encoding="utf-8"))
    archive_path = args.output.with_name(args.output.name + ".zip")
    measurement_path = args.output.with_name(args.output.name + ".measurements.json")
    for path in (args.output, archive_path, measurement_path):
        if path.exists():
            raise FileExistsError(f"Preserving source delivery: {path}")
    for directory, items in ((args.downloads, resources["source_companion_downloads"]),
                             (args.model, resources["source_files"])):
        for item in items:
            path = directory / item["file"]
            if path.stat().st_size != item["bytes"] or digest(path) != item["sha256"]:
                raise ValueError(f"source resource checksum mismatch: {path}")
    repository = evidence.parents[2]
    files = {Path("upstream") / item["file"]: args.downloads / item["file"]
             for item in resources["source_companion_downloads"]}
    files.update({Path("model") / item["file"]: args.model / item["file"]
                  for item in resources["source_files"]})
    project_files = ["LICENSE", "scripts/sherpa_probe.py", "scripts/speech_probe.py",
                     "scripts/test_sherpa_probe.py", "scripts/test_sherpa_sources.py"]
    project_files += [f"docs/evidence/issue-5/{name}" for name in (
        "prepare.py", "bundle.py", "sources.py", "measure.py", "resources.json", "inputs.json",
        "asr-model.json", "source-build.md", "piper-local-sources.patch", "source-delivery-research.md",
        "native-dependencies.json",
        "samples/resources.json", "samples/quality.json",
    )]
    project_files += [path.relative_to(repository).as_posix()
                      for path in sorted((evidence / "licenses").iterdir()) if path.is_file()]
    files.update({Path("project") / name: repository / name for name in project_files})
    for path in files.values():
        if not path.is_file():
            raise FileNotFoundError(f"missing source material: {path}")
    args.output.mkdir(parents=True)
    for name, source in files.items():
        destination = args.output / name
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, destination)
    (args.output / "README.md").write_text(
        "# Source materials for the French speech validation\n\n"
        "This companion accompanies the Windows runtime ZIP. It contains complete pinned Piper, eSpeak, "
        "Sonic and Sherpa archives, Eigen, the original model and both configurations, and project scripts. "
        "See [build and conversion steps](project/docs/evidence/issue-5/source-build.md) and "
        "[licenses](project/docs/evidence/issue-5/licenses/ATTRIBUTION.md).\n\n"
        "The model publisher has not established its preferred source form for modification. "
        "Source rebuilds and public redistribution clearance remain unverified.\n",
        encoding="utf-8",
    )
    inventory = {path.relative_to(args.output).as_posix(): {"bytes": path.stat().st_size, "sha256": digest(path)}
                 for path in sorted(args.output.rglob("*")) if path.is_file()}
    (args.output / "inventory.json").write_text(json.dumps(inventory, indent=2) + "\n", encoding="utf-8")
    with zipfile.ZipFile(archive_path, "x", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for name in sorted(set(inventory) | {"inventory.json"}):
            archive.write(args.output / name, name)
    with zipfile.ZipFile(archive_path) as archive:
        if archive.testzip() is not None or set(archive.namelist()) != set(inventory) | {"inventory.json"}:
            raise ValueError("source ZIP inventory or CRC mismatch")
    raw_source_bytes = sum(item["bytes"] for item in resources["source_files"])
    for item in resources["source_companion_downloads"]:
        if item["file"].endswith(".tar.gz"):
            with tarfile.open(args.downloads / item["file"]) as archive:
                raw_source_bytes += sum(member.size for member in archive if member.isfile())
        else:
            raw_source_bytes += item["bytes"]
    measurements = {
        "date": date.today().isoformat(),
        "additional_download_bytes": sum(item["bytes"] for item in resources["source_companion_downloads"]),
        "original_model_download_bytes": sum(item["bytes"] for item in resources["source_files"]),
        "raw_source_input_bytes": raw_source_bytes,
        "methods": {"download": "Distinct pinned source artifact bytes; original model inputs are already counted in runtime acquisition",
                    "raw": "Regular-file sizes in the five source tar archives, plus both original configs, ONNX and model card",
                    "installed": "Actual companion file logical sizes, including original archives, scripts, notices and inventory",
                    "compressed": "Whole companion ZIP_DEFLATED level 9; exact member set and CRC verified"},
        "package": {"path": archive_path.as_posix(), "files": len(inventory) + 1,
                    "installed_bytes": sum(path.stat().st_size for path in args.output.rglob("*") if path.is_file()),
                    "zip_bytes": archive_path.stat().st_size, "sha256": digest(archive_path), "inventory_verified": True},
        "limitations": ["The publisher's preferred model source form is unconfirmed",
                        "Native source rebuilds and public redistribution clearance are unverified"],
    }
    measurement_path.write_text(json.dumps(measurements, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(measurements))


if __name__ == "__main__":
    main()
