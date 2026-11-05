"""Local Piper comparison only; does not admit or package models for Lexique."""

import hashlib
import importlib.metadata
import json
from pathlib import Path
import sys
import time
import wave
import winsound

sys.path.insert(0, str(Path(__file__).resolve().parents[3]))
from scripts.speech_probe import network_denials, peak_memory


def main():
    from piper import PiperVoice

    model, output = map(Path, sys.argv[1:])
    evidence = Path(__file__).resolve().parent
    if output.exists():
        raise ValueError("output already exists")
    manifest = json.loads((evidence / "piper-models.json").read_text(encoding="utf-8"))
    for path in (model, Path(str(model) + ".json")):
        expected = next(item for item in manifest if item["file"] == path.name)
        assert hashlib.sha256(path.read_bytes()).hexdigest() == expected["sha256"], path
    before = network_denials()
    start = time.perf_counter()
    voice = PiperVoice.load(model, use_cuda=False)
    report = {
        "model": model.name, "load_seconds": time.perf_counter() - start,
        "packages": {item.metadata["Name"]: item.version for item in importlib.metadata.distributions()},
        "config": voice.config.to_dict(), "network_denials_before": before, "cases": [],
        "quality": "pending human comparison; not redistribution admission",
    }
    output.mkdir(parents=True)
    inputs = json.loads((evidence / "inputs.json").read_text(encoding="utf-8"))
    for index, text in enumerate([*inputs, "[[i]]"]):
        start = time.perf_counter()
        case = {"input": text, "file": f"{index + 1:02}.wav"}
        if index == len(inputs):
            case["purpose"] = "explicit /i/ phoneme control for vocabulary y; not unchanged-text inference"
        phonemes = voice.phonemize(text)
        case["phonemes"] = phonemes
        unknown = sorted({p for sentence in phonemes for p in sentence if p not in voice.config.phoneme_id_map})
        assert phonemes and not unknown, (text, unknown)
        path = output / case["file"]
        with wave.open(str(path), "wb") as wav:
            voice.synthesize_wav(text, wav)
        case["generation_seconds"] = time.perf_counter() - start
        with wave.open(str(path), "rb") as wav:
            assert wav.getnchannels() == 1 and wav.getsampwidth() == 2 and wav.getnframes() > 0
            assert any(wav.readframes(wav.getnframes())), "silent WAV"
            case["sample_rate"] = wav.getframerate()
            case["audio_seconds"] = wav.getnframes() / wav.getframerate()
        case["sha256"] = hashlib.sha256(path.read_bytes()).hexdigest()
        winsound.PlaySound(str(path.resolve()), winsound.SND_FILENAME | winsound.SND_NODEFAULT)
        case["playback"] = "synchronous Windows PlaySound completed; quality unconfirmed"
        report["cases"].append(case)
        print(json.dumps(case, ensure_ascii=False), flush=True)
    report["network_denials_after"] = network_denials()
    report["peak_working_set_bytes"] = peak_memory()
    (output / "report.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
