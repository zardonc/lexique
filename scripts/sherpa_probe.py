"""Independent French Sherpa/Piper probe; writes audio and measurement evidence."""

import argparse
import ctypes
import hashlib
import importlib.metadata
import json
import os
from pathlib import Path
import platform
import sys
import time
import wave

sys.path.insert(0, str(Path(__file__).resolve().parent))
from speech_probe import network_denials, peak_memory


EVIDENCE = Path(__file__).resolve().parents[1] / "docs/evidence/issue-5"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("assets", type=Path)
    parser.add_argument("output", type=Path, help="must not exist")
    parser.add_argument("--require-network-denied", action="store_true")
    parser.add_argument("--play", action="store_true")
    args = parser.parse_args()
    if args.output.exists():
        raise FileExistsError(f"Preserving previous trial: {args.output}")
    manifest = json.loads((EVIDENCE / "resources.json").read_text(encoding="utf-8"))
    for item in manifest["assets"]:
        path = args.assets / item["file"]
        if not path.is_file():
            raise FileNotFoundError(f"missing resource: {path.name}")
        with path.open("rb") as source:
            digest = hashlib.file_digest(source, "sha256").hexdigest()
        if path.stat().st_size != item["bytes"] or digest != item["sha256"]:
            raise ValueError(f"resource checksum mismatch: {path.name}")
    input_bytes = (EVIDENCE / "inputs.json").read_bytes()
    if hashlib.sha256(input_bytes).hexdigest() != manifest["inputs_sha256"]:
        raise ValueError("trial input checksum mismatch")
    inputs = json.loads(input_bytes)
    if not isinstance(inputs, list) or len(inputs) != 8 or any(not isinstance(text, str) or not 1 <= len(text) <= 200 for text in inputs):
        raise ValueError("Invalid trial inputs")
    config = json.loads((args.assets / "fr_FR-mls-medium.onnx.json").read_text(encoding="utf-8"))
    args.output.mkdir(parents=True)
    report = {
        "status": "failed", "engine": "sherpa-onnx", "model": "fr_FR-mls-medium",
        "speaker_id": manifest["speaker_id"], "source_speaker_id": "3698",
        "frontend_voice": manifest["frontend_voice"], "threads": 2,
        "python": platform.python_version(), "platform": platform.platform(),
        "input_version": manifest["input_version"], "inputs_sha256": manifest["inputs_sha256"],
        "offline_verified": False, "quality_status": "pending human listening",
        "assets": manifest["assets"], "cases": [],
    }
    try:
        report["network_denials_before"] = network_denials() if args.require_network_denied else []
        loading = time.perf_counter()
        import numpy as np
        import sherpa_onnx
        from piper.phonemize_espeak import EspeakPhonemizer, ESPEAK_DATA_DIR

        report["runtime_import_seconds"] = time.perf_counter() - loading
        report["packages"] = {name: importlib.metadata.version(name) for name in manifest["packages"]}
        if report["packages"] != manifest["packages"] or report["python"] != manifest["python_version"]:
            raise ValueError("runtime package version mismatch")
        from piper import espeakbridge

        library = ctypes.CDLL(espeakbridge.__file__)
        library.espeak_Info.argtypes = [ctypes.POINTER(ctypes.c_char_p)]
        library.espeak_Info.restype = ctypes.c_char_p
        report["espeak_version"] = library.espeak_Info(None).decode("ascii")
        if report["espeak_version"] != manifest["source"]["espeak_version"]:
            raise ValueError("native frontend version mismatch")
        start = time.perf_counter()
        frontend = EspeakPhonemizer(espeak_data_dir=ESPEAK_DATA_DIR)
        report["frontend_load_seconds"] = time.perf_counter() - start
        start = time.perf_counter()
        tts = sherpa_onnx.OfflineTts(sherpa_onnx.OfflineTtsConfig(
            model=sherpa_onnx.OfflineTtsModelConfig(
                vits=sherpa_onnx.OfflineTtsVitsModelConfig(
                    model=str(args.assets / "fr_FR-mls-medium.onnx"),
                    tokens=str(args.assets / "tokens.txt"),
                    noise_scale=config["inference"]["noise_scale"],
                    noise_scale_w=config["inference"]["noise_w"],
                    length_scale=config["inference"]["length_scale"],
                ), num_threads=2, provider="cpu",
            ),
        ))
        report["model_load_seconds"] = time.perf_counter() - start
        report["total_load_seconds"] = time.perf_counter() - loading
        if (tts.lang, tts.sample_rate, tts.num_speakers) != ("fr", 22050, 125):
            raise ValueError("unexpected Sherpa model metadata")
        for index, text in enumerate(inputs, 1):
            start = time.perf_counter()
            phonemes = frontend.phonemize(manifest["frontend_voice"], text)
            for sentence in phonemes:
                for phoneme in sentence:
                    if phoneme not in config["phoneme_id_map"]:
                        raise ValueError(f"unmapped French phoneme: {phoneme!r} in {text!r}")
            frontend_seconds = time.perf_counter() - start
            gen_config = sherpa_onnx.GenerationConfig()
            gen_config.sid = manifest["speaker_id"]
            gen_config.speed = 1.0
            gen_config.phoneme_codepoints = [[ord(phoneme) for phoneme in sentence] for sentence in phonemes]
            first_audio = []

            def receive_audio(samples, progress):
                if len(samples) and not first_audio:
                    first_audio.append(time.perf_counter() - start)
                return 1

            generation = time.perf_counter()
            audio = tts.generate("", gen_config, receive_audio)
            generation_seconds = time.perf_counter() - generation
            samples = np.asarray(audio.samples)
            if audio.sample_rate != 22050 or not samples.size or not np.isfinite(samples).all() or not np.any(samples):
                raise ValueError(f"invalid generated audio for {text!r}")
            path = args.output / f"{index:02}.wav"
            pcm = np.rint(np.clip(samples, -1, 1) * 32767).astype("<i2").tobytes()
            with wave.open(str(path), "wb") as wav:
                wav.setparams((1, 2, audio.sample_rate, 0, "NONE", "not compressed"))
                wav.writeframes(pcm)
            with wave.open(str(path), "rb") as wav:
                if wav.getnframes() != samples.size or not any(wav.readframes(wav.getnframes())):
                    raise ValueError(f"invalid saved WAV for {text!r}")
            case = {
                "input": text, "phonemes": ["".join(sentence) for sentence in phonemes],
                "phoneme_codepoints": gen_config.phoneme_codepoints, "file": path.name,
                "sha256": hashlib.sha256(path.read_bytes()).hexdigest(), "bytes": path.stat().st_size,
                "frontend_seconds": frontend_seconds, "generation_seconds": generation_seconds,
                "first_audio_seconds": first_audio[0] if first_audio else None,
                "audio_seconds": samples.size / audio.sample_rate,
                "peak_amplitude": float(np.max(np.abs(samples))), "played": False,
            }
            report["cases"].append(case)
            if args.play:
                import winsound

                playback = time.perf_counter()
                winsound.PlaySound(str(path.resolve()), winsound.SND_FILENAME | winsound.SND_NODEFAULT)
                case["played"] = True
                case["playback_seconds"] = time.perf_counter() - playback
        report["network_denials_after"] = network_denials() if args.require_network_denied else []
        report["offline_verified"] = args.require_network_denied
        report["status"] = "passed"
    except Exception as error:
        report["error"] = {"type": type(error).__name__, "message": str(error)}
        raise
    finally:
        report["peak_process_memory_bytes"] = peak_memory() if os.name == "nt" else None
        (args.output / "report.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": report["status"], "cases": len(report["cases"]), "output": str(args.output)}, ensure_ascii=False))


if __name__ == "__main__":
    main()
