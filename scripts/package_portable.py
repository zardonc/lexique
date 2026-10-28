"""Package a Windows release without installers or personal runtime data."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import struct
import zipfile


def package(root: Path, output_directory: Path | None = None, architecture: str = "x64"):
    targets = {"x64": ("target/release", 0x8664),
               "arm64": ("target/aarch64-pc-windows-msvc/release", 0xAA64)}
    if architecture not in targets:
        raise ValueError(f"Unsupported Windows architecture: {architecture}")
    release_path, machine = targets[architecture]
    release = root / release_path
    executable = release / "lexique-desktop.exe"
    binary = executable.read_bytes()
    pe_offset = struct.unpack_from("<I", binary, 0x3C)[0]
    if binary[:2] != b"MZ" or binary[pe_offset:pe_offset + 4] != b"PE\0\0":
        raise ValueError("Expected a Windows executable")
    if struct.unpack_from("<H", binary, pe_offset + 4)[0] != machine:
        raise ValueError(f"Executable does not target Windows {architecture}")
    config = json.loads((root / "apps/desktop/src-tauri/tauri.conf.json").read_text())
    name = f"Lexique-{config['version']}-windows-{architecture}-portable"
    output = (output_directory if output_directory is not None else root / "target/portable") / name
    archive = output.parent / f"{name}.zip"
    checksum_path = archive.with_suffix(".zip.sha256")
    if output.exists() or archive.exists() or checksum_path.exists():
        raise FileExistsError(f"Preserving existing delivery: {output}; move it before rebuilding")
    report = json.loads((root / "target/desktop-dictionary-sample/report.json").read_text())
    dictionary = release / "dictionary/dictionary.sqlite3"
    if hashlib.sha256(dictionary.read_bytes()).hexdigest() != report["sqlite_sha256"]:
        raise ValueError("Release dictionary does not match the prepared resource")
    output.mkdir(parents=True)
    shutil.copy2(executable, output / "Lexique.exe")
    shutil.copytree(root / "resources/freedict-fra-zho-2025.11.23", output / "dictionary")
    shutil.copy2(dictionary, output / "dictionary/dictionary.sqlite3")
    shutil.copy2(root / "LICENSE", output / "LICENSE")
    (output / "README.txt").write_text(
        f"Lexique {config['version']} - Windows 11 {architecture.upper()} portable development sample\n\n"
        "Extract the entire ZIP into a writable folder, then open Lexique.exe.\n"
        "Keep dictionary/ beside the executable. No installer or first-run download is used.\n"
        "Requires an existing Microsoft Edge WebView2 runtime (normally present on Windows 11).\n"
        "The unsigned executable may trigger Windows SmartScreen.\n\n"
        "All application-managed runtime files stay in Lexique-data/ beside Lexique.exe:\n"
        "data/lexique.sqlite3, webview/ (profile/cache/crash data), temp/, config/, cache/, logs/.\n"
        "Close the app before moving or backing up the whole folder. Preserve Lexique-data/\n"
        "when updating. Existing AppData files are neither imported nor removed.\n"
        "A read-only location fails; there is no fallback to AppData.\n\n"
        "This sample provides FreeDict fra-zho lookup and M1 vocabulary notes with explicit save,\n"
        "search, sorting and restart recovery. TTS and a full dictionary are not included.\n"
        "Try: école, œil, aujourd’hui, or, vacances.\n"
        "Project code: MIT (LICENSE). Dictionary: CC BY-SA 3.0 (dictionary/COPYING).\n",
        encoding="utf-8",
    )
    files = sorted(path for path in output.rglob("*") if path.is_file())
    checksums = {path.relative_to(output).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
                 for path in files}
    (output / "SHA256SUMS.json").write_text(json.dumps(checksums, indent=2) + "\n")
    with zipfile.ZipFile(archive, "x", zipfile.ZIP_DEFLATED, compresslevel=9) as zipped:
        for path in sorted(output.rglob("*")):
            if path.is_file():
                zipped.write(path, path.relative_to(output.parent))
    with zipfile.ZipFile(archive) as zipped:
        expected = {f"{name}/{path}": checksum for path, checksum in checksums.items()}
        assert set(zipped.namelist()) == {*expected, f"{name}/SHA256SUMS.json"}
        assert all(hashlib.sha256(zipped.read(path)).hexdigest() == checksum
                   for path, checksum in expected.items()), "Archive content differs from the delivery"
        assert not any("Lexique-data/" in path for path in zipped.namelist())
    checksum = hashlib.sha256(archive.read_bytes()).hexdigest()
    with checksum_path.open("x", encoding="utf-8") as checksum_file:
        checksum_file.write(f"{checksum}  {archive.name}\n")
    return {"archive": str(archive), "bytes": archive.stat().st_size,
            "sha256": checksum, "verified_files": len(expected) + 1}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, help="Separate delivery parent; existing artifacts are never overwritten")
    parser.add_argument("--architecture", choices=("x64", "arm64"), default="x64")
    args = parser.parse_args()
    print(json.dumps(package(Path(__file__).resolve().parent.parent, args.output_dir, args.architecture), indent=2))
