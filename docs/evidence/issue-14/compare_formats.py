"""Measure actual encodings of the fixed Kokoro samples, with decoding checks."""

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import wave


def main():
    import imageio_ffmpeg

    output = Path(sys.argv[1])
    output.mkdir(parents=True, exist_ok=False)
    ffmpeg = imageio_ffmpeg.get_ffmpeg_exe()
    evidence = Path(__file__).resolve().parent
    formats = {
        "flac": ["-c:a", "flac", "-metadata_header_padding", "0"],
        "opus": ["-c:a", "libopus", "-b:a", "24k", "-application", "voip"],
        "m4a": ["-c:a", "aac", "-b:a", "32k"],
        "mp3": ["-c:a", "libmp3lame", "-b:a", "48k"],
    }
    report = {
        "ffmpeg_version": subprocess.check_output([ffmpeg, "-version"], text=True).splitlines()[0],
        "ffmpeg_sha256": hashlib.sha256(Path(ffmpeg).read_bytes()).hexdigest(),
        "settings": formats, "cases": [],
        "quality": "decoding checks only; perceptual quality pending listening",
    }
    for source in sorted((evidence / "samples").glob("[0-9][0-9].wav")):
        with wave.open(str(source), "rb") as wav:
            assert (wav.getnchannels(), wav.getsampwidth(), wav.getframerate()) == (1, 2, 24000)
            pcm = wav.readframes(wav.getnframes())
        item = {"source": source.name, "wav_bytes": source.stat().st_size, "formats": {}}
        for suffix, args in formats.items():
            destination = output / f"{source.stem}.{suffix}"
            subprocess.run([ffmpeg, "-hide_banner", "-loglevel", "error", "-n", "-i", str(source),
                            "-map_metadata", "-1", *args, str(destination)], check=True)
            decoded = subprocess.check_output([
                ffmpeg, "-hide_banner", "-loglevel", "error", "-i", str(destination),
                "-f", "s16le", "-acodec", "pcm_s16le", "-ac", "1", "-ar", "24000", "pipe:1",
            ])
            assert decoded and any(decoded), (source, suffix, "empty or silent decode")
            if suffix == "flac":
                assert decoded == pcm, (source, "lossless round trip mismatch")
            item["formats"][suffix] = {
                "bytes": destination.stat().st_size,
                "sha256": hashlib.sha256(destination.read_bytes()).hexdigest(),
                "decoded_sample_delta": (len(decoded) - len(pcm)) // 2,
                "pcm_identical": decoded == pcm,
            }
        report["cases"].append(item)
    assert len(report["cases"]) == 17, "expected all 17 fixed samples"
    report["total_bytes"] = {
        "wav": sum(item["wav_bytes"] for item in report["cases"]),
        **{suffix: sum(item["formats"][suffix]["bytes"] for item in report["cases"]) for suffix in formats},
    }
    (output / "report.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report["total_bytes"], indent=2))


if __name__ == "__main__":
    main()
