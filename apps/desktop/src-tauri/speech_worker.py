"""Synthesize one word using the packaged offline Supertonic runtime."""

import ctypes
import hashlib
import json
from pathlib import Path
import sys

import numpy as np
import sherpa_onnx

ASSETS = {
    "duration_predictor": ("duration_predictor.int8.onnx", "c3eb91414d5ff8a7a239b7fe9e34e7e2bf8a8140d8375ffb14718b1c639325db"),
    "text_encoder": ("text_encoder.int8.onnx", "c7befd5ea8c3119769e8a6c1486c4edc6a3bc8365c67621c881bbb774b9902ff"),
    "vector_estimator": ("vector_estimator.int8.onnx", "20cd86fa5c6effedfda0e7cffe5b0569ca401c440a0c3a1d72bf39286c0db3fd"),
    "vocoder": ("vocoder.int8.onnx", "e923d60f53f95eb1ce235f1dc33ec56d9c057823c96fa6f8acf98f32b0da6152"),
    "tts_json": ("tts.json", "42078d3aef1cd43ab43021f3c54f47d2d75ceb4e75f627f118890128b06a0d09"),
    "unicode_indexer": ("unicode_indexer.bin", "8402ca48e5189a8950138580b0fff64db6f072f24ac07cd54ba8b2fbb9883b30"),
    "voice_style": ("voice.bin", "67d5209b0ee8ce6c74105ffbe12fe6a7628aea3b4ba2fcb308a4a67938a93ce8"),
}


def encode(library_path, samples, rate):
    library = ctypes.CDLL(str(library_path))
    library.ope_comments_create.restype = ctypes.c_void_p
    library.ope_comments_destroy.argtypes = [ctypes.c_void_p]
    library.ope_encoder_create_pull.argtypes = [ctypes.c_void_p, ctypes.c_int, ctypes.c_int,
                                                ctypes.c_int, ctypes.POINTER(ctypes.c_int)]
    library.ope_encoder_create_pull.restype = ctypes.c_void_p
    library.ope_encoder_ctl.argtypes = [ctypes.c_void_p, ctypes.c_int]
    library.ope_encoder_write_float.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_float), ctypes.c_int]
    library.ope_encoder_drain.argtypes = [ctypes.c_void_p]
    library.ope_encoder_get_page.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_void_p),
                                             ctypes.POINTER(ctypes.c_int32), ctypes.c_int]
    library.ope_encoder_destroy.argtypes = [ctypes.c_void_p]
    comments = library.ope_comments_create()
    if not comments:
        raise RuntimeError("Could not initialize Opus comments")
    encoder = None
    try:
        error = ctypes.c_int()
        encoder = library.ope_encoder_create_pull(comments, rate, 1, 0, ctypes.byref(error))
        if not encoder or error.value:
            raise RuntimeError(f"Could not create Opus encoder: {error.value}")
        for operation, call in (
            ("set bitrate", lambda: library.ope_encoder_ctl(encoder, 4002, ctypes.c_int(48000))),
            ("write samples", lambda: library.ope_encoder_write_float(
                encoder, samples.ctypes.data_as(ctypes.POINTER(ctypes.c_float)), samples.size)),
            ("finish stream", lambda: library.ope_encoder_drain(encoder)),
        ):
            result = call()
            if result:
                raise RuntimeError(f"Could not {operation}: {result}")
        pages = []
        pointer, length = ctypes.c_void_p(), ctypes.c_int32()
        while library.ope_encoder_get_page(encoder, ctypes.byref(pointer), ctypes.byref(length), 0) == 1:
            if length.value <= 0 or length.value > 1_000_000:
                raise RuntimeError("Invalid Opus page")
            pages.append(ctypes.string_at(pointer, length.value))
        result = b"".join(pages)
        if not result.startswith(b"OggS") or len(result) > 10_000_000:
            raise RuntimeError("Invalid Opus output")
        return result
    finally:
        if encoder:
            library.ope_encoder_destroy(encoder)
        library.ope_comments_destroy(comments)


def main():
    if len(sys.argv) != 3:
        raise ValueError("Expected model directory and encoder library")
    request = json.loads(sys.stdin.buffer.read(8193).decode("utf-8"))
    if not isinstance(request, dict) or set(request) != {"text", "voice"}:
        raise ValueError("Invalid synthesis request")
    text, voice = request["text"], request["voice"]
    if not isinstance(text, str) or not text.strip() or len(text) > 200 or any(ord(c) < 32 or 127 <= ord(c) <= 159 for c in text):
        raise ValueError("Invalid headword")
    if voice not in ("female", "male"):
        raise ValueError("Invalid voice")
    paths = {}
    for key, (name, expected) in ASSETS.items():
        path = Path(sys.argv[1]) / name
        with path.open("rb") as file:
            actual = hashlib.file_digest(file, "sha256").hexdigest()
        if actual != expected:
            raise ValueError(f"Model checksum mismatch: {name}")
        paths[key] = str(path)
    config = sherpa_onnx.OfflineTtsConfig(model=sherpa_onnx.OfflineTtsModelConfig(
        supertonic=sherpa_onnx.OfflineTtsSupertonicModelConfig(**paths), num_threads=2, provider="cpu", debug=False))
    if not config.validate():
        raise ValueError("Invalid Supertonic resources")
    tts = sherpa_onnx.OfflineTts(config)
    if tts.num_speakers != 10:
        raise ValueError("Unexpected speaker count")
    generation = sherpa_onnx.GenerationConfig()
    generation.sid, generation.speed, generation.num_steps = (0 if voice == "female" else 5), 1.0, 8
    generation.extra = {"lang": "fr"}
    if generation.extra.get("lang") != "fr":
        raise RuntimeError("Synthesis language was not forwarded")
    audio = tts.generate("i grec" if text == "y" else text, generation)
    if audio.sample_rate != 44_100 or tts.sample_rate != audio.sample_rate:
        raise ValueError("Unexpected synthesized sample rate")
    samples = np.ascontiguousarray(audio.samples, dtype=np.float32)
    if samples.ndim != 1 or not 0 < samples.size <= 44_100 * 120 or not np.isfinite(samples).all() or not np.any(samples):
        raise ValueError("Invalid synthesized audio")
    sys.stdout.buffer.write(encode(Path(sys.argv[2]), samples, audio.sample_rate))


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(f"Speech generation failed: {error}", file=sys.stderr)
        sys.exit(1)
