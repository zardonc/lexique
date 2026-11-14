# 手动备份与完整学习快照（#71）

2026-11-13，[#71](https://github.com/zardonc/lexique/issues/71) 实现 [规格 #68](https://github.com/zardonc/lexique/issues/68) 的手动备份导出，使用隔离合成数据库、合成 Ogg 字节和既有合法词典夹具；没有读取个人学习库。整份导入、受控重启和启动恢复入口由后续任务接入。

## 已实现边界

- v1 根对象完整保留 21 个字段；评分／批量的连续 sequence 来自原 rowid 提交顺序。双卡、未建立进度、撤销前队列、暂停、每日额度所用历史、原始保存请求及结果、部分／取消批量和全部已确认语音需求使用既有领域类型。
- 原生采集关闭新写入，排空完整已接受任务与回调，再暂停合成及独立 GC／对账。一致 SQLite 读取、词典清单规范化和受保护音频复制完成后恢复使用；打包不继续持有学习写入暂停。
- 当前／未到期上一版、共享哈希、初始已归属和未登记旧音频使用真实文件检查；缺失／损坏保留关联并明示。已替代旧音频、临时／例句、过期及未完成发布字节排除。验证封存但尚未发布的持久中断状态仍保留为 unpublished。
- Stored ZIP／ZIP64 只含 manifest、snapshot 和实际包含哈希的音频。目标必须为新文件，目标文件系统暂存、同步与完整 ZIP 回读通过后独占发布。损坏的包内 included 字节使整包拒绝，不能改称原有缺失。
- 主窗口专用命令使用原生保存选择器；任意路径及整包字节不进入 IPC 参数。单次操作、关闭保护、取消、失败与 worker panic 有真实终态。前端处理未保存草稿、离开保护及全部已知待确认评分／撤销／批量，再开始原生导出。
- 可迁移偏好为明确白名单，复用 #69 界面设置和 #70 词典清单；设备设置、路径、凭据、日志、MDX 正文和模型排除。新增依赖的原始通知通过 `backup-notices/` 随包交付。

源边界：[快照 DTO](../../../crates/storage/src/learning_snapshot.rs)、[一致导出](../../../crates/storage/src/snapshot_export.rs)、[严格校验](../../../crates/storage/src/snapshot_validation.rs)、[原生容器](../../../apps/desktop/src-tauri/src/backup.rs)、[操作状态](../../../apps/desktop/src-tauri/src/backup_operations.rs)、[维护租约](../../../apps/desktop/src-tauri/src/maintenance.rs)、[独立合成验收夹具](../../../crates/storage/tests/snapshot_admission.rs)。

## 检查与证据

完整日志保存在本次工作树的忽略目录 `target/issue-71/`；浏览器独立工作树日志在 `target/issue-71-frontend/`。原生／存储测试独立预编译，现有 runner 的硬性 60 秒执行上限保持。

| 检查 | 结果 |
| --- | --- |
| `pnpm lint`、`pnpm typecheck`、`pnpm build` | 合并前端后通过；Vite 保留既有体积／第三方注释警告 |
| 40 项备份浏览器测试、`pnpm test:e2e --workers=3` | 40／282 全部通过，使用严格 IPC 替身；服务与端口已释放 |
| `pnpm rust:test:build` 后 `pnpm test:rust` | 179 项核心／存储／词典测试通过，含 19 项完整快照验收夹具；独立夹具最终执行 1.38 秒 |
| `cargo test --locked --release -p lexique-desktop --test speech_cache --lib --no-run` 后 `pnpm test:desktop:lib` | 38 项通过，1.87 秒；含 15 项备份边界测试，覆盖完整批量排空、五阶段取消与真实继续、独立 GC 排空／继续、目标竞争、Windows 文件共享读取失败、不可用目标父目录、受保护副本损坏及恶意 ZIP／JSON／音频检查 |
| 同次 release 预编译后 `pnpm test:speech:cache` | 11 项通过，0.64 秒；崩溃子进程专用夹具一项按既有规则忽略 |
| 依赖通知打包回归 | 先复现 ZIP 缺通知，再通过随包回读检查 |
| `pnpm test:tooling` | 通过，包含完整 portable／speech／frequency 工具回归 |
| `cargo fmt --all`、`pnpm rust:fmt`、`pnpm rust:check` | 格式与锁定 workspace 编译通过；保留既有 speech-cache dead-code 及后续恢复接入 API 的警告 |
| `git diff --check`、`pnpm test:docs`、`pnpm test:docs:validator` | 差异和本地链接检查通过，13 项验证器检查通过 |

独立存储夹具复现并修正合法操作身份被拒绝、必需可空字段缺失被接受、当前／pending／目标／生成及验证关系矛盾被接受等问题；正面夹具保留历史保存结果、同秒评分／批量顺序、两个卡片的原进度、撤销、暂停、原音频保护期及已确认需求预算。验证后 pending 来源错误分类为 not_selected 的失败已改为 unpublished，支持 prepare → verify → capture 的真实持久边界。

命令写入准入的独立只读审查无待修复遗漏，覆盖全部注册命令及 blocking 任务／回调的租约生命周期。完整日志使用 `final-*` 命名；故意取消 ZIP 写入时 zip-rs 的 Drop 诊断保持可见，不影响取消终态与清理断言。

## 仍须分别验收

本次自动化原生检查证明隔离 SQLite、文件、真实 worker 与 Rust 容器边界。原生 GUI 保存选择器、系统实际音频播放、macOS／Windows ARM64 执行、物理跨系统迁移、整份恢复、发行包、首装／断网／签名验收尚未执行，由 #72–#75 和既有发行门槛继续验证；浏览器或编译结果不替代这些证据。
