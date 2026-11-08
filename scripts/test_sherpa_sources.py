"""Check the source companion through its CLI and delivered archives."""

import hashlib
import io
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile
import unittest
import zipfile


SCRIPT = Path(__file__).resolve().parents[1] / "docs/evidence/issue-5/sources.py"


class SherpaSourcesTests(unittest.TestCase):
    def run_sources(self, downloads, model, output):
        return subprocess.run(
            [sys.executable, "-B", "-X", "utf8", str(SCRIPT), str(downloads), str(model), str(output)],
            capture_output=True, text=True, encoding="utf-8", timeout=60,
        )

    def test_missing_source_is_reported_before_creating_delivery(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result = self.run_sources(root / "missing", root / "model", root / "delivery")
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("piper-639388b6317fc4731e91d53da42aea68fd4166ff.tar.gz", result.stderr)
            self.assertFalse((root / "delivery").exists())
            self.assertFalse((root / "delivery.zip").exists())

    @unittest.skipUnless(os.environ.get("LEXIQUE_SHERPA_SOURCE_DOWNLOADS") and
                         os.environ.get("LEXIQUE_SHERPA_MODEL_SOURCE"), "requires pinned source materials")
    def test_delivered_sources_include_native_build_inputs_and_original_model(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "delivery"
            result = self.run_sources(Path(os.environ["LEXIQUE_SHERPA_SOURCE_DOWNLOADS"]),
                                      Path(os.environ["LEXIQUE_SHERPA_MODEL_SOURCE"]), output)
            self.assertEqual(result.returncode, 0, result.stderr)
            with zipfile.ZipFile(output.with_suffix(".zip")) as archive:
                self.assertIsNone(archive.testzip())
                self.assertTrue({
                    "README.md", "inventory.json", "model/fr_FR-tom-medium.onnx",
                    "model/fr_FR-tom-medium.onnx.json", "model/MODEL_CARD",
                    "upstream/fr_FR-tom-author.onnx.json", "upstream/eigen-5.0.1.tar.gz",
                    "upstream/sherpa-onnx-af2d8d81dbfa61c4cea4c568da78f3982edca005.tar.gz",
                    "project/LICENSE", "project/scripts/sherpa_probe.py", "project/scripts/speech_probe.py",
                    "project/docs/evidence/issue-5/prepare.py", "project/docs/evidence/issue-5/bundle.py",
                    "project/docs/evidence/issue-5/source-build.md",
                    "project/docs/evidence/issue-5/piper-local-sources.patch",
                    "project/docs/evidence/issue-5/licenses/LICENSE-TOM-AGPL-3.0.txt",
                    "project/docs/evidence/issue-5/licenses/TOM-README.md",
                }.issubset(archive.namelist()))
                self.assertEqual(hashlib.sha256(archive.read("model/fr_FR-tom-medium.onnx")).hexdigest(),
                                 "bf65074ccdeeeeaa832e75edb1c0a513c01c9a972bdf085ff8a6e71ea234fd41")
                for name, required in (
                    ("piper-639388b6317fc4731e91d53da42aea68fd4166ff", {
                        "CMakeLists.txt", "src/piper/espeakbridge.c", "setup.py", "pyproject.toml",
                        ".github/workflows/wheels.yml", "COPYING"}),
                    ("espeak-ng-724808c5a83f9ef95fdd0db886ba7ba537ff224a", {
                        "CMakeLists.txt", "src/ucd-tools/CMakeLists.txt", "src/ucd-tools/COPYING.UCD",
                        "dictsource/fr_list", "dictsource/fr_rules", "phsource/phonemes", "COPYING"}),
                    ("sonic-fbf75c3d6d846bad3bb3d456cbc5d07d9fd8c104", {"sonic.c", "sonic.h", "LICENSE"}),
                ):
                    with tarfile.open(fileobj=io.BytesIO(archive.read(f"upstream/{name}.tar.gz"))) as source:
                        prefix = name.replace("piper-", "piper1-gpl-", 1) if name.startswith("piper-") else name
                        self.assertTrue({f"{prefix}/{path}" for path in required}.issubset(source.getnames()), name)
            measured = json.loads(output.with_suffix(".measurements.json").read_text(encoding="utf-8"))
            self.assertEqual(measured["package"]["zip_bytes"], output.with_suffix(".zip").stat().st_size)

    def test_existing_directory_zip_or_measurements_are_preserved(self):
        for suffix in ("", ".zip", ".measurements.json"):
            with self.subTest(suffix=suffix), tempfile.TemporaryDirectory() as directory:
                output = Path(directory) / "delivery"
                saved = output.with_name(output.name + suffix)
                if not suffix:
                    saved.mkdir()
                    saved /= "previous.txt"
                saved.write_bytes(b"previous evidence")
                result = self.run_sources(output.parent / "missing", output.parent / "model", output)
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("Preserving source delivery", result.stderr)
                self.assertEqual(saved.read_bytes(), b"previous evidence")

    @unittest.skipUnless(os.environ.get("LEXIQUE_SHERPA_SOURCE_DOWNLOADS"), "requires pinned source archive")
    def test_corrupt_same_size_source_fails_before_creating_delivery(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source_name = "piper-639388b6317fc4731e91d53da42aea68fd4166ff.tar.gz"
            shutil.copyfile(Path(os.environ["LEXIQUE_SHERPA_SOURCE_DOWNLOADS"]) / source_name, root / source_name)
            with (root / source_name).open("r+b") as source:
                value = source.read(1)
                source.seek(0)
                source.write(bytes([value[0] ^ 1]))
            result = self.run_sources(root, root / "model", root / "delivery")
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("source resource checksum mismatch", result.stderr)
            self.assertFalse((root / "delivery").exists())
            self.assertFalse((root / "delivery.zip").exists())


if __name__ == "__main__":
    unittest.main()
