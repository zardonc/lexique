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
