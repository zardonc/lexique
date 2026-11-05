"""Encode the verified Supertonic WAV samples as Ogg Opus for listening."""

import hashlib
import json
from pathlib import Path
import subprocess
import sys
import wave

import imageio_ffmpeg


def main():
    source_dir = Path(sys.argv[2]) if len(sys.argv) > 2 else Path(__file__).with_name("supertonic-full")
    report = json.loads((source_dir / "report.json").read_text(encoding="utf-8"))
    output = Path(sys.argv[1])
    output.mkdir(parents=True, exist_ok=False)
    ffmpeg = imageio_ffmpeg.get_ffmpeg_exe()
    result = {"ffmpeg_version": subprocess.check_output([ffmpeg, "-version"], text=True).splitlines()[0],
              "ffmpeg_sha256": hashlib.sha256(Path(ffmpeg).read_bytes()).hexdigest(),
              "settings": {"codec": "libopus", "container": "ogg", "application": "voip",
                           "bitrates_kbps": [24, 48]}, "cases": []}
    for case in report["cases"]:
        source = source_dir / case["file"]
        assert hashlib.sha256(source.read_bytes()).hexdigest() == case["sha256"]
        with wave.open(str(source), "rb") as wav:
            assert (wav.getnchannels(), wav.getsampwidth(), wav.getframerate()) == (1, 2, 44100)
            frames = wav.getnframes()
        row = {"input": case["input"], "voice": case["voice"], "wav": case["file"],
               "wav_bytes": source.stat().st_size, "formats": {}}
        for bitrate in (24, 48):
            destination = output / f"{source.stem}-{bitrate}k.opus"
            subprocess.run([ffmpeg, "-hide_banner", "-loglevel", "error", "-n", "-i", str(source),
                            "-map_metadata", "-1", "-c:a", "libopus", "-b:a", f"{bitrate}k",
                            "-application", "voip", str(destination)], check=True)
            decoded = subprocess.check_output([ffmpeg, "-hide_banner", "-loglevel", "error", "-i",
                                               str(destination), "-f", "s16le", "-ac", "1",
                                               "-ar", "44100", "pipe:1"])
            assert decoded and any(decoded), destination
            row["formats"][str(bitrate)] = {"file": destination.name, "bytes": destination.stat().st_size,
                "sha256": hashlib.sha256(destination.read_bytes()).hexdigest(),
                "decoded_sample_delta": len(decoded) // 2 - frames}
        result["cases"].append(row)
    assert len(result["cases"]) == 38
    result["total_bytes"] = {"wav": sum(c["wav_bytes"] for c in result["cases"]),
        **{str(b): sum(c["formats"][str(b)]["bytes"] for c in result["cases"]) for b in (24, 48)}}
    (output / "report.json").write_text(json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result["total_bytes"], ensure_ascii=False))


if __name__ == "__main__":
    main()
