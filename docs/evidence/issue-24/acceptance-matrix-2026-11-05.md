# #24 验收覆盖

2026-11-05，Windows x64。后端138项、完整浏览器100项、Windows缓存10项及桌面库3项通过；新增界面边界与恢复专项分别11项、12项通过。真实产品、选定音频、发布恢复、维护、独立队列参考、10k窗口、50k混合历史资源与原生worker超时均有执行结果。下表将[50专项](../../issue-24-test-plan.md)、[原场景与准入](github-design-review.md)及[14项验收](../../issue-24-spec.md)逐项对应到实际断言。

状态：**通过**表示本行所列范围已有执行证据；**条件**表示已有覆盖且保留表中明确缺口；**不适用**表示条件能力未启用；**后续**表示既定里程碑。干净系统断网测试由用户执行，2026-11-05报告整体通过。条件项不计为完整用例通过。业务规则由DB测试、界面状态由UI测试、宿主能力由原生执行共同证明，不要求将每个规则再次组合成原生UI用例。

## 证据入口

| 代号 | 测试源码／结果 |
| --- | --- |
| DB-R | [review.rs](../../../crates/storage/tests/review.rs)：双卡、额度、位置、评分／撤销、暂停、提交失败及重开 |
| DB-Q | [queue_cache.rs](../../../crates/storage/tests/queue_cache.rs)：修订、增量、WAL一致快照、连续修复写入 |
| DB-A | [selected_audio.rs](../../../crates/storage/tests/selected_audio.rs)：内容版本、选择代次、发布意图、共享归属、期限、GC及迁移 |
| DB-W | [speech_work.rs](../../../crates/storage/tests/speech_work.rs)、[review_audio_demand.rs](../../../crates/storage/tests/review_audio_demand.rs)：批量、预算、优先级、公平及原规格修复 |
| UI-Q | [automatic-queue.spec.ts](../../../apps/desktop/e2e/automatic-queue.spec.ts)、[queue-window.spec.ts](../../../apps/desktop/e2e/queue-window.spec.ts)、[review-queue.spec.ts](../../../apps/desktop/e2e/review-queue.spec.ts) |
| UI-A | [review.spec.ts](../../../apps/desktop/e2e/review.spec.ts)、[queue-audio-recovery.spec.ts](../../../apps/desktop/e2e/queue-audio-recovery.spec.ts)、[speech-playback.spec.ts](../../../apps/desktop/e2e/speech-playback.spec.ts)、[audio-migration.spec.ts](../../../apps/desktop/e2e/audio-migration.spec.ts) |
| UI-B | [preflight-24.spec.ts](../../../apps/desktop/e2e/preflight-24.spec.ts)、[notebook.spec.ts](../../../apps/desktop/e2e/notebook.spec.ts)：本页选择、提交快照、导航、部分失败及编辑保护 |
| FS | [speech_cache.rs](../../../apps/desktop/src-tauri/tests/speech_cache.rs)；[缓存结果](../../../target/issue-24-continuation/accept-cache-tests.log)：10 passed，含GC占用失败后释放重试；显式子进程crash fixture另行执行 |
| RUN | [后端结果](../../../target/issue-24-continuation/accept-final-rust-tests.log)、[完整浏览器结果](../../../target/issue-24-continuation/complete-e2e.log)、[新增界面边界](../../../target/issue-24-continuation/accept-ui-boundaries.log)、[lint](../../../target/issue-24-continuation/accept-final-lint.log)；命令为预编译后 `pnpm test:rust`、`pnpm test:e2e --workers=2` 及日志所列专项 |
| PRODUCT | [native-product-24.mjs](../../../apps/desktop/e2e/native-product-24.mjs)／[结果](../../../target/issue-24/accept-product-2/results.json)：六词真实CPU生成、完整非静音解码、实际播放、批量并发交互、重启 |
| SELECT | [native-selected-audio-27.mjs](../../../apps/desktop/e2e/native-selected-audio-27.mjs)／[结果](../../../target/issue-24/complete-native-selected-audio-27/results.json)：偏好不替换、normal／0.6实播、原女声修复、上一版及进度保留 |
| RECOVER | [native-recovery-24.mjs](../../../apps/desktop/e2e/native-recovery-24.mjs)／[结果](../../../target/issue-24/complete-native-recovery-24/results.json)；[native-selection-recovery-27.mjs](../../../apps/desktop/e2e/native-selection-recovery-27.mjs)／[结果](../../../target/issue-24/complete-native-selection-recovery-27/results.json)：旧库／旧字节、暂停、205个缺失意图之后的有效发布、重复重启 |
| DEMAND | [native-demand-24.mjs](../../../apps/desktop/e2e/native-demand-24.mjs)／[结果](../../../target/issue-24/complete-native-demand-24/results.json)：有界自动需求、去重及刷新不重置attempt／deadline |
| LIFE | [native-lifecycle-24.mjs](../../../apps/desktop/e2e/native-lifecycle-24.mjs)／[结果](../../../target/issue-24/complete-native-lifecycle-24/results.json)：301份过期内容、200MiB软目标、当前／上一版／未知文件及实际分配 |
| RETRY | [native-transient-window-24.mjs](../../../apps/desktop/e2e/native-transient-window-24.mjs)／[结果](../../../target/issue-24/complete-native-transient-window-24/results.json)；[坏legacy恢复](../../../target/issue-24/verified-native-corrupt-retry-24/results.json)、[积压错误](../../../target/issue-24/verified-native-single-errors-24/results.json)、[缺失ready重试](../../../target/issue-24/verified-native-ready-24/results.json) |
| CRASH | [native-crash-24.mjs](../../../apps/desktop/e2e/native-crash-24.mjs)：[intent](../../../target/issue-24/crash-intent-committed/results.json)、[tmp写入](../../../target/issue-24/crash-tmp_written/results.json)、[tmp同步](../../../target/issue-24/crash-tmp_synced/results.json)、[rename](../../../target/issue-24/crash-renamed/results.json)、[readback](../../../target/issue-24/crash-readback/results.json)、[DB完成](../../../target/issue-24/crash-db_complete/results.json)、[GC标记](../../../target/issue-24/crash-gc_marked/results.json)、[GC删除](../../../target/issue-24/crash-gc_deleted/results.json)、[GC完成](../../../target/issue-24/crash-gc_finished/results.json)；真实子进程kill及两次重启 |
| ORACLE | [native-oracle-24.mjs](../../../apps/desktop/e2e/native-oracle-24.mjs)／[Vancouver结果](../../../target/issue-24/accept-oracle-vancouver/results.json)：40词混合状态，22个原始JSON参考比较；识义队列、评分／撤销幂等、额度、暂停、跳过及重启 |
| PERF | [健康10k](../../../target/issue-24/accept-window-healthy/results.json)、[全坏10k](../../../target/issue-24/accept-corrupt-combined.json)、[100词](../../../target/issue-24/measured-window-100-healthy/results.json)、[1k词](../../../target/issue-24/measured-window-1000-healthy/results.json)、[50%缺失](../../../target/issue-24/measured-window-10000-missing-half/results.json)、[100%缺失](../../../target/issue-24/measured-window-10000-missing-all/results.json)、[50k元数据](../../../target/issue-24/measured-window-50000-windowonly/results.json)、[评分DOM计时](../../../target/issue-24/verified-ui-latency-6/results.json)、[15次空库启动](../../../target/issue-24/complete-startup/results.json) |
| PRE | [V01–V06结果与冻结预算](preflight-results.md)、[模型比较原始结果](preflight-summary.json)；[桌面场景映射](desktop-results.md) |
| AUDIO | [native-audio-boundaries-24.mjs](../../../apps/desktop/e2e/native-audio-boundaries-24.mjs)／[零额度](../../../target/issue-24/accept-audio-quota/results.json)、[header-only](../../../target/issue-24/accept-audio-header/results.json)、[字节上限](../../../target/issue-24/accept-audio-bytes/results.json)、[实际codec](../../../target/issue-24/accept-audio-codec/results.json)：最终二进制，双日内卡只probe一次；坏音频解码失败且无评分；10MB文件及32MiB累计预算；90,494字节／16秒音频完整解码实播 |
| AUDIO-ORACLE | [native-oracle-audio-24.mjs](../../../apps/desktop/e2e/native-oracle-audio-24.mjs)／[结果](../../../target/issue-24/accept-oracle-audio/results.json)：8词16卡、原始JSON与实体音频独立参考，7项比较通过；真实替换发布与有限非静音Opus解码、外部连接暂停／额度失效和旧评分拒绝、评分／撤销、重启后缺失／长度错误修复收敛 |
| RESUME | [自动队列专项结果](../../../target/issue-24-continuation/accept-resume-3.log)：12项通过，包括持续事件最大等待、隐藏两学习日后合并逾期唤醒并重新核验时区；使用受控时钟与可见性 |
| RESOURCE | [native-resources-24.mjs](../../../apps/desktop/e2e/native-resources-24.mjs)／[结果](../../../target/issue-24/accept-resources-4/results.json)：50k词／100k卡／100k历史，57.25秒内100次真实评分，逐次仅加载1行metadata／解析1项progress、0重建；内存／句柄／监听器及重启取样，owned日志无fixture词头／笔记／URL |
| TIMEOUT | [native-timeouts-24.mjs](../../../apps/desktop/e2e/native-timeouts-24.mjs)／[启动](../../../target/issue-24/accept-timeout-startup/results.json)、[执行](../../../target/issue-24/accept-timeout-runtime/results.json)：挂起实际owned worker，启动30秒超时终止；执行按持久4秒deadline由attempt 3→4进入failed，新请求恢复，重启不重置旧预算，无遗留worker |

