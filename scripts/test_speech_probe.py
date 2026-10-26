"""Exercise the speech probe through its public CLI, without model dependencies."""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
import wave


SCRIPT = Path(__file__).with_name("speech_probe.py")


class SpeechProbeTests(unittest.TestCase):
    def run_probe(self, root, inputs, env=None):
        source = root / "inputs.json"
        source.write_text(json.dumps(inputs), encoding="utf-8")
        return subprocess.run(
            [sys.executable, "-X", "utf8", str(SCRIPT), str(source), str(root / "assets"),
             str(root / "output")],
            capture_output=True, text=True, encoding="utf-8", timeout=10, env=env,
        )

    def test_invalid_inputs_fail_before_loading_models(self):
        for text in ("", "a\n", "a" * 201, "école🧪"):
            with self.subTest(text=text), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                result = self.run_probe(root, [text])
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("invalid input", result.stderr)
                self.assertFalse((root / "output").exists())

    def test_missing_resource_is_visible_without_creating_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result = self.run_probe(root, ["œil"])
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("kokoro-v1.0.int8.onnx", result.stderr)
            self.assertFalse((root / "output").exists())

    def test_corrupt_resource_does_not_generate_audio(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "assets").mkdir()
            (root / "assets/kokoro-v1.0.int8.onnx").write_bytes(b"corrupt model")
            result = self.run_probe(root, ["école"])
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("resource checksum mismatch", result.stderr)
            self.assertFalse((root / "output").exists())

    def test_external_frontend_override_is_rejected(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result = self.run_probe(root, ["école"], {
                **os.environ, "PHONEMIZER_ESPEAK_LIBRARY": "unverified.dll",
            })
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("frontend override is not allowed", result.stderr)
            self.assertFalse((root / "output").exists())

    @unittest.skipUnless(os.environ.get("LEXIQUE_SPEECH_BUNDLE"), "requires prepared speech bundle and restricted Windows network")
    def test_real_offline_cli_generates_readable_wav_for_word_and_unknown_spelling(self):
        bundle = Path(os.environ["LEXIQUE_SPEECH_BUNDLE"]).resolve()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "inputs.json"
            source.write_text(json.dumps(["œil", "qzxyplouf", "y"]), encoding="utf-8")
            result = subprocess.run(
                [str(bundle / "runtime/python.exe"), "-X", "utf8", str(SCRIPT), str(source),
                 str(bundle / "assets"), str(root / "output"), "--require-network-denied"],
                capture_output=True, text=True, encoding="utf-8", timeout=60,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            report = json.loads((root / "output/report.json").read_text(encoding="utf-8"))
            self.assertEqual([case["input"] for case in report["cases"]], ["œil", "qzxyplouf", "y"])
            self.assertEqual(report["cases"][2]["phonemes"], "i")
            self.assertIn("pronunciation_override", report["cases"][2])
            for case in report["cases"]:
                with wave.open(str(root / "output" / case["file"]), "rb") as audio:
                    self.assertEqual((audio.getnchannels(), audio.getsampwidth(), audio.getframerate()), (1, 2, 24000))
                    self.assertGreater(audio.getnframes(), 0)
                    self.assertTrue(any(audio.readframes(audio.getnframes())))
            for phase in ("network_denials_before", "network_denials_after"):
                self.assertEqual([item["winerror"] for item in report[phase]], [10013, 10013])

    def test_existing_evidence_is_never_overwritten(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "output").mkdir()
            evidence = root / "output" / "keep.txt"
            evidence.write_text("keep", encoding="utf-8")
            result = self.run_probe(root, ["être"])
            self.assertNotEqual(result.returncode, 0)
            self.assertIn("output already exists", result.stderr)
            self.assertEqual(evidence.read_text(encoding="utf-8"), "keep")


if __name__ == "__main__":
    unittest.main()
