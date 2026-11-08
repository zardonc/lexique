"""Check the standalone Sherpa probe through its CLI and saved evidence."""

import hashlib
from datetime import date
import json
import os
import platform
import re
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
            self.assertIn("missing resource: fr_FR-tom-medium.onnx", result.stderr)
            self.assertFalse((root / "output").exists())

    @unittest.skipUnless(os.environ.get("LEXIQUE_SHERPA_ASSETS") and os.environ.get("LEXIQUE_SHERPA_ASR_ASSETS"),
                         "requires pinned speech and ASR assets in a network-denied Windows process")
    def test_real_offline_generation_and_playback_preserve_exact_inputs(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "output"
            result = self.run_probe(Path(os.environ["LEXIQUE_SHERPA_ASSETS"]), output,
                                    "--require-network-denied", "--play")
            self.assertEqual(result.returncode, 0, result.stderr)
            report = json.loads((output / "report.json").read_text(encoding="utf-8"))
            self.assertEqual(report["status"], "passed")
            self.assertEqual(report["date"], date.today().isoformat())
            self.assertEqual(report["cpu"], platform.processor())
            self.assertEqual(report["logical_cpus"], os.cpu_count())
            self.assertEqual([case["input"] for case in report["cases"]], [
                "être", "sommes", "œil", "école", "aujourd'hui",
                "nous sommes ici", "les amis arrivent", "J'apprends le français tous les jours.",
            ])
            for phase in ("network_denials_before", "network_denials_after"):
                self.assertEqual([item["winerror"] for item in report[phase]], [10013, 10013])
            for case in report["cases"]:
                self.assertTrue(case["played"])
                with wave.open(str(output / case["file"]), "rb") as audio:
                    self.assertEqual((audio.getnchannels(), audio.getsampwidth(), audio.getframerate()), (1, 2, 44100))
                    self.assertGreater(audio.getnframes(), 0)
                    # The pinned Tom decoder emits 256-sample hops; retain its quiet phonemes.
                    self.assertEqual(audio.getnframes() % 256, 0, "Retiming removed part of the VITS audio")
                    self.assertTrue(any(audio.readframes(audio.getnframes())))
            import numpy as np
            import sherpa_onnx

            asr_assets = Path(os.environ["LEXIQUE_SHERPA_ASR_ASSETS"])
            manifest = json.loads((SCRIPT.parents[1] / "docs/evidence/issue-5/asr-model.json").read_text(encoding="utf-8"))
            for item in manifest["files"]:
                data = (asr_assets / item["path"]).read_bytes()
                self.assertEqual(len(data), item["bytes"])
                self.assertEqual(hashlib.sha256(data).hexdigest(), item["sha256"])
            recognizer = sherpa_onnx.OfflineRecognizer.from_whisper(
                encoder=str(asr_assets / "tiny-encoder.int8.onnx"),
                decoder=str(asr_assets / "tiny-decoder.int8.onnx"),
                tokens=str(asr_assets / "tiny-tokens.txt"),
                language="fr", task="transcribe", num_threads=2, provider="cpu",
            )
            with wave.open(str(output / report["cases"][-1]["file"]), "rb") as audio:
                samples = np.frombuffer(audio.readframes(audio.getnframes()), dtype="<i2").astype(np.float32) / 32768
                sample_rate = audio.getframerate()
            stream = recognizer.create_stream()
            stream.accept_waveform(sample_rate, samples)
            recognizer.decode_stream(stream)
            # ASR may omit the silent final s; both spellings have the same French pronunciation.
            self.assertIn(re.findall(r"\w+", stream.result.text.casefold()), (
                ["j", "apprends", "le", "français", "tous", "les", "jours"],
                ["j", "apprend", "le", "français", "tous", "les", "jours"],
            ), f"French sentence not recognized: {stream.result.text!r}")

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
            for name in ("fr_FR-tom-medium.onnx", "fr_FR-tom-medium.onnx.json", "tokens.txt"):
                (assets / name).write_bytes(b"corrupt")
            result = self.run_probe(assets, root / "output")
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("resource checksum mismatch: fr_FR-tom-medium.onnx", result.stderr)
            self.assertFalse((root / "output").exists())

    @unittest.skipUnless(os.environ.get("LEXIQUE_SHERPA_ASSETS"), "requires pinned speech assets")
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
