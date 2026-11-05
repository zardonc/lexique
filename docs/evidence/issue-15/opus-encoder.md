# Windows Opus encoder investigation

Current update, 2026-10-29: the fixed-source DLL is now integrated into the
Windows x64 speech worker and the development portable package. Fresh native
generation, independent Opus decoding, real WebView2 playback and restart
recovery passed. See [the #15 evidence](README.md) for hashes, samples and
remaining limits. Earlier statements below describe their historical stage;
clean-machine compatibility and final distribution obligations remain unverified.

On 2026-10-28, the Windows x64 `opus-tools 0.2` archive linked by the
[Xiph downloads page](https://www.opus-codec.org/downloads/) was retrieved from
[Mozilla's archive](https://archive.mozilla.org/pub/opus/win64/opus-tools-0.2-win64.zip).
Its SHA-256 is `84e643f1cc3fd26fc743ea7830392de765507f72846e24719804d3d07f1ad96f`.
The extracted `opusenc.exe` is 647,680 bytes and has SHA-256
`fa05b15daffdc70ad24bada408f30eaa3e9169bcb97bc9835fe1a6913a1919f0`.
The executable reports opus-tools 0.2, libopus 1.3-rc2 and libopusenc 0.2.

The archive's license identifies opusenc as BSD-style and separately identifies
opusinfo as GPL. The encoder's actual bundled dependencies, build provenance,
notices and product redistribution obligations still need verification; this
observation does not admit the archive as a product dependency. MSVC dumpbin
reports direct imports only from `KERNEL32.dll` and `SHELL32.dll`; this does not
identify statically linked components or rule out dynamic loading. The hashes
above identify the retrieved bytes and were not compared to an independently
published archive digest.

The existing v9 sample `female-normal-01.wav` was encoded with `opusenc
--bitrate 48 --speech` and decoded with the accompanying opusdec. Opusinfo
reported a single-channel Opus stream with 44,100 Hz original sample rate and
1.348 seconds playback length. The encoded file is 6,696 bytes and the decoded
WAV has 59,459 mono frames with a nonzero peak. Variable bitrate averaged
39.73 kbit/s including container overhead for this short sample; 48 kbit/s is
the requested target, not a fixed per-file rate. No synthesis or playback was
performed for this check. Artifacts remain in `.tooling/issue-15-opus/`.

## Fixed-source build

The official [Xiph downloads page](https://www.opus-codec.org/downloads/)
publishes SHA-256 values for Opus 1.6.1
(`6ffcb593207be92584df15b32466ed64bbec99109f007c82205f0194572411a1`)
and libopusenc 0.3
(`f616d3aff9b2034547894ccb8ab56c36cf1a4acb0d922c5d7119f97bbe58642c`);
both retrieved source archives matched. Their members were checked for paths
and links before extraction. On this machine, CMake and MSVC 19.44.35229 built
libopus as a static library, with runtime hardening enabled and fast math off.
The upstream Visual Studio project built libopusenc as a DLL against that
library, using the v143 toolset. MSVC ignored a `-msse4.1` option and reported
upstream conversion and signedness warnings; they remain uninvestigated.

The resulting DLL SHA-256 is
`b9f9ccabf5e00ce55e43079d921316747ba1c4ea730a5d06a31ebe64db147774`.
Its direct imports are KERNEL32, VCRUNTIME140 and Windows Universal CRT DLLs.
[The check](check_opus_encoder.py) encodes the same existing WAV with a target
of 48 kbit/s. Independent opusinfo identifies libopus 1.6.1 and libopusenc 0.3;
opusdec recovers all 59,459 frames at 44,100 Hz with a nonzero peak. No speech
was synthesized or played. The library has not yet been packaged, integrated,
tested on a clean machine, or admitted for distribution; its full notices and
the compiler warnings still require review.

## Offline synthesis integration check

The desktop speech worker was run with the v16-revised Supertonic runtime and
assets, the locally built encoder DLL, and a synthetic `ecole` request. The
worker checked the seven model hashes and returned a 7,971-byte Ogg stream.
Independent opusinfo identified libopus 1.6.1 and libopusenc 0.3; opusdec
decoded 59,509 mono frames at 44,100 Hz with a nonzero peak. A malformed
request returned an error and no audio. This was an automated integration
check, not a listening assessment; no audio was played. The result is not a
native desktop or portable packaging test.

`scripts/prepare_speech.py` stages the fixed v16-revised inventory, the locally
built DLL and the two verified source licenses into a new development resource
directory. An isolated stage contained 1,058 verified files; invoking the
worker using its Python runtime in isolated mode produced an Ogg stream. A
second staging attempt to the same path failed without overwriting it. The
portable packager can include these resources in a Windows x64 development
package after verifying them; this does not establish complete distribution
obligations or clean-machine compatibility.
