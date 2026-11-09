# MDX 读取依赖与通知

应用使用已验收的 `mdictlib 0.2.8 + lzo` 读取用户自备 MDX，按需定位并保留同键物理记录。格式、来源与 Windows x64 技术证据见 [#3](../../docs/evidence/issue-3/README.md)，产品行为与原形关联见 [D1／D3](../../docs/lookup-enhancement-slices.md)。第三方词典内容不进入默认包。

固定依赖来自根 [Cargo.lock](../../Cargo.lock)，新增依赖与独立探针采用相同版本。下列文件保留相应 crate 包的法律原文；双许可采用 MIT，`encoding_rs` 的 WHATWG BSD 条款同时适用。其余共用依赖的通知见 [原生通知](../tesseract-fra/licenses/native/README.md)。便携包将本目录复制为 `mdx-notices/`。

| 固定 crate | 上游 | 采用许可 | 法律原文 |
| --- | --- | --- | --- |
| mdictlib 0.2.8 | <https://github.com/lonelam/mdictlib> | MIT | [原文](licenses/mdictlib-0.2.8.txt) |
| lzokay 2.0.1 | <https://github.com/encounter/lzokay-rs> | MIT | [原文](licenses/lzokay-2.0.1.txt) |
| encoding_rs 0.8.42 | <https://github.com/hsivonen/encoding_rs> | MIT AND BSD-3-Clause | [MIT](licenses/encoding_rs-0.8.42.txt)、[WHATWG](licenses/encoding_rs-whatwg.txt) |
| core_detect 1.0.0 | <https://github.com/thomcc/core_detect> | MIT | [原文](licenses/core_detect-1.0.0.txt) |
| multiversion_no_op 1.0.0 | <https://github.com/hsivonen/multiversion_no_op> | MIT | [原文](licenses/multiversion_no_op-1.0.0.txt) |
| simdutf8 0.1.5 | <https://github.com/rusticstuff/simdutf8> | MIT | [原文](licenses/simdutf8-0.1.5.txt) |
| zerocopy 0.8.63 | <https://github.com/google/zerocopy> | MIT | [原文](licenses/zerocopy-0.8.63.txt) |
| zerocopy-derive 0.8.63（构建） | <https://github.com/google/zerocopy> | MIT | [原文](licenses/zerocopy-derive-0.8.63.txt) |

自定义词典的内容许可与解析器代码许可分别处理。未从 MDX 独立提取的许可、音标、词性与释义字段保持未知；原始正文仅在隔离窗口中显示。
