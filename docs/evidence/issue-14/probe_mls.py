"""Generate and play offline French MLS voice/speed comparison samples."""

import argparse
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
    from piper.config import SynthesisConfig

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("assets", type=Path)
    parser.add_argument("output", type=Path, help="must not exist")
    parser.add_argument("female_source_id")
    parser.add_argument("male_source_id")
    args = parser.parse_args()
    if args.output.exists():
        raise FileExistsError(f"Preserving existing output: {args.output}")

    evidence = Path(__file__).resolve().parent
    manifest = json.loads((evidence / "mls-model.json").read_text(encoding="utf-8"))
    files = {item["path"].rsplit("/", 1)[-1]: item for item in manifest["files"]}
    model_path = args.assets / "fr_FR-mls-medium.onnx"
    config_path = Path(str(model_path) + ".json")
    for path in (model_path, config_path):
        expected = files[path.name]
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        assert path.stat().st_size == expected["actual_bytes"] and actual == expected["actual_sha256"], path

    raw_config = json.loads(config_path.read_text(encoding="utf-8"))
    speaker_map = raw_config["speaker_id_map"]
    assert raw_config["language"]["code"] == "fr_FR" and len(speaker_map) == 125
    speaker_ids = {"female": args.female_source_id, "male": args.male_source_id}
    assert len(set(speaker_ids.values())) == 2 and all(source_id in speaker_map for source_id in speaker_ids.values())

    inputs = json.loads((evidence / "inputs.json").read_text(encoding="utf-8"))
    assert len(inputs) == 17 and all(isinstance(text, str) and text.strip() for text in inputs)
    cases = [*inputs, "[[i]]"]
    before = network_denials()
    started = time.perf_counter()
    voice = PiperVoice.load(model_path, use_cuda=False)
    normal_length_scale = voice.config.length_scale
    report = {
        "model": model_path.name,
        "model_sha256": hashlib.sha256(model_path.read_bytes()).hexdigest(),
        "packages": {item.metadata["Name"]: item.version for item in importlib.metadata.distributions()},
        "speaker_map": {
            label: {"source_id": source_id, "inference_id": speaker_map[source_id]}
            for label, source_id in speaker_ids.items()
        },
        "config": {
            "language": raw_config["language"]["code"],
            "sample_rate": raw_config["audio"]["sample_rate"],
            "speaker_count": len(speaker_map),
            "normal_length_scale": normal_length_scale,
            "slow_length_scale": normal_length_scale * 1.3,
            "noise_scale": raw_config["inference"]["noise_scale"],
            "noise_w_default": raw_config["inference"]["noise_w"],
            "noise_w_scale": 0.0,
        },
        "load_seconds": time.perf_counter() - started,
        "network_denials_before": before,
        "cases": [],
        "quality": "pending human listening; generation, playback, and duration checks do not establish voice gender or quality",
    }
    args.output.mkdir(parents=True)
    duration_by_profile = {f"{voice_label}-{speed}": 0.0 for voice_label in speaker_ids for speed in ("normal", "slow")}
    for voice_label, source_id in speaker_ids.items():
        inference_id = speaker_map[source_id]
        for speed in ("normal", "slow"):
            length_scale = normal_length_scale if speed == "normal" else normal_length_scale * 1.3
            syn_config = SynthesisConfig(
                speaker_id=inference_id,
                length_scale=length_scale,
                noise_w_scale=0.0,
            )
            profile = f"{voice_label}-{speed}"
            for index, text in enumerate(cases, start=1):
                start = time.perf_counter()
                phonemes = voice.phonemize(text)
                unknown = sorted({p for sentence in phonemes for p in sentence if p not in voice.config.phoneme_id_map})
                assert phonemes and not unknown, (text, unknown)
                filename = f"{profile}-{index:02}.wav"
                path = args.output / filename
                with wave.open(str(path), "wb") as wav:
                    voice.synthesize_wav(text, wav, syn_config=syn_config)
                generation_seconds = time.perf_counter() - start
                with wave.open(str(path), "rb") as wav:
                    assert wav.getnchannels() == 1 and wav.getsampwidth() == 2 and wav.getnframes() > 0
                    audio = wav.readframes(wav.getnframes())
                    assert any(audio), (filename, "silent WAV")
                    duration = wav.getnframes() / wav.getframerate()
                    assert wav.getframerate() == raw_config["audio"]["sample_rate"]
                duration_by_profile[profile] += duration
                report["cases"].append({
                    "input": text,
                    "purpose": "explicit /i/ phoneme control; not unchanged-text inference" if index == len(cases) else "fixed French comparison input",
                    "file": filename,
                    "voice_label_for_comparison": voice_label,
                    "source_speaker_id": source_id,
                    "inference_speaker_id": inference_id,
                    "speed": speed,
                    "length_scale": length_scale,
                    "phonemes": phonemes,
                    "generation_seconds": generation_seconds,
                    "sample_rate": raw_config["audio"]["sample_rate"],
                    "audio_seconds": duration,
                    "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                })
                winsound.PlaySound(str(path.resolve()), winsound.SND_FILENAME | winsound.SND_NODEFAULT)
                report["cases"][-1]["playback"] = "synchronous Windows PlaySound completed; human quality pending"
                print(json.dumps(report["cases"][-1], ensure_ascii=False), flush=True)

    for voice_label in speaker_ids:
        normal = duration_by_profile[f"{voice_label}-normal"]
        slow = duration_by_profile[f"{voice_label}-slow"]
        assert slow > normal, (voice_label, normal, slow)
        normal_cases = {
            case["input"]: case["audio_seconds"]
            for case in report["cases"]
            if case["voice_label_for_comparison"] == voice_label and case["speed"] == "normal"
        }
        slow_cases = {
            case["input"]: case["audio_seconds"]
            for case in report["cases"]
            if case["voice_label_for_comparison"] == voice_label and case["speed"] == "slow"
        }
        assert normal_cases.keys() == slow_cases.keys()
        assert all(slow_cases[text] > duration for text, duration in normal_cases.items()), (
            voice_label,
            {text: (normal_cases[text], slow_cases[text]) for text in normal_cases if slow_cases[text] <= normal_cases[text]},
        )
        report.setdefault("pairwise_duration_validation", {})[voice_label] = {
            "cases": len(normal_cases),
            "all_slow_exceed_normal": True,
        }
    report["aggregate_duration_by_profile"] = duration_by_profile
    report["slow_to_normal_duration_ratio"] = {
        voice_label: duration_by_profile[f"{voice_label}-slow"] / duration_by_profile[f"{voice_label}-normal"]
        for voice_label in speaker_ids
    }
    report["network_denials_after"] = network_denials()
    report["peak_working_set_bytes"] = peak_memory()
    (args.output / "report.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
