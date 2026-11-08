"""Measure the admitted CPU Supertonic voices using complete French sentences."""
import argparse
import json
from pathlib import Path
import sys
import time
import wave

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "apps/desktop/src-tauri"))
from speech_resident import create_model
from speech_worker import encode
import numpy as np
import sherpa_onnx

SENTENCES = [
    "Aujourd’hui, nous allons au marché pour acheter du pain, des fruits et quelques légumes frais.",
    "Après le petit-déjeuner, Marie prend le train de huit heures et retrouve ses amis devant la bibliothèque.",
    "Quand il fait beau, les enfants jouent dans le jardin, puis toute la famille se promène au bord de la rivière.",
]


def word_count(text):
    return sum(any(character.isalnum() for character in word) for word in text.split())


def main():
    assert word_count("… l’école porte-monnaie 42 !") == 3
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bundle", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    args.output.mkdir(parents=True, exist_ok=False)
    configuration = args.output / "ort-cpu.conf"
    configuration.write_text("SessionConfig.session.intra_op.allow_spinning=0\n"
                             "SessionConfig.session.inter_op.allow_spinning=0\n", encoding="utf-8")
    model = create_model(args.bundle / "assets", str(configuration.resolve()))
    results = []
    for voice, sid in [("female", 0), ("male", 5)]:
        for index, text in enumerate(SENTENCES, 1):
            generation = sherpa_onnx.GenerationConfig()
            generation.sid, generation.speed, generation.num_steps = sid, 1.0, 8
            generation.extra = {"lang": "fr"}
            started = time.perf_counter()
            audio = model.generate(text, generation)
            elapsed = time.perf_counter() - started
            samples = np.asarray(audio.samples, dtype=np.float32)
            assert samples.size > 0 and np.isfinite(samples).all()
            stem = f"{voice}-{index}"
            (args.output / f"{stem}.ogg").write_bytes(encode(args.bundle / "opusenc.dll", samples, audio.sample_rate))
            with wave.open(str(args.output / f"{stem}.wav"), "wb") as output:
                output.setparams((1, 2, audio.sample_rate, 0, "NONE", "not compressed"))
                output.writeframes((np.clip(samples, -1, 1) * 32767).astype("<i2").tobytes())
            results.append({"voice": voice, "text": text, "words": word_count(text),
                            "frames": int(samples.size), "sample_rate": audio.sample_rate,
                            "duration_seconds": samples.size / audio.sample_rate,
                            "generation_seconds": elapsed, "file": f"{stem}.ogg"})
    baselines = {}
    for voice in ["female", "male"]:
        entries = [entry for entry in results if entry["voice"] == voice]
        baselines[voice] = sum(entry["words"] for entry in entries) * 60 / sum(entry["duration_seconds"] for entry in entries)
    report = {"engine": "Supertonic 3 int8", "cpu": True, "speed": 1.0, "steps": 8,
              "bitrate": 48000, "baselines_wpm": baselines, "samples": results}
    (args.output / "calibration.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"baselines_wpm": baselines, "samples": len(results)}))


if __name__ == "__main__":
    main()
