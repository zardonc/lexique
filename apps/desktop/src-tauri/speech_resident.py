"""Resident CPU-only Supertonic worker. No network or learning-database access."""
import hashlib
from pathlib import Path
import sys

import numpy as np
import sherpa_onnx

from speech_worker import ASSETS, encode
from speech_protocol import serve


def create_model(assets, configuration):
    paths = {}
    for key, (name, expected) in ASSETS.items():
        path = Path(assets) / name
        with path.open("rb") as file:
            if hashlib.file_digest(file, "sha256").hexdigest() != expected:
                raise ValueError(f"Model checksum mismatch: {name}")
        paths[key] = str(path)
    expected = "SessionConfig.session.intra_op.allow_spinning=0\nSessionConfig.session.inter_op.allow_spinning=0\n"
    if Path(configuration).read_text(encoding="utf-8") != expected:
        raise ValueError("Invalid CPU session configuration")
    config = sherpa_onnx.OfflineTtsConfig(model=sherpa_onnx.OfflineTtsModelConfig(
        supertonic=sherpa_onnx.OfflineTtsSupertonicModelConfig(**paths), num_threads=2,
        provider="cpu:" + configuration, debug=False))
    if not config.validate():
        raise ValueError("Invalid Supertonic resources")
    model = sherpa_onnx.OfflineTts(config)
    if model.num_speakers != 10 or model.sample_rate != 44100:
        raise ValueError("Unexpected Supertonic model identity")
    return model


def synthesize(model, encoder, text, voice):
    generation = sherpa_onnx.GenerationConfig()
    generation.sid, generation.speed, generation.num_steps = (0 if voice == "female" else 5), 1.0, 8
    generation.extra = {"lang": "fr"}
    if generation.extra.get("lang") != "fr":
        raise ValueError("Synthesis language was not forwarded")
    audio = model.generate("i grec" if text == "y" else text, generation)
    samples = np.ascontiguousarray(audio.samples, dtype=np.float32)
    if audio.sample_rate != 44100 or samples.ndim != 1 or not 0 < samples.size <= 44100 * 120 or not np.isfinite(samples).all() or not np.any(samples):
        raise ValueError("Invalid synthesized audio")
    return encode(Path(encoder), samples, audio.sample_rate)


def main():
    if len(sys.argv) != 4:
        raise ValueError("Expected resources, encoder and CPU configuration")
    model = create_model(sys.argv[1], sys.argv[3])
    serve(lambda text, voice: synthesize(model, sys.argv[2], text, voice), sys.stdin.buffer, sys.stdout.buffer, capacity=3)


if __name__ == "__main__":
    try:
        main()
    except Exception as error:
        print(str(error)[:2048], file=sys.stderr)
        sys.exit(1)
