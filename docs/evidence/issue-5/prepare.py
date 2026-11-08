"""Prepare the pinned MLS model for Sherpa using its upstream metadata format."""

import argparse
import hashlib
import json
from pathlib import Path
import shutil


def main():
    import onnx

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("output", type=Path, help="must not exist")
    args = parser.parse_args()
    if args.output.exists():
        raise FileExistsError(f"Preserving prepared assets: {args.output}")
    manifest = json.loads(Path(__file__).with_name("resources.json").read_text(encoding="utf-8"))
    for item in manifest["source_files"]:
        path = args.source / item["file"]
        with path.open("rb") as source:
            digest = hashlib.file_digest(source, "sha256").hexdigest()
        if path.stat().st_size != item["bytes"] or digest != item["sha256"]:
            raise ValueError(f"source resource checksum mismatch: {path.name}")
    name = "fr_FR-mls-medium.onnx"
    config = json.loads((args.source / (name + ".json")).read_text(encoding="utf-8"))
    if (config["espeak"]["voice"], config["audio"]["sample_rate"], config["num_speakers"]) != ("fr", 22050, 125):
        raise ValueError("unexpected MLS configuration")
    model = onnx.load(args.source / name)
    original_graph = model.graph.SerializeToString()
    metadata = {
        "model_type": "vits", "comment": "piper", "language": "French",
        "voice": "fr", "version": "1", "has_espeak": "1", "has_g2pw": "0",
        "n_speakers": "125", "sample_rate": "22050",
    }
    onnx.helper.set_model_props(model, metadata)
    args.output.mkdir(parents=True)
    onnx.save(model, args.output / name)
    if onnx.load(args.output / name).graph.SerializeToString() != original_graph:
        raise ValueError("prepared ONNX graph differs from the original")
    shutil.copyfile(args.source / (name + ".json"), args.output / (name + ".json"))
    tokens = []
    for phoneme, ids in config["phoneme_id_map"].items():
        if phoneme == "\n":
            continue
        if len(phoneme) != 1 or len(ids) != 1:
            raise ValueError(f"unsupported phoneme mapping: {phoneme!r}: {ids}")
        tokens.append(f"{phoneme} {ids[0]}\n")
    (args.output / "tokens.txt").write_text("".join(tokens), encoding="utf-8", newline="\n")
    for path in sorted(args.output.iterdir()):
        print(json.dumps({"file": path.name, "bytes": path.stat().st_size,
                          "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}))


if __name__ == "__main__":
    main()