最终产品二进制为 `accepted-binary.exe`，SHA-256 `61aae370b2e04fd0efec69fdafe142bf7ee99d918036857b4e195236493111e4`，RESOURCE／TIMEOUT、PRODUCT、最终健康／全坏10k及两组oracle使用该二进制。SELECT、RECOVER、DEMAND、LIFE及早期音频边界使用 `complete-binary.exe`（`33a7e17521fc89457826e42659f839681c452026e8f1203090192dfc947463d1`）。CRASH与部分独立性能／错误分片使用 `finish-binary.exe`（`564c4a907ee3bc44987ae43b9b7ed7a282ae89d265527e50e7345308bff7a712`）。每个原生入口硬限60秒，编译单独执行。详细命令、环境和数字见[验证报告](continuation-2026-11-04.md)。

## 50个专项用例

| ID | 状态 | 已执行的断言／对应证据 | 尚缺的具体断言或条件 |
| --- | --- | --- | --- |
| T24-Q01 | 通过 | DB-R `batch_progress_preserves_order_duplicates_defaults_and_errors`；DB-Q；ORACLE混合新卡；PERF空库启动 | — |
| T24-Q02 | 通过 | DB-R实际类别混排及重开；ORACLE FNV／完整顺序；AUDIO-ORACLE双卡类别混排、准入计数／四卡窗口及音频故障收敛；UI-Q独立摘要、未播放禁评分 | — |
| T24-Q03 | 通过 | DB-R `anki_*`、每日新卡计数、失败提交；ORACLE额度与忽略开关 | — |
| T24-Q04 | 通过 | DB-R `zero_limits_preserve_both_due_intraday_cards`；DB-W本日需求；AUDIO零额度100词只准入两张日内卡、单次probe | — |
| T24-Q05 | 通过 | DB-R `raising_review_limit_interleaves_newly_admitted_cards`、限额保序与持久化；ORACLE改限额／重启 | — |
| T24-Q06 | 通过 | DB-R分钟优先、旅行及撤销；核心学习日／DST／时钟倒退评分拒绝向量；Vancouver ORACLE；RESUME两学习日后逾期唤醒与动态换区 | — |
| T24-Q07 | 通过 | DB-R双卡独立、暂停／恢复、重复加入；SELECT重启后卡片和评分进度完全相同 | — |
| T24-Q08 | 通过 | 最终全坏10k：100次响应0失败，strict≤8、缓存≤32MiB；UI-A修复中／失败显示；全坏p95=885.0ms，数据库／修复争用计入总延迟 | — |
| T24-Q09 | 通过 | DB-R音频过滤先于额度；UI-Q失败跳过不评分；UI-A定向修复；RETRY共享锁恢复 | — |
| T24-Q10 | 通过 | DB-R `warm_queue_reuses_the_loaded_positions`、writer占用时稳定读取、首次50卡零位置UPDATE；DB-Q自身位置不重建；健康10k热响应零probe／payload／重载／解析 | — |
| T24-A01 | 通过 | FS nofollow／固定目录／完整读取及并发发布；PRODUCT／SELECT／AUDIO解码并播放窗口返回的同一字节 | — |
| T24-A02 | 通过 | FS目录／重解析／并发发布；SELECT错内容；AUDIO超10MB重启拒绝；AUDIO-ORACLE实际文件长度错误后拒用并修复；完整读取所得同一buffer校验hash后交解码，变化字节不能取得校验许可 | — |
| T24-A03 | 通过 | UI-A播放失败／取消／重开／重播在途禁评分；UI-Q progress revision失效；AUDIO实际header-only解码失败、评分不可用且rating表为零 | — |
| T24-A04 | 通过 | UI-A离卡后晚到替换不播放、重开取消旧autoplay；UI-Q revision delta、乱序／不匹配错误 | — |
| T24-A05 | 通过 | PRODUCT六份有限非静音完整解码和实际播放；SELECT两种速度；AUDIO 16秒／90,494字节实际codec及header-only禁评分 | — |
| T24-A06 | 通过 | SELECT偏好变化／原规格修复；DB-A同hash新物理身份、迟到GC、失败选择与代次守卫 | — |
| T24-A07 | 通过 | AUDIO八份各10,000,000字节的文件压力，缓存／payload保持≤32MiB，超限文件重启拒绝；90,494字节真实codec另验；UI-A取消旧播放／释放请求 | — |
| T24-A08 | 通过 | UI-A正面隐藏词头、翻面不重播、失败重播恢复；SELECT normal／0.6同内容实播 | — |
| T24-C01 | 通过 | UI-Q单查询、末尾刷新、1000事件、最大等待、隐藏／空闲零读取、当前作答保留；DB评分资格 | — |
| T24-C02 | 通过 | DB-Q已提交修订、no-op／回滚、两连接一致快照、连续音频修订；DB-R文件检查不持DB快照与评分／撤销失败回滚 | — |
| T24-C03 | 不适用 | 当前使用原始progress JSON及修订缓存，DB-Q／ORACLE覆盖该路径 | 未启用独立归一化进度投影；同步写点属于M6 |
| T24-C04 | 通过 | DB-R过期评分／撤销与幂等重试；UI-Q过期played presentation拒绝；ORACLE评分／撤销重试；AUDIO-ORACLE独立连接暂停／改额度使热缓存失效，暂停后旧评分拒绝 | — |
| T24-C05 | 通过 | DB-R `queue_reindexes_positions_near_the_integer_limit`及 `exhausted_position_gap_rolls_back_and_preserves_undo_after_reopening`：整数边界、间隙耗尽、过滤锚点、重编提交失败回滚、重开后幂等撤销保序 | — |
| T24-C06 | 不适用 | 当前每请求连接＋短快照；DB-Q真实WAL并发写入通过 | 未启用WAL小读池；该条件门槛在启用时适用 |
| T24-W01 | 通过 | DB-W满积压保留双卡、交互保留容量；PRODUCT单常驻进程；积压错误native及正常退出无owned worker | — |
| T24-W02 | 通过 | DB-W四次／退避／deadline、恢复／刷新不重置；DEMAND刷新保持预算；TIMEOUT真实启动／执行超时终止owned worker，attempt 4终态、新需求恢复与重启预算保持 | — |
| T24-W03 | 通过 | DB-A连续替换、执行后继、取消后共享订阅及operation幂等；DB-W取消不删卡；SELECT进度保留 | — |
| T24-W04 | 通过 | DB-A不可变实例／hash分离、同spec不同输出；DB-W原spec键与不支持规格可见错误 | — |
| T24-W05 | 通过 | DB-W `background_fairness_preserves_explicit_batches_and_the_interaction_slot`、本日只读规划、未来排除；DEMAND有界持久需求 | — |
| T24-W06 | 通过 | CRASH六个发布断点；RECOVER缺失意图后有效发布与重复重启；DB-A cancelled／failed／旧generation不能切换 | — |
| T24-G01 | 通过 | DB-A删卡不解除生词归属／当前永久；DB-W无卡生词保护；LIFE压力下保留当前内容及业务状态 | — |
| T24-G02 | 通过 | DB-A成功切换十四天／多次替换；SELECT真实上一版及修订；LIFE上一版文件保留 | — |
| T24-G03 | 通过 | DB-A最后归属七天及到期边界；DB-W临时完成起算；native LRU／LIFE回收不改业务数据 | — |
| T24-G04 | 通过 | DB-A marked不复活、mark generation及迟到删除；RECOVER旧gc_pending；CRASH三GC断点；FS占用／并发发布 | — |
| T24-G05 | 通过 | DB-A共享归属／最后解除／创建词删除后仍可绑定其它词、重复事件不延长期限 | — |
| T24-G06 | 通过 | DB-A软容量与保护；LIFE真实超过200MiB、301过期文件、当前／上一版／未知保留，allocated／logical分列 | — |
| T24-G07 | 通过 | FS真实Windows占用使GC删除返回错误，`fail_audio_gc`登记诊断、保留marked及原文件，释放后删除／finish成功；CRASH GC恢复；LIFE未知文件保留；正常退出worker清理 | — |
| T24-L01 | 通过 | DB-A recent-first有界兼容登记、迁移保护；RECOVER v10→17保留旧字节、双卡、暂停与笔记 | — |
| T24-L02 | 通过 | RECOVER可用legacy不启动模型、不换字节；坏legacy native恢复后其它preview仍成功 | — |
| T24-L03 | 通过 | DB-W未知规格修复例外；坏legacy native可重试；UI-Q `unknown legacy repair discloses the voice change in each language` 实际核验中／英／法提示可见 | — |
| T24-L04 | 通过 | DB-W `automatic_repair_keeps_known_voice_and_reports_unsupported_specs`；SELECT偏好改变后仍按female修复 | — |
| T24-L05 | 通过 | DB-A v11／v12／v14迁移、FK完整性、回滚；storage测试更高schema拒绝；RECOVER重开 | — |
| T24-P01 | 通过 | PERF 100／1k／10k；最终健康10k p95=92.7ms；首次响应另存checkpoint，15次空库启动另列 | — |
| T24-P02 | 通过 | DOM评分端到端p95=11.6ms可作post-return切卡耗时上界；RESOURCE连续100次普通评分，每次1行metadata／1项progress、0重建；50k评分IPC p95=220.3ms，队列IPC另报 | — |
| T24-P03 | 通过 | PERF 0／50／100%缺失与全坏，每次窗口／probe／缓存预算、失败次数独立记录；DB-Q四种readiness下100／1k词热快照SQL条数不变且不重载；RESOURCE连续评分有界增量 | — |
| T24-P04 | 通过 | RESOURCE 50k词／100k卡／100k历史、100次评分；队列p95=228.9ms／p99=239.6ms；句柄366→372→371、重启366，监听器185→183，各取样0worker；工作集33.3→63.5MB、重启32.9MB，私有提交10.4→41.5MB、重启10.2MB；RESUME受控跨日恢复 | — |
| T24-S01 | 通过 | FS固定portable目录／拒绝重解析；LIFE只删登记文件、未知保留；RESOURCE逐份读取owned日志，无fixture词头／笔记／来源URL；原生错误保留分类诊断 | — |
| T24-S02 | 通过 | PRODUCT／SELECT已有固定binary真实生成、解码、播放与重启；ProcessFailed日志接入；用户2026-11-05确认干净系统离线测试通过 | 用户报告整体通过，未提供测试机版本或逐项日志；RTSS兼容性按既定范围排除 |
| T24-F01 | 后续 | Windows ARM64、macOS x64／ARM64既定平台计划 | 路径／编码／解码／安全打开及离线须各平台实机验收 |
| T24-F02 | 后续 | M6网页端及codec／系统准入记录 | 存储、配额、多标签、WASM复习集成、浏览器实际播放另验 |

