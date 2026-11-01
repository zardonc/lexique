# PR #23 复核与批量语音重试修复

2026-11-01。复核 [PR #23](https://github.com/zardonc/lexique/pull/23)，基点 `45fc84af8dd4223422d3d25ec1ae5cbc8dbca4e5`，原 HEAD `210f8a38780823f8505e2fd5a7414fe58e1498a6`。用户确认继续核验并修复这个 PR。原标题及说明只描述 #13，但分支已包含 #13–#22 的 M2 实现；本轮按当前 [M2 规格](../../m2-spec.md)复核，并同步说明实际范围。

## 问题与修复

批量加入复习先调用 `cachedSpeech`。坏缓存使这个只读命令抛出 `Invalid cached speech audio`，随后不会调用生成命令；重复点击“重试”仍停在同一错误。规格要求再次加入能修复缺失或失败音频，并逐词反馈真实结果。

批量路径现直接调用已有 `generateSpeech`：原生命令在锁内复用有效缓存，缺失时生成，损坏时安全替换，其他读写或合成错误继续返回。未添加依赖、IPC、迁移或新的缓存策略；卡片仍先通过已有幂等加入接口保存，语音失败不会撤回卡片或重置进度。

`notebook.spec.ts` 的新回归先失败：期待可重试的合成错误 `voice unavailable`，实际仅出现缓存损坏错误。修复后同一测试通过，验证生成失败可见、重试成功且重试入口移除。生词本单文件 22 项和完整浏览器 72 项均通过。初次失败截图及上下文保存在本机 `target/pr23-red/`。

原生 `native-speech-check.mjs` 使用既有合成数据库的独立副本和本轮重建的程序，覆盖真实坏缓存的批量修复。初次快照比较的是新卡默认状态；复审后加强准备：通过公开评分接口建立真实已学习进度，再主动暂停，明确断言非新卡、评分修订大于零及暂停为真，然后确认批量修复后这些状态完全相同。随后禁用副本中的语音引擎，重复批量加入仍成功，有效音频字节未变。加强后的原生检查全部通过；既有并发生成、失败替换保留旧音频、成功替换、实际播放、0.6 倍重播、隐藏听音正面词头及重启恢复继续通过。用户学习数据及既有交付未改写。

## 代码审查

两名全新上下文审查代理分别检查原 PR 的 Standards 和 Spec，遵循仓库规范、领域文档及最新决定。Standards：0 项确定违规、0 项需处理的基线异味。Spec：1 项 P2，即上述批量坏缓存问题；已复现并修复。修复差异由另外两名全新上下文代理复审：规范 0 项、规格 0 项未解决问题。复审指出默认新卡快照不足以验证既有学习状态，已通过上述真实评分及主动暂停准备补齐，并单独重跑原生检查通过。

## 本轮验证

PowerShell 7、Node 24.22.1、pnpm 11.25.0、uv 管理的 Python 3.14.10、Rust/Cargo 1.99.1。验证进程使用仓库 `.tooling/corepack` 缓存、真实 Python 路径及项目测试临时目录；未改系统设置。浏览器为项目 Playwright Chromium headless shell，固定 UTC；浏览器 IPC fixture 不作为原生存储证据。

| 命令 / 边界 | 结果 |
| --- | --- |
| `pnpm lint`、`pnpm typecheck`、`pnpm build` | 通过 |
| `pnpm rust:fmt`、`cargo check --locked --workspace` | 通过 |
| `cargo clippy --locked --workspace --all-targets -- -D warnings` | 通过 |
| `pnpm rust:test:build` 后 `pnpm test:rust` | 70 项通过；预编译独立，执行硬 60 秒限制保持 |
| `cargo test --locked -p lexique-core --test scheduling` | 6 项通过 |
| `pnpm scheduling:build`、`node scripts/check-wasm.mjs`、`node scripts/check-scheduling.mjs` | 重建后实际执行；71 固定 + 4,096 校准向量排程严格一致，最大记忆数值误差为 0 |
| `pnpm test:e2e -- --workers=1 notebook.spec.ts --grep 'batch enrollment retries corrupt audio'` | 修复前失败、修复后通过 |
| `pnpm test:e2e -- --workers=2 notebook.spec.ts` | 22 项通过 |
| `pnpm test:e2e -- --workers=2` | 完整 72 项通过 |
| `pnpm test:resources` | 7 项通过 |
| `pnpm test:tooling` | 构建参数 2 项、portable 7 项、语音 portable 2 项通过，真实语音资源打包项初次因输入未设置而跳过 |
| 设置真实 bundle / encoder / sources 后 `pnpm test:speech:portable` | 3 项全部通过，补齐上述跳过项 |
| `cargo check --locked --workspace --target aarch64-pc-windows-msvc` | 通过；未推断实机运行通过 |
| `pnpm desktop:build --no-bundle` | locked Windows x64 release 通过 |
| `native-speech-check.mjs`，指定本轮程序、既有语音包和合成库的独立副本 | 批量坏缓存修复、有效缓存复用、进度 / 暂停保留及既有真实语音检查通过；文件 symlink 分支因权限跳过 |

最终原生副本位于 `target/pr23-native-speech-learned-20261101/`；初次副本保留在 `target/pr23-native-speech-20261101/`。程序 SHA-256 为 `ea796007bad7b06e8326f2842f105035e826ccb51355ecf823ae55ac312f8694`。调度产物哈希及体积报告位于 `target/issue-13/conformance.json`，模型修订仍为 `fsrs-rs/6.6.2+libm-0.2.16`。既有数值容差未扩大，日期、整数间隔、步骤、阶段及元数据继续严格比较。

## CI 与限制

实时读取原 HEAD 的 GitHub check runs：[PR CI](https://github.com/zardonc/lexique/actions/runs/41282397166) 与 [push CI](https://github.com/zardonc/lexique/actions/runs/41282393117) 各 10 项全部通过，包含 Windows / macOS × x64 / ARM64 编译。本轮本机没有 macOS 环境，不把这些 CI 编译结果描述成实机运行或离线发行通过。

默认 pnpm 启动器的联网下载和 WindowsApps Python 占位程序最初失败，使用既有缓存和 uv 解释器后恢复。受限环境的 Vite 子进程未就绪及浏览器 localhost 访问被拒绝；相关尝试均结束。普通权限请求曾因自动审批服务错误未执行，随后权限审批恢复，在普通权限下完成标准回归。另在受限环境用本地构建资源的离线路由复现了同一界面错误；它不替代上表标准回归。既有 Rust 硬链接复制和 Rollup 注释警告保留，没有压低诊断。

本轮没有施加网络限制或重做干净 Windows 验收；既有用户断网实测结论及平台限制沿用 [#22 收尾报告](../issue-22/review-2026-11-01.md)。本轮文件 symlink 检查跳过，既有 junction 拒绝证据保持，不冒称本轮补测。macOS / ARM64 实机、连续 24 小时缓存清理、完整默认资源及正式完整版 / Lite 发行仍未验收。没有自动合并 PR。
