# 原生 OCR 依赖通知

2026-11-09，依据锁定的 Windows x64 `cargo tree --locked`：XCap 0.9.9 及 image 0.25.10 的 PNG／JPEG 依赖，另含 base64 0.23.1。各文件保留上游法律原文；双许可依赖采用下表列出的许可，Unicode 许可同时适用。平台专属依赖在对应发行物中另行补齐。

| 固定 crate | 上游 | 采用许可 | 法律原文 |
| --- | --- | --- | --- |
| adler2-2.0.1 | <https://github.com/oyvindln/adler2> | MIT | [原文](adler2-2.0.1.txt) |
| base64-0.23.1 | <https://github.com/marshallpierce/rust-base64> | MIT | [原文](base64-0.23.1.txt) |
| bitflags-2.13.2 | <https://github.com/bitflags/bitflags> | MIT | [原文](bitflags-2.13.2.txt) |
| bytemuck-1.25.2 | <https://github.com/Lokathor/bytemuck> | MIT | [原文](bytemuck-1.25.2.txt) |
| byteorder-lite-0.1.0 | <https://github.com/image-rs/byteorder-lite> | MIT | [原文](byteorder-lite-0.1.0.txt) |
| cfg-if-1.0.5 | <https://github.com/rust-lang/cfg-if> | MIT | [原文](cfg-if-1.0.5.txt) |
| crc32fast-1.5.3 | <https://github.com/srijs/rust-crc32fast> | MIT | [原文](crc32fast-1.5.3.txt) |
| fdeflate-0.3.7 | <https://github.com/image-rs/fdeflate> | MIT | [原文](fdeflate-0.3.7.txt) |
| flate2-1.1.10 | <https://github.com/rust-lang/flate2-rs> | MIT | [原文](flate2-1.1.10.txt) |
| image-0.25.10 | <https://github.com/image-rs/image> | MIT | [原文](image-0.25.10.txt) |
| log-0.4.34 | <https://github.com/rust-lang/log> | MIT | [原文](log-0.4.34.txt) |
| miniz_oxide-0.8.9 | <https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide> | MIT | [原文](miniz_oxide-0.8.9.txt) |
| miniz_oxide-0.9.1 | <https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide> | MIT | [原文](miniz_oxide-0.9.1.txt) |
| moxcms-0.8.1 | <https://github.com/awxkee/moxcms.git> | BSD-3-Clause | [原文](moxcms-0.8.1.txt) |
| num-traits-0.2.19 | <https://github.com/rust-num/num-traits> | MIT | [原文](num-traits-0.2.19.txt) |
| png-0.18.1 | <https://github.com/image-rs/image-png> | MIT | [原文](png-0.18.1.txt) |
| proc-macro2-1.0.107 | <https://github.com/dtolnay/proc-macro2> | MIT | [原文](proc-macro2-1.0.107.txt) |
| pxfm-0.1.30 | <https://github.com/awxkee/pxfm> | BSD-3-Clause | [原文](pxfm-0.1.30.txt) |
| quote-1.0.47 | <https://github.com/dtolnay/quote> | MIT | [原文](quote-1.0.47.txt) |
| scopeguard-1.2.0 | <https://github.com/bluss/scopeguard> | MIT | [原文](scopeguard-1.2.0.txt) |
| serde_core-1.0.229 | <https://github.com/serde-rs/serde> | MIT | [原文](serde_core-1.0.229.txt) |
| simd-adler32-0.3.10 | <https://github.com/mcountryman/simd-adler32> | MIT | [原文](simd-adler32-0.3.10.txt) |
| syn-2.0.119 | <https://github.com/dtolnay/syn> | MIT | [原文](syn-2.0.119.txt) |
| syn-3.0.9 | <https://github.com/dtolnay/syn> | MIT | [原文](syn-3.0.9.txt) |
| thiserror-2.0.21 | <https://github.com/dtolnay/thiserror> | MIT | [原文](thiserror-2.0.21.txt) |
| thiserror-impl-2.0.21 | <https://github.com/dtolnay/thiserror> | MIT | [原文](thiserror-impl-2.0.21.txt) |
| unicode-ident-1.0.27 | <https://github.com/dtolnay/unicode-ident> | MIT AND Unicode-3.0 | [原文](unicode-ident-1.0.27.txt)、[Unicode 原文](unicode-3.0.txt) |
| widestring-1.2.1 | <https://github.com/VoidStarKat/widestring-rs> | MIT | [原文](widestring-1.2.1.txt) |
| windows-0.62.2 | <https://github.com/microsoft/windows-rs> | MIT | [原文](windows-0.62.2.txt) |
| windows-collections-0.3.2 | <https://github.com/microsoft/windows-rs> | MIT | [原文](windows-collections-0.3.2.txt) |
| windows-core-0.62.2 | <https://github.com/microsoft/windows-rs> | MIT | [原文](windows-core-0.62.2.txt) |
| windows-future-0.3.2 | <https://github.com/microsoft/windows-rs> | MIT | [原文](windows-future-0.3.2.txt) |
| windows-implement-0.60.2 | <https://github.com/microsoft/windows-rs> | MIT | [原文](windows-implement-0.60.2.txt) |
| windows-interface-0.59.3 | <https://github.com/microsoft/windows-rs> | MIT | [原文](windows-interface-0.59.3.txt) |
| windows-link-0.2.1 | <https://github.com/microsoft/windows-rs> | MIT | [原文](windows-link-0.2.1.txt) |
| windows-numerics-0.3.1 | <https://github.com/microsoft/windows-rs> | MIT | [原文](windows-numerics-0.3.1.txt) |
| windows-result-0.4.1 | <https://github.com/microsoft/windows-rs> | MIT | [原文](windows-result-0.4.1.txt) |
| windows-strings-0.5.1 | <https://github.com/microsoft/windows-rs> | MIT | [原文](windows-strings-0.5.1.txt) |
| windows-threading-0.2.1 | <https://github.com/microsoft/windows-rs> | MIT | [原文](windows-threading-0.2.1.txt) |
| xcap-0.9.9 | <https://github.com/nashaofu/xcap.git> | Apache-2.0 | [原文](../xcap.txt) |
| zune-core-0.5.3 | <https://github.com/etemesi254/zune-image> | MIT | [原文](zune-core-0.5.3.txt) |
| zune-jpeg-0.5.15 | <https://github.com/etemesi254/zune-image/tree/dev/crates/zune-jpeg> | MIT | [原文](zune-jpeg-0.5.15.txt) |
