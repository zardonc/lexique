# Piper French MLS medium — attribution and provenance

Prepared 2026-10-26 for Lexique issue #14. This is the model attribution material; it does not certify that the Piper engine and all runtime dependencies have completed their separate redistribution requirements.

## Model

`fr_FR-mls-medium` — Piper French (France), 125 speakers, 22,050 Hz.
Credit: **Michael Hansen and the Piper contributors**; publisher: **rhasspy/piper-voices**.
Michael Hansen's association with Piper is recorded in the [original Piper repository history](https://github.com/rhasspy/piper/commit/73c04d81d5590ecc46e522de3601ce7fb29fc2be); this credit does not claim an unverified exclusive authorship of this specific training run.

- [Fixed model card](https://huggingface.co/rhasspy/piper-voices/raw/5041a7a49ae5b3a0f30e6837769eb03fdbb231bd/fr/fr_FR/mls/medium/MODEL_CARD), also preserved verbatim as [MODEL_CARD](MODEL_CARD).
- [Fixed original ONNX](https://huggingface.co/rhasspy/piper-voices/resolve/5041a7a49ae5b3a0f30e6837769eb03fdbb231bd/fr/fr_FR/mls/medium/fr_FR-mls-medium.onnx): 76,733,750 bytes; SHA-256 `0ed223f78466917f2bae05ee90096ce69ab1fdeb251f55590d0e7422d234e162`.
- [Fixed original configuration](https://huggingface.co/rhasspy/piper-voices/resolve/5041a7a49ae5b3a0f30e6837769eb03fdbb231bd/fr/fr_FR/mls/medium/fr_FR-mls-medium.onnx.json): 7,036 bytes; SHA-256 `252b0b0a6e4cc4949e23eccb956f9c779986c32f934f2f7e2191e5fdc2edca61`.
- Weight and configuration files are **unmodified**: no fine-tuning, quantization or format conversion. Selecting speaker `0` / `1` and setting synthesis speed do not modify those files. The model card records training from scratch.

The project's CC BY 4.0 admission interpretation combines the specific model card with [Piper's fixed official direction to inspect each voice's model card](https://github.com/OHF-Voice/piper1-gpl/blob/74ac24e7e8e2e57a5e1fada5505c74e3d9c1843a/docs/VOICES.md) and its statement that no additional restrictions are imposed on voice models. It does not rely on the repository-wide MIT metadata or the dataset license alone. The model card places the CC BY 4.0 field in its Dataset section; see the explicit reasoning and limits in [the admission note](../replacement-admission.md).

## Training data

**MLS: A Large-Scale Multilingual Dataset for Speech Research** (2020).
Authors: **Vineel Pratap, Qiantong Xu, Anuroop Sriram, Gabriel Synnaeve and Ronan Collobert**.
Source: [OpenSLR SLR94](https://openslr.org/94/), published as **Creative Commons Attribution 4.0 International (CC BY 4.0)**. MLS is derived from read audiobooks from **LibriVox**; credit also belongs to the contributing audiobook readers and authors as supplied with the original resources. No individual reader identity is asserted for the product's generic “French female/male” labels.

The original corpus is not included in Lexique. The trained model and inference configuration are redistributed without modification. [Official Facebook MLS metadata](https://huggingface.co/datasets/facebook/multilingual_librispeech/resolve/2e83e61823b4c47dcbcb1980bb88601274127609/data/mls_french/metainfo.txt) identifies source speaker `1840` as M and `3698` as F; the model configuration maps those IDs to inference speakers `0` and `1` respectively.

## License, notices and modifications

License: [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). The original official legal text, including its disclaimers, is preserved in [CC-BY-4.0.txt](CC-BY-4.0.txt). Preserve supplied upstream notices, source links and this attribution with the model. This attribution does not imply endorsement of Lexique by the authors, speakers, Piper, Meta/Facebook, OpenSLR or LibriVox. Personality, privacy, patent and trademark rights are outside the copyright license's general grant.

The original [PIPER-VOICES.md](PIPER-VOICES.md) is an archived provenance document, not an additional license applied by Lexique. Its links and examples describe upstream material and do not select other voice models for the product. Original HTTP bodies were saved without text changes; their retrieval URLs, byte counts and SHA-256 hashes are in [sources.json](sources.json).

This folder concerns the model and its source notices only. The engine is `piper-tts 1.8.0` under GPL-3.0-or-later; the project's final engine/dependency notices and corresponding-source delivery must be assembled and verified separately before release. Model admission does not prove human listening acceptance, target-platform execution, offline playback or the final compressed package size.
