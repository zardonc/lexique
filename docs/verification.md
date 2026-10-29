# 骨架验证记录

## M2 评分撤销（issue #18）

2026-10-29：隔离 Windows x64 原生窗口通过评分、重启、撤销和继续评分；检查使用合成数据库副本，不触及用户库。存储回归覆盖新卡、分钟步骤、长期评分、重启、提交失败、重复撤销、跨学习日及 UTC 到东京的额度归属。Edge 45 项通过，包含响应丢失和明确拒绝后的重试；浏览器替身不证明原生持久化。lint、typecheck、build、Rust 格式与 workspace 编译通过；后端先预编译再执行全套测试，保留 60 秒超时。Windows 原生无 bundle 构建及隔离演示通过。

旧版本评分可能只保存队列数字位置，无法可靠重建已变化的原顺序。用户同意此时撤销进度和额度、按当前规则重建队列，并提示顺序可能变化；构造的 schema 5 文件升级、跨日撤销及重启重试通过。该旧库不是由历史二进制生成，原生旧快照、跨时区失败路径及其他目标平台尚未实机验收；不据此宣称 M2 或发行就绪。

## M2 每日设置与复习队列（issue #17）

2026-10-29 验收复核：规范和规格审查发现队列检查延长未播放音频的缓存寿命，以及评分前的数值位置不足以支持后续撤销恢复。修复后，Windows 原生隔离测试确认队列检查不更新访问时间、实际音频读取会更新访问时间，设置和两张队列卡重启恢复；存储回归确认评分事务保存评分前的卡片顺序。此前两项检查均先复现失败。复核后 `pnpm lint`、`pnpm typecheck`、`pnpm build`、`cargo fmt --all --check`、`cargo check --locked --workspace`、`pnpm desktop:build --no-bundle`、预编译后 `pnpm test:rust`（43 项）通过；Edge 42 项在修复前通过，修复未改前端行为。真实撤销及保留后续编辑仍由 #18 验证。完整队列快照随评分次数和队列长度增长，尚未测量大词库成本。

2026-10-29：schema 5 增量保存每日设置及队列顺序；分钟卡优先，新旧卡按实际额度穿插，并在评分事务中保存队列上下文。音频不可用的听音卡不阻塞识义卡；播放失败可跳过而不评分。

- `pnpm lint`、`pnpm typecheck`、`pnpm build`、`cargo fmt --all --check`、`cargo check --locked --workspace` 和 `pnpm desktop:build --no-bundle`：通过。构建保留既有 Zod / Rollup 注释警告和 Rust 增量缓存硬链接回退警告。
- `pnpm rust:test:build` 后 `pnpm test:rust`：43 项通过，包含固定时间下的额度、等待、同词暂缓、混排、上调额度及数据库重启恢复；测试执行保留 60 秒上限。
- `pnpm test:e2e`：Edge 42 项通过，包含设置保存失败和听音播放失败后跳过；浏览器使用原生接口 fixture，不证明原生持久化。
- `pnpm wasm:build` 后 `node scripts/check-wasm.mjs`：WASM ABI 实际执行通过，不作为排程一致性验收。
- `node apps/desktop/e2e/native-queue-check.mjs`：Windows x64 release 在隔离合成数据库副本上启动真实窗口；通过原生 IPC 保存设置，重启后设置及一张队列卡保持一致。测试进程已停止，未修改用户学习数据。

本次原生检查未覆盖多卡混排、原生播放失败或禁网；多卡顺序由 SQLite 测试覆盖。跨重启撤销属于后续 #18，其他平台和发行验收尚未验证。构建使用 Node.js 24.22.1 和 Python 3.14.10；资源准备输出中的 Python 3.12.13 是既有缓存清单的生成版本，不代表本次执行版本。

## M2 单卡评分与事务恢复（issue #16）

2026-10-29：schema 4 增量保存卡片进度、评分前状态、排程结果和评分上下文。识义卡翻面及听音卡成功播放并翻面后可评分；同词暂缓、每日额度、到期和时钟倒退在资格查询及评分事务内检查。失败保留操作身份以便重试，明确拒绝可刷新状态。此前 #15 尚无复习队列，完整队列及撤销分别由后续工单实现。

- `pnpm lint`、`pnpm typecheck`、`pnpm desktop:build --no-bundle`、`cargo check --locked --workspace`、`cargo fmt --all --check`：通过。
- `pnpm rust:test:build`、`pnpm test:rust`：通过，包括事务失败回滚、重试、时区变化、额度和同词暂缓测试；测试执行沿用 60 秒上限。
- `pnpm scheduling:build`、`pnpm test:scheduling`、`node scripts/check-wasm.mjs`：通过；71 个固定向量及 4096 个校准向量的原生与 WASM 结果一致。
- `pnpm test:e2e`：Edge 37 项通过；浏览器测试模拟原生接口，不证明原生持久化。
- `node apps/desktop/e2e/native-rating-check.mjs`：Windows x64 release 隔离副本中，真实窗口翻面并评分，重启后显示未到期；原生 IPC 回读进度、同操作重试及同词卡暂缓均通过。使用 #15 测试数据库副本，未修改用户学习数据，测试进程已停止。

原生界面本轮仅实测识义卡的“记得”路径；其他评分、听音门槛及失败路径由自动化测试覆盖。其他平台、完整复习队列、撤销和发行验收不在本次验证范围。

## M2 单个生词双卡与原生语音（issue #15）

2026-10-29 人工验收补记：用户确认交付音频无问题，新生成 `école` 的听感通过。下文“待用户确认”保留为技术验收时的记录；完整桌面禁网、连续 24 小时及其他平台等限制不因此改变。实现提交为 `e544dd9`，下一条主线为 #16 的四档评分、真实排程与事务恢复。

2026-10-29：#15 的实现及 Windows x64 技术验收已完成。schema 3 增量保留真实 schema 2 生词记录的完整公开内容，不批量建卡；加入复习通过立即事务创建两卡，重复及重启重试保持身份。听音卡接入真实 Supertonic 3 / 目标 48 kbit/s Ogg Opus，缺失和失败可重试，识义卡不受语音失败影响。卡面、机器合成标识、0.6 倍重播、播放请求取消、七天缓存清理和持久诊断已接入。

| 检查 | 结果 |
| --- | --- |
| `pnpm lint` / `pnpm typecheck` / `pnpm build` | 通过；保留 Vite 对依赖 PURE 注释的警告 |
| `pnpm test:e2e`，已安装 Edge | 28 项通过，其中复习界面 10 项；包含关闭后重开取消旧播放、成功重播后再次加入复用音频 |
| `pnpm rust:test:build` 后 `pnpm test:rust` | 29 项通过；预编译独立，执行阶段硬 60 秒超时保持不变 |
| `cargo fmt --all --check`、锁定离线全 workspace Clippy / 编译、`pnpm desktop:build --no-bundle` | 通过；保留增量缓存硬链接回退复制警告 |
| `pnpm wasm:build` 后 `node scripts/check-wasm.mjs` | 实际 WASM ABI 运行通过，schema 版本为 3；不当作新增 FSRS 验收 |
| `pnpm test:resources` / `pnpm test:tooling`，配置真实语音输入 | 7 项资源及 12 项工具检查通过，语音打包集成项未跳过 |
| `pnpm test:speech:cleanup`，配置最新 release | 4 项 Windows 原生检查通过：启动清理、占用 / 无关 / 非规范名保留、目录联接保护、锁协调、无 stderr 的持久诊断及重启重试 |
| `node apps/desktop/e2e/native-speech-check.mjs …` | 真实 WebView2 播放、正面隐藏、翻面不重播、0.6 倍重播、跳过及两卡 / 生词 / 音频重启恢复通过；损坏缓存、并发生成及失败 / 成功替换通过 |
| 便携包及独立音频解码 | 新语音开发 ZIP 165,205,335 字节，逐项校验 1,072 个文件；新 `école` 音频经 opusinfo / opusdec 确认有效，留给用户试听 |

