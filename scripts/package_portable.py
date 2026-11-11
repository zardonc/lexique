"""Package a Windows release without installers or personal runtime data."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil
import struct
import sys
import tempfile
import zipfile

sys.path.insert(0, str(Path(__file__).resolve().parent))
from prepare_speech import stage
from freedict_mdx import verify


def package(root: Path, output_directory: Path | None = None, architecture: str = "x64",
            speech_bundle: Path | None = None, speech_encoder: Path | None = None,
            speech_sources: Path | None = None):
    speech_inputs = (speech_bundle, speech_encoder, speech_sources)
    if any(speech_inputs) and not all(speech_inputs):
        raise ValueError("Speech bundle, encoder, and sources must be supplied together")
    if all(speech_inputs) and architecture != "x64":
        raise ValueError("Speech development resources require Windows x64")
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
    suffix = "-speech" if all(speech_inputs) else ""
    name = f"Lexique-{config['version']}-windows-{architecture}-portable{suffix}"
    output = (output_directory if output_directory is not None else root / "target/portable") / name
    archive = output.parent / f"{name}.zip"
    checksum_path = archive.with_suffix(".zip.sha256")
    if output.exists() or archive.exists() or checksum_path.exists():
        raise FileExistsError(f"Preserving existing delivery: {output}; move it before rebuilding")
    report = json.loads((root / "target/desktop-dictionary-sample/report.json").read_text())
    dictionary = release / "dictionary/dictionary.sqlite3"
    if hashlib.sha256(dictionary.read_bytes()).hexdigest() != report["sqlite_sha256"]:
        raise ValueError("Release dictionary does not match the prepared resource")
    conjugation_report = json.loads((root / "target/desktop-conjugation-v6-stats/report.json").read_text())
    conjugation = release / "conjugation"
    if hashlib.sha256((conjugation / "conjugation.sqlite3").read_bytes()).hexdigest() != conjugation_report["sqlite_sha256"]:
        raise ValueError("Release conjugation does not match the prepared resource")
    frequency_report = json.loads((root / "target/desktop-frequency-v1/report.json").read_text())
    frequency = release / "frequency"
    if hashlib.sha256((frequency / "frequency.sqlite3").read_bytes()).hexdigest() != frequency_report["sqlite_sha256"]:
        raise ValueError("Release frequency does not match the prepared resource")
    for filename, expected in frequency_report["inputs"].items():
        if hashlib.sha256((frequency / filename).read_bytes()).hexdigest() != expected:
            raise ValueError(f"Release frequency source or notice differs: {filename}")
    verify(root, conjugation / "freedict-fra-eng-0.4.1")
    if all(speech_inputs):
        with tempfile.TemporaryDirectory(prefix="lexique-speech-stage-") as temporary:
            staged = Path(temporary) / "speech"
            stage(speech_bundle, speech_encoder, speech_sources, staged)
            output.mkdir(parents=True)
            shutil.copytree(staged, output / "speech")
    else:
        output.mkdir(parents=True)
    shutil.copy2(executable, output / "Lexique.exe")
    shutil.copytree(root / "resources/freedict-fra-zho-2025.11.23", output / "dictionary")
    shutil.copy2(dictionary, output / "dictionary/dictionary.sqlite3")
    shutil.copytree(root / "resources/freedict-fra-eng-mdx-0.4.1",
                    output / "dictionary/freedict-fra-eng-mdx")
    shutil.copy2(root / "scripts/freedict_mdx.py",
                 output / "dictionary/freedict-fra-eng-mdx/freedict_mdx.py")
    shutil.copy2(root / "LICENSE", output / "dictionary/freedict-fra-eng-mdx/LICENSE-MIT")
    shutil.copytree(conjugation, output / "conjugation")
    shutil.copytree(frequency, output / "frequency")
    shutil.copytree(root / "resources/tesseract-fra/licenses", output / "ocr-notices/licenses")
    shutil.copy2(root / "resources/tesseract-fra/README.md", output / "ocr-notices/README.md")
    shutil.copytree(root / "resources/mdictlib-0.2.8", output / "mdx-notices")
    shutil.copy2(root / "LICENSE", output / "LICENSE")
    (output / "README.txt").write_text(
        f"Lexique {config['version']} - Windows 11 {architecture.upper()} portable development sample\n\n"
        "Extract the entire ZIP into a writable folder, then open Lexique.exe.\n"
        "Keep dictionary/, conjugation/ and frequency/ beside the executable. No installer or first-run download is used.\n"
        "Requires an existing Microsoft Edge WebView2 runtime (normally present on Windows 11).\n"
        "If Lexique cannot open because WebView2 is missing, install the Microsoft Edge\n"
        "WebView2 Evergreen Runtime from https://developer.microsoft.com/microsoft-edge/webview2/\n"
        "For an offline computer, download the Evergreen Standalone Installer for its\n"
        "architecture on another computer, transfer it, and run it yourself. Lexique\n"
        "does not download, install, or bundle WebView2. Restart Lexique after installation.\n"
        "The unsigned executable may trigger Windows SmartScreen.\n\n"
        "All application-managed runtime files stay in Lexique-data/ beside Lexique.exe:\n"
        "data/lexique.sqlite3, webview/ (profile/cache/crash data), temp/, config/, cache/, logs/.\n"
        "Close the app before moving or backing up the whole folder. Preserve Lexique-data/\n"
        "when updating. Existing AppData files are neither imported nor removed.\n"
        "A read-only location fails; there is no fallback to AppData.\n\n"
        "This sample provides FreeDict fra-zho lookup and M1 vocabulary notes with explicit save,\n"
        "search, sorting and restart recovery. Saved words can be added to two review cards;\n"
        "review scoring and card progress survive restart. A full dictionary is not included.\n"
        "Selection shortcut: Ctrl+Alt+Shift+D. Screen OCR shortcut: Ctrl+Alt+Shift+O.\n"
        "Both shortcuts can be changed in Settings. Selection and OCR enter the Translation\n"
        "input, retaining the first 5,000 Unicode characters; dictionary lookup is explicit.\n"
        "Offline French OCR runtime and model are embedded. Licenses and sources: ocr-notices/.\n"
        "Add MDX dictionaries in Settings. Imported copies stay in Lexique-data/.\n"
        "Dictionary order is saved; other tab bodies load on selection. Parser licenses: mdx-notices/.\n"
        "One supplemental French-English MDX is bundled: FreeDict 0.4.1 (original content 2018-08-11).\n"
        "Its GPL-2.0-or-later notices, provenance and MIT converter are in dictionary/freedict-fra-eng-mdx/.\n"
        "Complete corresponding FreeDict source is in conjugation/freedict-fra-eng-0.4.1/; see the MDX README.md to rebuild.\n"
        "Settings includes optional source/download links. Other dictionaries are not bundled or downloaded automatically.\n"
        "MDX imports are limited to 128 MiB each and 15 custom entries; MDD media are unavailable.\n"
        "Conjugation uses the compact format-6 resource with 23 ordinary/pronominal reference blocks.\n"
        "Query normalization, three color modes, saved block order and dictionary navigation are included.\n"
        "Keep the format-6 resource with its matching rebuilt reader when updating.\n"
        + ("Offline speech is included for development testing. See speech/docs for its licenses.\n"
           "Each complete conjugation expression and spelling/gender alternative can be played separately.\n"
           "The voice is machine-generated; model use restrictions apply.\n"
           if all(speech_inputs) else "TTS is not included.\n") +
        "Try: école, œil, aujourd’hui, or, vacances.\n"
        "Project code: MIT (LICENSE). Dictionary: CC BY-SA 3.0 (dictionary/COPYING).\n"
        "Conjugation: Lefff forms LGPL-LR (conjugation/LICENSE-LGPLLR), Grammalecte forms/metadata\n"
        "GPL-3.0-only (conjugation/grammalecte-2.3.0/LICENSE-GPL3).\n"
        "Lefff syntax (LGPL-LR) and RosaeNLG elision data (CC BY-SA 3.0) are in\n"
        "conjugation/lefff-syntax/ with attribution and separate reference-code licenses.\n"
        "English verb meanings: FreeDict fra-eng 0.4.1, GPL-2.0-or-later; complete source and\n"
        "notices in conjugation/freedict-fra-eng-0.4.1/. Missing meanings leave tables usable.\n"
        "Raw sources, provenance, corrections, decoder, LGPL-LR ending rules and eight MIT converters\n"
        "accompany the resource.\n\n"
        "Notebook frequency labels use Lexique 3.83 subtitle lemma/POS statistics.\n"
        "Frequency data and its derived database: CC BY-SA 4.0 (frequency/COPYING).\n"
        "Original source, attribution, provenance and the MIT converter accompany frequency/.\n"
        "Rebuild frequency: copy frequency/Lexique383.tsv.gz, COPYING, UPSTREAM-README.txt,\n"
        "README.md and provenance.json into resources/lexique-3.83/ in a new workspace;\n"
        "copy frequency/frequency_sample.py into scripts/ and LICENSE-MIT into LICENSE.\n"
        "Run python -B -X utf8 scripts/frequency_sample.py target/rebuilt-frequency.\n\n"
        "Rebuild conjugation with Python 3.12+ (standard library only): in a new workspace,\n"
        "copy conjugation/conjugation_*.py and conjugation/conjugation_endings.json into scripts/,\n"
        "LICENSE into the workspace root,\n"
        "and conjugation/grammalecte-2.3.0/ into resources/grammalecte-2.3.0/. Copy\n"
        "conjugation/lefff-syntax/ into resources/lefff-syntax/. Copy\n"
        "conjugation/freedict-fra-eng-0.4.1/ into resources/freedict-fra-eng-0.4.1/. Copy\n"
        "conjugation/lefff-3.4.mlex.gz, provenance.json, README.md and LICENSE-LGPLLR into\n"
        "resources/lefff-3.4/. Run python -X utf8 scripts/conjugation_sample.py build target/rebuilt.\n",
        encoding="utf-8",
    )
    files = sorted(path for path in output.rglob("*") if path.is_file())
    checksums = {path.relative_to(output).as_posix(): hashlib.sha256(path.read_bytes()).hexdigest()
                 for path in files}
    (output / "SHA256SUMS.json").write_text(json.dumps(checksums, indent=2) + "\n")
    with zipfile.ZipFile(archive, "x", zipfile.ZIP_DEFLATED, compresslevel=9, strict_timestamps=False) as zipped:
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
    parser.add_argument("--speech-bundle", type=Path, help="Verified speech evidence bundle")
    parser.add_argument("--speech-encoder", type=Path, help="Fixed Windows x64 Opus encoder DLL")
    parser.add_argument("--speech-sources", type=Path, help="Directory containing fixed Opus source archives")
    args = parser.parse_args()
    print(json.dumps(package(Path(__file__).resolve().parent.parent, args.output_dir, args.architecture,
                             args.speech_bundle, args.speech_encoder, args.speech_sources), indent=2))
