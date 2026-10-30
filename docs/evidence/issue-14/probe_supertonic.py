"""Small offline listening trial, using sherpa-onnx's official Supertonic API."""

import argparse
import hashlib
import importlib.metadata
import json
from pathlib import Path
import sys
import time
import wave
import winsound

import numpy as np
import sherpa_onnx

sys.path.insert(0, str(Path(__file__).resolve().parents[3]))
from scripts.speech_probe import network_denials, peak_memory


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("assets", type=Path)
    parser.add_argument("output", type=Path, help="must not already exist")
    args = parser.parse_args()
    if args.output.exists():
        raise FileExistsError(f"Preserving previous trial: {args.output}")
    inputs = json.loads(Path(__file__).with_name("inputs.json").read_text(encoding="utf-8"))
    if not isinstance(inputs, list) or len(inputs) != 17 or not all(isinstance(text, str) and text for text in inputs):
        raise ValueError("Invalid trial inputs")
    names = {
        "duration_predictor": "duration_predictor.int8.onnx",
        "text_encoder": "text_encoder.int8.onnx",
        "vector_estimator": "vector_estimator.int8.onnx",
        "vocoder": "vocoder.int8.onnx",
        "tts_json": "tts.json",
        "unicode_indexer": "unicode_indexer.bin",
        "voice_style": "voice.bin",
    }
    paths = {key: str(args.assets / name) for key, name in names.items()}
    hashes = {name: hashlib.sha256((args.assets / name).read_bytes()).hexdigest() for name in names.values()}
    manifest = json.loads(Path(__file__).with_name("supertonic3-model.json").read_text(encoding="utf-8"))
    expected = {item["path"]: item["actual_sha256"] for item in manifest["files"]}
    for name, digest in hashes.items():
        if expected[name] != digest:
            raise ValueError(f"resource checksum mismatch: {name}")
    config = sherpa_onnx.OfflineTtsConfig(model=sherpa_onnx.OfflineTtsModelConfig(
        supertonic=sherpa_onnx.OfflineTtsSupertonicModelConfig(**paths),
        num_threads=2, provider="cpu", debug=False,
    ))
    if not config.validate():
        raise ValueError("Invalid Supertonic resources")
    before = network_denials()
    started = time.perf_counter()
    tts = sherpa_onnx.OfflineTts(config)
    if tts.num_speakers != 10:
        raise ValueError(f"Unexpected speaker count: {tts.num_speakers}")
    report = {
        "model": "sherpa-onnx-supertonic-3-tts-int8-2026-05-11",
        "source_example": "https://github.com/k2-fsa/sherpa-onnx/blob/af2d8d81dbfa61c4cea4c568da78f3982edca005/python-api-examples/supertonic-tts.py",
        "resource_sha256": hashes,
        "packages": {name: importlib.metadata.version(name) for name in ("sherpa-onnx", "sherpa-onnx-core", "numpy")},
        "load_seconds": time.perf_counter() - started,
        "network_denials_before": before,
        "quality": "not accepted; pending human listening, no automated pronunciation verdict",
        "cases": [],
    }
    args.output.mkdir(parents=True)
    for gender, sid in (("female", 0), ("male", 5)):
        for index, text in enumerate(inputs, 1):
            generate(tts, args.output, report, gender, sid, "normal", 1.0, index, text, "fr")
        generate(tts, args.output, report, gender, sid, "normal", 1.0, 18,
                 "Hello. I am learning French every day.", "en")
        generate(tts, args.output, report, gender, sid, "normal", 1.0, 19, "i grec", "fr")
    if len(report["cases"]) != 38 or not all(case["speed_parameter"] == 1.0 for case in report["cases"]):
        raise ValueError("Incomplete trial results")
    report["network_denials_after"] = network_denials()
    report["peak_working_set_bytes"] = peak_memory()
    (args.output / "report.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def generate(tts, output, report, gender, sid, speed_name, speed, index, text, lang):
    config = sherpa_onnx.GenerationConfig()
    config.sid, config.speed, config.num_steps = sid, speed, 8
    # The Python binding returns a copy; mutating its dict drops the language.
    config.extra = {"lang": lang}
    if config.extra.get("lang") != lang:
        raise ValueError("Synthesis language was not forwarded to the native runtime")
    started = time.perf_counter()
    audio = tts.generate(text, config)
    elapsed = time.perf_counter() - started
    samples = np.asarray(audio.samples)
    if samples.ndim != 1 or samples.size == 0 or not np.isfinite(samples).all():
        raise ValueError("Invalid audio output")
    if float(np.max(np.abs(samples))) == 0:
        raise ValueError("Silent output")
    filename = f"{gender}-{speed_name}-{index:02}.wav"
    path = output / filename
    with wave.open(str(path), "wb") as wav:
        wav.setparams((1, 2, audio.sample_rate, 0, "NONE", "not compressed"))
        wav.writeframes((np.clip(samples, -1, 1) * 32767).astype("<i2").tobytes())
    with wave.open(str(path), "rb") as wav:
        if wav.getnframes() != samples.size or wav.getframerate() != tts.sample_rate:
            raise ValueError("WAV readback mismatch")
    winsound.PlaySound(str(path.resolve()), winsound.SND_FILENAME | winsound.SND_NODEFAULT)
    case = {"input": text, "language": lang, "voice": gender, "sid": sid,
            "speed": speed_name, "speed_parameter": speed, "num_steps": 8,
            "file": filename, "sample_rate": audio.sample_rate,
            "audio_seconds": samples.size / audio.sample_rate,
            "generation_seconds": elapsed, "clipped_samples": int(np.count_nonzero(np.abs(samples) > 1)),
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
            "playback": "synchronous PlaySound completed; does not establish intelligibility"}
    report["cases"].append(case)
    print(json.dumps(case, ensure_ascii=False), flush=True)


if __name__ == "__main__":
    main()
