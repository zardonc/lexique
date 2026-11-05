"""Build an isolated Windows x64 Supertonic evidence package from pinned local inputs."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import zipfile


EVIDENCE = Path(__file__).resolve().parent
REPOSITORY = EVIDENCE.parents[2]
RUNTIME = {
    "python-3.14.10-embed-amd64.zip": "90905e64a840557d8c1bc4f5b74db3d7ee85e3ffb217611cac1c7813acea4159",
    "numpy-2.5.4-cp314-cp314-win_amd64.whl": "2ca144f15135b6212a5c47b1e2aeca6e412f102f95a2d5d88d8aec77eb255de3",
    "sherpa_onnx-2.0.1-cp314-cp314-win_amd64.whl": "4aa2668fa24aa3223bae9d1363aa3274ef2201f2f1560542b6d3c282130a3831",
    "sherpa_onnx_core-2.0.1-py3-none-win_amd64.whl": "08890e9ea23ed2f59f81d5840c63b6b5cc8674d0f6e48d9069126a848aa0e022",
}


def digest(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("materials", type=Path)
    parser.add_argument("wheelhouse", type=Path)
    parser.add_argument("assets", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    archive = args.output.with_suffix(".zip")
    if args.output.exists() or archive.exists():
        raise FileExistsError("Preserving existing output or archive")

    inputs = [(args.materials / name if index < 2 else args.wheelhouse / name, expected)
              for index, (name, expected) in enumerate(RUNTIME.items())]
    model = json.loads((EVIDENCE / "supertonic3-model.json").read_text(encoding="utf-8"))
    inputs.extend((args.assets / item["path"], item["actual_sha256"]) for item in model["files"][:7])
    licenses = json.loads((EVIDENCE / "supertonic-licenses/sources.json").read_text(encoding="utf-8"))
    inputs.extend((EVIDENCE / "supertonic-licenses" / item["file"], item["actual_sha256"])
                  for item in licenses["evidence"])
    for source, expected in inputs:
        if digest(source) != expected:
            raise ValueError(f"checksum mismatch: {source.name}")

    args.output.mkdir(parents=True)
    for index, (source, _) in enumerate(inputs[:4]):
        destination = args.output / "runtime"
        if index:
            destination /= "Lib/site-packages"
        with zipfile.ZipFile(source) as package:
            for member in package.namelist():
                target = (destination / member).resolve()
                if not target.is_relative_to(destination.resolve()):
                    raise ValueError(f"unsafe archive path: {member}")
            for member in package.namelist():
                if index == 3 and member.startswith("sherpa_onnx_core-2.0.1.data/data/Scripts/"):
                    continue
                package.extract(member, destination)
    (args.output / "runtime/python314._pth").write_text(
        "python314.zip\n.\nLib/site-packages\n", encoding="utf-8")
    (args.output / "assets").mkdir()
    for source, _ in inputs[4:11]:
        shutil.copy2(source, args.output / "assets" / source.name)
    evidence = args.output / "docs/evidence/issue-14"
    evidence.mkdir(parents=True)
    for name in ("probe_supertonic.py", "supertonic3-model.json", "inputs.json",
                 "supertonic-static-comparison.md", "compare_onnx_static.py",
                 "test_compare_onnx_static.py"):
        shutil.copy2(EVIDENCE / name, evidence / name)
    shutil.copytree(EVIDENCE / "supertonic-licenses", evidence / "supertonic-licenses")
    (args.output / "scripts").mkdir()
    shutil.copy2(REPOSITORY / "scripts/speech_probe.py", args.output / "scripts/speech_probe.py")
    shutil.copy2(REPOSITORY / "LICENSE", args.output / "LICENSE")
    inventory = {path.relative_to(args.output).as_posix(): digest(path)
                 for path in sorted(args.output.rglob("*")) if path.is_file()}
    (args.output / "inventory.json").write_text(json.dumps(inventory, indent=2) + "\n", encoding="utf-8")
    with zipfile.ZipFile(archive, "x", zipfile.ZIP_DEFLATED, compresslevel=6) as package:
        for path in sorted(args.output.rglob("*")):
            if path.is_file():
                package.write(path, path.relative_to(args.output))
    with zipfile.ZipFile(archive) as package:
        if package.testzip() is not None:
            raise ValueError("package CRC verification failed")
    print(json.dumps({"archive": str(archive), "compressed_bytes": archive.stat().st_size,
                      "installed_bytes": sum(path.stat().st_size for path in args.output.rglob("*") if path.is_file()),
                      "sha256": digest(archive), "files": len(inventory) + 1}, indent=2))


if __name__ == "__main__":
    main()
