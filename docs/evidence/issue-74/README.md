# 损坏学习数据的独立恢复（#74）

对应[规格 #68](https://github.com/zardonc/lexique/issues/68)与[票据 #74](https://github.com/zardonc/lexique/issues/74)。启动失败保留真实诊断与预期路径，用户可从已验证备份恢复；原始学习文件完整保留后才进入同一持久切换流程。

## 数据与运行边界

`ready`／`recovery_required` 启动状态独立于数据库连接、正常运行状态、词典缓存和后台 worker。SQLite 损坏、当前更高 schema、完整性／迁移失败以及不可用的数据／临时目录均保持学习关闭，不创建替代空库。普通初始化先在自有副本中验证，失败重试保留原数据库及 sidecar 字节；实际学习连接仍由正常初始化入口打开并校验。

正式确认前保存可读的已知评分／撤销／批量 WebView 键及原始字符串，保留格式错误的值。原生只接受已知应用键的身份，不解析这些旧值。切换关闭相关句柄后，将原数据库、所有存在的 WAL／SHM／rollback journal、音频及应用托管 MDX 逐字节保存至独立保留目录；原始请求先持久保存，再发布 `prepared`。

原始文件保留副本显示实际路径、字节数与类型，不作为已验证可导入备份，也不进入逻辑导入或垃圾清理。读取、复制、同步、目录或请求保存失败停止替换，并保留可找回位置及错误。目标设备设置独立配置；资源缺口保持实际能力。

恢复复用 [#73](../issue-73/README.md) 的 `prepared`／`swapping`／`committed`、组件存在性、移动意图和回退决定。原数据库原本不存在时，回退保留其不存在，不初始化空学习库。已提交启动在学习开放前完成已知旧请求的删除、读回与原生确认；原始字符串持续保留，无关 WebView 数据保持。通知绑定当前恢复身份，迟到的旧身份及同身份退后状态不覆盖当前进度，当前诊断保持可见。

## 原生与浏览器证据

原生测试使用真实 SQLite、生产启动／恢复协调、Windows 文件操作与共享锁，覆盖损坏／不支持／完整性／迁移失败、缺少健康状态对象、原始字节与 sidecar、不可写目录、保留读取／复制／同步失败、普通恢复启动及重复恢复。

独立恢复父测试实际终止自有子进程：准备／切换 10 次、回退 3 次、请求清理 2 次，并另行终止 WAL 播种子进程。#73 既有恢复父测试保留 45 次实际退出。子进程在隔离路径中按同步标记执行，父进程回收后调用生产启动入口，核实完整原状态或完整已验证状态。

浏览器验证独立恢复入口、严格 IPC 字段、原始字符串、旧请求保护、资源能力、错误、当前身份、重复确认、键盘与关闭入口。恢复后健康页面允许正常读取当前批量状态；旧身份的评分／撤销／批量请求不能重放。

## 最终检查

2026-11-14，工作树 `issue-74`、分支 `codex/backup-restore-74`，基于集成提交 `c698797272b8474189773fc77cd66aae3fcec94c`。完整日志位于本工作树 `target/issue-74/`，使用 Rust／Cargo 1.99.1、Node 24.22.1、pnpm 11.25.0、Python 3.14.10；共享 Cargo target 为相邻 `issue-72/target/`。原生编译与执行分别计时，所有执行 runner 保留硬性 60 秒上限。

| 命令／检查 | 最终结果 | 完整日志 |
| --- | --- | --- |
| `pnpm lint` | exit 0 | `final-lint-diagnostics-green.log` |
| `pnpm typecheck` | exit 0 | `final-typecheck-diagnostics-green.log` |
| `pnpm build` | exit 0，25.44 s | `final-frontend-build-diagnostics-green.log` |
| `pnpm test:e2e backup-restore backup-recovery --workers=2` | 49/49，29.2 s，exit 0 | `final-browser-restore-green.log` |
| `pnpm rust:fmt` | exit 0，1.54 s | `final-original-open-rustfmt-check.log` |
| `pnpm rust:test:build` | 独立预编译，exit 0，44.14 s | `final-backend-precompile.log` |
| `pnpm test:rust` | 185 通过，exit 0，23.75 s | `final-backend-tests.log` |
| `cargo test --locked --release -p lexique-desktop --test speech_cache --lib --no-run` | 独立预编译，exit 0，69.28 s | `final-original-open-release-precompile.log` |
| Windows 原始文件共享锁回归 | 1/1，七项实际共享锁失败，exit 0，1.14 s | `final-native-source-lock-green.log` |
| 普通恢复启动共享锁回归 | 1/1，exit 0，0.80 s | `ordinary-restore-startup-lock-green.log` |
| `pnpm test:desktop:lib` | 88 通过／2 子进程入口忽略，exit 0，70.53 s | `final-desktop-five-pass-green.log` |
| `pnpm test:speech:cache` | 11 通过／1 既有崩溃夹具忽略，exit 0 | `final-speech-cache-tests.log` |
| `pnpm rust:check` | locked workspace，exit 0，23.62 s | `final-rust-workspace-check.log` |
| `node target/issue-74/desktop-selection-check.mjs` | 90 个 Windows 条目各选一次，exit 0 | `final-desktop-selection-check.log` |
| `node --test target/issue-74/runner-regression.mjs` | 2/2，重复 skip 传递、完整选择与失败短路通过 | `final-desktop-runner-regression.log` |
| `pnpm test:docs` | 163 份文档／1303 个仓库链接通过，exit 0 | `final-docs-check.log` |
| `pnpm test:docs:validator` | 13/13，exit 0，18.95 s | `final-docs-validator.log` |

五个公开分区顺序执行，每个显式单线程。实际 Rust `--list` 及公开 runner 参数核实全部 90 个条目恰好各选一次，包含两个由父测试显式调用的忽略子入口；注入任一分区 exit 19 后，后续分区不启动。

| 分区 | 实际通过 | 忽略 | 所选条目 | Rust 执行时间 |
| --- | ---: | ---: | ---: | ---: |
| 其他桌面库测试 | 53 | 0 | 53 | 11.61 s |
| 普通恢复 | 16 | 1 | 17 | 8.31 s |
| 三项中断循环 | 3 | 0 | 3 | 20.35 s |
| 回退中断循环 | 1 | 0 | 1 | 12.26 s |
| 独立启动／原始文件恢复 | 15 | 1 | 16 | 10.98 s |
| 合计 | 88 | 2 | 90 | |

## 验证范围

集成提交 `ca7d31eb8caa475ac806427f5e199b0daec3b5f0` 的 [PR CI](https://github.com/zardonc/lexique/actions/runs/43654344670) 与 [push CI](https://github.com/zardonc/lexique/actions/runs/43654340882) 共 22 项检查通过。[完整前端 job](https://github.com/zardonc/lexique/actions/runs/43654344670/job/132629480751) 明确记录 375/375 浏览器测试通过（3.2 min），包含 lint、typecheck、build；[文档 job](https://github.com/zardonc/lexique/actions/runs/43654344670/job/132629480747) 通过。本票恢复／启动套件 49/49 与截图保留于原有档案。原生严格 Clippy 未在本票重复执行，#73 的既有诊断与最终集成审查边界保持。

本票使用合成隔离学习数据、真实 Windows 文件／子进程、合法资源及严格浏览器 IPC 替身。实际 Windows GUI、系统选择器、受控重启、真实播放与独立目录迁移通过 [#68／#75 原生验收](../issue-68/README.md)。其他 OS／架构、物理跨系统迁移、正式发行／首装／断网／签名按既有门槛分别验证。