## 原T01–T12场景与V01–V06准入

| ID | 状态 | 实际覆盖与具体边界 |
| --- | --- | --- |
| T01 | 通过 | UI-Q空／非空、单查询、1000事件、末尾事件、持续流、隐藏／恢复及无idle轮询；UI-B／PRODUCT真实加入与批量；vocabulary.rs单个／批量／自动加入、暂停／评分／撤销／设置与发布沿同一changed事件，接入UI-Q已验证协调器 |
| T02 | 通过 | DB-R／核心时间向量／UI-Q绝对due和04:00／Vancouver ORACLE；RESUME隐藏两学习日后合并为一次读取、重核验动态时区且随后不重复唤醒 |
| T03 | 通过 | UI-Q当前作答／progress revision／丢响应identity；DB-R撤销保留后续编辑；ORACLE幂等与重启 |
| T04 | 通过 | UI-B默认20／50页、末页53词中的51–53固定提交、排序／搜索／翻页清选择；晚到搜索响应不恢复旧页或旧选择；非法ID与积压由DB-W／native覆盖 |
| T05 | 通过 | UI-B提交前不报成功、丢响应原快照重试、两阶段计数；DB-W savepoint／commit失败及混合新旧暂停；PRODUCT识义卡不等音频 |
| T06 | 通过 | PRODUCT真实生成期间查词／编辑／评分／P0缺失请求并记录逐项时间；SELECT缓存实播；DB-W容量／同键合并 |
| T07 | 通过 | UI-Q卸载／remount及UI-B未保存保护；DB-W取消／预算；PRODUCT退出／重启及CRASH发布恢复；TIMEOUT真实启动／执行超时终止worker，新请求恢复与重启预算保持 |
| T08 | 通过 | PRODUCT临时转生词复用；DB-A／DB-W归属、到期拒用、共享／解除与GC；SELECT normal／0.6 |
| T09 | 通过 | FS、DB-A、RECOVER、CRASH、RETRY、AUDIO覆盖迁移／权限／链接／共享／发布、I/O失败保留旧内容、同字节校验及解码失败禁评分；物理故障边界见限制 |
| T10 | 通过 | DB-W近期窗口／七天候选规划、本日额度／未来排除与只读需求；UI-A旧请求不串卡、例句完整性；真实例句按F01留待M3 |
| T11 | 通过 | PRE本地／受控协议分类、DB-W终态精确预算和定向重试、native积压／坏legacy恢复；当前仅本地CPU实现，未接入在线请求或竞速路径；实际transport属F02 |
| T12 | 通过 | DB-W 100／1k／10k分块；PERF多规模／故障；RESOURCE混合100k历史、100次评分逐批采样及重启，报告内存／句柄／worker和p95／p99 |
| V01 | 通过 | PRE改前阻塞／导航时序、固定输入及原生基线；PRODUCT当前交互结果分别记录 |
| V02 | 通过 | PRE冻结CPU／线程／槽位／资源预算；36轮432份真实音频完整解码；后台0／1／2／4及P0比较 |
| V03 | 通过 | PRE真实SQLite 20／50／100短事务、savepoint、竞争与commit失败；DB-W当前产品100词分块／幂等 |
| V04 | 通过 | PRE UTC／语义规格；DB-A期限、版本、归属、迁移和GC；SELECT／LIFE／RECOVER真实文件 |
| V05 | 通过 | PRE受控协议／预算；DB-W身份、优先级、取消、后继执行；CRASH九断点及重复恢复；TIMEOUT真实超时／终止／恢复与重启预算保持 |
| V06 | 后续 | F01：M3真实例句；F02：M4在线认证／翻译／TTS；F03／F04：M6网页端／浏览器缓存；F05：M4／M6可选音频同步 |

