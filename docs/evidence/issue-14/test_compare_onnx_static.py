"""Verify static graph comparison without evaluating either model."""

from pathlib import Path
import tempfile
import unittest

import numpy as np
from onnx import TensorProto, helper, numpy_helper

from compare_onnx_static import compare, read_model


def tensor(name, values, dtype):
    return numpy_helper.from_array(np.array(values, dtype=dtype), name=name)


def model(nodes, initializers):
    value = helper.make_tensor_value_info("value", TensorProto.FLOAT, [1, 2])
    return helper.make_model(helper.make_graph(nodes, "test", [value], [value], initializers))


class CompareOnnxStaticTests(unittest.TestCase):
    def test_quantized_weights_match_original_without_inference(self):
        original = model([], [tensor("weight", [[0.0, 1.25]], np.float32)])
        converted = model(
            [helper.make_node("DequantizeLinear", ["weight_wq", "weight_scale", "weight_zp"], ["w"])],
            [tensor("weight_wq", [[0, 2]], np.uint8),
             tensor("weight_scale", 0.5, np.float32),
             tensor("weight_zp", 0, np.uint8)],
        )
        result = compare("test", original, converted)
        self.assertTrue(result["input_signatures_equal"])
        self.assertTrue(result["output_signatures_equal"])
        weights = result["matched_dequantized_weights"]
        self.assertEqual(len(weights), 1)
        self.assertEqual(weights[0]["name"], "weight")
        self.assertEqual(weights[0]["count"], 2)
        self.assertAlmostEqual(weights[0]["max_abs_error"], 0.25)
        self.assertAlmostEqual(weights[0]["max_error_in_steps"], 0.5)

    def test_wrong_hash_is_rejected_before_parsing(self):
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "invalid.onnx"
            path.write_bytes(b"not a model")
            with self.assertRaisesRegex(ValueError, "SHA-256 mismatch"):
                read_model(path, "0" * 64)

    def test_per_axis_dequantization_matches_each_channel(self):
        original = model([], [tensor("weight", [[0.0, 1.25], [3.0, 4.25]], np.float32)])
        converted = model(
            [helper.make_node("DequantizeLinear", ["weight_wq", "weight_scale", "weight_zp"], ["w"], axis=0)],
            [tensor("weight_wq", [[0, 2], [3, 4]], np.uint8),
             tensor("weight_scale", [0.5, 1.0], np.float32),
             tensor("weight_zp", [0, 0], np.uint8)],
        )
        weight = compare("test", original, converted)["matched_dequantized_weights"][0]
        self.assertAlmostEqual(weight["max_abs_error"], 0.25)
        self.assertAlmostEqual(weight["max_error_in_steps"], 0.5)

    def test_matmul_integer_resolves_weight_and_scale_names(self):
        original = model([], [tensor("weight", [[0.0, 1.25]], np.float32)])
        converted = model(
            [helper.make_node("MatMulInteger", ["x", "weight_quantized", "x_zp", "weight_zero_point"], ["y"])],
            [tensor("weight_quantized", [[0, 2]], np.uint8),
             tensor("weight_scale", 0.5, np.float32),
             tensor("weight_zero_point", 0, np.uint8)],
        )
        weight = compare("test", original, converted)["matched_dequantized_weights"][0]
        self.assertEqual(weight["operator"], "MatMulInteger")
        self.assertEqual(weight["name"], "weight")
        self.assertAlmostEqual(weight["max_error_in_steps"], 0.5)

    def test_nonpositive_or_nonfinite_scale_is_rejected(self):
        original = model([], [tensor("weight", [[1.0]], np.float32)])
        for value in (0.0, -0.5, float("nan"), float("inf")):
            with self.subTest(scale=value):
                converted = model(
                    [helper.make_node("DequantizeLinear", ["weight_wq", "weight_scale", "weight_zp"], ["w"])],
                    [tensor("weight_wq", [[2]], np.uint8),
                     tensor("weight_scale", value, np.float32),
                     tensor("weight_zp", 0, np.uint8)],
                )
                with self.assertRaisesRegex(ValueError, "Invalid quantization scale"):
                    compare("test", original, converted)


if __name__ == "__main__":
    unittest.main()
