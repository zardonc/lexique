# Portable scheduling math

Base: the official `fsrs 6.6.2` crates.io archive, commit
`6088949b99d44f46f5328fd7289d209b9a842b68`, SHA-256
`89836c0a26b347b585b23824034600b69bfc7f0b0112d0cd656791799eb3a717`.
The original BSD-3-Clause [LICENSE](LICENSE) applies to this vendored source.
Published auxiliary files are kept verbatim, including upstream workflow
whitespace; they are not enabled as this workspace's CI or toolchain.

Only two upstream files change:

- `Cargo.toml` adds exact `libm = "=0.2.16"`, with default features disabled.
- `src/model.rs` routes the scalar scheduling functions' `ln`, `exp`, and
  `powf` through `libm::logf`, `libm::expf`, and `libm::powf`. The expressions,
  operation order, f32 types, clamps, and official parameters remain unchanged.

This patch covers the application's `FSRS::default().next_states` path with
official FSRS-6 parameters. Other upstream APIs, training, older parameter
formats, and optimizer behavior are outside this compatibility claim.
`Cargo.toml.orig` and the upstream lockfile retain their published contents;
the workspace manifest and lockfile select the patched dependency and libm.

The application records `fsrs-rs/6.6.2+libm-0.2.16` and rejects unpatched
historical model identifiers. This is a local compatibility revision of
6.6.2, not an unmodified upstream release. It does not interpose global C
math symbols or change the application's learning steps or rounding rules.

Source audit, failure reproduction, and validation:
[issue #13 evidence](../../docs/evidence/issue-13/math-diagnosis.md).
