"""Build the isolated Windows probe from already downloaded, hash-verified inputs."""

import hashlib
import json
from pathlib import Path
import shutil
import sys
import zipfile


def main():
    materials, output = map(Path, sys.argv[1:])
    archive = output.with_suffix(".zip")
    if output.exists() or archive.exists():
        raise ValueError("output or archive already exists")
    evidence = Path(__file__).resolve().parent
    repository = evidence.parents[2]
    resources = json.loads((evidence / "resources.json").read_text(encoding="utf-8"))
    for item in resources:
        with (materials / item["file"]).open("rb") as source:
            if hashlib.file_digest(source, "sha256").hexdigest() != item["sha256"]:
                raise ValueError(f"checksum mismatch: {item['file']}")
    output.mkdir(parents=True)
    for item in resources:
        source = materials / item["file"]
        role = item["role"]
        if role in ("wheel", "python"):
            destination = output / "runtime"
            if role == "wheel":
                destination /= "Lib/site-packages"
            with zipfile.ZipFile(source) as package:
                for member in package.namelist():
                    target = (destination / member).resolve()
                    if not target.is_relative_to(destination.resolve()):
                        raise ValueError(f"unsafe archive path: {member}")
                package.extractall(destination)
        else:
            destination = output / ("assets" if role == "model" else role)
            destination.mkdir(exist_ok=True)
            shutil.copy2(source, destination / source.name)
    (output / "runtime/python312._pth").write_text(
        "python312.zip\n.\nLib/site-packages\n", encoding="utf-8",
    )
    for name in ("inputs.json", "resources.json", "README.md", "prepare.py", "checks.md",
                 "report.json", "first-attempt.json", "y-before.json"):
        shutil.copy2(evidence / name, output / name)
    shutil.copytree(evidence / "samples", output / "samples")
    for name in ("speech_probe.py", "test_speech_probe.py"):
        shutil.copy2(repository / "scripts" / name, output / name)
    shutil.copy2(repository / "LICENSE", output / "LICENSE-project.txt")
    files = sorted(path for path in output.rglob("*") if path.is_file())
    inventory = []
    for path in files:
        with path.open("rb") as source:
            digest = hashlib.file_digest(source, "sha256").hexdigest()
        inventory.append({"file": path.relative_to(output).as_posix(),
                          "bytes": path.stat().st_size, "sha256": digest})
    (output / "inventory.json").write_text(json.dumps(inventory, indent=2) + "\n", encoding="utf-8")
    with zipfile.ZipFile(archive, "x", zipfile.ZIP_DEFLATED, compresslevel=6) as package:
        for path in sorted(output.rglob("*")):
            if path.is_file():
                package.write(path, path.relative_to(output).as_posix())
    with zipfile.ZipFile(archive) as package:
        assert package.testzip() is None, "corrupt output ZIP"
    with archive.open("rb") as source:
        digest = hashlib.file_digest(source, "sha256").hexdigest()
    print(json.dumps({"archive": str(archive), "bytes": archive.stat().st_size,
                      "installed_bytes": sum(p.stat().st_size for p in output.rglob("*") if p.is_file()),
                      "sha256": digest}, indent=2))


if __name__ == "__main__":
    main()
