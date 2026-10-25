# 骨架验证记录

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

## 未包含的验收

没有真实词典、完整变位、TTS、FSRS、第三方接口、Google 授权、Sheets 同步或浏览器学习流程；没有验证完整离线资源包、签名安装器、四架构安装或生产运行。CI 原生编译检查也不能替代这些验收。

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
