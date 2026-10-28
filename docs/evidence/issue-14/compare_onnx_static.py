"""Compare fixed Supertonic ONNX graphs and weights without running inference."""

import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path

import numpy as np
import onnx
from onnx import numpy_helper


FILES = {
    "vector_estimator": (
        "883ac868ea0275ef0e991524dc64f16b3c0376efd7c320af6b53f5b780d7c61c",
        "20cd86fa5c6effedfda0e7cffe5b0569ca401c440a0c3a1d72bf39286c0db3fd",
    ),
    "vocoder": (
        "085de76dd8e8d5836d6ca66826601f615939218f90e519f70ee8a36ed2a4c4ba",
        "e923d60f53f95eb1ce235f1dc33ec56d9c057823c96fa6f8acf98f32b0da6152",
    ),
}


def read_model(path, expected):
    with path.open("rb") as source:
        actual = hashlib.file_digest(source, "sha256").hexdigest()
    if actual != expected:
        raise ValueError(f"SHA-256 mismatch: {path}")
    model = onnx.load(path, load_external_data=False)
    if any(t.data_location == onnx.TensorProto.EXTERNAL for t in model.graph.initializer):
        raise ValueError(f"External data not supported: {path}")
    return model


def signature(values):
    return [(v.name, v.type.SerializeToString().hex()) for v in values]


def compare(name, original, quantized):
    old = {t.name: t for t in original.graph.initializer}
    new = {t.name: t for t in quantized.graph.initializer}
    same = [key for key in old.keys() & new.keys()
            if old[key].SerializeToString() == new[key].SerializeToString()]
    results = []
    for node in quantized.graph.node:
        if node.op_type == "DequantizeLinear" and len(node.input) >= 3:
            weight, scale, zero = node.input[:3]
        elif node.op_type == "MatMulInteger" and len(node.input) >= 4:
            weight, zero = node.input[1], node.input[3]
            scale = weight.removesuffix("_quantized") + "_scale"
        else:
            continue
        if not all(x in new for x in (weight, scale, zero)):
            continue
        original_name = weight.removesuffix("_wq").removesuffix("_quantized")
        if original_name == weight:
            continue
        if original_name not in old:
            continue
        a = numpy_helper.to_array(old[original_name])
        q = numpy_helper.to_array(new[weight])
        s = numpy_helper.to_array(new[scale])
        z = numpy_helper.to_array(new[zero])
        if not np.isfinite(s).all() or not (s > 0).all():
            raise ValueError(f"Invalid quantization scale: {scale}")
        if a.shape != q.shape or not np.issubdtype(a.dtype, np.floating):
            continue
        if s.ndim:
            axis = next((attr.i for attr in node.attribute if attr.name == "axis"), 1)
            shape = [1] * q.ndim
            shape[axis] = s.size
            s, z = s.reshape(shape), z.reshape(shape)
        restored = (q.astype(np.float32) - z.astype(np.float32)) * s
        delta = np.abs(a - restored)
        results.append({"name": original_name, "operator": node.op_type, "count": int(a.size),
                        "max_abs_error": float(delta.max()),
                        "mean_abs_error": float(delta.mean()),
                        "scale_max": float(np.max(s)),
                        "max_error_in_steps": float(np.max(delta / s))})
    return {
        "name": name,
        "input_signatures_equal": signature(original.graph.input) == signature(quantized.graph.input),
        "output_signatures_equal": signature(original.graph.output) == signature(quantized.graph.output),
        "original_operators": dict(Counter(n.op_type for n in original.graph.node)),
        "quantized_operators": dict(Counter(n.op_type for n in quantized.graph.node)),
        "original_initializers": len(old), "quantized_initializers": len(new),
        "identical_named_initializers": len(same),
        "matched_dequantized_weights": results,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("original", type=Path)
    parser.add_argument("assets", type=Path)
    args = parser.parse_args()
    for name, (raw_hash, int8_hash) in FILES.items():
        raw = read_model(args.original / f"{name}.onnx", raw_hash)
        converted = read_model(args.assets / f"{name}.int8.onnx", int8_hash)
        print(json.dumps(compare(name, raw, converted), sort_keys=True), flush=True)


if __name__ == "__main__":
    main()
