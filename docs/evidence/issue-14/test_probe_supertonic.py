"""Check the Supertonic trial through its command-line interface."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest
import wave


SCRIPT = Path(__file__).with_name("probe_supertonic.py")


class SupertonicProbeTests(unittest.TestCase):
    def run_probe(self, assets, output):
        return subprocess.run(
            [sys.executable, "-X", "utf8", str(SCRIPT), str(assets), str(output)],
            capture_output=True, text=True, encoding="utf-8", timeout=120,
        )

    def test_missing_assets_fail_without_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result = self.run_probe(root / "missing", root / "output")
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("duration_predictor.int8.onnx", result.stderr)
            self.assertFalse((root / "output").exists())

    def test_corrupt_asset_fails_without_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            assets = root / "assets"
            assets.mkdir()
            for name in ("duration_predictor.int8.onnx", "text_encoder.int8.onnx",
                         "vector_estimator.int8.onnx", "vocoder.int8.onnx", "tts.json",
                         "unicode_indexer.bin", "voice.bin"):
                (assets / name).write_bytes(b"corrupt")
            result = self.run_probe(assets, root / "output")
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("resource checksum mismatch: duration_predictor.int8.onnx", result.stderr)
            self.assertFalse((root / "output").exists())

    def test_existing_output_is_preserved(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            output = root / "output"
            output.mkdir()
            evidence = output / "keep.txt"
            evidence.write_text("keep", encoding="utf-8")
            result = self.run_probe(root / "missing", output)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("Preserving previous trial", result.stderr)
            self.assertEqual(evidence.read_text(encoding="utf-8"), "keep")

    def test_optimized_python_rejects_invalid_inputs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            evidence = root / "docs/evidence/issue-14"
            evidence.mkdir(parents=True)
            (root / "scripts").mkdir()
            for name in ("probe_supertonic.py", "supertonic3-model.json"):
                shutil.copy2(SCRIPT.with_name(name), evidence / name)
            shutil.copy2(SCRIPT.parents[3] / "scripts/speech_probe.py", root / "scripts/speech_probe.py")
            (evidence / "inputs.json").write_text("[]", encoding="utf-8")
            result = subprocess.run(
                [sys.executable, "-O", str(evidence / "probe_supertonic.py"),
                 str(root / "assets"), str(root / "output")],
                capture_output=True, text=True, encoding="utf-8", timeout=120,
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("Invalid trial inputs", result.stderr)

    @unittest.skipUnless(os.environ.get("LEXIQUE_SUPERTONIC_ASSETS"), "requires pinned assets and restricted Windows network")
    def test_real_offline_generation_preserves_inputs_and_produces_audio(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "output"
            result = self.run_probe(Path(os.environ["LEXIQUE_SUPERTONIC_ASSETS"]), output)
            self.assertEqual(result.returncode, 0, result.stderr)
            report = json.loads((output / "report.json").read_text(encoding="utf-8"))
            inputs = json.loads(SCRIPT.with_name("inputs.json").read_text(encoding="utf-8"))
            self.assertEqual([case["input"] for case in report["cases"][:17]], inputs)
            self.assertEqual([case["input"] for case in report["cases"][19:36]], inputs)
            for phase in ("network_denials_before", "network_denials_after"):
                self.assertEqual([item["winerror"] for item in report[phase]], [10013, 10013])
            for case in report["cases"]:
                with wave.open(str(output / case["file"]), "rb") as audio:
                    self.assertEqual((audio.getnchannels(), audio.getsampwidth(), audio.getframerate()), (1, 2, 44100))
                    self.assertTrue(any(audio.readframes(audio.getnframes())))


if __name__ == "__main__":
    unittest.main()
