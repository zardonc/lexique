# 整份恢复与持久启动恢复（#73）

对应[规格 #68](https://github.com/zardonc/lexique/issues/68)与[票据 #73](https://github.com/zardonc/lexique/issues/73)。正式确认消费已验证的预览身份，先保留当前状态 B，再受控重启到完整 A。恢复前备份可再次恢复 B；原数据库及音频副本持续保留，并提供按原生副本 ID 打开目录的入口。

## 数据与运行边界

普通启动在 SQLite 初始化、迁移、缓存和 worker 之前处理 `prepared`／`swapping`／`committed`。移动前保存组件存在性和意图；未提交状态回退完整 B，已提交状态保留 A 并继续清理。活动记录仍为 `prepared` 时，也遵循保留目录中匹配的持久回退决定。无法读回决定或完成回退时显示错误及副本位置，保持学习入口关闭。

正式切换排空已接受的读写回调并关闭旧 worker。终止关闭后的失败仍保持旧学习入口关闭，同时释放备份关闭保留权，使恢复窗口可以关闭／退出。相同身份的重复请求返回原操作状态；迟到事件和回调不能重新开放旧学习页面或启动第二次切换。

便携偏好使用 A，支持的目标设备快捷键保持；MDX 按实际内容接回，同名错误文件不关联，冻结出处和预留身份保持。原评分／撤销／批量结果、提交顺序、绝对时间、额度、音频选择和保护期限保持。原缺失音频继续待修复；已确认需求保留取消、预算和原期限，旧发布回调不能覆盖新意图。

已提交启动先持久登记旧请求清理，前端只删除已知评分／撤销／批量待确认键并验证不存在，再由原生确认并初始化学习。存储访问、身份、清理或原生确认失败继续关闭学习和旧请求重试入口；无关 WebView 数据保持。

## 独立领域与中断证据

领域夹具通过真实 SQLite 和原生协调完成 A → B → 恢复 A → 普通启动，并实际从恢复前备份恢复 B。历史预期来自恢复前独立 SQL 和既有领域接口，音频来自已有合法 Ogg 夹具的真实字节。

45 次实际终止自有子进程覆盖准备交接、SQLite 主库／WAL／SHM／rollback journal、音频隔离／安装、回退、提交及旧请求清理边界。父进程等待精确同步标记后终止并回收子进程，再调用生产普通启动入口。原数据库／音频不存在的持久记录夹具验证不会创建空学习库，也不会将新空目录误认为完整旧状态。

| 检查 | 通过结果 | 本机完整日志 |
| --- | --- | --- |
| 真 WAL 主库隔离 | 1/1，0.39 s | `target/issue-73/finish-real-wal-tracer-test.log` |
| 13 项正向移动中断 | 1/1，8.93 s | `target/issue-73/finish-forward-interruptions-test.log` |
| 14 项回退移动中断 | 1/1，10.64 s | `target/issue-73/finish-rollback-interruptions-test.log` |
| 8 项持久 rollback journal 中断 | 1/1，5.83 s | `target/issue-73/finish-journal-interruptions-green-test.log` |
| prepared 交接中断 | 1/1，0.69 s | `target/issue-73/finish-prepared-interruption-test.log` |
| 6 项提交／清理中断 | 1/1，3.30 s | `target/issue-73/finish-committed-interruptions-test.log` |
| 原音频／数据库不存在 | 各 1/1，0.58／0.55 s | `target/issue-73/finish-audio-absence-test.log`、`finish-database-absence-test.log` |
| Windows 保留读取／新状态同步／回退／清理共享锁故障 | 各 1/1，0.12／0.72／0.82／0.60 s | `target/issue-73/finish-preservation-sharing-test.log`、`finish-switch-sync-sharing-test.log`、`finish-rollback-sharing-test.log`、`finish-cleanup-sharing-test.log` |
| 失败释放关闭保留权／相同身份重复开始 | 各 1/1，0.06／0.05 s | `target/issue-73/finish-close-reservation-green-test.log`、`finish-repeated-start-green-test.log` |
| 历史／便携偏好／设备 MDX／音频需求与 A、B 真实恢复 | 3/3，4.09 s | `target/issue-73/finish-domain-all-green-test.log` |
| 七项命令生成与本地主窗口权限 | 7/7 | `target/issue-73/finish-restore-admission-generated.log` |

## 最终合并检查

前端提交 `d7d0a7578cbdfc07405184be7bb366a5bc8f37f4` 已以 `97747cfaa3a6f2337734a84851fbdeed426fd6eb` 合入本票；前端源码与通过检查的提交保持一致。原生完整日志位于本票工作树 `target/issue-73/`，前端完整日志位于 `issue-73-frontend/target/issue-73-frontend/`。Rust 1.99.1、Cargo 1.99.1、Node 24.22.1、pnpm 11.25.0、Python 3.14.10 的实际解析及串行测试环境记录在 `target/issue-73/final-tool-versions.json`。

| 命令／检查 | 最终结果 | 完整日志 |
| --- | --- | --- |
| `cargo fmt --all`；`pnpm rust:fmt` | 通过 | `final-reviewed-cargo-fmt.log`、`final-reviewed-rust-fmt.log` |
| `pnpm rust:check` | 通过，16.92 s | `final-reviewed-rust-check.log` |
| `cargo clippy --locked -p lexique-core -p lexique-storage -p lexique-dictionary --all-targets -- -D warnings` | 通过，14.35 s | `final-core-clippy.log` |
| `pnpm rust:test:build` | 独立预编译通过，5.79 s | `final-rust-test-build.log` |
| `pnpm test:rust` | 185 通过，19.57 s | `final-backend-tests.log` |
| `cargo test --locked --release -p lexique-desktop --test speech_cache --lib --no-run` | 独立预编译通过，68.71 s | `final-reviewed-desktop-test-build.log` |
| `pnpm test:desktop:lib` | 73 通过／1 子进程入口忽略，50.62 s | `final-desktop-lib-tests.log` |
| `pnpm test:speech:cache` | 11 通过／1 既有崩溃夹具忽略，3.69 s | `final-speech-cache-tests.log` |
| `pnpm lint`；`pnpm typecheck`；`pnpm build` | 前端通过 | 前端 `lint-final-delivery.log`、`typecheck-final.log`、`build-final.log` |
| 恢复浏览器检查；聚焦变位检查 | 39／39，44.9 s；2／2，18.9 s | 前端 `restore-final-focused.log`、`conjugation-status-focused.log` |
| 完整浏览器检查 | 365／365，7.1 min，exit 0 | 前端 `full-browser-final.log` |
| `node target/issue-73/check-restore-admission.mjs --generated` | 七项命令注册／生成权限／本地主窗口准入通过 | `final-restore-admission.log` |
| `cargo clippy --locked -p lexique-desktop --all-targets --message-format=json -- -D clippy::all` | 严格失败，14 项规格前诊断；没有新诊断或其他编译错误 | `final-native-clippy-final.jsonl`、`final-native-clippy-final.log` |
| `git diff --cached --check` | 通过，28 个本票文件 | `final-delivery-diff-check.log` |
| `pnpm test:docs` | 162 份文档／1301 个仓库链接通过 | `final-docs-delivery.log` |

桌面库完整检查包含三项独立历史／偏好／MDX／音频领域证明及所有中断父测试。`restore_interruption_child` 仅供这些父测试在隔离路径中显式启动，普通库测试将该入口忽略；缓存的 `publication_crash_fixture` 保持既有忽略规则。编译与实际执行分开，三个 Rust runner 的硬性 60 秒上限保持。原生与前端的七项命令、两个事件、状态和字段协议已逐项对照。

最终补充 Clippy 的 14 项逐行来源见 `target/issue-73/final-native-clippy-final-provenance.json`，均来自规格前 main。#70 的 MDX 条件与 #71 的缺失音频索引两项诊断已作行为保持的修正；修正前的完整 16 项报告及来源分别保留在 `final-native-clippy-baseline.jsonl`、`final-native-clippy-provenance.json`。前端构建中的既有第三方注释与包体警告继续记录在构建日志。

## 验证范围

本票自动化使用隔离合成学习库、真实 Windows 文件操作、生产普通启动协调和合法资源夹具。浏览器检查使用严格 IPC 替身，验证浏览器本地存储键、保护、进度、失败和迟到回调。原数据库不存在的持久记录夹具不证明 #74 的损坏来源恢复准备流程。

完整 Windows 原生设置／选择器／重启／播放与整份操作验收由 #75 继续完成；#74、其他 OS／架构、物理跨系统迁移、正式发行／首装／断网／签名仍分别验收。语音需求恢复检查不执行模型合成或实际播放。
