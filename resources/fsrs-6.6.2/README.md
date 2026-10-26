# 固定 FSRS 调度依赖

状态：Windows x64 原生 / 实际 WASM 对照通过，使用显式本地数学兼容修订 `fsrs-rs/6.6.2+libm-0.2.16`。未修改的上游包存在跨端取整差异，其失败证据仍保留。

- 上游：[open-spaced-repetition/fsrs-rs v6.6.2](https://github.com/open-spaced-repetition/fsrs-rs/tree/v6.6.2)，crates.io 包名 `fsrs`，精确版本 `=6.6.2`，默认 features 关闭。
- 发布包对应提交：`6088949b99d44f46f5328fd7289d209b9a842b68`，来自包内 `.cargo_vcs_info.json`。
- 许可：BSD-3-Clause，Copyright (c) 2023, Open Spaced Repetition；保留未修改的 [LICENSE](LICENSE)。项目 MIT 不覆盖本依赖。分发含该实现的二进制 / WASM 时必须附带此许可、版权及免责声明，不得借上游名称背书。
- 官方 21 个默认参数直接使用该版本的 `DEFAULT_PARAMETERS`，完整数值随每次输出记录。没有复制 Anki 代码、参数优化或间隔扰动。
- [本地源码及补丁范围](../../vendor/fsrs-6.6.2/LEXIQUE-PATCH.md)：只修改包 manifest 与标量调度数学函数，使用固定 libm 0.2.16，保留公式和官方参数；不冒称未修改的上游发行版。该目录保留上游 BSD 许可，[libm 许可](../libm-0.2.16/LICENSE.txt)单独保留。
- [Cargo.lock](../../Cargo.lock) 固定本地 fsrs 与 libm 版本；原发布包哈希及逐文件来源对照见 [源码审计](../../docs/evidence/issue-13/vendor-audit.json)。实算、传递依赖许可、构建体积与复验入口见 [#13 证据](../../docs/evidence/issue-13/README.md)。
