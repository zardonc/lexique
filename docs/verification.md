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

## 本次结果

本机 Windows 验证：`pnpm lint`、`pnpm typecheck`、`pnpm build`、Rustfmt、WASM release 编译与实际执行检查均通过。使用已安装的 Microsoft Edge 运行 2 项完整浏览器 E2E，通过；检查产生桌面与手机截图，结束后测试服务已停止。

SQLite 集成测试的本机预编译未通过：环境缺少 MSVC `link.exe`，因此没有执行后端测试，也没有宣称原生桌面已启动或已构建成功。对应集成测试与四目标原生编译将在 GitHub Actions 中运行，实际状态以该提交的检查结果为准。

构建显示 Zod 依赖的两条 Rollup 注释解析警告；构建退出码为 0，未关闭或降级警告。pnpm 11 构建脚本白名单已使用 `allowBuilds`，只放行 esbuild。

## 未包含的验收

没有真实词典、完整变位、TTS、FSRS、第三方接口、Google 授权、Sheets 同步或浏览器学习流程；没有验证完整离线资源包、签名安装器、四架构安装或生产运行。CI 原生编译检查也不能替代这些验收。
