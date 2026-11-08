Kokoro French standalone comparison

Kokoro v1.0 int8, ff_siwis, kokoro-onnx 0.6.1, eSpeak NG 1.52.0,
CPython 3.12.10 and ONNX Runtime 1.31.0. CPU, two threads, speed 1.0.
inputs.json contains the eight separate fr-common-v1 comparison inputs.

Run speech_probe.py with runtime/python.exe, inputs.json, assets and a new
output directory. --require-network-denied requires an external process
network restriction; --play uses synchronous Windows WAV playback.
The repository documents the Codex Windows AppContainer invocation:
https://github.com/zardonc/lexique/tree/main/docs/evidence/issue-6

The voice tensor is identified by the official ff_siwis.pt in provenance/.
Model and converted voice assets retain the model repository's Apache-2.0
terms; notice/model-card.md and notice/voices.md retain SIWIS CC BY
attribution and the official warning about short utterances.
kokoro-onnx and ONNX Runtime are MIT; phonemizer and eSpeak include GPL
components. Dependency licenses remain in runtime/Lib/site-packages,
notices in notice/, and original frontend and wrapper sources in source/.
resources.json records each acquisition artifact and its separate license.
LICENSE-project.txt applies to the project probe, not third-party assets.

prerequisite/vc_redist.x64.exe is the separate Microsoft runtime installer.
It is not installed by the probe. This is a local comparison package;
human French quality, clean-machine execution, other targets, and final
product redistribution are assessed separately in the repository evidence.