测试使用隔离的合成生词库，没有修改用户学习数据；任务启动的测试进程已关闭。规范与规格两项代码审查未发现剩余缺陷，之前的播放失败标志、持久日志和废弃临时文件问题已修复并通过回归。命令、哈希、截图与试听文件见 [#15 完整证据](evidence/issue-15/README.md)。

限制：新音频的主观听感由用户确认；本轮原生窗口未施加网络限制，不能把实播结果描述为完整桌面禁网验收，既有 #14 禁网证据单独保留。缓存文件符号链接实测因权限跳过，目录联接保护已通过；连续运行 24 小时触发、跨进程同时生成、长时间播放清理及不支持硬链接的文件系统未测。其他平台、干净机器及 M5 完整版 / Lite 发行仍待后续；评分 / 排程持久化属于 #16。

### #15 实施过程（2026-10-28，历史）

以下“未完成”和“待验证”描述保留为当时的过程记录，当前结论以上节为准。

本轮补充 Windows x64 语音开发包入口：固定资源和编码器校验后打包，归档内容、许可、CRC 及不含 `Lexique-data` 的集成检查通过；损坏清单会失败并保留已有文件，原无语音打包测试 7 项通过。复习界面新增播放失败后再次加入的重新生成路径，浏览器完整测试 26 项、`pnpm lint`、`pnpm typecheck`、`cargo fmt --all --check`、桌面 `cargo check` / Clippy 和 Rust 测试通过。Windows 原生清理测试单独运行 3 项通过；并行运行时两项因 WebView2 临时日志尚未释放导致测试目录清理报错。

开发包隔离副本通过真实桌面 IPC 验证：损坏缓存可重新生成，同一进程内并发生成返回一致结果，禁用运行库后重新生成失败且旧文件不变，恢复运行库后可替换。测试使用独立测试数据库并保留音频供用户试听。现读取完成即释放音频文件句柄，播放使用内存中的副本；跨进程同时生成及长时间播放期间的清理仍需原生验证，24 小时定时触发尚未实测。开发包不是正式发行验收，实际音质由用户试听确认。

识义卡和听音卡翻面共用摘要、缺例句提示及按需展开的笔记和来源；听音卡缺少展开内容的回归测试先失败，修复后 `e2e/review.spec.ts` 的 5 项通过。`pnpm typecheck`、`pnpm lint` 和 `pnpm desktop:build --no-bundle` 通过；release 构建保留 Vite 对依赖注释的警告。

后续缓存恢复切片：界面调用 `speech_cached` 恢复音频，重复加入时复用已有音频；Edge fixture 的复习卡 6 项通过，`pnpm typecheck` 与离线 `cargo check --locked --workspace` 通过。缓存原生重启、并发、重解析点和不支持硬链接的文件系统尚未验收；目前无七天清理，不得视为 #15 完成。

本次接入 Windows 启动及每 24 小时的七天清理，并保留已读取音频的文件句柄直至进程退出。Windows release 的隔离启动测试 2 项通过：过期音频删除，新文件、占用文件及无关文件保留；语音目录指向目录联接时不清理目标。`cargo fmt --all --check`、`cargo check --locked --offline --workspace` 和 `pnpm desktop:build --no-bundle` 通过；编译保留缓存硬链接回退警告。24 小时周期、清理失败重试、生成 / 播放过程中的并发和完整发行资源仍未验收。

随后新增隔离失败路径：语音目录不可用时应用仍初始化，错误写入标准错误；修复目录后重启可清理过期文件。Windows 原生清理测试共 3 项通过；`pnpm rust:test:build` 后 `pnpm test:rust` 的 29 项测试通过，执行阶段保留 60 秒硬超时。24 小时周期、实际生成 / 播放期间并发和完整发行资源仍未验收。

继续补充缓存缺失后的恢复测试：已有两卡且音频缺失时，再次加入触发语音生成并显示可用，识义卡仍可见。Edge 中 `e2e/review.spec.ts` 7 项通过，`pnpm lint`、`pnpm typecheck` 通过；此测试模拟原生接口，不证明清理或真实播放。

Windows release 使用 `.tooling/issue-15-cleanup-retry-20261028/` 中独立测试库和开发暂存资源：启动时删除预置的过期音频，原生界面保留识义卡并显示听音卡待生成；再次加入后重新生成 6,709 字节的 Ogg，界面显示可用，数据库回读仍为原识义卡和听音卡 ID。打开听音卡后将测试文件访问时间设为八天前，第二个实例启动清理后文件仍在；测试进程已关闭。这不证明实际声音输出、禁网或标准发行包。

隔离目录 `.tooling/issue-15-native-cache-check/` 的 Windows x64 release 实测：打开既有测试词条并重新加入复习后，界面显示语音已生成，专用目录出现 9,311 字节的 `1.ogg`。关闭并保持启动会话存活后重启，不再次生成即可显示语音已生成；本次未测实际声音输出或断网行为。测试进程已关闭。

Windows x64 在 `.tooling/issue-15-native-check-20261028/` 使用独立数据目录启动 release 样例：首次窗口持续黑屏；以 `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--disable-gpu` 重启后界面正常。通过真实原生接口查找并保存 `école`、创建两卡、生成语音；听音正面未显示词头，打开后未显示播放错误，翻面显示释义、缺例句提示和笔记来源入口。未测量实际音频输出，也未限制网络，不能据此声称原生实播或离线验收通过。测试进程已关闭，黑屏根因尚未确定。浏览器测试使用原生接口 fixture，不证明实际播放。音频缓存及重启恢复、七天清理、正式资源集成及发行许可仍未完成；本票不能验收或视为完成。

本文件保留各阶段历史结果；较早记录中“待验收”或“未完成”的描述以当时为准。#13 后续独立规格审查及 #14 最终准入决定分别见[收尾记录](evidence/issue-13/closeout-2026-10-28.md)和[语音证据](evidence/issue-14/README.md)。

## Windows ARM64 开发样例交叉构建

2026-10-26：在 Windows x64 主机交叉构建 `aarch64-pc-windows-msvc` release 成功；构建前端时 TypeScript 和 Vite 构建通过。使用 `scripts/package_portable.py --architecture arm64` 生成开发样例并校验 10 个包内文件，ZIP 为 4,904,286 字节，SHA-256 `afcc8733be720dc66c35b8b964dc913e970e84b3c0522650ce2f7967dab1af8b`；包含现有词典小样，不含 TTS 或个人运行数据。七项隔离打包测试和两项构建参数测试通过。实际 ARM64 启动、WebView2、词典与语音均未验收，该包不是正式发行或 Lite 版。

2026-10-28 收尾复验：`pnpm test:tooling` 的两项构建参数及七项隔离打包测试通过；回读已有 ZIP 的 SHA-256、CRC 和清单中九个文件哈希均匹配。未重新构建或在 ARM64 设备上执行该样例。

日期：2026-10-24。此文件只记录本仓库骨架检查，不继承历史原型的通过结论。

## 可重复检查

| 范围 | 命令 / 证据 |
| --- | --- |
| 前端约束 | `pnpm lint` / `pnpm typecheck` / `pnpm build` |
| 浏览器 | `pnpm test:e2e`；三语、导航、主题、收纳、手机宽度、页面错误；报告与截图输出到忽略目录 |
| SQLite | `pnpm rust:test:build` 后 `pnpm test:rust`；重复初始化保留数据、高版本拒绝、迁移失败回滚 |
| Rust | `pnpm rust:fmt` / `pnpm rust:check`，CI 额外运行 Clippy |
| WASM | `pnpm wasm:build` 后 `node scripts/check-wasm.mjs`，实际装载并调用编译产物 |
| 原生目标 | CI 的 Windows / macOS × x64 / ARM64 `cargo check` |

## 早先骨架检查结果

本机 Windows 验证：`pnpm lint`、`pnpm typecheck`、`pnpm build`、Rustfmt、WASM release 编译与实际执行检查均通过。使用已安装的 Microsoft Edge 运行 2 项完整浏览器 E2E，通过；检查产生桌面与手机截图，结束后测试服务已停止。

SQLite 集成测试的本机预编译未通过：环境缺少 MSVC `link.exe`，因此没有执行后端测试，也没有宣称原生桌面已启动或已构建成功。对应集成测试与四目标原生编译将在 GitHub Actions 中运行，实际状态以该提交的检查结果为准。

构建显示 Zod 依赖的两条 Rollup 注释解析警告；构建退出码为 0，未关闭或降级警告。pnpm 11 构建脚本白名单已使用 `allowBuilds`，只放行 esbuild。

## Node.js 安全升级后的环境复核

复核日期：2026-10-24。

- 现有 nvm-windows 1.2.2 将 Node.js v24.15.0 升级并切换至 v24.22.1 x64；`nvm current`、`node --version` 和 `C:\Program Files\nodejs` 链接均确认生效。该版本为 [2026-10-21 官方安全更新](https://nodejs.org/en/blog/release/v24.22.1)，修复高危 DNS / TLS / Brotli 等问题；旧版本仍留在 nvm 中，不再是当前版本。
- 新 `node.exe` SHA-256 为 `d702daa815eabbff763c7d3808e235cf031cf2ca70211b56847bc72f51e240e5`，与官方发布值一致；Authenticode 显示 OpenJS Foundation 签名有效。
- npm 11.19.1、Corepack 和仓库锁定的 pnpm 11.25.0 可用。`pnpm install --frozen-lockfile`、`pnpm lint`、`pnpm typecheck`、`pnpm build` 均退出 0；锁文件未改变。构建仍有上述两条 Zod / Rollup 注释警告。
- 使用已安装 Edge 执行 `pnpm test:e2e`，2 项通过；仅验证界面骨架与浏览器开发服务。
- `tauri info` 确认 WebView2 155.0.4283.62；未检测到 MSVC / Windows SDK，PATH 中没有 Rust / Cargo / rustup。在用户 `.rustup` 中直接执行遗留二进制得到 Rust / Cargo 1.91.1，未满足项目锁定的 1.99.1，且该工具链没有 WASM 目标。因此未运行当前原生构建、SQLite 后端测试或 WASM 重建。
- 正常权限下 npm、Rust 1.99.1 manifest 和 crates.io 官方端点均返回 HTTP 200。受限工具环境会出现 DNS 错误，不能据此判定主机断网。

上述缺项已在下面的原生环境补齐中处理；保留此段检查时的结果，以区分升级前后证据。

## 原生开发环境补齐与实测

日期：2026-10-24。使用官方安装器，rustup 安装器 SHA-256 与官方值一致，Build Tools 安装器 Microsoft 签名有效。Build Tools 安装退出码为 0，无需重启。

| 工具 | 已安装版本 / 组件 |
| --- | --- |
| rustup / Rust / Cargo | 1.29.2 / 1.99.1 / 1.99.1；项目工具链自动生效，用户 PATH 已加入 `.cargo/bin` |
| Rust 组件 | rustfmt、Clippy；`wasm32-unknown-unknown`、`x86_64-pc-windows-msvc`、`aarch64-pc-windows-msvc` |
| Microsoft Build Tools | Visual Studio 2022 Build Tools 17.14.42；MSVC 14.44.35207，包含 x64 / x86 和 ARM64 工具 |
| Windows SDK | 10.0.26100.0 目标头文件、库及资源编译器 |
| 既有运行环境 | Node.js 24.22.1、pnpm 11.25.0、WebView2 155.0.4283.62 |

实测通过：

- `pnpm rust:fmt`。
- `cargo clippy --locked -p lexique-core -p lexique-storage --all-targets -- -D warnings`。
- `pnpm rust:test:build` 与 `pnpm test:rust`：3 项存储集成测试通过；0 项单元 / 文档测试不计作额外测试通过。
- `pnpm wasm:build` 与 `node scripts/check-wasm.mjs`：实际编译、装载并调用当前版本 ABI；不代表 FSRS 验收。
- `pnpm rust:check`：普通终端环境无需手动加载 VS Developer Shell 即可通过；当前旧会话临时补入新安装的 Cargo PATH。
- `cargo check --locked --workspace --target aarch64-pc-windows-msvc`：Windows ARM64 编译检查通过，未在 ARM64 硬件运行。
- `pnpm --filter @lexique/desktop exec tauri build --debug --no-bundle`：x64 调试程序成功构建，包含前端生产构建；仍有前述两条 Zod / Rollup 注释警告。
- 实际启动 `target/debug/lexique-desktop.exe`，原生窗口加载 `http://tauri.localhost/`，显示“本地存储已就绪”；用户应用数据目录创建 12,288 字节的 `lexique.sqlite3`，文件头检查通过，标准错误日志为空。测试窗口正常关闭，数据库保留；这不代表笔记业务的重启恢复验收。
- `tauri info` 的环境项全部通过；`vswhere` 确认 x64、ARM64、SDK 组件已注册，安装完整。

重新打开终端后会读取持久化的 Rust PATH；旧终端或继承旧环境的 Codex 进程可能需要重启。开发命令仍为 `pnpm desktop`。本轮不修改应用业务代码或锁文件；安装器与临时日志位于已忽略的 `.tooling/`。

## M0 资源小样探查

日期：2026-10-24。详情与来源见[资源小样调查](resource-feasibility.md)。这是独立资源探查，没有新增应用业务接口，也没有改变工程依赖或项目锁文件。

- 从用户指定的 mdx.mdict.org 获取 328,668 字节的 MDX 1.2 样本；头部长度 / ADLER32 校验通过。隔离 Cargo 工具固定 `mdictlib = 0.2.8` 并启用 LZO，编译退出 0；逐条解码 18,085 条正文并完成真实查询，执行退出 0。原始 HTML 未被执行。内容再分发许可尚未核实，不能默认随包发行。
- FreeDict 法英 0.4.1、法中 2025.11.23 的 Dictd 包下载成功，本地 SHA-256 已记录。标准库脚本只读取归档、不提取执行；全部索引行的偏移 / 长度检查及正文 UTF-8 解码通过，分别得到 8,505 / 10,947 条词条记录。未对照官方发布校验值，未完成结构化转换或准入。
- Python 首次输出受终端 GBK 对 `œ` 的显示限制失败；显式 `-X utf8` 后重跑成功，源文件没有改写。完整脚本、断言、源包和日志保存在忽略目录中。
- 已读取 FreeMdict 指定目录及两个子目录，尚未下载其大型全集或 MDD。TTS 仅核验官方文档和模型元数据，没有下载权重、生成 / 播放音频或接受 Pocket 的访问条件。
- 用户确认 TTS 拆为三票后，七份任务模板、父规格引用及依赖检查通过。已发布为 [#1](https://github.com/zardonc/lexique/issues/1) 的七张原生子任务 #2–#8，均为 open / ready-for-agent；GitHub 逐张回读正文与本地批准稿一致，父规格正文未变。原生依赖仅为 #8 blocked by #2，父页面显示七张子任务且只有 #8 为 blocked。页面凭证已保存；本轮没有执行这些任务的功能实现。

## 未包含的验收

应用尚未接入真实词典、完整变位、TTS、FSRS、第三方接口、Google 授权、Sheets 同步或浏览器学习流程；没有验证完整离线资源包、签名安装器、四架构安装或生产运行。独立资源解码和 CI 原生编译检查也不能替代这些验收。

## M0 默认词典小样准入（issue #2）

日期：2026-10-24。本票新增独立资源构建 / 查询入口，不改变上述应用接入边界。完整来源、原始许可、覆盖缺项、查询契约、体积和重建步骤见 [FreeDict 法中小样](../resources/freedict-fra-zho-2025.11.23/README.md)。

- 已从 FreeDict 官方获取法中 2025.11.23 原始 TEI 源包及发布 SHA-512，实算一致；保留未改动源包、包内 CC BY-SA 3.0 正文、Karl Bartel / FreeDict / WikDict / Wiktionary / DBnary 归属与转换、再分发说明。源码 MIT 不覆盖词典数据。
- Python 3.12.13 / SQLite 3.50.4 将全部 10,947 条记录转换为只读查询的 SQLite，保留 10,777 个原始键、同键记录及完整原始词条片段。先前 Dictd 的 10,589 个不同规范化索引键不等于原始词头数。`vacances` 存在；`sommes`、`yeux`、`chevaux` 等缺项未补造；弯撇号不改写为 ASCII。
- `pnpm test:resources`：6 项通过；包含经公开查询逐条比对所有源记录的元素 / 属性 / 文本、真实重音 / 撇号 / œ / 中文、多记录、无结果、错误 / 损坏 / 篡改资源、校验失败、保留既有输出和用户库、查询前后资源不变、连续两次生成逐字节一致。此前每条新增行为均先观察对应检查失败，再实现通过。
- `python -X utf8 scripts/dictionary_sample.py build .tooling/dictionary-admission/result`：通过；公开查询已实际读取生成库。构建及查询只使用本地输入，不调用网络。源包 464,676 字节、TEI 4,346,160、SQLite 5,062,656、完整压缩资源载荷 1,341,112、安装载荷逻辑字节 5,551,649；详细 SHA-256、计量组成和环境限制在小样记录 / `report.json`。
- `pnpm lint`、`pnpm typecheck`、`pnpm build`：全部通过；保留既有两条 Zod / Rollup 注释警告。没有新增应用依赖或改变锁文件。
- `pnpm rust:test:build` 后 `pnpm test:rust`：3 项存储集成测试通过，原有硬性 60 秒执行限制不变。受限环境报告增量缓存硬链接失败后复制的警告，未抑制；本票未修改 Rust 源码。
- `pnpm test:e2e`：受限环境中首次运行卡住，已明确终止本次 Playwright 及其 Vite 子进程树，未当作通过；正常权限下使用 Edge 重跑，2 项通过，Playwright 管理的开发服务已停止。这验证既有界面回归，不证明原生词典集成。
- 本票源码 / 文档差异和本地 Markdown 链接检查通过。上游 `COPYING` 保持字节原样；`git diff --check` 对其第 2 行 `=======` 标题误报冲突标记，另报告原始结尾空行。这两项作为保留第三方许可原文的已知例外，其余本票差异检查通过。
- 独立规范审查与规格审查各 0 项问题。新增 Windows / Linux / macOS 资源测试 CI 配置，远端尚未运行；本机只验证 Windows x64。未新增或重建 CodeGraph 索引。

本次准入限于该固定开发小样及其附带分发义务，不表示完整默认词典、桌面查词 #8、原生 / 四架构运行、首装断网安装或正式发行已验收；未更改用户学习数据库，也未冻结全部后续资源格式。

## M0–M1 原生小样离线查词（issue #8）

日期：2026-10-25。此节更新上述“应用尚未接入真实词典”的历史边界：现在仅固定 FreeDict 法中小样已接入原生窗口；完整默认词典、笔记、变位、TTS 与发行验收仍未完成。系统、版本、资源 / 可执行文件哈希、禁网方法、逐项观察与原生截图见 [Windows 验证证据](evidence/issue-8/README.md)。

- `pnpm lint`、`pnpm typecheck`、`pnpm build`：通过；保留既有两条 Zod / Rollup 注释警告。
- `pnpm test:e2e`：4 项通过，包含浏览器明确边界、平台 IPC 响应校验、真实来源字段、无结果 / 缺失 / 损坏 / 查询失败清除旧结果，以及既有三语、主题、导航与手机宽度回归。IPC 替身仅用于浏览器边界检查，不作为原生成功证据。
- `pnpm test:resources`：7 项通过；新增重复准备仅复用已验证生成物、损坏时失败且保留原件的检查。源包、内容与转换的既有完整比对仍通过。
- `pnpm rust:fmt`、`pnpm rust:check`、`cargo clippy --offline --locked -p lexique-core -p lexique-storage -p lexique-dictionary --all-targets -- -D warnings`：通过。受限文件系统的缓存硬链接警告仍可能出现，未抑制。
- `pnpm rust:test:build` 预编译后 `pnpm test:rust`：2 项词典集成测试与 3 项存储集成测试通过；执行约 1.5 秒，既有硬性 60 秒执行限制保留。覆盖真实 SQLite / 原始拼写 / 多记录 / 无匹配 / 注入字符串 / 无效输入 / 缺失 / 损坏 / 读取失败 / 替换恢复；逐字节确认资源和既有用户库不变且没有 SQLite sidecar。
- `pnpm desktop:build --debug --no-bundle`：通过。原生窗口在无互联网能力的 AppContainer 中首次启动并完成真实查词，应用和 WebView 后代令牌已核查；正常关闭退出 0 后在同等禁网环境重启，查询相同词条成功。原生资源错误分支已实际替换本次生成副本验证，结束后恢复原哈希，用户库哈希不变。
- TDD 记录：公开 Rust 查询入口先编译失败，准备入口先导入失败，页面先因查词输入缺失失败；实现后通过。损坏资源带匹配测试哈希的回归先得到错误分类，补充 SQLite 文件头校验后通过。浏览器 IPC 测试曾因缺少实际 `isTauri` 标志进入预览，修正测试的系统边界后通过。
- 独立规范审查 0 项、规格审查 0 项；差异及本次 Markdown 本地链接检查通过。验证启动的应用均已退出，Playwright 管理的服务已停止；既有未提交调查文档保留且未纳入本次提交。

这次只证明本机 Windows x64 开发构建的小样查询；没有重跑其他平台运行、完整词库 / 三语覆盖、WASM、安装器、缺 WebView2 的离线依赖安装、笔记恢复或正式发行。原生查词不接受用户资源路径、不执行原始 HTML；仅新增本地 main 窗口的查词命令，没有开放文件、shell、HTTP 插件或远程窗口权限。CI 已为资源准备与 Rust 词典测试补齐配置，本票提交尚未在远端执行。

## Windows portable 交付（issue #8 补充）

日期：2026-10-25。本节补充前述 debug 验收：Windows release 现按用户要求构建免安装 portable ZIP，运行数据、WebView profile、临时文件及框架配置 / 缓存 / 日志均定位到 `.exe` 同级 `Lexique-data/`。来源、尺寸、最终二进制与 ZIP 哈希、截图及逐项观察见 [portable 交付证据](evidence/issue-8/README.md#windows-portable-交付补充)。

- `pnpm desktop:portable`：通过；locked release 构建，无安装器，10 个交付文件逐项回读校验，ZIP 为 4,972,902 字节。重复打包和只遗留 `.zip.sha256` 均明确拒绝覆盖；旧交付保留。
- `pnpm lint`、`pnpm typecheck`、构建中的 `pnpm build`：通过；`pnpm test:resources` 7 项、`pnpm test:e2e` 4 项通过。保留原有 Zod / Rollup 注释与终端颜色环境警告。
- `pnpm rust:fmt`、`pnpm rust:check`、`cargo clippy --offline --locked --workspace --all-targets -- -D warnings`：通过；入口修正后再次通过格式与全 workspace Clippy。保留受限文件系统缓存硬链接复制警告。
- `pnpm rust:test:build` 后 `pnpm test:rust`：2 项词典与 3 项存储测试通过，执行仍受硬性 60 秒限制。
- 原生回归先确认旧构建不生成 exe 同级数据库；最终 ZIP 在带中文 / 空格路径、不同工作目录、外部 WebView 环境变量覆盖条件下禁网启动并查询成功。关闭退出 0 后整目录移动、同等禁网重启仍可查词并退出 0；实际应用及两轮各六个 WebView 后代令牌已检查。无法创建便携目录时退出 101，不改旧库、不回退；旧用户库哈希和旧 WebView 文件元数据保持不变。
- 独立规范审查发现校验 sidecar 可被覆盖，规格审查发现继承的 WebView 数据目录可逸出；两项已修复并以公开打包入口 / 实际 native 进程验证，独立复查各剩余 0 项。差异、链接与远端 CI 结果在交付时另行核对；原有未提交的资源调查文档保持原样。

这仅交付 Windows 11 x64 小样 portable，不证明签名、缺 WebView2 的干净机器、四目标发行、完整资源、笔记或同步。操作系统自己的诊断 / 安全记录不受应用目录配置控制；程序不写系统安装目录，不自动迁移旧 AppData。验证启动的原生进程和 Playwright 服务均已关闭。CI 已新增 `windows-x64-portable` ZIP 产物；远端结果见 issue #8 最终交付记录。

## M1 生词记录与独立笔记（issue #10）

日期：2026-10-25。此节更新上述笔记未实现的历史边界：Windows x64 开发构建现已完成三层记录、显式保存、幂等重试、版本冲突、搜索排序与重启恢复。完整命令、哈希、故障观察及截图见 [M1 验证证据](evidence/issue-10/README.md)。

- `pnpm lint`、`pnpm typecheck`、`pnpm build`，Rust 格式 / workspace 编译 / Clippy，以及 WASM 编译与实际装载均通过；既有警告保留。
- `pnpm rust:test:build` 后 `pnpm test:rust`：15 项通过，包含真实文件库重开、并发 / 幂等 / 冲突、写入与提交失败、增量迁移；执行硬限仍为 60 秒。资源检查 7 项、Edge E2E 13 项通过。
- 最终 locked debug 构建在 AppContainer 禁网环境真实重启，身份、来源、摘要、换行多语言笔记和排序偏好恢复。原生关闭三种选择及保存失败不退出均实测；词典缺失 / 校验失败时仍可编辑保存笔记，摘要不变。测试仅使用隔离副本，原始资源和已有学习数据保留。
- 独立审查的未确认保存保护与 macOS 菜单 Quit / Cmd+Q 问题已修复并复查。macOS Dock Quit 仍可能绕过未保存提示，须在该平台验收前处理；macOS 专属代码未在本机编译或运行。

本轮仅 Windows x64 开发构建验收；其他架构 / 平台、远端 CI、release portable 重打及正式发行未验证。没有新增草稿、删除、FSRS、同步、TTS 或服务。验证进程均已关闭；本轮开始前的文档编辑未纳入本次实现提交。

## M1 Windows 复盘修复与 macOS 阻塞

日期：2026-10-25。Windows 可执行的复盘项已修复：公开原生构建入口默认 locked，portable 复用同一入口；独立打包测试与构建参数测试接入 CI，交付说明更新 M1 笔记能力。新包输出到隔离目录，旧交付与既有学习数据保留。详见 [Windows 后续证据](evidence/issue-10/windows-follow-up.md)。

- 工具测试 7 项、资源检查 7 项、Edge E2E 13 项、预编译后的 Rust 测试 15 项通过；lint / typecheck 与 release 内嵌前端生产构建通过。后端执行硬限仍为 60 秒。
- Windows x64 locked release portable 成功生成并逐项校验；真实禁网窗口验证 Alt+F4、标题栏关闭、继续编辑、保存失败不退出、恢复数据库后重试，以及多语言笔记与排序的重启恢复。Windows ARM64 workspace 编译通过，但无该架构实机运行证据。
- macOS 专属编译、菜单 Quit / Cmd+Q / Dock Quit 及原生数据恢复集中标记为 **blocked — 等待 macOS 执行环境**。退出矩阵、已知 Dock Quit 缺陷与解除阻塞条件见 [macOS 清单](macos-m1-blockers.md)。Windows 结果不替代其验收。

本轮未推送、未运行远端 CI；没有验证 macOS / Windows ARM64 实机、缺 WebView2 干净机器、签名或正式发行。验证仅使用隔离合成数据，原有未提交文档修改保持原样。

## M2 固定 FSRS 与原生 / 实际 WASM（issue #13）

日期：2026-10-26。共享核心候选使用固定 `fsrs-rs 6.6.2` 官方默认参数执行四档评分、初学 / 重学步骤与日历排程，不含优化或间隔扰动。Windows x64 原生 CLI 和无宿主导入的实际 WASM 通过同一公开入口执行，最终一致性准入 **blocked**：两个合法反例出现一天排程差异。来源、许可、完整命令、输入契约、误差与产物体积见 [#13 证据](evidence/issue-13/README.md)。

- 首轮 69 个固定向量及 4,096 个固定种子校准向量通过；独立审查扩大输入后发现浮点误差跨过取整边界。反例已加入固定集，最终 71+4,096 对照有 2 项严格排程不一致，`pnpm test:scheduling` 退出 1 并保留完整报告；没有放宽日期 / 整数间隔比较或将失败改为通过。
- 5 项核心集成测试、既有 15 项 Rust 测试通过；先 `pnpm rust:test:build`，再执行 `pnpm test:rust`，硬限仍为 60 秒。lint / typecheck / build、Rust 格式、workspace check / Clippy、资源 7 项、工具 7 项及 Edge E2E 13 项均通过。
- Windows ARM64 workspace 编译通过，尚无实机运行证据。本机 Python Store 别名失效导致首次资源准备 / workspace check 失败；仅在验证进程中将已有 Python 3.12.13 目录加入 PATH 后重跑通过，没有修改全局设置。首次 WASM 构建暴露上游随机数依赖限制，最终使用其明确报错的 unsupported 后端；实际产物没有宿主导入。

未接入学习数据存储、评分 UI 或 TTS，未更改数据库 schema。#13 保持未通过，依赖它的后续集成继续阻塞；没有换库、补丁替换上游算法或改变已确认取整规则。TTS 仍由 #14 单独准入，生成后需人工试听校验。macOS 实机、Windows ARM64 实机、远端 CI、完整离线资源及发行仍未验证；本次不据此关闭 M2 或解除其他里程碑门槛。

### Issue #13 数学兼容修复

日期：2026-10-26。本节更新上段历史阻塞结论。优先检查依赖与开发环境后，以零第三方依赖的最小程序确认 Windows / WASM 的标准 `exp` / `powf` 相差 1 ULP；固定包哈希、隔离 target 重建、Debug / Release 及 Rust 1.91.1 交叉检查没有发现应用代码、缓存或配置导致偏差的证据。完整假设、探针与原失败结果见 [诊断](evidence/issue-13/math-diagnosis.md)。

保留 FSRS 6.6.2 原公式与官方参数，显式 vendored 补丁只将标量调度路径的数学函数统一为固定 libm 0.2.16，关闭其默认 features；源码审计仅 manifest / model.rs 与官方包不同。模型标识为 `fsrs-rs/6.6.2+libm-0.2.16`，新增回归拒绝未打补丁的历史版本。当前 Windows x64 release 原生 / 实际 WASM 的 71+4,096 向量通过、记忆差为零，原两条反例继续要求整数排程严格相等。Rust 格式、workspace Clippy、预编译后 21 项 Rust 测试和 Windows ARM64 workspace 编译通过。

本次未改 UI，未重跑前端 / E2E 或重打 portable；macOS / ARM64 实机、远端 CI 仍未运行。没有修改用户数据、迁移存储或集成 TTS；#14 的语音准入及人工试听仍独立保留，不能据此宣称完整 M2 通过。上游标准数学的独立诊断命令仍在本机退出 1，这是刻意保留的原因复现，正式 `pnpm test:scheduling` 已通过。

### Issue #13 验收收尾

2026-10-28：五项技术验收通过。重新构建 Windows x64 原生 / 实际 WASM，71 + 4,096 条向量排程严格一致、记忆差为零，产物哈希与既有机器报告相同。锁定安装、lint / typecheck / build、Rust 格式 / workspace check / Clippy、预编译后 24 项 Rust、7 项资源、9 项工具和当前工作区 18 项 Edge E2E 通过；runner 硬 60 秒限制不变，Windows ARM64 仅编译通过。

独立规范审查无需处理问题；独立规格子审查三次限流失败，由主代理完成逐项规格复核，未冒称独立审查通过。完整来源 / 依赖回读、命令与限制见[收尾记录](evidence/issue-13/closeout-2026-10-28.md)。本次只提交 #13 文档，保留已有 #14 编辑；不改用户库、不启动后续实现。远端 CI、macOS / ARM64 实机及发行验收仍未运行，当前分支未推送。Playwright 服务已停止。

## M2 单词离线语音探针（issue #14）

2026-10-27：历史语音验证包固定的 Python 3.12.10 缺少后续安全修复，已将 Supertonic 准备脚本和 CI 改为 Python 3.14.10；旧包仅留作历史证据。v7 六项打包检查及三项失败路径检查通过，受限进程实际生成并调用播放 38 条音频；远端 CI 与干净机器未验证。版本及限制见[试验记录](evidence/issue-14/supertonic-trial.md)。

本机使用 v7 的 Python 3.14.10 另运行 `test_dictionary_sample.py` 和 `test_package_portable.py`，各七项通过；这不代替 CI 的其他系统运行。旧 Kokoro 探针的历史资源清单仍指向 Python 3.12.10，不应作为当前安全发行材料。

同日从 v7 ZIP 解压副本在受限 Windows x86-64 进程新生成 38 条 WAV 并调用播放，逐文件哈希匹配，前后联网探测均被拒绝。Eigen 5.0.1 官方源码归档哈希与上游声明一致，但实际 wheel 使用的 Eigen 版本及分发义务仍未确认；见[试验记录](evidence/issue-14/supertonic-trial.md)和[许可说明](evidence/issue-14/supertonic-licenses/ATTRIBUTION.md)。

2026-10-27 用户反馈试听 v7 生成音频，男女声均正常；未逐项说明文件、设备或速度，不能据此宣称全部 38 条逐条验收或干净机器已验证。

同日核验 Eigen 3.4.0 官方源码归档 SHA-256 与上游声明相同；该归档包含 LGPL 文件，并提供 `EIGEN_MPL2_ONLY` 编译限制。实际 wheel 采用的版本、源码范围和编译选项尚未证明，不能据此认定其发行义务已闭合。

2026-10-27：补充固定 CPython 3.14.10 官方许可文档中的 Expat 和 Unicode 通知，v7 缺失该文档的回归检查失败，v8 六项打包检查通过。NumPy wheel 元数据所列 17 项许可文件在 v7 中均存在；CPython 的 `libtommath.dll`、OpenSSL 归属以及 sherpa-onnx wheel 的实际构建依赖仍需核查，v8 尚未实播。详见[许可说明](evidence/issue-14/supertonic-licenses/ATTRIBUTION.md)。

同日 v8 准备目录在受限进程重新生成并调用播放 38 条 WAV，前后联网探测均被拒绝；没有从 ZIP 解压运行，也未人工试听 v8 样本。

2026-10-27 用户反馈 v8 生成音频的男女声均正常；未逐项说明试听文件、设备或播放速度。

同日复核 `libtommath.dll`：CPython 配置将其关联 Tcl/Tk，但嵌入包中未发现 Tcl/Tk DLL 或根目录 PE 文件的直接导入。官方 `cpython-bin-deps` 的同名 DLL 与嵌入包的 DLL 大小及 Git blob 均不同，不能按相同二进制认定；动态加载及确切来源仍未排除或确认。详见[许可说明](evidence/issue-14/supertonic-licenses/ATTRIBUTION.md)。

后续复核证实上述文件差异来自官方 Authenticode 签名：验证签名有效，去除证书表并还原 PE 相关字段后与 CPython 上游二进制 Git blob 一致。Tcl 源码的 LibTomMath 许可加入 v9，新增检查在 v8 失败、v9 六项通过；该记录不证明动态加载情况或全部运行库义务。后续 v9 准备目录的生成与播放见下方当前状态。

日期：2026-10-26。Kokoro v1.0 int8 / ff_siwis、kokoro-onnx 0.6.1、eSpeak NG 1.52.0
组合在 Windows x64 的受限禁网进程中完成 17 个准确输入的真实生成、有效 WAV 回读及同步
Windows 播放调用。完整独立 Python 运行包也重跑通过，不借用开发虚拟环境或预生成音频。
模型、声线、前端、运行库、对应源码、许可与下载校验清单、可试听音频和性能报告见
[#14 证据](evidence/issue-14/README.md)。

准入仍 **blocked — 待修正后的完整样本试听验收**。用户认可声音可继续评估，并要求先修复
孤立 y；已将其词汇读音显式修正为 /i/，保留原前端音素、修正标记与前后音频。
播放 API 成功不证明声学输出
或法语质量，未知词 G2P 推断也不代表正确发音。首轮两项短句因前端分隔符被严格拒绝，
已保留失败证据并显式处理；其他未知音素继续报错，不静默丢弃。

完整压缩 / 安装量及依赖明细见该票 measurements；单独语音包不使用 WebView2，不能据其体积
推断正式桌面首装总量。缺 VC++ 的干净机器、其他三个目标、产品语音 IPC / UI 及 M5 未验收。
18 项 Python、lint / typecheck / build、21 项 Rust、71+4,096 调度向量、工具与 13 项 Edge E2E
回归通过，具体命令和环境失败重跑见 [检查记录](evidence/issue-14/checks.md)。未改用户库或产品依赖。

## Piper 语音偏好设置

2026-10-26：新增三语设置页，选择法语女声／男声、正常／慢速，默认女声／正常；通过受限桌面 IPC 保存到现有 SQLite 元数据。不存在的配置使用默认值，非法配置保持错误。无 schema 迁移、新依赖、学习数据重写或音频删除。规则见[语音偏好](speech-preferences.md)。

- `pnpm lint`、`pnpm typecheck`、`pnpm build`：通过。首次 lint 发现测试夹具复杂度超限，提取默认回复后复验通过。
- `pnpm rust:test:build` 后 `pnpm test:rust`：23 项通过，执行保持硬性 60 秒上限；新增 2 项覆盖默认值、数据库重开恢复、其他偏好保留、非法字段和损坏配置。
- `cargo fmt --all --check`、`cargo check --locked --workspace`：通过，包含 Windows x64 桌面命令与权限生成。
- `pnpm test:e2e`（本机 Edge）：17 项通过，新增 4 项覆盖保存／重新进入、失败重试、非法响应／重新读取和三语手机布局；桌面与手机截图已检查。
- 首次受限环境浏览器检查无法连接本机服务，已停止并在允许回环连接的环境重跑。初版顺序回复夹具被 React 严格模式的重复读取消耗；改为持有保存值的夹具后通过。没有据此修改生产读取行为。

浏览器测试模拟 IPC，不证明原生窗口完整端到端链路；Rust 测试证明实际 SQLite 文件的重开恢复。未重打 portable 或做原生窗口交互、其他平台运行、TTS 合成／播放／缓存、慢速倍率听感校准。偏好设置可用不表示模型已获默认包再分发准入或完整 M2 已通过。


## Piper MLS 双说话人准入与偏好兼容

2026-10-26：替代模型选择 `fr_FR-mls-medium`，源说话人 `3698/F` → 推理编号 `1`、`1840/M` → `0`。依据固定官方模型许可指引、从零训练模型卡及 MLS 来源，按 CC BY 4.0 路径继续准入；[补查记录](evidence/issue-14/replacement-admission.md)明确许可解释边界，[原始材料](evidence/issue-14/mls-licenses/ATTRIBUTION.md)保留来源、署名、全文和哈希。不是完整引擎 GPL 分发义务或正式发行验收。

设置保存稳定的 `female`／`male`，兼容旧 `siwis`／`tom` 对应的男女声及原语速；没有数据迁移或音频删除。新增回归先复现旧模型标识泄漏（断言失败），再加入 Rust 反序列化别名并通过。

- `pnpm lint`、`pnpm typecheck`、`pnpm build`：通过。
- `pnpm rust:test:build` 后 `pnpm test:rust`：24 项通过；硬性 60 秒执行超时保持，首次编译独立。
- `cargo fmt --all --check`、`cargo check --locked --workspace`：通过；仅文件系统硬链接缓存回退复制警告。
- `pnpm test:e2e`（本机 Edge）：17 项通过；设置页的选择、保存失败、重新进入和响应校验沿用既定边界。
- 原始许可归档 SHA-256、模型权重／配置哈希与上游固定快照校验通过。

独立合成探针及四组合结果见[试听页](evidence/issue-14/mls-samples/listen.html)。正常时长系数 `1.0`、慢速 `1.3`，两档均保留模型声学噪声 `0.333`、将时长扰动 `noise_w_scale` 固定为 `0`。校准后 72 个 WAV 有效，36 对样本逐条慢速更长；女声／男声总时长倍率为 `1.276`／`1.275`。同一受限进程前后各两次联网探测返回 WinError 10013，72 次同步 Windows PlaySound 调用完成。首轮及校准轮合计生成播放 144 次；不把播放调用完成当作人工听感验收。首轮默认时长噪声产生部分慢速样本反而更短，不能据总时长断言通过速度验收；保留原始报告作校准对照。裸文本 `y` 仍为字母名，另有显式 `[[i]]` 音素对照，不能把该对照当作未改文本的正确合成。

未接入产品 TTS 合成／播放／缓存，浏览器 IPC 模拟不等于原生完整链路；未重打 portable、计量最终整包或运行其他三个目标。双声线音质、慢速清晰度及显式 /i/ 听感仍待人工验收。


### MLS 人工验收失败

2026-10-26：用户实际试听上述四组合，确认全部不是正确法语词读音，而是混乱人声且音质不佳。撤回 MLS 候选，不认定 TTS 质量准入或 #14 完成。72 个有效 WAV、36 对时长检查及播放成功只是容器／调用证据，未覆盖真实可懂度；后续替换需重新试听。原始失败样本和报告保留，不以调速校准通过覆盖人工否决。


### Supertonic 3 替代小批筛查

2026-10-26：MLS 已因人工试听失败撤回。Supertonic 3 int8 / F1 女声、M1 男声 × 正常 1.0／慢速 0.8，在禁网受限 Windows x64 进程生成、回读并调用播放 26 个 WAV；前后联网探测均为 WinError 10013。12 对法语样本逐条慢速更长；不认定声音质量或完整 #14 验收通过。固定资源哈希、原始许可证、模型／代码许可区分见[执行记录](evidence/issue-14/supertonic-trial.md)。

语言参数原生绑定 getter 返回副本：CLI 断言先复现失败，再改完整字典赋值、保留断言并重跑通过。Whisper tiny 本地对 28 个样本作识别对照，支持 MLS 目标句偏离严重及新组合例句有所改善；短词、y／i 识别不稳定，不代替人工。ASR 归一化的重音／œ 自检先失败后修复，最终全量对照通过运行。仅本地试验环境新增两个 sherpa-onnx 包，未改产品依赖。

本轮未再改前端／Rust 源码；沿用本轮此前 lint、typecheck、build、24 项 Rust 及 17 项 Edge E2E 通过记录。独立脚本校验、音频哈希与浏览器试听页另行检查。未重打 portable、未通过完整包体／四平台／实际产品音频链路；短词与 y 的质量及人工试听仍是明确门槛。

补充检查：26 个试听控件在浏览器均可加载有效时长且没有自动播放；独立逐文件哈希、PCM、12 对时长、许可证原始字节及本地链接检查通过。手写改动通过 Git 空白检查；逐字归档的 CC BY 全文与 Piper VOICES 原件各自带末尾空行，保持上游字节及哈希，不为消除该提示修改原文。

### 慢速与字母名试听候选

用户确认原英语与法语音频可用，要求慢速更慢，并倾向单独字母 `y` 使用字母名。本轮通过 `probe_supertonic.py` 在 Windows x64 受限进程生成并回读 32 个 WAV，包含男女声、正常 `1.0`、候选慢速 `0.6`、英语及 `i grec`；32 次同步播放调用完成，前后两次网络探测均为 WinError 10013。逐文件 SHA-256 核对通过；16 对相同输入的慢速时长均大于正常速度，试听页 17 个本地引用均存在，`git diff --check` 通过。[报告与试听](evidence/issue-14/supertonic-slower/listen.html)记录本轮结果。

`0.6` 和 `i grec` 的听感尚待用户确认；此试验没有实现产品 TTS、字母语境识别或音频缓存，也没有完成原 #14 全量样本和跨平台验收。

后续用户要求播放时变速：试听页常速和慢速使用同一个正常生成的音频，慢速播放倍率设为 `0.6`。设置保存原有 `normal`／`slow` 值并明确显示倍率；这不表示应用已经具备语音生成或播放能力。前述 `speed=0.6` 合成样本仅作历史记录保留。

验证：`pnpm run lint`、`pnpm run typecheck`、`pnpm run build` 通过；Windows Edge 下 `pnpm --filter @lexique/desktop exec playwright test e2e/speech.spec.ts e2e/speech-playback.spec.ts` 的 5 项通过，覆盖设置保存恢复、失败处理及试听页同文件的实际播放倍率。试听探针的 Python 语法检查通过；未为调速重新生成音频。

用户随后确认 `0.6` 倍播放清晰自然，`i grec` 可作为单独字母 `y` 的读音；该确认不扩展为完整单词集、产品合成与播放或发行验收。

完整输入补测：Windows x64 禁网受限进程下运行探针，原定 17 项、英语和字母名分别由男女声以正常速度生成并同步调用播放，共 38 个 WAV；网络探测前后各两次均为 WinError 10013，峰值工作集 314,765,312 字节。逐文件 SHA-256、原始输入顺序及倍率核对通过，削波计数均为零。浏览器检查[试听页](evidence/issue-14/supertonic-full/listen.html)的 38 个控件，切换慢速后播放倍率均为 0.6，资源未报缺失。CLI 对缺失资源及已有输出目录均非零退出，已有输出被保留。完整样本的读音仍需人工试听；独立候选的完整发行包、其他平台及产品链路仍未验收。

用户随后确认完整音频样本没有问题。使用现有 FFmpeg 7.1 将 38 个常速 WAV 分别编码为 Ogg Opus 24 和 48 kbit/s（libopus、voip），共 76 个文件。原始文件共 5,467,662 字节，两种 Opus 分别共 145,852 和 286,705 字节，节省 97.33% 和 94.76%。所有文件均解码为非静音 PCM，重采样回 44.1 kHz 的帧数与源文件相差 0 至 1 帧。浏览器逐个加载 76 个 Opus 文件并成功以 0.6 倍播放一个 24 kbit/s 文件；[试听对照](evidence/issue-14/supertonic-opus/listen.html)供人工比较音质。未验证其他平台的播放器，也未将编解码接入产品或确认 FFmpeg 随产品分发的许可。

用户最初确认上述 Opus 24 和 48 kbit/s 的音质可接受。2026-10-27 进一步复听后，用户明确指出 24 kbit/s 在同一句内声音时轻时响，48 kbit/s 没有这一问题，并确定应用新生成语音默认采用 Ogg Opus 48 kbit/s；此前优先评估 24 kbit/s 的建议不再适用。新设备试听常速 / 0.6 倍慢速播放正常，但产品编码与其他目标平台兼容性、发行许可仍未验证。

追加 CLI 回归：4 项通过，包括在受限 Windows x64 进程重新生成并播放 38 个音频，以及缺失资源、损坏资源和既有输出的失败检查。运行库 wheel 已按 PyPI 摘要核对；六个候选原始下载文件合计 196,958,331 字节，但完整桌面包及许可尚未核算，详见[追加记录](evidence/issue-14/supertonic-trial.md)。

### Supertonic 独立验证包

2026-10-27 当前状态：v9 准备目录在受限 Windows x86-64 进程生成并调用播放 38 条音频，用户确认 v9 声音正确；反馈未逐项列出试听文件、设备或播放速度。v9 补充 LibTomMath 许可，六项打包检查通过，ZIP 为 159,650,967 字节，解压逻辑大小为 240,989,324 字节；并非从 ZIP 解压后执行。本票剩余的来源和再分发准入缺口见[当前准入决定](evidence/issue-14/README.md#当前准入决定)。完整桌面包体和其他三个目标实机验证留待后续发行验收，不因其未完成阻塞 #14。

后续在不生成音频的前提下增加 hclust 许可及版权说明，并构建 v11 验证包。v11 与 v9 的模型及运行时逐文件哈希一致，包 CRC、六项打包检查及四项无音频探针检查通过；真实合成测试按用户要求跳过。v11 ZIP 为 159,653,525 字节，解压逻辑大小 240,995,571 字节，哈希及差异见[试验记录](evidence/issue-14/supertonic-trial.md)。上游 wheel 的实际构建依赖、选项和量化发布物的完整转换来源仍未证实，不能据此宣称 #14 通过。

进一步补充 Darts 和 ThreadPool 上游通知及模型静态对照后，v15 ZIP 为 159,665,936 字节、解压逻辑大小 241,018,108 字节，1,048 个文件的清单及 CRC 检查通过；与已试听的 v9 有 1,016 个模型和运行时文件哈希一致。六项打包检查、四项无音频探针检查及五项模型比较测试通过，真实合成项按用户要求跳过。比较仅支持部分同名权重具有数值关系，未证明完整模型转换及 wheel 的实际构建依赖；详见[试验记录](evidence/issue-14/supertonic-trial.md)及[静态对照](evidence/issue-14/supertonic-static-comparison.md)。#14 的资源准入仍未完成。

2026-10-28：按用户提供的研究资料补充 Eigen 5.0.1 上游源码归档中的 MPL、BSD 及许可概览原文，并将其加入资源来源清单；归档和 24 项许可材料的 SHA-256 校验通过。打包失败路径测试四项通过，两个需要现成包的检查本次跳过；未重建 v15、未生成或播放音频。源码许可证据不能证明 wheel 使用的 Eigen 版本或其完整依赖，资源准入仍待核实。

同日补齐 Eigen 两版候选许可，构建 v16：ZIP 159,715,206 字节，解压逻辑大小 241,154,340 字节，1,060 文件；CRC 和六项打包检查通过，1,016 个模型 / 运行时文件与 v9 哈希一致。四项无音频探针检查、五项静态比较测试通过；真实合成按用户要求跳过。OpenSSL 两个 DLL 与固定 CPython 二进制依赖逐字节相同；其全部第三方通知和 sherpa-onnx wheel 实际依赖仍未完整审计，见[许可记录](evidence/issue-14/supertonic-licenses/ATTRIBUTION.md)。

同日固定提交的独立源码 checkout 完成 Windows x64 Release C API 构建；编译日志保留 C4804、C4068 警告。新 DLL 的 `.text` 哈希和 `.rdata` 长度与 v9 不同，不能作为已试听 wheel 的构建证明，亦未执行该新 DLL 的音频验证。构建依赖和日志见[受控构建记录](evidence/issue-14/controlled-build-20261028.md)；v16 不含该后续记录。

用户随后确认：接受 v9 wheel 构建证明缺失及量化校准过程未独立复现为 #14 已知限制，按候选依赖保守归档通知，并将 OpenRAIL-M 协议及机器合成披露纳入 M2 / M5 验收。v16-revised 包 159,725,647 字节、解压 241,177,527 字节、1,067 文件，六项打包、四项无音频探针及五项静态比较通过，1,016 个模型 / 运行库文件与 v9 一致；未再次合成音频。项目准入决定不等同完整依赖证明或正式发行法律批准。详见[准入记录](evidence/issue-14/README.md)。

2026-10-26：精简重复运行库后的 Windows x64 v2 ZIP 为 158,381,008 字节，解压文件逻辑总量 238,534,083 字节；1,026 项文件哈希及 ZIP CRC 通过。解压副本在受限进程重生成 38 条并调用播放，前后联网探测返回 WinError 10013；五项打包检查通过。完整产品体积、干净机器、其他平台及全部传递依赖许可仍未验收，详见[执行记录](evidence/issue-14/supertonic-trial.md)。

ONNX Runtime 1.28.2 DLL 与上游 MT Release 版本逐字节哈希一致，已收录其许可及第三方通知。加入许可材料的 v3 ZIP 为 158,414,965 字节、解压文件逻辑总量 238,868,060 字节，1,028 项清单哈希与 ZIP CRC 通过；v3 未重跑解压播放。其他依赖的许可仍需确认。

固定模型、运行时与许可材料的独立 Windows x64 验证包已构建：压缩 166,691,207 字节、文件逻辑总量 261,179,033 字节；ZIP CRC 和 1,031 项文件哈希通过。包内运行时在受限进程重生成 38 条并完成同步播放调用，前后联网探测均被拒绝。打包失败路径测试 4 项通过。命令、摘要和局限见[执行记录](evidence/issue-14/supertonic-trial.md)。

这不是完整桌面包，不包括系统 VC++、WebView2、词典或应用；未验证 ZIP 解压到干净机器、其他架构、完整传递许可及产品集成。完整版 / Lite 分发和 Lite 模型下载均尚未实现，#14 不宣称完成。

2026-10-27：补充 KissFFT 固定上游版权声明和 BSD-3-Clause 文本；Windows x86-64 v5 验证包 ZIP CRC、1,034 项清单哈希与五项打包测试通过，新增许可检查在 v4 先失败。v5 尚未实播，源码依赖声明不等于 wheel 的实际链接清单；详细数据见[执行记录](evidence/issue-14/supertonic-trial.md)。

2026-10-27：补充固定 kaldifst 和 OpenFST 许可、版权及作者文件；Windows x86-64 v6 验证包五项打包测试通过，新增文件检查在 v5 先失败。v6 未实播，实际二进制依赖与完整分发义务仍待核验；详细数据见[执行记录](evidence/issue-14/supertonic-trial.md)。

同日复测 v6 ZIP 解压副本：受限 Windows x86-64 进程新生成 38 条 WAV 并完成同步播放调用，音频哈希均匹配，生成前后两地址联网探测均为 WinError 10013。v6 新生成音频未获人工试听验收，实际二进制依赖和其他平台仍未核验；详见[执行记录](evidence/issue-14/supertonic-trial.md)。

同日用户确认 Windows ARM64、macOS x86-64 和 macOS ARM64 留待后续验证，不阻塞 #14；正式桌面发行仍须满足四目标要求。v6 Python 扩展及 C API DLL 自报同一 `2.0.1` 版本与 `af2d8d81` 源码提交前缀；完整依赖和发行许可不能由自报版本代替，详见[来源记录](evidence/issue-14/supertonic-licenses/ATTRIBUTION.md)。

2026-10-27：新增 portable 语音临时目录与定期清理的产品要求及后续接入验收项；当前没有产品语音生成或清理实现，本次不声称已验证文件存放、过期清理或移动后重试。macOS portable 路径与四架构实机验证仍待落实。