## AC01–AC14

| ID | 状态 | 判定与证据 |
| --- | --- | --- |
| AC01 | 通过 | PRE方案／预算、PERF多规模与故障分片、隔离fixture、硬件／缓存状态和可复现入口齐备；报告区分实测与未测范围 |
| AC02 | 通过 | UI-Q自动刷新／定向重试、UI-A／UI-B及三语legacy提示通过；自动触发的接入及协调器覆盖见T01 |
| AC03 | 通过 | DB-R、UI-Q、SELECT及ORACLE覆盖双卡、独立排程、额度、暂停、跳过、持久顺序、撤销及幂等 |
| AC04 | 通过 | UI-Q单查询、合并／最大等待／持续流、空闲零读取、隐藏暂停与remount释放通过 |
| AC05 | 通过 | DB-W短事务／两阶段／失败／取消／容量，FS安全目录／发布，PRODUCT真实交互及同字节解码／播放 |
| AC06 | 通过 | 本表逐项记录八类稳定性相关场景和50专项的源码、命令入口、断言／结果／缺口；DB、FS、播放与原生证据分别列出，后续平台单列 |
| AC07 | 通过 | 健康10k p95=92.7ms、评分DOM p95=11.6ms；全坏p95=885.0ms且0失败；RESOURCE 50k混合历史下报告100次评分、资源增长／重启与p95／p99，实测范围见限制 |
| AC08 | 通过 | RUN、缓存／库测试、锁定Rust构建、格式、lint／typecheck／build、100项完整浏览器、138后端、原生和实际WASM校准；命令与验证结论见验证报告 |
| AC09 | 通过 | PRE V01–V04、V05本地／发布断点／真实超时通过；12红回归映射本表；ORACLE及AUDIO-ORACLE独立参考通过 |
| AC10 | 通过 | UI-B、DB-W、PRODUCT／SELECT／RECOVER、TIMEOUT覆盖T04–T09本页选择、提交快照、幂等双卡、应用任务、容量、取消／重启及真实失败 |
| AC11 | 通过 | DB-A／DB-W精确期限与共享、SELECT选定版本、LIFE软容量与分配、RECOVER非破坏迁移、CRASH GC边界 |
| AC12 | 通过 | 本地分类失败／精确重试／预算由PRE、DB-W、RETRY、TIMEOUT覆盖；忙碌与终态分别处理，实际超时终止owned worker；当前无在线请求实现，F02后续 |
| AC13 | 通过 | 本期需求筛选／有界规划／旧请求隔离由DB-W、UI-A覆盖；真实例句与F01–F05按V06里程碑后续 |
| AC14 | 通过 | R01–R10保留映射见[规格复核](github-design-review.md)，T01–T12／V01–V06及50专项在本表有明确状态；Windows验收与未来里程碑／正式发行边界分别记录 |

## 限制

干净系统离线通过依据用户报告，未提供测试机版本或逐项日志。原生产品附加检查记录一次WebView2进程访问异常（ProcessFailed kind=0，exit=-1073741819）；崩溃模块为RTSSHooks64.dll＋0x1490af，与已确认的RTSS兼容性排除范围一致。资源结果覆盖本机57.25秒／100次评分与重启：工作集及私有提交在预热后增长，重启回到初始水平；未取得小时级趋势、物理休眠恢复、其它硬件／平台或物理冷盘结果。受控跨两学习日／时区恢复已通过。未诱发操作系统磁盘满或物理掉电；已验证I/O错误传播、旧内容保留与发布／清理进程kill恢复。50k资源fixture共享一份选定音频，完整codec与单文件／累计字节压力另由PRODUCT／AUDIO证明。M3／M4／M6能力按既定后续范围交付。
