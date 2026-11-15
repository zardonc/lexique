# 整份备份／恢复的 Windows 原生验收（#68／#75）

对应[规格 #68](https://github.com/zardonc/lexique/issues/68)与[验收票据 #75](https://github.com/zardonc/lexique/issues/75)。2026-11-15 UTC，Windows x64 原生设置、系统文件选择器、整份替换、受控重启、正常关闭／重开、缓存播放与独立目录迁移通过。源码基于集成提交 `ca7d31eb8caa475ac806427f5e199b0daec3b5f0`，包含本次启动状态与关闭协调修正。

恢复清理与快捷键初始化保持原有串行规则；`startup_status`、`finish_close` 以及窗口关闭／退出协调离开 GUI 线程执行。清理初始化先取得已有关闭保留权，再取得启动锁，防止关闭与 worker 初始化交错。就绪页面仍经过未保存编辑保护；恢复页面可直接正常关闭。

## 数据与执行边界

A、B 均使用三条合成词汇、六张卡、两条进度及四张尚无进度的卡，含法语重音、中文笔记、例句、冻结出处、评分提交顺序、已撤销评分与批量记录。A 使用女性音频／100 WPM，B 使用男性音频／240 WPM；当前与未到期上一版音频由两条词汇共享，原始绝对保护期限保持。每份预期来自独立固定 SQL 投影、完整性／外键检查和文件字节读取。

原生夹具使用实际构建的可执行文件、当前词典／变位／词频资源及已有合法 Opus 夹具。A／MDX 导出输入构建的 SHA256 为 `5abcceeeaf5a505752487cc05a0239182e065f14dbaff5e3a46ed559421218a3`；19 个恢复、关闭与失败验收构建为 `1ba8aab398d8915244a637892825518818d89243b3e5ace3f3f7ae8234491bca`，导出实现保持一致。各项实际来源见 `run-metadata.json`。模型编码器在隔离目录中不可用；恢复后的听力卡实际播放至 `ended`，公开缓存返回的字节另经 Web Audio 解码确认非静音，未启动合成 worker。这里验证保存的单词音频经听力卡播放。

全部 21 个最终原生夹具通过。每个夹具独立保留硬性 60 秒计时至清理结束，成功或失败均只回收自有程序、worker 与辅助进程；最终清理读回均为零。系统选择器先核实自有 PID、可执行路径、模态窗口及文件名控件，再操作其窗口；受控重启接管自有后继进程并重新连接已完成导航的 WebView2。

原生驱动与独立投影位于 [native-backup-restore-check.mjs](../../../apps/desktop/e2e/native-backup-restore-check.mjs)、[native-backup-state.mjs](../../../apps/desktop/e2e/native-backup-state.mjs)。重启／进程清理与系统选择器分别复用 [native-24-harness.mjs](../../../apps/desktop/e2e/native-24-harness.mjs)、[native-input-helper.ps1](../../../apps/desktop/e2e/native-input-helper.ps1)。

## 原生验收矩阵

封存目录为集成工作树的 `target/issue-68-archive/native-75/`。`source-inventory.json` 记录原始路径、封存相对路径及字节数；各行目录内保留 `results.json`、`run-metadata.json`、`lifecycle.jsonl`、`cleanup.json`、独立预期／读回、备份原始字节与截图。运行日志位于 `logs/`，无需保留任务工作树即可检查封存证据。

| 行为 | 封存中的 `runs/` 目录 | 通过的检查 |
| --- | --- | --- |
| A 原生导出 | `export-A-03` | 实际保存选择器；完整笔记、出处、卡与历史；`A.lexique-backup`、`state-A.json`、`oracle-A.json` |
| populated B → A | `restore-A-09` | 实际预览／确认／后继就绪；独立 A 读回；已知待确认键清除、无关键保留；实际 `B-retained.lexique-backup` |
| 保存的 B 包导入 | `restore-B-01` | 实际恢复保留包，独立 B、缓存与操作重放读回 |
| A 正常重开后播放 | `playback-A-02` | 正常关闭／重开、原声字节、解码非静音、产品听力卡播放结束，无合成 worker |
| populated A → 保留 B → 正常重开 | `playback-B-over-A-01` | 克隆已关闭的完整恢复 A；`populated-A-before-B.json` 独立核实；实际 B 选择器／切换；A 音频身份消失，B 偏好、队列、历史与音频替换；重开后实际播放 |
| MDX 来源导出／迁移 | `export-mdx-01`、`migrate-mdx-01` | 三个自定义身份／冻结出处；目标精确内容自动接回，缺失内容手动接回；不同／未知内容均实际取消后确认；目标快捷键和原始 MDX 字节保留，新身份避让已有身份 |
| 正常原生关闭保护 | `close-guard-01` | 实际窗口关闭；继续编辑保留草稿，保存并离开落库后退出 |
| 版本、数量、未知字段 | `invalid1-02` | 四份包经实际打开选择器失败，错误可见，B 与原始待确认字符串保持，后续写入成功 |
| 数值、关系、JSON／ZIP 重复 | `invalid2-02` | 四份包拒绝，B 与原始待确认字符串保持，后续写入成功 |
| 路径、截断、缺音频、CRC | `invalid3-02` | 四份包拒绝，B 与原始待确认字符串保持，后续写入成功 |
| 音频内容不匹配、压缩 | `invalid4-02` | 两份包拒绝，B 与原始待确认字符串保持，后续写入成功 |
| 原来源缺失／损坏音频 | `miss-01`、`bad-01` | `source_missing`／`source_invalid` 计数与包内容独立读回；对应字节省略，学习不变，包仍可预览 |
| 导出失败与选择器取消 | `efail-01` | 实际来源采集错误可见，不发布可选部分包；保存选择器取消；恢复原偏好后 B 保持、写入与导出成功 |
| 损坏／更高 schema／迁移失败恢复 | `rcorr-01`、`rnew-01`、`rmig-01` | `recovery_required` 与真实错误；无学习 worker；原 DB／WAL／SHM／journal／音频／MDX 字节及待确认原字符串完整保留；实际恢复 A，旧键在学习就绪前清除 |
| 恢复页面正常关闭 | `rclose-01` | 实际“Close app”按钮正常退出，原始文件不变 |
| 新评分与正常重开 | `rate-01` | 原保存／评分／撤销／批量重放与冲突；恢复后提交新评分；正常重开后独立学习状态、保存的评分结果与最近撤销身份保持 |
| 启动清理与正常关闭 | `startup-probe-03` | 后继完成清理与就绪，正常关闭成功 |

`efail-01` 是来源采集失败与用户取消证明。实际 Windows 文件读取、同步、目录失败以及中断边界复用下方生产协调测试。

## 领域与持久故障矩阵

下列生产协调／真实 SQLite 证明随本次完整后端与桌面库执行保留，并复用 [#73](../issue-73/README.md)、[#74](../issue-74/README.md) 的既有原始日志与中断档案。

| 行为 | 当前源中的精确证明 |
| --- | --- |
| 同时间评分提交顺序、最近撤销、队列位置／额度、尚无进度卡 | [snapshot_import.rs](../../../crates/storage/tests/snapshot_import.rs) 的 `imported_equal_time_history_keeps_undo_order_quota_queue_and_absent_progress`；[restore_domain_tests.rs](../../../apps/desktop/src-tauri/src/restore_domain_tests.rs) 的 `ordinary_restore_keeps_history_replays_preferences_and_a_backup_that_restores_b`，提交顺序 600–609、额度／撤销／后续评分与重开 626–724；原生补充 `runs/rate-01/` |
| 相同保存／评分身份的请求冲突 | `restore_domain_tests.rs` 650–674：保存重放与不同笔记拒绝、评分重放与不同评分拒绝；原生 `restore-A-09`、`restore-B-01`、`rate-01` 的 `verifyReplays` |
| 相同音频替换身份的请求冲突 | `restore_domain_tests.rs` 的 `ordinary_restore_pairs_audio_bytes_with_original_confirmed_demand_and_retained_b`，1459–1470：原结果重放，改词汇或强制标志拒绝；[selected_audio.rs](../../../crates/storage/tests/selected_audio.rs) 的 `retrying_the_same_audio_operation_does_not_create_a_successor`，582–610：重复保持原意图，不同强制标志或声音拒绝 |
| 真实来源／目标／同步失败和恢复使用 | [backup.rs](../../../apps/desktop/src-tauri/src/backup.rs) 的 `genuine_source_read_errors_fail_export_and_release_the_maintenance_guards`、`an_unusable_destination_parent_fails_without_changing_learning_data`；[restore_tests.rs](../../../apps/desktop/src-tauri/src/restore_tests.rs) 的 `genuine_preservation_read_failure_keeps_b_and_releases_normal_learning_admission`、`genuine_new_state_sync_failure_rolls_back_complete_b_before_commit`、`genuine_rollback_copy_failure_blocks_learning_until_b_can_be_completed`、`genuine_cleanup_directory_failure_keeps_committed_a_and_retries_without_reimporting` |
| 持久准备／切换／回退／提交清理中断 | `restore_tests.rs` 的 `forced_exits_at_uncommitted_database_sidecar_and_audio_boundaries_restore_b`、`forced_exits_during_rollback_repeat_without_losing_b_or_the_original_copies`、`forced_exit_at_prepared_keeps_the_confirmed_stage_for_ordinary_startup`、`forced_exits_after_commit_and_during_cleanup_keep_a_and_gate_learning`、`forced_exits_at_persistent_sqlite_journal_moves_and_rollback_restore_b`；#73 的 45 次实际子进程退出档案 |
| 原始文件恢复与确认清理中断 | [recovery_tests.rs](../../../apps/desktop/src-tauri/src/recovery_tests.rs) 的 `raw_subprocess_interruptions_recover_only_complete_original_or_verified_timelines`、`raw_subprocess_rollback_interruptions_keep_invalid_originals_and_raw_requests`、`raw_preserved_requests_survive_interrupted_acknowledged_cleanup`；#74 的准备／切换 10 次、回退 3 次及清理 2 次实际退出档案 |

## 最终检查

工具链为 PowerShell 7、pnpm 11.25.0、Node 24.22.1、Rust／Cargo 1.99.1 与 uv Python 3.14.10。完整命令和日志封存于 `checks/`、`logs/`；编译独立于执行，存储／缓存 runner 与桌面库各分区均保持硬性 60 秒上限。

| 命令／检查 | 结果 | 封存日志 |
| --- | --- | --- |
| `pnpm lint` | exit 0，6.88 s | `logs/final-lint-green.log` |
| `pnpm typecheck` | exit 0，22.84 s | `logs/final-typecheck.log` |
| `pnpm build` | exit 0，30.38 s | `logs/final-build.log` |
| `pnpm rust:fmt` | exit 0，2.22 s | `logs/final-rustfmt.log` |
| `pnpm rust:test:build` | 独立预编译，exit 0，54.51 s | `logs/final-storage-precompile.log` |
| `pnpm test:rust` | 185 通过，exit 0，26.06 s | `logs/final-storage-tests.log` |
| `cargo test --locked --release -p lexique-desktop --test speech_cache --lib --no-run` | 独立预编译，exit 0，90.00 s | `logs/final-native-test-precompile.log` |
| `pnpm test:desktop:lib` | 六次顺序执行，88 通过／2 子进程入口按用途忽略，exit 0，合计 68.74 s | `logs/final-desktop-six.log` |
| `pnpm test:speech:cache` | 11 通过／1 既有崩溃夹具按用途忽略，exit 0，3.64 s | `logs/final-speech-cache.log` |
| `pnpm rust:check` | locked workspace，exit 0，29.81 s | `logs/final-workspace-check.log` |
| `pnpm test:e2e backup-restore backup-recovery --workers=2` | 使用已核实的 Edge 可执行文件；49/49，31.9 s，exit 0；本地启动探针先通过 1/1 | `logs/final-browser-restore-all.log`、`logs/final-browser-probe.log` |
| `pnpm test:docs` | 164 份文档／1321 个仓库链接通过，exit 0，6.73 s | `logs/final-docs.log` |
| `git diff --cached --check` | 本票 11 个文件，exit 0 | `logs/final-staged-diff.log` |

| 桌面库分区 | 所选条目 | 通过 | 忽略 | Rust 执行时间 |
| --- | ---: | ---: | ---: | ---: |
| 其他桌面库／领域 | 34 | 34 | 0 | 6.24 s |
| 备份 | 19 | 19 | 0 | 4.34 s |
| 普通恢复 | 17 | 16 | 1 | 7.58 s |
| 三项强制退出循环 | 3 | 3 | 0 | 19.71 s |
| 回退强制退出循环 | 1 | 1 | 0 | 11.60 s |
| 独立启动／原始文件恢复 | 16 | 15 | 1 | 10.94 s |
| 合计 | 90 | 88 | 2 | |

两个忽略入口由中断父测试显式启动。完整日志核实全部 90 个条目各选一次，记录于 `checks/desktop-selection-results.json`。各分区显式单线程，使用既有 [test-rust.mjs](../../../scripts/test-rust.mjs) 的独立 60 秒执行计时；汇总时间不作为单次 runner 预算。

集成基线 `ca7d31eb8caa475ac806427f5e199b0daec3b5f0` 的 [PR CI](https://github.com/zardonc/lexique/actions/runs/43654344670) 与 [push CI](https://github.com/zardonc/lexique/actions/runs/43654340882) 共 22 项检查通过。[前端 job](https://github.com/zardonc/lexique/actions/runs/43654344670/job/132629480751) 记录完整 375/375（3.2 min）及 lint／typecheck／build，[文档 job](https://github.com/zardonc/lexique/actions/runs/43654344670/job/132629480747) 通过。这些 CI 结果对应该集成 SHA；本次源码的本机检查如上。

## 恢复记录、词典预览与便携路径

2026-11-15 UTC，当前 Windows x64 构建的 SHA256 为 `8012803edac7648e1122f04f3d2fb0da0465036129ca3986d7030e5b8a9d8100`，集成基线为 `34040078f101d9973df93aee38e552010799fca4`。本节证据位于工作树 `target/review-fixes68/`，原生夹具的紧凑副本位于其中的 `native/runs/`；各自的 `run-metadata.json` 记录完整命令、实际输入和构建身份。上面的验收表与 CI 链接保留各自的源码与执行身份。

预览与实际恢复共用目标词典内容验证；匹配条目保持可用，缺失、内容不同或来源哈希未知的自定义条目按 catalog 顺序待重连。预览只读目标库和文件；legacy 设备 catalog 在内存中归一化。独立恢复跳过失败的原学习库。恢复前已有日志以完整原始字节保留至 `raw-files/.lexique-restore.json`，与原始文件和待确认字符串同步后才能发布新的恢复记录。损坏 JSON、不可解码日志及未完成回滚均覆盖；已有保留数据库、音频、逻辑备份、决策、旧 staging 和待确认证据保持，日志复制或读取失败可见并阻止替换。Portable 启动始终将 `TEMP` 和 `TMP` 绑定到预期便携目录，目录创建失败的诊断保持。

| 检查 | 结果 | `target/review-fixes68/` 中的证据 |
| --- | --- | --- |
| `cargo fmt --all --check`、`cargo check --locked --workspace` | 通过 | `logs/rust-format-green-check.log`、`logs/rust-workspace-check-final.log` |
| `pnpm rust:test:build`、`pnpm test:rust` | 独立预编译后 185 通过 | `logs/storage-precompile-final.log`、`logs/storage-tests-final.log` |
| 桌面库 release 独立预编译与完整八分区 | 94 个条目无遗漏／重复；92 通过，两个子进程入口按用途忽略 | `desktop-selection-results.json`、`logs/desktop-partition-*.log` |
| 缓存 release 独立预编译、`pnpm test:speech:cache` | 11 通过，一个既有崩溃入口按用途忽略 | `logs/cache-precompile-final.log`、`logs/cache-tests-final.log` |
| `pnpm lint`、`pnpm typecheck`、`pnpm desktop:build --no-bundle` | 通过；原生构建包含 `pnpm build` | `logs/lint-final.log`、`logs/typecheck-final.log`、`logs/desktop-build-final.log` |
| 浏览器启动探针；`pnpm test:e2e backup-restore backup-recovery --workers=2` | 已核实的 Playwright headless shell；1/1、49/49 通过 | `logs/browser-startup-probe.log`、`logs/browser-recovery-final.log` |
| 原生 `migrate-mdx` | 8 项检查、实际打开选择器；待重连列表只含未验证的自定义条目，原目标文件与冻结学习内容保持 | `native/runs/migrate-mdx/`、`logs/native-migrate-mdx-accepted.log` |
| 原生 `recovery-journal` | 4 项检查、实际打开选择器；损坏日志／文件／待确认字符串逐字节保留，独立 A 读回通过 | `native/runs/recovery-journal/`、`logs/native-recovery-journal-green.log` |
| Windows portable 进程环境 | 当前构建在临时目录被文件阻挡时，实际 `TEMP`／`TMP` 均为预期便携路径；硬限 60 秒 | `portable-final-01-result.json`、`logs/portable-environment-final.log` |

| 当前桌面库分区 | 所选条目 | 通过 | 忽略 | Rust 执行时间 |
| --- | ---: | ---: | ---: | ---: |
| 其他桌面库／领域 | 34 | 34 | 0 | 6.45 s |
| 备份 | 20 | 20 | 0 | 5.32 s |
| 普通恢复 | 18 | 17 | 1 | 9.05 s |
| 未提交状态强制退出循环 | 1 | 1 | 0 | 9.74 s |
| 提交后／清理强制退出循环 | 1 | 1 | 0 | 3.91 s |
| SQLite 持久日志强制退出循环 | 1 | 1 | 0 | 6.04 s |
| 回退强制退出循环 | 1 | 1 | 0 | 11.91 s |
| 独立启动／原始文件恢复 | 18 | 17 | 1 | 12.15 s |
| 合计 | 94 | 92 | 2 | |

三个强制退出循环各自独立执行，原有案例、runner 与每次 60 秒硬限保持。完整名单、各分区名单、命令与实际结果见 `desktop-selection-results.json`；Windows 库存不用于推定 macOS 条目数量。原生夹具独立保留 60 秒硬限至清理结束，当前夹具自有 host／worker／helper 的清理读回均为零。

## 验证范围

本次为 Windows x64 原生合成隔离数据验收，使用真实 SQLite、文件操作、系统选择器、自有进程重启与产品播放。独立目录迁移验证便携状态与目标设备设置；ARM／macOS GUI、物理跨 OS 迁移、整机断网、正式发行／首装／签名及资源准入继续按 [development-readiness.md](../../development-readiness.md) 的独立门槛验证。
