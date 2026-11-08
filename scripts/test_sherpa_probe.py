"""Check the standalone Sherpa probe through its CLI and saved evidence."""

import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
import wave
from pathlib import Path


SCRIPT = Path(__file__).with_name("sherpa_probe.py")


class SherpaProbeTests(unittest.TestCase):
    def run_probe(self, assets, output, *options):
        return subprocess.run(
            [sys.executable, "-X", "utf8", str(SCRIPT), str(assets), str(output), *options],
            capture_output=True, text=True, encoding="utf-8", timeout=60,
        )

    def test_missing_resource_is_reported_without_creating_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result = self.run_probe(root / "missing", root / "output")
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("missing resource: fr_FR-mls-medium.onnx", result.stderr)
            self.assertFalse((root / "output").exists())

    @unittest.skipUnless(os.environ.get("LEXIQUE_SHERPA_ASSETS"), "requires pinned MLS assets and a network-denied Windows process")
    def test_real_offline_generation_and_playback_preserve_exact_inputs(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "output"
            result = self.run_probe(Path(os.environ["LEXIQUE_SHERPA_ASSETS"]), output,
                                    "--require-network-denied", "--play")
            self.assertEqual(result.returncode, 0, result.stderr)
            report = json.loads((output / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(report["status"], "passed")
            self.assertEqual([case["input"] for case in report["cases"]], [
                "être", "sommes", "œil", "école", "aujourd'hui",
                "nous sommes ici", "les amis arrivent", "J'apprends le français tous les jours.",
            ])
            for phase in ("network_denials_before", "network_denials_after"):
                self.assertEqual([item["winerror"] for item in report[phase]], [10013, 10013])
            for case in report["cases"]:
                self.assertTrue(case["played"])
                with wave.open(str(output / case["file"]), "rb") as audio:
                    self.assertEqual((audio.getnchannels(), audio.getsampwidth(), audio.getframerate()), (1, 2, 22050))
                    self.assertGreater(audio.getnframes(), 0)
                    self.assertTrue(any(audio.readframes(audio.getnframes())))

    def test_existing_output_is_preserved(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            output = root / "output"
            output.mkdir()
            saved = output / "report.json"
            saved.write_text("previous evidence", encoding="utf-8")
            result = self.run_probe(root / "missing", output)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("Preserving previous trial", result.stderr)
            self.assertEqual(saved.read_text(encoding="utf-8"), "previous evidence")

    def test_corrupt_resource_fails_before_creating_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            assets = root / "assets"
            assets.mkdir()
            for name in ("fr_FR-mls-medium.onnx", "fr_FR-mls-medium.onnx.json", "tokens.txt"):
                (assets / name).write_bytes(b"corrupt")
            result = self.run_probe(assets, root / "output")
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("resource checksum mismatch: fr_FR-mls-medium.onnx", result.stderr)
            self.assertFalse((root / "output").exists())

    @unittest.skipUnless(os.environ.get("LEXIQUE_SHERPA_ASSETS"), "requires pinned MLS assets")
    def test_altered_common_input_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            scripts = root / "scripts"
            scripts.mkdir()
            for name in ("sherpa_probe.py", "speech_probe.py"):
                shutil.copyfile(SCRIPT.with_name(name), scripts / name)
            evidence = root / "docs/evidence/issue-5"
            evidence.mkdir(parents=True)
            original = SCRIPT.parents[1] / "docs/evidence/issue-5"
            shutil.copyfile(original / "resources.json", evidence / "resources.json")
            inputs = json.loads((original / "inputs.json").read_text(encoding="utf-8"))
            inputs[0] = "être."
            (evidence / "inputs.json").write_text(json.dumps(inputs), encoding="utf-8")
            output = root / "output"
            result = subprocess.run(
                [sys.executable, "-X", "utf8", str(scripts / "sherpa_probe.py"),
                 os.environ["LEXIQUE_SHERPA_ASSETS"], str(output)],
                capture_output=True, text=True, encoding="utf-8", timeout=60,
            )
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("trial input checksum mismatch", result.stderr)
            self.assertFalse(output.exists())


if __name__ == "__main__":
    unittest.main()
