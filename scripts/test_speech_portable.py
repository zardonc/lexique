"""Check the public Windows speech-enabled portable packaging command."""

import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
import zipfile


class SpeechPortableTest(unittest.TestCase):
    def test_incomplete_inputs_and_unsupported_architecture_preserve_delivery(self):
        root = Path(__file__).resolve().parent.parent
        with tempfile.TemporaryDirectory(prefix="lexique-speech-inputs-") as temporary:
            directory = Path(temporary)
            delivery = directory / "existing.zip"
            delivery.write_bytes(b"existing delivery")
            for arguments, message in (
                (["--speech-bundle", temporary], "must be supplied together"),
                (["--architecture", "arm64", "--speech-bundle", temporary,
                  "--speech-encoder", temporary, "--speech-sources", temporary], "require Windows x64"),
            ):
                with self.subTest(arguments=arguments):
                    result = subprocess.run([
                        sys.executable, "-X", "utf8", str(root / "scripts/package_portable.py"),
                        "--output-dir", temporary, *arguments,
                    ], cwd=root, capture_output=True, text=True, timeout=30)
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn(message, result.stderr)
                    self.assertEqual(delivery.read_bytes(), b"existing delivery")
                    self.assertEqual(list(directory.iterdir()), [delivery])

    def test_invalid_resources_preserve_existing_delivery(self):
        root = Path(__file__).resolve().parent.parent
        if not (root / "target/release/lexique-desktop.exe").is_file():
            self.skipTest("A compiled Windows x64 release is required for the resource preflight")
        with tempfile.TemporaryDirectory(prefix="lexique-invalid-package-") as temporary:
            directory = Path(temporary)
            bundle = directory / "bundle"
            bundle.mkdir()
            (bundle / "inventory.json").write_text("{}", encoding="utf-8")
            delivery = directory / "existing.zip"
            delivery.write_bytes(b"existing delivery")
            result = subprocess.run([
                sys.executable, "-X", "utf8", str(root / "scripts/package_portable.py"),
                "--output-dir", temporary, "--speech-bundle", str(bundle),
                "--speech-encoder", str(directory / "encoder.dll"),
                "--speech-sources", str(directory),
            ], cwd=root, capture_output=True, text=True, timeout=30)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("inventory checksum mismatch", result.stderr)
            self.assertEqual(delivery.read_bytes(), b"existing delivery")
            self.assertEqual(list(directory.glob("Lexique*")), [])

    def test_speech_package_contains_resources_and_licenses(self):
        bundle = os.environ.get("LEXIQUE_SPEECH_BUNDLE")
        encoder = os.environ.get("LEXIQUE_OPUS_ENCODER")
        sources = os.environ.get("LEXIQUE_OPUS_SOURCES")
        if not all((bundle, encoder, sources)):
            self.skipTest("Verified speech bundle and Opus inputs are required")
        root = Path(__file__).resolve().parent.parent
        with tempfile.TemporaryDirectory(prefix="lexique-speech-package-") as temporary:
            result = subprocess.run([
                sys.executable, "-X", "utf8", str(root / "scripts/package_portable.py"),
                "--output-dir", temporary, "--speech-bundle", bundle,
                "--speech-encoder", encoder, "--speech-sources", sources,
            ], cwd=root, capture_output=True, text=True, timeout=300)
            self.assertEqual(result.returncode, 0, result.stderr)
            archives = list(Path(temporary).glob("*.zip"))
            self.assertEqual(len(archives), 1)
            with zipfile.ZipFile(archives[0]) as archive:
                names = archive.namelist()
                for suffix in ("speech/runtime/python.exe", "speech/assets/voice.bin",
                               "speech/opusenc.dll", "speech/opus-1.6.1.tar.gz.LICENSE",
                               "speech/libopusenc-0.3.tar.gz.LICENSE",
                               "speech/docs/evidence/issue-14/supertonic-licenses/LICENSE-SUPERTONE-model-OpenRAIL-M.txt"):
                    self.assertTrue(any(name.endswith("/" + suffix) for name in names), suffix)
                self.assertFalse(any("/Lexique-data/" in name for name in names))
                self.assertIsNone(archive.testzip())


if __name__ == "__main__":
    unittest.main()
