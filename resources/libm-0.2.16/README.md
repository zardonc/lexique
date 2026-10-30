# Portable math dependency

Exact crate: [`libm 0.2.16`](https://docs.rs/libm/0.2.16/libm/), from
[`rust-lang/compiler-builtins`](https://github.com/rust-lang/compiler-builtins).
Archive SHA-256: `b6d2cec3eae94f9f509c767b45932f1ada8350c4bdb85af2fcab4a3c14807981`.
The lockfile fixes the version and checksum. Default features are disabled,
so the FSRS scheduling patch uses the generic Rust math implementations.

The crate as a whole is available under MIT. The unmodified upstream
[LICENSE.txt](LICENSE.txt) preserves its full licensing and attribution
notices, including the contribution terms. Include it when distributing
binaries or WASM containing this implementation. Project licensing does
not replace the third-party notice.

This dependency supplies the scalar math functions for the explicit
`fsrs-rs/6.6.2+libm-0.2.16` compatibility revision. See the
[patch scope](../../vendor/fsrs-6.6.2/LEXIQUE-PATCH.md) and
[diagnosis](../../docs/evidence/issue-13/math-diagnosis.md).
