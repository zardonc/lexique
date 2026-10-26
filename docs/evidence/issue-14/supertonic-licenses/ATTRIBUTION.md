# Supertonic 3 License Evidence

The sherpa-onnx release archive contains four `*.int8.onnx` files, `tts.json`, `unicode_indexer.bin`, `voice.bin`, a README and an MIT `LICENSE`. Its README identifies the accompanying model as OpenRAIL-M and the MIT text as applying to sample code. Keep those scopes separate.

The extracted package contains model assets, not the sherpa-onnx inference engine. The sherpa-onnx source repository's Apache-2.0 license applies to that code when used separately; it does not replace the model license. The exact license files and source hashes are recorded in [sources.json](sources.json).

The ONNX files are distributed as int8 artifacts by the sherpa-onnx release. This evidence does not independently audit how each weight was transformed, and it does not conclude that redistribution under OpenRAIL-M is suitable for Lexique. Isolated Windows generation and playback calls for 26 samples were checked; human French intelligibility and naturalness remain unaccepted. See the [trial](../supertonic-trial.md).

The official voice SDK documentation describes five male styles (`M1`-`M5`) and five female styles (`F1`-`F5`). The pinned sherpa-onnx generator sorts style JSON filenames before building `voice.bin`, giving `sid` 0-4 to `F1`-`F5` and `sid` 5-9 to `M1`-`M5`. This license folder contains notices only; the [listening samples](../supertonic-samples/listen.html) are separate.

## Attribution Draft

Supertonic 3 model assets by Supertone Inc., distributed in the sherpa-onnx int8 release. Model and voice assets are subject to OpenRAIL-M. sherpa-onnx engine source is separately licensed under Apache-2.0. The bundled MIT license covers sample code as described by the release README.

The maintainer quantized/repackaged the upstream model and combined preset voice styles into `voice.bin`; Lexique used those published bytes without further model changes. Preserve the model OpenRAIL-M text, existing MIT notices and source links, and identify the maintainer modifications. A final product distribution must incorporate OpenRAIL-M use restrictions as enforceable terms and disclose machine synthesis; merely shipping this notice is not a completed product license flow. No endorsement by Supertone or sherpa-onnx is implied.
