# 隔离备份校验与预览（#72）

对应[规格 #68](https://github.com/zardonc/lexique/issues/68)与[票据 #72](https://github.com/zardonc/lexique/issues/72)。设置页选择备份后，显示已验证的文件信息、全部内容数量、原有音频缺失／损坏、待补齐词典和恢复后的重启要求。合法内容完整准备为独立 SQLite 与托管音频；当前学习状态和草稿保留。

## 数据与操作边界

v1 导入使用当前初始化 schema 的空目标和一次立即事务，参数化保存全部学习集合。记录、逐句出处、双卡、实际进度、评分／撤销／原始队列、暂停、批量、语音来源／归属／任务、音频版本／选择／导入／旧关系／操作／发布／修复、便携偏好与逻辑词典清单均保留。评分和批量保持提交顺序，原始 f32 结果可重放；冻结的移除词典出处、未建立的进度、旧零生成／未知时间及失败批量缺失 ID 均可导入。

原绝对 expiry／orphan／previous 期限、取消／终态需求、尝试预算与执行意图保持。物理绑定和运行投影在隔离库重建；`included` 版本要求已知长度，可保留 `NULL` 生成时间和未验证状态；`source_missing` 可保留 `NULL` 长度／生成时间。原验证／封存状态保留。托管音频只从各自 `included` 引用发布并完整回读，同哈希的原缺失／损坏／排除引用保持无文件。正式恢复继续使用已准备对象。

主窗口的选择／状态／取消命令与进度事件只传递操作身份和完整只读事实。前端在启动前订阅，启动后核对状态，按身份和修订保护新事件；取消、重新选择及迟到响应遵守原生暂存生命周期。三语界面提供可访问进度、错误、重新核对和重试，学习页继续使用时设置进度保持挂载。导出／预览／关闭统一协调。

实现：[快照导入](../../../crates/storage/src/snapshot_import.rs)、[导入领域检查](../../../crates/storage/tests/snapshot_import.rs)、[严格快照校验](../../../crates/storage/src/snapshot_validation.rs)、[原生容器](../../../apps/desktop/src-tauri/src/backup.rs)、[操作协调](../../../apps/desktop/src-tauri/src/backup_operations.rs)、[只读 IPC](../../../apps/desktop/src/platform/backup.ts)、[预览界面](../../../apps/desktop/src/backup-preview.tsx)。

## 检查与证据

存储日志位于本次工作树的忽略目录 `target/issue-72/`，源码核对、协议及文档检查日志另存于 `target/issue-72-merge/`。存储 `b313512` 的 52 份证据已归档至集成工作树的 `target/issue-68-archive/storage-b313512/`，最终后端 `c74d708` 的六份日志位于 `target/issue-68-archive/backend-c74d708/`。前端 `bc2b19c` 的完整 36 份日志已归档至同一工作树的 `target/issue-68-archive/preview-frontend-bc2b19c/`，原始日志保留在前端工作树 `target/issue-72-frontend/`。原生 `c1bc882` 的最终测试／编译日志保留在原生工作树的 `target/issue-72-native/`，命令准入、源码匹配和 Clippy 来源清单位于其 `target/issue-72/`。Rust 测试分别预编译，既有 runner 的硬性 60 秒执行上限保持。

| 检查 | 结果 |
| --- | --- |
| `pnpm rust:test:build`；精确 locked `snapshot_import --no-run` 后运行导入检查 | 六项通过，0.16 秒；分别用现有记录／保存重放、评分／撤销／队列／额度、批量／语音及音频读接口核实 |
| SQLite 容量故障 | `PRAGMA max_page_count` 在合成内存库触发真实 `SQLITE_FULL`；部分写入发生后完整回滚，原默认偏好／schema／外键保留 |
| 目标拒绝 | 已有学习、仅有设置、未初始化及 schema 17／19／99 均在写入前拒绝，内容和 schema 保持 |
| 核心／存储／词典 Clippy（`--locked --all-targets -- -D warnings`） | Cargo／Rust 1.99.1 通过 |
| 最终 `pnpm rust:test:build` 后独立 `pnpm test:rust` | `c74d708` 185 项通过，包含六项导入检查；runner 19.809 秒，硬性 60 秒上限保持 |
| 前端 `pnpm lint`、`pnpm typecheck`、`pnpm build` | `bc2b19c` 通过；既有第三方注释和包体警告保留 |
| 预览／导出／完整浏览器检查 | `bc2b19c` 分别 44／40／326 项通过，使用严格 IPC 替身 |
| 原生独立 release 预编译后 `pnpm test:desktop:lib`／`pnpm test:speech:cache` | `c1bc882` 分别 48／11 项通过，5.88／1.30 秒；缓存原有崩溃子进程夹具一项忽略 |
| 原生暂存与生命周期 | 真实 ZIP／SQLite／托管音频回读、取消与清理排他、文件失败、源文件替换／删除、迟到进度／结果／panic、一次性消费／关闭保留权及根目录所有权转移通过 |
| 最终 `cargo fmt --all --check`／`cargo check --locked --workspace` | `c1bc882` 通过，锁定编译 9.81 秒；Cargo／Rust 1.99.1 |
| 原生命令／事件准入与 TypeScript 协议 | 三个命令注册、精确生成授权、本地主窗口参数与事件接收方四项通过；字段、枚举／空值和命令／事件形状与前端严格协议一致 |
| 文档差异与仓库链接 | 通过 |
| 原生补充 Clippy（`-D clippy::all`） | 16 项既有诊断：14 项来自规格前 main，2 项来自 #70／#71；完整日志和逐行来源清单保留供 #68 最终审查 |

## 仍须分别验收

本票的自动化证据使用合成学习数据、隔离数据库／文件和合法资源夹具。正式替换与启动恢复由 #73／#74 接入；原生 GUI 选择器、系统实际音频播放、Windows x64 整份恢复／重启、其他 OS／架构、物理跨系统迁移及发行／首装／断网／签名由 #75 和既有门槛分别验收。
