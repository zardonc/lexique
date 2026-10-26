"""Isolated Kokoro French admission probe; no desktop or learning-data access."""

import argparse
import ctypes
import hashlib
import json
import os
from pathlib import Path
import platform
import socket
import sys
import time
import unicodedata
import wave


ASSETS = {
    "kokoro-v1.0.int8.onnx": "6e742170d309016e5891a994e1ce1559c702a2ccd0075e67ef7157974f6406cb",
    "voices-v1.0.bin": "bca610b8308e8d99f32e6fe4197e7ec01679264efed0cac9140fe9c29f1fbf7d",
}


def network_denials():
    """Observe the external sandbox; this check does not install a firewall."""
    results = []
    for host in ("1.1.1.1", "8.8.8.8"):
        try:
            with socket.create_connection((host, 443), timeout=3):
                raise RuntimeError(f"network is reachable: {host}")
        except OSError as error:
            if getattr(error, "winerror", None) != 10013:
                raise RuntimeError(f"expected network access denial: {host}: {error}") from error
            results.append({"host": host, "port": 443, "winerror": error.winerror})
    return results


def peak_memory():
    from ctypes import wintypes

    class Counters(ctypes.Structure):
        _fields_ = [("cb", wintypes.DWORD), ("PageFaultCount", wintypes.DWORD)] + [
            (name, ctypes.c_size_t) for name in (
                "PeakWorkingSetSize", "WorkingSetSize", "QuotaPeakPagedPoolUsage",
                "QuotaPagedPoolUsage", "QuotaPeakNonPagedPoolUsage",
                "QuotaNonPagedPoolUsage", "PagefileUsage", "PeakPagefileUsage",
            )
        ]

    counters = Counters()
    counters.cb = ctypes.sizeof(counters)
    get_process = ctypes.windll.kernel32.GetCurrentProcess
    get_process.restype = wintypes.HANDLE
    get_info = ctypes.windll.psapi.GetProcessMemoryInfo
    get_info.argtypes = [wintypes.HANDLE, ctypes.POINTER(Counters), wintypes.DWORD]
    if not get_info(get_process(), ctypes.byref(counters), counters.cb):
        raise ctypes.WinError()
    return counters.PeakWorkingSetSize


