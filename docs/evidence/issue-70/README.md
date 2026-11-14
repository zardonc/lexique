# 自定义词典身份与 MDX 补齐（#70）

2026-11-13，基于已合入界面偏好的集成分支验证。逻辑清单、内容身份与本机托管文件关联分开保存；缺文件保留身份、标题、顺序和学习记录。相同内容直接补齐，不同或未知内容须确认；旧托管文件保留。

| 命令 | 结果 |
| --- | --- |
| `pnpm install --frozen-lockfile` | 通过，锁文件未修改 |
| `pnpm lint`、`pnpm typecheck`、`pnpm build` | 通过 |
| `pnpm rust:fmt`、`pnpm rust:check` | 通过，锁定 workspace 检查 |
| `pnpm rust:test:build` → `pnpm test:rust` | 通过，157 项；测试执行保留 60 秒硬限 |
| `pnpm mdict:test:build` → `pnpm test:mdict` | 通过，9 项 |
| `cargo test --locked --manifest-path apps/desktop/src-tauri/Cargo.toml --release --lib --no-run` → `pnpm test:desktop:lib` | 通过，21 项 |
| `pnpm test:e2e dictionary-relink.spec.ts dictionary-tabs.spec.ts dictionary.spec.ts dictionary-lemmas.spec.ts dictionary-resources.spec.ts example-excerpts.spec.ts --workers=3` | 通过，28 项 |
| `pnpm test:e2e --workers=3` | 合入界面偏好后通过，242 项 |
| `pnpm test:docs`、`git diff --check HEAD` | 通过；158 份文档，无链接问题 |

原生模块使用隔离 SQLite 与项目既有合法／损坏 MDX。验证恢复清单不凭标题／编号选文件、新增导入避开托管文件和冻结／历史出处、同内容保持原身份和顺序、不同／未知内容先确认、确认后重查候选内容、旧文件与学习记录保留，以及解析／发布／登记与设置验证失败回滚。新注册表实例回读同一数据，验证缓存与持久关联。

浏览器使用已安装的 Edge 和 Playwright 管理的服务器，验证缺文件标签、原身份补齐、不同内容确认与取消、提交期间禁用操作／Escape、满容量未知内容补齐、错误和不匹配 IPC 身份可见。检查结束后服务器停止。

完整日志位于本次工作树忽略的 `target/issue-70/logs/`。浏览器 IPC 替身不证明原生文件发布；SQLite／注册表回读不证明原生 GUI 重启。原生 GUI、跨平台、组合备份恢复及发行检查由 #75 继续执行。
