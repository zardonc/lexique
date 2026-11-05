"""Check the isolated evidence package without touching existing deliveries."""

import hashlib
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path
import unittest
import zipfile


SCRIPT = Path(__file__).with_name("prepare_supertonic.py")


class PrepareSupertonicTests(unittest.TestCase):
    @unittest.skipUnless(os.environ.get("LEXIQUE_SUPERTONIC_BUNDLE"), "no evidence bundle supplied")
    def test_native_components_report_pinned_source_revision(self):
        root = Path(os.environ["LEXIQUE_SUPERTONIC_BUNDLE"])
        program = """
import ctypes
import pathlib
import sherpa_onnx
import sys
library = ctypes.WinDLL(str(pathlib.Path(sys.argv[1]).resolve()), winmode=0)
names = ('SherpaOnnxGetVersionStr', 'SherpaOnnxGetGitSha1', 'SherpaOnnxGetGitDate')
for name in names:
    getattr(library, name).restype = ctypes.c_char_p
native = [getattr(library, name)().decode() for name in names]
assert [sherpa_onnx.__version__, sherpa_onnx.git_sha1, sherpa_onnx.git_date] == native
assert native[0] == '2.0.1' and native[1] == 'af2d8d81'
"""
        library = root / "runtime/Lib/site-packages/sherpa_onnx/lib/sherpa-onnx-c-api.dll"
        result = subprocess.run([str(root / "runtime/python.exe"), "-c", program, str(library)],
                                capture_output=True, text=True, encoding="utf-8", timeout=60)
        self.assertEqual(result.returncode, 0, result.stderr)

    @unittest.skipUnless(os.environ.get("LEXIQUE_SUPERTONIC_BUNDLE"), "no evidence bundle supplied")
    def test_bundle_has_one_runtime_copy_and_valid_inventory(self):
        root = Path(os.environ["LEXIQUE_SUPERTONIC_BUNDLE"])
        library = root / "runtime/Lib/site-packages/sherpa_onnx/lib/onnxruntime.dll"
        self.assertTrue(library.is_file())
        self.assertFalse((root / "runtime/Lib/site-packages/sherpa_onnx_core-2.0.1.data").exists())
        with zipfile.ZipFile(root.with_suffix(".zip")) as package:
            inventory = json.loads(package.read("inventory.json"))
            self.assertEqual(set(package.namelist()), set(inventory) | {"inventory.json"})
            for name in ("supertonic-static-comparison.md", "compare_onnx_static.py",
                         "test_compare_onnx_static.py"):
                self.assertIn("docs/evidence/issue-14/" + name, inventory)
            license_dir = "docs/evidence/issue-14/supertonic-licenses/"
            for name in ("NOTICE-CONSERVATIVE.md", "ATTRIBUTION.md", "sources.json"):
                current = SCRIPT.parent / "supertonic-licenses" / name
                self.assertEqual(inventory.get(license_dir + name), hashlib.sha256(current.read_bytes()).hexdigest(), name)
            sources = json.loads((SCRIPT.parent / "supertonic-licenses/sources.json").read_text(encoding="utf-8"))
            for item in sources["evidence"]:
                self.assertEqual(inventory.get(license_dir + item["file"]), item["actual_sha256"], item["file"])
            for name in ("NOTICE-DARTS-CLONE.txt", "NOTICE-THREADPOOL.txt", "LICENSE-HCLUST-CPP.txt", "LICENSE-LIBTOMMATH-TCL-9.0.4.0.txt", "CPYTHON-3.14.10-LICENSES.rst", "LICENSE-UTF8PROC-2.12.0.md", "LICENSE-PYBIND11-3.0.0.txt",
                         "LICENSE-NLOHMANN-JSON-3.12.0.txt", "LICENSE-SIMPLE-SENTENCEPIECE-0.7.txt",
                         "COPYING-KISSFFT.txt", "LICENSE-KISSFFT-BSD-3-Clause.txt",
                         "LICENSE-KALDIFST-1.8.0.txt", "COPYING-OPENFST.txt", "AUTHORS-OPENFST.txt"):
                self.assertTrue(license_dir + name in inventory, name)
            for name, expected in inventory.items():
                self.assertEqual(hashlib.sha256(package.read(name)).hexdigest(), expected, name)

    def run_prepare(self, root):
        return subprocess.run(
            [sys.executable, str(SCRIPT), str(root), str(root), str(root), str(root / "delivery")],
            capture_output=True, text=True, encoding="utf-8", timeout=60,
        )

    def test_missing_inputs_do_not_create_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result = self.run_prepare(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertFalse((root / "delivery").exists())
            self.assertFalse((root / "delivery.zip").exists())

    def test_existing_output_is_preserved(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            output = root / "delivery"
            output.mkdir()
            (output / "keep").write_text("keep", encoding="utf-8")
            result = self.run_prepare(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual((output / "keep").read_text(encoding="utf-8"), "keep")

    def test_existing_archive_is_preserved(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            archive = root / "delivery.zip"
            archive.write_bytes(b"keep")
            result = self.run_prepare(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(archive.read_bytes(), b"keep")
            self.assertFalse((root / "delivery").exists())

    def test_corrupt_runtime_does_not_create_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "python-3.14.10-embed-amd64.zip").write_bytes(b"corrupt")
            result = self.run_prepare(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("checksum mismatch", result.stderr)
            self.assertFalse((root / "delivery").exists())


if __name__ == "__main__":
    unittest.main()
