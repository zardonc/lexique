"""Compare the current per-word worker with model reuse in an isolated process."""
import argparse
import ast
import hashlib
import json
import os
from pathlib import Path
import statistics
import subprocess
import time

from measure_review import REPO, run

WORDS = ("école", "bonjour", "œil", "vacances", "aujourd’hui")


def reuse_probe(source):
    # Split the existing worker at model initialization; keep its hash, model,
    # French language, voice, synthesis, and encoder checks exactly as written.
    module = ast.parse(source)
    main = next(node for node in module.body if isinstance(node, ast.FunctionDef) and node.name == "main")
    lines = source.splitlines(True)
    header = "".join(lines[:main.lineno - 1])
    body = "".join(lines[main.body[0].lineno - 1:main.body[-1].end_lineno])
    body = "\n".join(line[4:] for line in body.splitlines())
    setup, generation = body.split("generation = sherpa_onnx.GenerationConfig()", 1)
    input_line = 'request = json.loads(sys.stdin.buffer.read(8193).decode("utf-8"))'
    if setup.count(input_line) != 1:
        raise ValueError("Worker input changed; review the experiment")
    setup = setup.replace(input_line, 'request = {"text": "école", "voice": "female"}')
    generation = "generation = sherpa_onnx.GenerationConfig()" + generation
    output_line = 'sys.stdout.buffer.write(encode(Path(sys.argv[2]), samples, audio.sample_rate))'
    if generation.count(output_line) != 1:
        raise ValueError("Worker output changed; review the experiment")
    generation = generation.replace(output_line, 'encoded = encode(Path(sys.argv[2]), samples, audio.sample_rate)')
    return header + "\nimport time\nstarted = time.perf_counter()\n" + setup + "\nsetup_ms = (time.perf_counter()-started)*1000\n" + \
        f"for text in {WORDS!r}:\n    started = time.perf_counter()\n" + \
        "\n".join("    " + line for line in generation.splitlines()) + \
        '\n    print(json.dumps({"text":text,"setup_ms":setup_ms,"word_ms":(time.perf_counter()-started)*1000,"audio_bytes":len(encoded)}), flush=True)\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("resources", type=Path, help="existing packaged speech directory, read-only")
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    resources = args.resources.resolve()
    output = args.output.resolve()
    owned = (REPO / "target/issue-24").resolve()
    if not output.is_relative_to(owned) or output == owned:
        raise ValueError("Output must be a new child of target/issue-24")
    output.mkdir(parents=True, exist_ok=False)
    worker = REPO / "apps/desktop/src-tauri/speech_worker.py"
    source = worker.read_text(encoding="utf-8")
    probe = reuse_probe(source)
    (output / "reuse_probe.py").write_text(probe, encoding="utf-8")
    python = resources / "runtime/python.exe"
    base = [str(python), "-I", "-X", "utf8", "-c"]
    assets, encoder = str(resources / "assets"), str(resources / "opusenc.dll")
    baseline = []
    reuse_runs = []
    env = os.environ.copy()
    (output / "temp").mkdir()
    env["TEMP"] = env["TMP"] = str(output / "temp")
    for repeat in range(3):
        for word in WORDS:
            start = time.perf_counter()
            process = subprocess.Popen(base + [source, assets, encoder], stdin=subprocess.PIPE,
                                       stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env,
                                       creationflags=subprocess.CREATE_NO_WINDOW if os.name == "nt" else 0)
            try:
                audio, error = process.communicate(json.dumps({"text": word, "voice": "female"}).encode(), timeout=60)
            except subprocess.TimeoutExpired:
                subprocess.run(["taskkill.exe", "/PID", str(process.pid), "/T", "/F"], check=True, capture_output=True)
                process.communicate()
                raise RuntimeError("Speech measurement exceeded hard 60s") from None
            elapsed = (time.perf_counter() - start) * 1000
            if process.returncode or not audio.startswith(b"OggS"):
                raise RuntimeError(f"Worker failed: {error.decode(errors='replace')[:2048]}")
            (output / f"word-{repeat}-{WORDS.index(word)}.ogg").write_bytes(audio)
            baseline.append({"repeat":repeat,"word":word,"total_ms":elapsed,"audio_bytes":len(audio)})
            print(f"baseline {repeat} {word!a}: {elapsed:.1f} ms", flush=True)
        start = time.perf_counter()
        run(base + [probe, assets, encoder], output / f"reuse-{repeat}.jsonl", env=env)
        reuse_runs.append({"repeat":repeat,"process_total_ms":(time.perf_counter()-start)*1000})
        print(f"reuse {repeat} passed", flush=True)
    result = {"worker_sha256":hashlib.sha256(source.encode()).hexdigest(),
              "note":"Real packaged worker and synthesis; no IPC, cache publication or playback. First process follows existing local resource state, no cold-cache claim.",
              "words":WORDS,"baseline":baseline,"baseline_word_median_ms":statistics.median(row["total_ms"] for row in baseline),
              "reuse_process_runs":reuse_runs,
              "reuse":[json.loads(line) for repeat in range(3) for line in (output / f"reuse-{repeat}.jsonl").read_text().splitlines()]}
    (output / "summary.json").write_text(json.dumps(result, ensure_ascii=False, indent=2), encoding="utf-8")


if __name__ == "__main__":
    main()
