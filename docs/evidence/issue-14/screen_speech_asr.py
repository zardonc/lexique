"""Local diagnostic comparison, not a substitute for human pronunciation review."""

import argparse
import hashlib
import json
from pathlib import Path
import re
import sys
import unicodedata
import wave

import numpy as np
import sherpa_onnx

sys.path.insert(0, str(Path(__file__).resolve().parents[3]))
from scripts.speech_probe import network_denials


def words(text):
    text = unicodedata.normalize("NFKD", text.lower().replace("œ", "oe").replace("æ", "ae"))
    text = "".join(char for char in text if unicodedata.category(char) != "Mn")
    return re.findall("[a-z]+", text)


def word_error_rate(reference, actual):
    reference, actual = words(reference), words(actual)
    previous = list(range(len(actual) + 1))
    for i, expected in enumerate(reference, 1):
        current = [i]
        for j, observed in enumerate(actual, 1):
            current.append(min(previous[j] + 1, current[-1] + 1,
                               previous[j - 1] + (expected != observed)))
        previous = current
    return previous[-1] / max(1, len(reference))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("asr_assets", type=Path)
    parser.add_argument("tts_run", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    if args.output.exists():
        raise FileExistsError(f"Preserving previous screen: {args.output}")
    assert words("école français œil") == ["ecole", "francais", "oeil"]
    assert word_error_rate("bonjour ici", "bonjour ici") == 0
    assert word_error_rate("bonjour ici", "bonjour") == 0.5
    assert word_error_rate("bonjour", "salut") == 1
    evidence = Path(__file__).resolve().parent
    manifest = json.loads((evidence / "asr-model.json").read_text(encoding="utf-8"))
    for item in manifest["files"]:
        data = (args.asr_assets / item["path"]).read_bytes()
        assert len(data) == item["bytes"] and hashlib.sha256(data).hexdigest() == item["sha256"]
    before = network_denials()
    recognizer = sherpa_onnx.OfflineRecognizer.from_whisper(
        encoder=str(args.asr_assets / "tiny-encoder.int8.onnx"),
        decoder=str(args.asr_assets / "tiny-decoder.int8.onnx"),
        tokens=str(args.asr_assets / "tiny-tokens.txt"),
        language="fr", task="transcribe", num_threads=2, provider="cpu",
    )
    text = "J'apprends le français tous les jours."
    cases = [("accepted-siwis", evidence / "comparison/siwis/08.wav", text),
             ("accepted-tom", evidence / "comparison/tom/08.wav", text),
             ("rejected-mls-female", evidence / "mls-samples/female-normal-08.wav", text),
             ("rejected-mls-male", evidence / "mls-samples/male-normal-08.wav", text)]
    generated = json.loads((args.tts_run / "report.json").read_text(encoding="utf-8"))
    cases += [(f"supertonic-{case['file']}", args.tts_run / case["file"], case["input"])
              for case in generated["cases"] if case["language"] == "fr"]
    report = {"recognizer": "Whisper tiny int8", "language": "fr", "network_denials_before": before,
              "limitation": "ASR can mishear or hallucinate; word errors are a diagnostic, not a quality admission verdict",
              "cases": []}
    for label, path, text in cases:
        with wave.open(str(path), "rb") as wav:
            assert wav.getnchannels() == 1 and wav.getsampwidth() == 2
            sample_rate = wav.getframerate()
            samples = np.frombuffer(wav.readframes(wav.getnframes()), dtype="<i2").astype(np.float32) / 32768
        stream = recognizer.create_stream()
        stream.accept_waveform(sample_rate, samples)
        recognizer.decode_stream(stream)
        result = {"label": label, "input_file": str(path), "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                  "expected": text, "recognized": stream.result.text,
                  "word_error_rate": word_error_rate(text, stream.result.text)}
        report["cases"].append(result)
        print(json.dumps(result, ensure_ascii=False), flush=True)
    report["network_denials_after"] = network_denials()
    args.output.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
