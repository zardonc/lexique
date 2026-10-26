# 固定 FSRS 核心与原生 / WASM 验证

日期：2026-10-26。范围：[子工单 #13](https://github.com/zardonc/lexique/issues/13)，来自 [M2 规格 #12](https://github.com/zardonc/lexique/issues/12)。本票只交付共享调度及其验证链，不改用户库、IPC 或评分界面。TTS 由 #14 独立验证；生成后需人工试听确认内容及发音，当前没有生成或宣称语音通过。

**最终结论：准入 blocked，#13 尚未完成。** 初始 69+4,096 样例通过，但独立审查扩大合法输入后发现原生与 WASM 排程相差一天。两个反例已固化；最终 71+4,096 样例有 2 项失败，`pnpm test:scheduling` 写出证据后退出 1，CI 同样保持失败并上传报告。不会将调查完毕冒充准入通过。

| 固定反例 | 原生间隔 / 到期 Unix 秒 | WASM 间隔 / 到期 Unix 秒 |
| --- | --- | --- |
| regression-half-day-hard-native-wasm | 10,742 / 2,721,006,000 | 10,741 / 2,720,919,600 |
| regression-half-day-good-native-wasm | 1,425 / 1,916,013,600 | 1,424 / 1,915,927,200 |

请求、双方记忆状态和完整结果保存在 [最终机器报告](conformance-windows-x64.json)。例如第一个反例的 stability 分别为 1156.5043 / 1156.5032，仍在记忆容差内，但原始 FSRS 间隔跨过半天取整点。固定版本源码使用平台的 f32 `powf` / `exp` / `ln`，Cargo features 没有可用的确定性数学后端选项。此轮未静默换库、拦截全局数学符号、修改算法或用新的取整容差掩盖偏差；在获得经验证的跨端确定性方案前，依赖 #13 的集成继续阻塞。

## 来源与许可

固定 `fsrs = "=6.6.2"`，上游提交 `6088949b99d44f46f5328fd7289d209b9a842b68`；[官方调度示例](https://github.com/open-spaced-repetition/fsrs-rs/blob/v6.6.2/examples/schedule.rs)及[推理 API / 默认参数](https://github.com/open-spaced-repetition/fsrs-rs/blob/v6.6.2/src/inference.rs)为参考。没有复制 Anki 的 AGPL 实现，也没有静默换库。

BSD-3-Clause 许可原文、版权与再分发义务保留在 [资源声明](../../../resources/fsrs-6.6.2/README.md)。发布包及许可 SHA-256 见下方；包哈希与 Cargo.lock 一致。默认参数直接来自上游 21 个 `f32` 常量，输出逐次携带具体数值；接口拒绝其他版本、自定义参数和无法识别的历史时区库。

直接依赖为 `fsrs 6.6.2`、`chrono 0.4.45`、`chrono-tz 0.10.4`（内置 IANA 2025b）、既有 `serde 1.0.229` 与 `serde_json 1.0.154`；完整普通 / 宏依赖及声明许可见 [依赖表](dependencies.txt)，精确版本及校验和由 [Cargo.lock](../../../Cargo.lock)固定。fsrs 自身没有可关闭普通优化器依赖的 feature；虽然上游含训练 / 模拟模块，此入口只调用 `FSRS::default().next_states`，没有暴露这些能力。

WASM 中上游 rand 传递依赖的 getrandom 使用其[官方 unsupported 后端](https://docs.rs/getrandom/0.4.3/getrandom/#unsupported-backend)，任何意外取随机数均报 `UNSUPPORTED`。这只配置 `wasm32-unknown-unknown`；不影响原生随机源。实际调度 WASM 无宿主导入，无须模拟随机数、浏览器绑定或 wasm-bindgen CLI。将来若新增 WASM 随机能力，必须显式重新验证后端，不能复用本票结论。

## 公开契约

共享入口为 `lexique_core::scheduling::schedule(ScheduleRequest)`，原生 CLI 与 WASM 均经 `schedule_json` 严格 JSON 边界调用它。返回 `{"Ok": ...}` 或保留原因的 `{"Err": "..."}`。输入上限 16,384 UTF-8 字节，拒绝未知字段、非法评分、非整数或范围外保持率、无效日期 / 时区、无效记忆状态、矛盾阶段、未到期评分及卡片 / 全局最近评分之后的时钟倒退。时间为 Unix 整秒、有效 UTC 年份 1–9999，不读取本机当前时钟或时区。

`card` 包含 `phase`（new / learning_first / learning_last / review / relearning）、`memory`（stability / difficulty）、`last_review_at` 和 `due_at`。新卡后三项为 null；已学卡三项必须有效，记忆范围为上游的 stability 0.001–36,500、difficulty 1–10。请求另含 `rating`（again / hard / good / easy）、`now`、IANA `timezone`、可选全局 `latest_review_at`。`retention_percent` 默认 90、接受 70–99 的整数。可显式携带记录的 `model` 和 `timezone_database`，只接受当前固定模型及 2025b。

每次评分先执行 FSRS，再套用 #12 的完整步骤表，因此分钟步骤同样更新记忆。分钟以实际时刻加 60 / 330 / 600 / 900 秒；长期用该次 FSRS 间隔四舍五入后限于 1–36,500 天，以当前时区日历学习日加天数，于目标日 04:00 到期。不是将 UTC 秒数除以 86,400；已有 `due_at` 始终按绝对时刻判断，换时区不移动它。极少数历史时区若目标 04:00 不存在或歧义，明确返回错误，不猜选；固定样例覆盖 Apia 跳日失败。

调用方须保存完整请求和结果，包括原始状态、实际评分时间 / 时区、返回的经过学习日、模型版本 / 参数、保持率和时区库。重放使用原记录，不用最新设置重建历史。此票验证重复执行同一请求确定，以及改变保持率只影响后续排程、不会改变既有输入或所用记忆模型；不声称已交付历史持久化。

WASM 同步 ABI：`schedule_request(len)` 返回输入缓冲区指针（超限返回 0）；宿主写入准确长度后调用 `schedule_execute()` 取得输出字节数，再以 `schedule_result()` 和导出的 `memory` 读取 UTF-8 JSON。下一次请求使旧输入指针失效，下一次执行使旧输出指针失效；必须在调用后重新取得 memory.buffer 视图。无调用期间并发访问。Rust 端不从宿主传入的任意指针创建 slice，没有新增 unsafe 内存操作。

## 覆盖与误差

[固定向量](../../../crates/core/tests/fixtures/scheduling.json)共 71 条：20 个阶段 / 评分组合；凌晨 4 点前后、同日 / 跨日、23 / 25 小时学习日、DST 缺失 / 重复小时中的完整分钟、跨 DST 长期间隔；三个时区的原到期时间；默认及保持率端点、取整进位及上下界；非法输入、时钟倒退、模型和时区库版本错误，以及两个跨端取整反例。初始四档记忆常量来自上游公开示例，步骤和 UTC 期望来自 #12 及对应日历；没有运行本项目调度器来生成这些期望。反例只要求跨端严格一致，没有将某端结果硬写为正确答案。

另以固定种子 13 的 4,096 个合法向量校准 native f32 数学库与 WASM 的差异，覆盖所有阶段、评分、保持率及 0.001–36,500 稳定度。完整实测见 [Windows x64 对照](conformance-windows-x64.json)：最大绝对差 0.037，最大相对差 `1.176814165593972e-6`。仅 stability / difficulty 使用 `abs(a-b) <= 1e-6 + 2e-6 * abs(native)`；相对项略高于此次实测上界，并非无证据地放宽。阶段、步骤秒数、整数间隔、绝对到期秒、经过天数和元数据要求严格相等；两个反例相差一天，最终检查真实失败。该校准集不是对全部可能输入或未运行平台的误差证明。

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

已通过：lint / typecheck / build、Rust 格式 / workspace check / Clippy、5 项核心及既有 15 项 Rust 测试、7 项资源及 7 项工具测试、Edge E2E 13 项；Windows ARM64 workspace 编译通过，无实机运行证据。失败：最终 71+4,096 原生 / WASM 对照的两个取整反例。独立规范轴 1 项 P1、规格轴 1 项 P1，指向同一未解决的一天偏差，未以文档解释将其关闭。macOS 无本机执行环境，未编译或运行。本次 UI 未修改，E2E 只作既有行为回归。

首次失败与处理：首条测试先因公开入口不存在编译失败；非法保持率回归先得到成功结果，补齐边界校验后通过；历史时区库 / 全局非法时间向量先失败，增加显式版本和时间校验后通过。首次 WASM 因 getrandom 缺目标支持失败，wasm_js 试编译仍带宿主导入，最终改为明确失败的官方 unsupported 后端并实际验证无导入。原生首条记忆断言曾误将 f32 扩宽后的数值当 JSON 字面量，改用上游独立常量与浮点断言。资源准备和 workspace 首次因 Python 别名失败，使用已安装解释器后通过。既有增量缓存硬链接回退、Zod / Rollup 注释与终端颜色警告未抑制。

## 产物与限制

产物尺寸 / gzip 与 SHA-256 在 [机器报告](conformance-windows-x64.json)中。当前原生参考 CLI 为 1,544,192 字节，gzip 292,335；WASM 为 1,109,194 字节，gzip 204,994。压缩口径是单产物 gzip，不是最终桌面下载量；原生包含 CLI / JSON / 日历层，不能视为纯 FSRS 算法大小。

发布包 / 许可的 SHA-256 在同目录 `source-checksums.txt`。依赖由锁定 Cargo 获取，运行评分不需要网络、模型文件、系统动态推理库或首次下载；Windows CLI 仍有通常的系统运行依赖，此处没有验证干净机器分发。已有 portable 未重打，用户数据库未打开或迁移。

未验证 macOS 实际编译 / 运行、Windows ARM64 实机、浏览器学习产品、完整 M2 存储 / 额度 / 队列 / 撤销或 TTS；不替代既有 M1 macOS 阻塞及 M5 四目标发行门槛。无需启动原生 UI 或新增常驻服务，E2E 自有服务由 Playwright 关闭。
