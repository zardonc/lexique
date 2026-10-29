"""Stage the verified Windows x64 speech resources for a development build."""

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import shutil
import stat
import tarfile

INVENTORY = "6c6ae859642f879c697e4726fdceadf2f915a71137626b5eb449f67e08d1f7ff"
ENCODER = "b9f9ccabf5e00ce55e43079d921316747ba1c4ea730a5d06a31ebe64db147774"
SOURCE_ARCHIVES = {
    "opus-1.6.1.tar.gz": ("6ffcb593207be92584df15b32466ed64bbec99109f007c82205f0194572411a1", "opus-1.6.1/COPYING"),
    "libopusenc-0.3.tar.gz": ("f616d3aff9b2034547894ccb8ab56c36cf1a4acb0d922c5d7119f97bbe58642c", "libopusenc-0.3/COPYING"),
}


def digest(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def regular(path):
    info = path.lstat()
    if not stat.S_ISREG(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
        raise ValueError(f"Not a regular resource file: {path}")


def stage(bundle, encoder, sources, destination):
    if destination.exists() or destination.is_symlink():
        raise FileExistsError(f"Preserving existing destination: {destination}")
    bundle = bundle.resolve(strict=True)
    if destination.resolve().is_relative_to(bundle):
        raise ValueError("Destination must be outside the input bundle")
    inventory_path = bundle / "inventory.json"
    regular(inventory_path)
    if digest(inventory_path) != INVENTORY:
        raise ValueError("Speech inventory checksum mismatch")
    inventory = json.loads(inventory_path.read_text(encoding="utf-8"))
    selected = {}
    for name, expected in inventory.items():
        relative = PurePosixPath(name)
        if relative.is_absolute() or ".." in relative.parts or "\\" in name:
            raise ValueError("Unsafe resource path")
        if not (name.startswith("runtime/") or name.startswith("assets/") or
                name.startswith("docs/evidence/issue-14/supertonic-licenses/")):
            continue
        source = bundle.joinpath(*relative.parts)
        for parent in [source.parent, *source.parents]:
            if parent == bundle.parent:
                break
            info = parent.lstat()
            if not stat.S_ISDIR(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
                raise ValueError(f"Unsafe resource directory: {parent}")
        regular(source)
        if digest(source) != expected:
            raise ValueError(f"Speech resource checksum mismatch: {name}")
        selected[name] = source
    if "runtime/python.exe" not in selected or "assets/voice.bin" not in selected:
        raise ValueError("Speech resources are incomplete")
    regular(encoder)
    if digest(encoder) != ENCODER:
        raise ValueError("Opus encoder checksum mismatch")
    licenses = {}
    for name, (expected, member) in SOURCE_ARCHIVES.items():
        archive = sources / name
        regular(archive)
        if digest(archive) != expected:
            raise ValueError(f"Source archive checksum mismatch: {name}")
        with tarfile.open(archive, "r:gz") as tar:
            file = tar.extractfile(member)
            if file is None:
                raise ValueError(f"Missing license: {name}")
            licenses[name + ".LICENSE"] = file.read()
    destination.mkdir()
    for name, source in selected.items():
        target = destination.joinpath(*PurePosixPath(name).parts)
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
        if digest(target) != inventory[name]:
            raise ValueError(f"Staged speech resource checksum mismatch: {name}")
    shutil.copyfile(encoder, destination / "opusenc.dll")
    if digest(destination / "opusenc.dll") != ENCODER:
        raise ValueError("Staged Opus encoder checksum mismatch")
    for name, content in licenses.items():
        (destination / name).write_bytes(content)
    shutil.copyfile(inventory_path, destination / "inventory.json")
    return len(selected)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bundle", type=Path)
    parser.add_argument("encoder", type=Path)
    parser.add_argument("sources", type=Path)
    parser.add_argument("destination", type=Path)
    args = parser.parse_args()
    count = stage(args.bundle, args.encoder, args.sources, args.destination)
    print(f"Staged {count} verified files to {args.destination}")
