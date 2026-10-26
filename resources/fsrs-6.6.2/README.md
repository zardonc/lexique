# 固定 FSRS 调度依赖

状态：**准入 blocked**。实际 native / WASM 的合法输入会因平台浮点数学差异取整到不同天数；保留固定候选和失败证据，不用于正式评分链路。

- 上游：[open-spaced-repetition/fsrs-rs v6.6.2](https://github.com/open-spaced-repetition/fsrs-rs/tree/v6.6.2)，crates.io 包名 `fsrs`，精确版本 `=6.6.2`，默认 features 关闭。
- 发布包对应提交：`6088949b99d44f46f5328fd7289d209b9a842b68`，来自包内 `.cargo_vcs_info.json`。
- 许可：BSD-3-Clause，Copyright (c) 2023, Open Spaced Repetition；保留未修改的 [LICENSE](LICENSE)。项目 MIT 不覆盖本依赖。分发含该实现的二进制 / WASM 时必须附带此许可、版权及免责声明，不得借上游名称背书。
- 官方 21 个默认参数直接使用该版本的 `DEFAULT_PARAMETERS`，完整数值随每次输出记录。没有复制 Anki 代码、参数优化或间隔扰动。
- 包校验和由 [Cargo.lock](../../Cargo.lock) 固定；实算、传递依赖许可、构建体积与复验入口见 [#13 证据](../../docs/evidence/issue-13/README.md)。此目录没有另存或修改算法源码。
