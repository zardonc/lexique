"""Exercise packaging with temporary build outputs, never the user's delivery."""

import hashlib
import json
from pathlib import Path
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
        self.write("target/desktop-dictionary-sample/report.json", json.dumps({
            "sqlite_sha256": hashlib.sha256(b"verified dictionary").hexdigest(),
        }).encode())
        self.write("apps/desktop/src-tauri/tauri.conf.json", b'{"version":"0.1.0"}')
        self.write("resources/freedict-fra-zho-2025.11.23/COPYING", b"original resource license")
        self.write("LICENSE", b"project license")

    def write(self, name, content):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)

    def files(self):
        return {p.relative_to(self.root).as_posix(): p.read_bytes()
                for p in self.root.rglob("*") if p.is_file()}

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
            self.assertEqual(zipped.read(prefix + "LICENSE"), b"project license")
            self.assertFalse(any("Lexique-data" in path for path in zipped.namelist()))
            description = zipped.read(prefix + "README.txt").decode("utf-8")
            self.assertIn("notes", description)
            self.assertNotIn("not notes", description)
        after = self.files()
        self.assertTrue(all(after[path] == content for path, content in before.items()))

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
        self.assertFalse(self.output.exists())

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

    def test_separate_output_keeps_default_delivery(self):
        self.write("target/portable/" + self.name + ".zip", b"old delivery")
        result = package(self.root, self.root / "new delivery")
        self.assertTrue(Path(result["archive"]).is_file())
        self.assertEqual((self.output / (self.name + ".zip")).read_bytes(), b"old delivery")


if __name__ == "__main__":
    unittest.main()
