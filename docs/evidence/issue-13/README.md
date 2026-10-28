# 固定 FSRS 核心与原生 / WASM 验证

日期：2026-10-26。范围：[子工单 #13](https://github.com/zardonc/lexique/issues/13)，来自 [M2 规格 #12](https://github.com/zardonc/lexique/issues/12)。本票只交付共享调度及其验证链，不改用户库、IPC 或评分界面。TTS 由 #14 独立验证；生成后需人工试听确认内容及发音，当前没有生成或宣称语音通过。

**当前结论：Windows x64 原生 / 实际 WASM 一致性通过，使用显式数学兼容修订 `fsrs-rs/6.6.2+libm-0.2.16`。** 未修改上游包的两个取整反例已定位到平台 `exp` / `powf` 差异；隔离重建、Debug / Release 及旧编译器验证没有发现环境错配。仅修复依赖的标量数学路径后，71+4,096 样例全部通过，记忆数值实测差为零。它是固定 6.6.2 的本地兼容补丁，不是未修改的官方包；[诊断、独立复现与源码审计](math-diagnosis.md)单独保留。未验证平台和完整 M2 不据此宣称完成。

2026-10-28：五项条件重新验收通过；完整检查、审查结果与未运行边界见[收尾记录](closeout-2026-10-28.md)。以下保留初次实现及修复的历史记录。

| 固定反例 | 原生间隔 / 到期 Unix 秒 | WASM 间隔 / 到期 Unix 秒 |
| --- | --- | --- |
| regression-half-day-hard-native-wasm（修复前） | 10,742 / 2,721,006,000 | 10,741 / 2,720,919,600 |
| regression-half-day-good-native-wasm（修复前） | 1,425 / 1,916,013,600 | 1,424 / 1,915,927,200 |
| regression-half-day-hard-native-wasm（修复后） | 10,741 / 2,720,919,600 | 10,741 / 2,720,919,600 |
| regression-half-day-good-native-wasm（修复后） | 1,424 / 1,915,927,200 | 1,424 / 1,915,927,200 |

修复前请求及双方状态保存在 [原失败报告](conformance-unpatched-windows-x64.json)，修复后产物及全量结果见 [当前机器报告](conformance-windows-x64.json)。默认 Cargo features 无可用的统一数学后端选项，因此显式 [vendored patch](../../../vendor/fsrs-6.6.2/LEXIQUE-PATCH.md)仅把原公式的数学函数改用固定 libm；未拦截全局符号、修改应用学习步骤或改变取整规则。历史标识区分该修订与原上游包，拒绝混用。

## 来源与许可

固定 `fsrs = "=6.6.2"`，上游提交 `6088949b99d44f46f5328fd7289d209b9a842b68`；[官方调度示例](https://github.com/open-spaced-repetition/fsrs-rs/blob/v6.6.2/examples/schedule.rs)及[推理 API / 默认参数](https://github.com/open-spaced-repetition/fsrs-rs/blob/v6.6.2/src/inference.rs)为参考。没有复制 Anki 的 AGPL 实现，也没有静默换库。

BSD-3-Clause 许可原文、版权与再分发义务保留在 [资源声明](../../../resources/fsrs-6.6.2/README.md)；新增 [libm 许可](../../../resources/libm-0.2.16/README.md)单独保留。原 fsrs 包的哈希及逐文件对照见 [源码审计](vendor-audit.json)，本地源码由 Git 固定；libm 的包版本和 checksum 由 Cargo.lock 固定。默认参数直接来自上游 21 个 `f32` 常量，输出逐次携带具体数值；接口拒绝其他版本、自定义参数和无法识别的历史时区库。

直接依赖为本地兼容修订的 `fsrs 6.6.2`、`chrono 0.4.45`、`chrono-tz 0.10.4`（内置 IANA 2025b）、既有 `serde 1.0.229` 与 `serde_json 1.0.154`；fsrs 新增精确 `libm 0.2.16`、关闭其默认 features。完整普通 / 宏依赖及声明许可见 [依赖表](dependencies.txt)，精确版本由 [Cargo.lock](../../../Cargo.lock)固定。fsrs 自身没有可关闭普通优化器依赖的 feature；虽然上游含训练 / 模拟模块，此入口只调用 `FSRS::default().next_states`，没有暴露这些能力。

WASM 中上游 rand 传递依赖的 getrandom 使用其[官方 unsupported 后端](https://docs.rs/getrandom/0.4.3/getrandom/#unsupported-backend)，任何意外取随机数均报 `UNSUPPORTED`。这只配置 `wasm32-unknown-unknown`；不影响原生随机源。实际调度 WASM 无宿主导入，无须模拟随机数、浏览器绑定或 wasm-bindgen CLI。将来若新增 WASM 随机能力，必须显式重新验证后端，不能复用本票结论。

## 公开契约

共享入口为 `lexique_core::scheduling::schedule(ScheduleRequest)`，原生 CLI 与 WASM 均经 `schedule_json` 严格 JSON 边界调用它。返回 `{"Ok": ...}` 或保留原因的 `{"Err": "..."}`。输入上限 16,384 UTF-8 字节，拒绝未知字段、非法评分、非整数或范围外保持率、无效日期 / 时区、无效记忆状态、矛盾阶段、未到期评分及卡片 / 全局最近评分之后的时钟倒退。时间为 Unix 整秒、有效 UTC 年份 1–9999，不读取本机当前时钟或时区。

`card` 包含 `phase`（new / learning_first / learning_last / review / relearning）、`memory`（stability / difficulty）、`last_review_at` 和 `due_at`。新卡后三项为 null；已学卡三项必须有效，记忆范围为上游的 stability 0.001–36,500、difficulty 1–10。请求另含 `rating`（again / hard / good / easy）、`now`、IANA `timezone`、可选全局 `latest_review_at`。`retention_percent` 默认 90、接受 70–99 的整数。可显式携带记录的 `model` 和 `timezone_database`，只接受当前固定模型及 2025b。

每次评分先执行 FSRS，再套用 #12 的完整步骤表，因此分钟步骤同样更新记忆。分钟以实际时刻加 60 / 330 / 600 / 900 秒；长期用该次 FSRS 间隔四舍五入后限于 1–36,500 天，以当前时区日历学习日加天数，于目标日 04:00 到期。不是将 UTC 秒数除以 86,400；已有 `due_at` 始终按绝对时刻判断，换时区不移动它。极少数历史时区若目标 04:00 不存在或歧义，明确返回错误，不猜选；固定样例覆盖 Apia 跳日失败。

调用方须保存完整请求和结果，包括原始状态、实际评分时间 / 时区、返回的经过学习日、模型版本 / 参数、保持率和时区库。重放使用原记录，不用最新设置重建历史。此票验证重复执行同一请求确定，以及改变保持率只影响后续排程、不会改变既有输入或所用记忆模型；不声称已交付历史持久化。

WASM 同步 ABI：`schedule_request(len)` 返回输入缓冲区指针（超限返回 0）；宿主写入准确长度后调用 `schedule_execute()` 取得输出字节数，再以 `schedule_result()` 和导出的 `memory` 读取 UTF-8 JSON。下一次请求使旧输入指针失效，下一次执行使旧输出指针失效；必须在调用后重新取得 memory.buffer 视图。无调用期间并发访问。Rust 端不从宿主传入的任意指针创建 slice，没有新增 unsafe 内存操作。

## 覆盖与误差

[固定向量](../../../crates/core/tests/fixtures/scheduling.json)共 71 条：20 个阶段 / 评分组合；凌晨 4 点前后、同日 / 跨日、23 / 25 小时学习日、DST 缺失 / 重复小时中的完整分钟、跨 DST 长期间隔；三个时区的原到期时间；默认及保持率端点、取整进位及上下界；非法输入、时钟倒退、模型和时区库版本错误，以及两个跨端取整反例。初始四档记忆常量来自上游公开示例，步骤和 UTC 期望来自 #12 及对应日历；没有运行本项目调度器来生成这些期望。反例只要求跨端严格一致，没有将某端结果硬写为正确答案。

另以固定种子 13 的 4,096 个合法向量校准跨端差异，覆盖所有阶段、评分、保持率及 0.001–36,500 稳定度。原上游包最大绝对差 0.037、最大相对差 `1.176814165593972e-6`，见 [原记录](conformance-unpatched-windows-x64.json)；补丁后两项实测均为零，见 [当前记录](conformance-windows-x64.json)。仅 stability / difficulty 继续保留原经校准的 `abs(a-b) <= 1e-6 + 2e-6 * abs(native)`，没有扩大容差。阶段、步骤秒数、整数间隔、绝对到期秒、经过天数和元数据仍严格相等；原两条反例现在通过。该校准集不是对全部可能输入或未运行平台的误差证明。

额外公开入口检查覆盖逐步学习 / 遗忘 / 重学 / 再毕业、每次记忆更新、记录模型与保持率重放、坏 JSON / 超限、WASM 非 UTF-8、空请求和超限缓冲区。保留 schema_version ABI 检查，但不以它代替调度执行。

## 复验

本机 Windows x64、Rust/Cargo 1.99.1（LLVM 23.1.1）、Node 24.22.1、pnpm 11.25.0、Python 3.12.13。全局 Python Store 别名失效，验证 shell 临时将 `C:\Soft\1A-Coding\PythonPortable\python\cpython-3.12.13-windows-x86_64-none` 加到 PATH；不改变用户全局设置。其他机器使用 README 要求的 Python 3.12+ 即可。

```sh
pnpm lint
pnpm typecheck
pnpm build
pnpm rust:fmt
pnpm rust:check
cargo clippy --locked --workspace --all-targets -- -D warnings
pnpm rust:test:build
pnpm test:rust
pnpm scheduling:build
node scripts/check-wasm.mjs
pnpm test:scheduling
pnpm test:resources
pnpm test:tooling
pnpm test:e2e
cargo check --locked --workspace --target aarch64-pc-windows-msvc
```

原生 / WASM 编译在执行前完成；Rust runner 保留硬 60 秒总执行限制，对照原生子进程也有 60 秒 timeout。对照报告写入被忽略的 `target/issue-13/conformance.json`，提交副本保留本机证据。CI backend 已新增核心测试、原生 release 参考程序、实际 WASM 对照及报告上传；本轮未推送或运行远端 CI。

初次交付通过：lint / typecheck / build、Rust 格式 / workspace check / Clippy、5 项核心及既有 15 项 Rust 测试、7 项资源及 7 项工具测试、Edge E2E 13 项；对照最终失败的两个反例保存在原报告，独立审查的两项 P1 均指向该偏差。此次修复重跑 Rust 格式、workspace Clippy、预编译后 6 项核心及既有 15 项 Rust 测试、release 71+4,096 对照，全部通过；Windows ARM64 workspace 编译也通过，尚无实机证据。UI 未改，此次不重复前端 / E2E；没有再次委派独立审查。macOS 无本机执行环境，未编译或运行。

首次失败与处理：首条测试先因公开入口不存在编译失败；非法保持率回归先得到成功结果，补齐边界校验后通过；历史时区库 / 全局非法时间向量先失败，增加显式版本和时间校验后通过。首次 WASM 因 getrandom 缺目标支持失败，wasm_js 试编译仍带宿主导入，最终改为明确失败的官方 unsupported 后端并实际验证无导入。原生首条记忆断言曾误将 f32 扩宽后的数值当 JSON 字面量，改用上游独立常量与浮点断言。资源准备和 workspace 首次因 Python 别名失败，使用已安装解释器后通过。既有增量缓存硬链接回退、Zod / Rollup 注释与终端颜色警告未抑制。

## 产物与限制

产物尺寸 / gzip 与 SHA-256 在 [机器报告](conformance-windows-x64.json)中。当前原生参考 CLI 为 1,547,776 字节，gzip 294,610；WASM 为 1,109,906 字节，gzip 205,917。压缩口径是单产物 gzip，不是最终桌面下载量；原生包含 CLI / JSON / 日历层，不能视为纯 FSRS 算法大小。相对未修改的候选，gzip 分别增加 2,275 / 923 字节。

发布包 / 许可的 SHA-256 在同目录 `source-checksums.txt`。依赖由锁定 Cargo 获取，运行评分不需要网络、模型文件、系统动态推理库或首次下载；Windows CLI 仍有通常的系统运行依赖，此处没有验证干净机器分发。已有 portable 未重打，用户数据库未打开或迁移。

未验证 macOS 实际编译 / 运行、Windows ARM64 实机、浏览器学习产品、完整 M2 存储 / 额度 / 队列 / 撤销或 TTS；不替代既有 M1 macOS 阻塞及 M5 四目标发行门槛。无需启动原生 UI 或新增常驻服务，E2E 自有服务由 Playwright 关闭。
