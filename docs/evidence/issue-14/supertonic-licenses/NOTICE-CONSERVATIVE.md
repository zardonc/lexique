# Supertonic Windows x64 candidate notices

This is a conservative notice for the v9 Supertonic 3 and sherpa-onnx 2.0.1 Windows x64 evidence candidate. It supplements the original license and copyright files in this directory and the license files shipped in the runtime wheels. It is not a complete binary bill of materials or legal approval for product distribution.

## Model and engine

- Supertonic 3 model and voice assets by Supertone Inc. are subject to `LICENSE-SUPERTONE-model-OpenRAIL-M.txt`. The release sample code is separately subject to `LICENSE-SUPERTONE-sample-code-MIT.txt`; sherpa-onnx source is separately subject to `LICENSE-SHERPA-ONNX-Apache-2.0.txt`.
- Lexique uses the published model bytes without further modification. Two published graphs and `tts.json` match the recorded initial source snapshot; `voice.bin` and `unicode_indexer.bin` match the recorded packing comparison. The vector estimator and vocoder are published int8-labelled files; their exact conversion inputs and process have not been independently established. See `../supertonic3-model.json` and `../supertonic-static-comparison.md`.
- A product distributing the model must address the license's use restrictions, recipient license, notices, and modified-file requirements, as applicable. Product terms and disclosure requirements are recorded in `../../../development-readiness.md` in the repository; they have not been implemented or legally approved here.

## Candidate dependency notices

- ONNX Runtime 1.28.2's Windows DLL matches the recorded upstream artifact; its license and third-party notices are retained. CPython's pinned OpenSSL DLLs and LibTomMath binary have the provenance described in `ATTRIBUTION.md`. CPython and NumPy licenses and wheel notices remain separate.
- The directory also retains upstream notices for sherpa-onnx's declared dependencies and potential transitive dependencies, including utf8proc, pybind11, nlohmann/json, simple-sentencepiece and its Darts and ThreadPool headers, hclust-cpp, kaldi-decoder, kaldifst, OpenFST and KissFFT. Presence of a notice does not prove the component is in the wheel.
- Notices for Asio, PortAudio including its C++ bindings, and WebSocket++ are also retained from the fixed-source local build's fetched dependencies. GoogleTest 1.13.0 is an optional test dependency and its upstream notice is also retained. The public wheel workflow disables some features that use these components, but the actual wheel configuration is unverified. Other nested source-file notices or build-time dependencies are not proven fully enumerated.
- Eigen 5.0.1 and 3.4.0 source-archive notices are retained conservatively, including GPL and LGPL texts present in the 3.4.0 archive. Their inclusion does not assert that either version or any particular source file was used in the wheel. Actual build options and applicable obligations remain unknown.
- The OpenFST `COPYING` and `AUTHORS` files archived here match the files in the locally built `v1.8.5-2026-07-09` dependency as well as the previously recorded candidate. The local build's dependency selection does not prove the wheel's selection.

The exact file identities, attribution, evidence and unresolved provenance are recorded in `sources.json` and `ATTRIBUTION.md`. Any final distributor must identify applicable obligations for the actual components and honor them; including candidate notices alone does not establish compliance.