def fixed_frontend():
    import espeakng_loader
    from kokoro_onnx.config import EspeakConfig

    library = Path(espeakng_loader.get_library_path()).resolve()
    data = Path(espeakng_loader.get_data_path()).resolve()
    digest = hashlib.sha256(library.read_bytes()).hexdigest()
    if digest != "646d387acbc7ac2aa45e3625aa00a6835ae5d446ff8b0748298c3900b4dde258":
        raise ValueError("frontend DLL checksum mismatch")
    tree = hashlib.sha256()
    for path in sorted(data.rglob("*")):
        if path.is_file():
            tree.update(path.relative_to(data).as_posix().encode("utf-8"))
            tree.update(hashlib.sha256(path.read_bytes()).digest())
    if tree.hexdigest() != "cb4cda14d18557524f3fc3e3b7a45849a7481494a1fd78493fdd00721213f26c":
        raise ValueError("frontend data checksum mismatch")
    # Preflight the exact DLL so the upstream system-library fallback cannot hide a load failure.
    loaded = ctypes.CDLL(str(library))
    loaded.espeak_Info.argtypes = [ctypes.POINTER(ctypes.c_char_p)]
    loaded.espeak_Info.restype = ctypes.c_char_p
    version = loaded.espeak_Info(None).decode("ascii")
    if version != "1.52.0":
        raise ValueError(f"unexpected frontend version: {version}")
    return EspeakConfig(str(library), str(data)), {
        "version": version, "dll_sha256": digest, "data_tree_sha256": tree.hexdigest(),
        "library": str(library), "data": str(data),
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("inputs", type=Path, help="UTF-8 JSON array of exact French inputs")
    parser.add_argument("assets", type=Path)
    parser.add_argument("output", type=Path, help="must not exist")
    parser.add_argument("--require-network-denied", action="store_true")
    parser.add_argument("--play", action="store_true", help="synchronous Windows WAV playback")
    args = parser.parse_args()
    if args.output.exists():
        raise ValueError("output already exists")
    if os.environ.get("PHONEMIZER_ESPEAK_LIBRARY"):
        raise ValueError("frontend override is not allowed")
    inputs = json.loads(args.inputs.read_text(encoding="utf-8"))
    if not isinstance(inputs, list) or not 1 <= len(inputs) <= 100:
        raise ValueError("invalid input: expected 1-100 strings")
    for text in inputs:
        if not isinstance(text, str) or not text.strip() or len(text) > 200 or any(
            char not in " '-’.!?" and not (
                unicodedata.category(char).startswith("L")
                and "LATIN" in unicodedata.name(char, "")
                or unicodedata.category(char).startswith("M")
            ) for char in text
        ):
            raise ValueError(f"invalid input: expected 1-200 French text characters: {text!r}")
    for name, expected in ASSETS.items():
        path = args.assets / name
        with path.open("rb") as resource:
            actual = hashlib.file_digest(resource, "sha256").hexdigest()
        if actual != expected:
            raise ValueError(f"resource checksum mismatch: {name}")
    if sys.platform != "win32":
        raise RuntimeError("this probe measures Windows only")
    denials = network_denials() if args.require_network_denied else []

    import importlib.metadata
    import numpy as np
    import phonemizer
    from kokoro_onnx import Kokoro

    frontend_config, frontend = fixed_frontend()
    start = time.perf_counter()
    kokoro = Kokoro(str(args.assets / "kokoro-v1.0.int8.onnx"), str(args.assets / "voices-v1.0.bin"),
                    espeak_config=frontend_config)
    report = {
        "platform": platform.platform(), "cpu": platform.processor(),
        "logical_cpus": os.cpu_count(), "python": platform.python_version(),
        "packages": {dist.metadata["Name"]: dist.version for dist in importlib.metadata.distributions()},
        "voice": "ff_siwis", "language": "fr-fr", "speed": 1.0,
        "frontend": frontend,
        "load_seconds": time.perf_counter() - start, "network_denials_before": denials,
        "cases": [],
    }
    args.output.mkdir(parents=True)
    failed = False
    for index, text in enumerate(inputs):
        case = {"input": text, "quality": "pending human listening"}
        start = time.perf_counter()
        try:
            # Upstream filters unknown phonemes silently; inspect the unfiltered frontend output.
            raw = phonemizer.phonemize(text, "fr-fr", preserve_punctuation=True, with_stress=True).strip()
            case["frontend_phonemes"] = raw
            # eSpeak's French hyphens are word/link separators, not Kokoro phoneme tokens.
            phonemes = raw.replace("-", "")
            if text == "y":
                # The isolated vocabulary pronoun is /i/, not the letter name returned by eSpeak.
                phonemes = "i"
                case["pronunciation_override"] = "French vocabulary y (pronoun/adverb): /i/"
            case["phonemes"] = phonemes
            unknown = sorted(set(phonemes) - kokoro.tokenizer.vocab.keys())
            if not phonemes or unknown:
                raise ValueError(f"empty or unsupported phonemes: {unknown!r}")
            audio, rate = kokoro.create(phonemes, voice="ff_siwis", is_phonemes=True)
            if rate != 24000 or not audio.size or not np.isfinite(audio).all() or not np.any(audio):
                raise ValueError("invalid or silent audio")
            pcm = (np.clip(audio, -1, 1) * 32767).astype("<i2").tobytes()
            path = args.output / f"{index + 1:02}.wav"
            with wave.open(str(path), "wb") as output:
                output.setparams((1, 2, rate, 0, "NONE", "not compressed"))
                output.writeframes(pcm)
            with wave.open(str(path), "rb") as output:
                if output.getnframes() != len(audio) or output.readframes(len(audio)) != pcm:
                    raise ValueError("WAV read-back mismatch")
            case.update({
                "file": path.name, "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
                "generation_seconds": time.perf_counter() - start,
                "audio_seconds": len(audio) / rate, "peak_amplitude": float(np.max(np.abs(audio))),
            })
            if args.play:
                import winsound
                start = time.perf_counter()
                winsound.PlaySound(str(path.resolve()), winsound.SND_FILENAME | winsound.SND_NODEFAULT)
                case["playback_seconds"] = time.perf_counter() - start
                case["playback"] = "PlaySound completed; acoustic quality unconfirmed"
        except Exception as error:
            case["error"] = f"{type(error).__name__}: {error}"
            failed = True
        report["cases"].append(case)
        print(json.dumps(case, ensure_ascii=False), flush=True)
    report["peak_working_set_bytes"] = peak_memory()
    report["network_denials_after"] = network_denials() if args.require_network_denied else []
    (args.output / "report.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    return int(failed)


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, ValueError, RuntimeError) as error:
        print(f"{type(error).__name__}: {error}", file=sys.stderr)
        sys.exit(1)
