"""Exercise packaging with temporary build outputs, never the user's delivery."""

import hashlib
import json
import os
from pathlib import Path
import shutil
import struct
import tempfile
import unittest
import zipfile

from package_portable import package


class PortableTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.name = "Lexique-0.1.0-windows-x64-portable"
        self.output = self.root / "target/portable"
        binary = bytearray(256)
        binary[:2] = b"MZ"
        struct.pack_into("<I", binary, 0x3C, 0x80)
        binary[0x80:0x84] = b"PE\0\0"
        struct.pack_into("<H", binary, 0x84, 0x8664)
        self.write("target/release/lexique-desktop.exe", binary)
        self.write("target/release/dictionary/dictionary.sqlite3", b"verified dictionary")
        self.write("target/release/conjugation/conjugation.sqlite3", b"verified conjugation")
        self.write("target/release/conjugation/LICENSE-LGPLLR", b"source forms license")
        self.write("target/release/conjugation/grammalecte-2.3.0/LICENSE-GPL3", b"metadata license")
        self.write("target/release/frequency/frequency.sqlite3", b"verified frequency")
        self.write("target/release/frequency/COPYING", b"frequency data license")
        self.write("target/release/frequency/frequency_sample.py", b"frequency converter source")
        self.write("target/desktop-frequency-v1/report.json", json.dumps({
            "sqlite_sha256": hashlib.sha256(b"verified frequency").hexdigest(),
            "inputs": {"COPYING": hashlib.sha256(b"frequency data license").hexdigest(),
                       "frequency_sample.py": hashlib.sha256(b"frequency converter source").hexdigest()},
        }).encode())
        self.write("target/desktop-conjugation-v6-stats/report.json", json.dumps({
            "sqlite_sha256": hashlib.sha256(b"verified conjugation").hexdigest(),
        }).encode())
        self.write("target/desktop-dictionary-sample/report.json", json.dumps({
            "sqlite_sha256": hashlib.sha256(b"verified dictionary").hexdigest(),
        }).encode())
        self.write("apps/desktop/src-tauri/tauri.conf.json", b'{"version":"0.1.0"}')
        self.write("resources/freedict-fra-zho-2025.11.23/COPYING", b"original resource license")
        self.write("resources/tesseract-fra/README.md", b"fixed offline OCR sources")
        self.write("resources/tesseract-fra/licenses/tessdata-fast.txt", b"French OCR model license")
        self.write("resources/tesseract-fra/licenses/native/image.txt", b"native image license")
        self.write("resources/tesseract-fra/fra.traineddata", b"model embedded in the executable")
        self.write("resources/mdictlib-0.2.8/README.md", b"fixed MDX parser sources")
        self.write("resources/mdictlib-0.2.8/licenses/mdictlib-0.2.8.txt", b"MDX parser license")
        self.write("resources/backup-dependencies/README.md", b"Pinned backup dependency provenance")
        self.write("resources/backup-dependencies/zip-9.5.1/LICENSE", b"Original ZIP license")
        self.write("resources/backup-dependencies/cfg_aliases-0.2.2/NOTICES.md", b"Original transitive notices")
        repository = Path(__file__).resolve().parents[1]
        for name in ("freedict-fra-eng-mdx-0.4.1", "freedict-fra-eng-0.4.1"):
            shutil.copytree(repository / "resources" / name, self.root / "resources" / name)
        shutil.copytree(repository / "resources/freedict-fra-eng-0.4.1",
                        self.root / "target/release/conjugation/freedict-fra-eng-0.4.1")
        self.write("scripts/freedict_mdx.py", (repository / "scripts/freedict_mdx.py").read_bytes())
        self.write("LICENSE", b"project license")

    def write(self, name, content):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)

    def files(self):
        return {p.relative_to(self.root).as_posix(): p.read_bytes()
                for p in self.root.rglob("*") if p.is_file()}

    def test_backup_dependency_notices_are_delivered_unchanged(self):
        result = package(self.root)
        with zipfile.ZipFile(result["archive"]) as zipped:
            prefix = self.name + "/backup-notices/"
            self.assertEqual(zipped.read(prefix + "README.md"), b"Pinned backup dependency provenance")
            self.assertEqual(zipped.read(prefix + "zip-9.5.1/LICENSE"), b"Original ZIP license")
            self.assertEqual(zipped.read(prefix + "cfg_aliases-0.2.2/NOTICES.md"), b"Original transitive notices")

    def test_archive_hashes_licenses_notes_description_and_excludes_runtime_data(self):
        self.write("target/release/Lexique-data/data/lexique.sqlite3", b"private learning data")
        self.write("Lexique-data/webview/profile", b"private profile")
        before = self.files()
        result = package(self.root)
        archive = Path(result["archive"])
        self.assertEqual(result["sha256"], hashlib.sha256(archive.read_bytes()).hexdigest())
        self.assertEqual(archive.with_suffix(".zip.sha256").read_text().split()[0], result["sha256"])
        with zipfile.ZipFile(archive) as zipped:
            prefix = self.name + "/"
            checksums = json.loads(zipped.read(prefix + "SHA256SUMS.json"))
            self.assertEqual(set(zipped.namelist()), {prefix + p for p in checksums} | {prefix + "SHA256SUMS.json"})
            for path, digest in checksums.items():
                self.assertEqual(hashlib.sha256(zipped.read(prefix + path)).hexdigest(), digest)
            self.assertEqual(zipped.read(prefix + "dictionary/COPYING"), b"original resource license")
            self.assertEqual(zipped.read(prefix + "conjugation/conjugation.sqlite3"), b"verified conjugation")
            self.assertEqual(zipped.read(prefix + "conjugation/grammalecte-2.3.0/LICENSE-GPL3"), b"metadata license")
            self.assertEqual(zipped.read(prefix + "frequency/frequency.sqlite3"), b"verified frequency")
            self.assertEqual(zipped.read(prefix + "frequency/COPYING"), b"frequency data license")
            self.assertEqual(zipped.read(prefix + "frequency/frequency_sample.py"), b"frequency converter source")
            self.assertEqual(zipped.read(prefix + "ocr-notices/README.md"), b"fixed offline OCR sources")
            self.assertEqual(zipped.read(prefix + "ocr-notices/licenses/tessdata-fast.txt"), b"French OCR model license")
            self.assertEqual(zipped.read(prefix + "ocr-notices/licenses/native/image.txt"), b"native image license")
            self.assertEqual(zipped.read(prefix + "mdx-notices/README.md"), b"fixed MDX parser sources")
            self.assertEqual(zipped.read(prefix + "mdx-notices/licenses/mdictlib-0.2.8.txt"), b"MDX parser license")
            bundled = "dictionary/freedict-fra-eng-mdx/"
            metadata = json.loads(zipped.read(prefix + bundled + "manifest.json"))
            self.assertEqual(hashlib.sha256(zipped.read(prefix + bundled + metadata["file"])).hexdigest(), metadata["sha256"])
            self.assertEqual([name for name in zipped.namelist() if name.endswith(".mdx")], [prefix + bundled + metadata["file"]])
            self.assertEqual(zipped.read(prefix + bundled + "COPYING"), zipped.read(prefix + "conjugation/freedict-fra-eng-0.4.1/COPYING"))
            self.assertEqual(zipped.read(prefix + bundled + "AUTHORS"), zipped.read(prefix + "conjugation/freedict-fra-eng-0.4.1/AUTHORS"))
            self.assertIn(prefix + bundled + "README.md", zipped.namelist())
            self.assertIn(prefix + bundled + "freedict_mdx.py", zipped.namelist())
            self.assertEqual(zipped.read(prefix + bundled + "LICENSE-MIT"), b"project license")
            self.assertFalse(any(path.endswith("fra.traineddata") for path in zipped.namelist()))
            self.assertEqual(zipped.read(prefix + "LICENSE"), b"project license")
            self.assertFalse(any("Lexique-data" in path for path in zipped.namelist()))
            description = zipped.read(prefix + "README.txt").decode("utf-8")
            self.assertIn("notes", description)
            self.assertNotIn("not notes", description)
            self.assertIn("scoring", description.lower())
            self.assertIn("Evergreen Standalone Installer", description)
            self.assertIn("https://developer.microsoft.com/microsoft-edge/webview2/", description)
            self.assertIn("does not download, install, or bundle WebView2", description)
            self.assertNotIn("Scoring and a full dictionary are not included", description)
        after = self.files()
        self.assertTrue(all(after[path] == content for path, content in before.items()))

    def test_pre_1980_notice_is_packaged_without_changing_source(self):
        notice = self.root / "resources/mdictlib-0.2.8/licenses/mdictlib-0.2.8.txt"
        os.utime(notice, (0, 0))
        timestamp = notice.stat().st_mtime_ns
        result = package(self.root)
        with zipfile.ZipFile(result["archive"]) as zipped:
            path = self.name + "/mdx-notices/licenses/mdictlib-0.2.8.txt"
            self.assertEqual(zipped.read(path), b"MDX parser license")
            self.assertEqual(zipped.getinfo(path).date_time, (1980, 1, 1, 0, 0, 0))
        self.assertEqual(notice.read_bytes(), b"MDX parser license")
        self.assertEqual(notice.stat().st_mtime_ns, timestamp)

    def test_existing_delivery_archive_or_checksum_is_preserved(self):
        for suffix in ["/keep.txt", ".zip", ".zip.sha256"]:
            with self.subTest(suffix=suffix):
                path = "target/portable/" + self.name + suffix
                self.write(path, b"preserve existing artifact")
                before = self.files()
                with self.assertRaises(FileExistsError):
                    package(self.root)
                self.assertEqual(self.files(), before)
                (self.root / path).unlink()
                if suffix.startswith("/"):
                    (self.output / self.name).rmdir()

    def test_mismatched_dictionary_fails_before_creating_delivery(self):
        self.write("target/release/dictionary/dictionary.sqlite3", b"changed resource")
        before = self.files()
        with self.assertRaisesRegex(ValueError, "dictionary"):
            package(self.root)
        self.assertEqual(self.files(), before)

    def test_mismatched_conjugation_fails_before_creating_delivery(self):
        self.write("target/release/conjugation/conjugation.sqlite3", b"changed conjugation")
        before = self.files()
        with self.assertRaisesRegex(ValueError, "conjugation"):
            package(self.root)
        self.assertEqual(self.files(), before)
        self.assertFalse(self.output.exists())

    def test_mismatched_bundled_mdx_fails_before_creating_delivery(self):
        self.write("resources/freedict-fra-eng-mdx-0.4.1/freedict-fra-eng-0.4.1.mdx", b"changed MDX")
        before = self.files()
        with self.assertRaisesRegex(ValueError, "FreeDict MDX"):
            package(self.root)
        self.assertEqual(self.files(), before)
        self.assertFalse(self.output.exists())

    def test_mismatched_bundled_notice_or_corresponding_source_preserves_inputs(self):
        for name in ("resources/freedict-fra-eng-mdx-0.4.1/COPYING",
                     "target/release/conjugation/freedict-fra-eng-0.4.1/AUTHORS"):
            with self.subTest(name=name):
                path = self.root / name
                original = path.read_bytes()
                path.write_bytes(b"changed notice")
                before = self.files()
                with self.assertRaises(ValueError):
                    package(self.root)
                self.assertEqual(self.files(), before)
                self.assertFalse(self.output.exists())
                path.write_bytes(original)

    def test_invalid_executable_and_wrong_architecture_preserve_inputs(self):
        executable = self.root / "target/release/lexique-desktop.exe"
        original = executable.read_bytes()
        arm64 = bytearray(original)
        struct.pack_into("<H", arm64, 0x84, 0xAA64)
        for binary in [b"invalid" * 40, arm64]:
            with self.subTest(binary=binary[:2]):
                executable.write_bytes(binary)
                before = self.files()
                with self.assertRaises(ValueError):
                    package(self.root)
                self.assertEqual(self.files(), before)
                self.assertFalse(self.output.exists())

    def test_mismatched_frequency_or_notice_fails_before_creating_delivery(self):
        for name in ("frequency.sqlite3", "COPYING", "frequency_sample.py"):
            with self.subTest(name=name):
                path = self.root / "target/release/frequency" / name
                original = path.read_bytes()
                path.write_bytes(b"changed frequency resource")
                before = self.files()
                with self.assertRaisesRegex(ValueError, "frequency"):
                    package(self.root)
                self.assertEqual(self.files(), before)
                self.assertFalse(self.output.exists())
                path.write_bytes(original)

    def test_separate_output_keeps_default_delivery(self):
        self.write("target/portable/" + self.name + ".zip", b"old delivery")
        result = package(self.root, self.root / "new delivery")
        self.assertTrue(Path(result["archive"]).is_file())
        self.assertEqual((self.output / (self.name + ".zip")).read_bytes(), b"old delivery")

    def test_arm64_package_checks_architecture_and_preserves_dictionary(self):
        binary = bytearray((self.root / "target/release/lexique-desktop.exe").read_bytes())
        struct.pack_into("<H", binary, 0x84, 0xAA64)
        self.write("target/aarch64-pc-windows-msvc/release/lexique-desktop.exe", binary)
        self.write("target/aarch64-pc-windows-msvc/release/dictionary/dictionary.sqlite3", b"verified dictionary")
        self.write("target/aarch64-pc-windows-msvc/release/conjugation/conjugation.sqlite3", b"verified conjugation")
        self.write("target/aarch64-pc-windows-msvc/release/conjugation/LICENSE-LGPLLR", b"source forms license")
        self.write("target/aarch64-pc-windows-msvc/release/conjugation/grammalecte-2.3.0/LICENSE-GPL3", b"metadata license")
        shutil.copytree(self.root / "resources/freedict-fra-eng-0.4.1",
                        self.root / "target/aarch64-pc-windows-msvc/release/conjugation/freedict-fra-eng-0.4.1")
        self.write("target/aarch64-pc-windows-msvc/release/frequency/frequency.sqlite3", b"verified frequency")
        self.write("target/aarch64-pc-windows-msvc/release/frequency/COPYING", b"frequency data license")
        self.write("target/aarch64-pc-windows-msvc/release/frequency/frequency_sample.py", b"frequency converter source")
        result = package(self.root, architecture="arm64")
        with zipfile.ZipFile(result["archive"]) as zipped:
            prefix = "Lexique-0.1.0-windows-arm64-portable/"
            self.assertEqual(zipped.read(prefix + "dictionary/dictionary.sqlite3"), b"verified dictionary")
            self.assertIn("Windows 11 ARM64", zipped.read(prefix + "README.txt").decode("utf-8"))
            self.assertFalse(any("Lexique-data/" in name for name in zipped.namelist()))

    def test_arm64_rejects_x64_binary_without_creating_delivery(self):
        binary = (self.root / "target/release/lexique-desktop.exe").read_bytes()
        self.write("target/aarch64-pc-windows-msvc/release/lexique-desktop.exe", binary)
        before = self.files()
        with self.assertRaisesRegex(ValueError, "arm64"):
            package(self.root, architecture="arm64")
        self.assertEqual(self.files(), before)
        self.assertFalse(self.output.exists())


if __name__ == "__main__":
    unittest.main()
