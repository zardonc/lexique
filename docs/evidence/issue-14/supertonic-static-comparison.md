# Supertonic static model comparison

On 2026-10-27, the fixed initial [Supertone model snapshot](https://huggingface.co/Supertone/supertonic-3/tree/724fb5abbf5502583fb520898d45929e62f02c0b/onnx) was compared with the published sherpa-onnx assets without inference or audio generation. The comparator checks SHA-256 before parsing and rejects external tensor data.

| File | Original SHA-256 | Published int8 SHA-256 |
| --- | --- | --- |
| vector_estimator | `883ac868ea0275ef0e991524dc64f16b3c0376efd7c320af6b53f5b780d7c61c` | `20cd86fa5c6effedfda0e7cffe5b0569ca401c440a0c3a1d72bf39286c0db3fd` |
| vocoder | `085de76dd8e8d5836d6ca66826601f615939218f90e519f70ee8a36ed2a4c4ba` | `e923d60f53f95eb1ce235f1dc33ec56d9c057823c96fa6f8acf98f32b0da6152` |

The [comparator](compare_onnx_static.py) reports serialized tensor identity for same-named initializers and compares recognized `DequantizeLinear` or `MatMulInteger` candidate weights with same-named, same-shaped original floating-point initializers. It reports 96 identical named initializers and 116 candidate quantized weight comparisons for vector estimator, and 45 identical initializers and 58 candidate comparisons for vocoder. The largest errors for those comparisons were 0.5000089407 and 0.5000042319 quantization steps respectively. The vector input and output signatures and vocoder input signatures matched. The vocoder output remains float32, named `wav_tts`, and rank two, but its symbolic dimensions differ.

Unrecognized nodes, unmatched names, shapes and other initializers are not included in the comparison. For `MatMulInteger` the script infers the scale initializer by name; it does not verify the downstream scale operation. Counts do not establish full coverage, graph equivalence, the input actually used by the publisher, calibration data or the exact conversion chain. Full report generated locally: `.tooling/issue-14/static-onnx/report-v2.jsonl`, SHA-256 `4d7e6bb4dcdb6b855d5f19b1fdaccba69b33084703d3e715f2858bd932ad653a`.

The run used Python 3.12.13, ONNX 1.19.1, NumPy 2.5.4, protobuf 7.36.2, ml_dtypes 0.5.3 and typing_extensions 4.16.0. The separately acquired ONNX, ml_dtypes and typing_extensions wheels had SHA-256 `bc7e2e4e163e679721e547958b5a7db875bf822cad371b7c1304aa4401a7c7a4`, `da65e5fd3eea434ccb8984c3624bc234ddcc0d9f4c81864af611aaebcc08a50e` and `481caa481374e813c1b176ada14e97f1f67a4539ce9cfeb3f350d78d6370c2e8` respectively. These parsing dependencies are isolated trial tools, not product dependencies.

With those existing local materials, reproduce the comparison and its focused tests in PowerShell:

```powershell
$env:TEMP = "$PWD/.tooling/issue-14"
$env:TMP = $env:TEMP
$env:PYTHONPATH = "$PWD/.tooling/issue-14/static-onnx/parser;$PWD/.tooling/issue-14/materials/typing_extensions-4.16.0-py3-none-any.whl"
$env:PYTHONDONTWRITEBYTECODE = '1'
.tooling/issue-14/comparison/venv/Scripts/python.exe -X utf8 docs/evidence/issue-14/compare_onnx_static.py .tooling/issue-14/static-onnx .tooling/issue-14/supertonic-evidence-v9/assets
.tooling/issue-14/comparison/venv/Scripts/python.exe -X utf8 -m unittest discover -s docs/evidence/issue-14 -p test_compare_onnx_static.py -v
```

Five tests cover scalar and per-axis dequantization, a `MatMulInteger` naming case, rejection of invalid scales, and rejection of an incorrect input hash before parsing. They do not run either model.
