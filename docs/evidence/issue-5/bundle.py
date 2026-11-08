"""Assemble a local Windows x64 evidence package from pinned archives."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import zipfile


def digest(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("downloads", type=Path)
    parser.add_argument("assets", type=Path)
    parser.add_argument("output", type=Path, help="must not exist")
    args = parser.parse_args()
    if args.output.exists():
        raise FileExistsError(f"Preserving evidence package: {args.output}")
    evidence = Path(__file__).resolve().parent
    repository = evidence.parents[2]
    manifest = json.loads((evidence / "resources.json").read_text(encoding="utf-8"))
    for directory, items in ((args.downloads, manifest["downloads"]), (args.assets, manifest["assets"])):
        for item in items:
            path = directory / item["file"]
            if path.stat().st_size != item["bytes"] or digest(path) != item["sha256"]:
                raise ValueError(f"resource checksum mismatch: {path}")
    args.output.mkdir(parents=True)
    for item in manifest["downloads"]:
        if item["file"].endswith(".exe"):
            (args.output / "prerequisites").mkdir(exist_ok=True)
            shutil.copyfile(args.downloads / item["file"], args.output / "prerequisites" / item["file"])
            continue
        if not item["file"].endswith((".whl", ".zip")):
            continue
        destination = args.output / "runtime"
        if item["file"].endswith(".whl"):
            destination /= "Lib/site-packages"
        with zipfile.ZipFile(args.downloads / item["file"]) as archive:
            for member in archive.namelist():
                if not (destination / member).resolve().is_relative_to(destination.resolve()):
                    raise ValueError(f"unsafe archive path: {member}")
                if member.startswith("sherpa_onnx_core-2.0.1.data/data/Scripts/"):
                    continue
                archive.extract(member, destination)
    (args.output / "runtime/python314._pth").write_text("python314.zip\n.\nLib/site-packages\n", encoding="utf-8")
    (args.output / "assets").mkdir()
    for item in manifest["assets"]:
        shutil.copyfile(args.assets / item["file"], args.output / "assets" / item["file"])
    (args.output / "scripts").mkdir()
    for name in ("sherpa_probe.py", "speech_probe.py", "test_sherpa_probe.py"):
        shutil.copyfile(repository / "scripts" / name, args.output / "scripts" / name)
    shutil.copytree(evidence, args.output / "docs/evidence/issue-5",
                    ignore=shutil.ignore_patterns("__pycache__", "README.md", "measurements.json", "package-report.json"))
    shutil.copyfile(repository / "LICENSE", args.output / "LICENSE")
    (args.output / "README.md").write_text(
        "# Sherpa / Piper French Windows x64 validation\n\n"
        "Listen to all eight approved samples in docs/evidence/issue-5/samples-tom-v2/listen.html. "
        "Their listening verdict is recorded in samples-tom-v2/quality.json. "
        "The original MLS recordings and the partial Tom verdict remain in samples and samples-tom. "
        "Successful generation and sentence recognition do not establish human pronunciation approval.\n\n"
        "From this extracted directory, run in PowerShell 7 with Codex 0.167.0-alpha.3.2:\n\n"
        "```powershell\n"
        "codex sandbox -P issue5 "
        "-c 'permissions.issue5.filesystem={\":root\"=\"read\",\":project_roots\"=\"write\"}' "
        "-c 'permissions.issue5.network.enabled=false' -C $PWD -- "
        "runtime/python.exe -B -X utf8 scripts/sherpa_probe.py assets recordings "
        "--require-network-denied --play\n```\n\n"
        "The output directory must not exist. Licenses are in docs/evidence/issue-5/licenses. "
        "The offline VC prerequisite installer is retained in prerequisites and has not been executed.\n",
        encoding="utf-8",
    )
    inventory = {path.relative_to(args.output).as_posix(): {"bytes": path.stat().st_size, "sha256": digest(path)}
                 for path in sorted(args.output.rglob("*")) if path.is_file()}
    (args.output / "inventory.json").write_text(json.dumps(inventory, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"files": len(inventory), "installed_bytes": sum(item["bytes"] for item in inventory.values())}))


if __name__ == "__main__":
    main()
