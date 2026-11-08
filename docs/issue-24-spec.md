# #24 复习队列、选定单词音频与安全清理规格

2026-11-08 后续播放偏好改为[80–240 WPM](speech-preferences.md)，步长 20、默认 180，尚未接入；换算不改变选定内容、版本与保护关系。本 #24 的历史规格和验收保持原播放条件，新设置按[独立切片](lookup-notes-speech-plan.md)实施与回归。

当前实现与验收结果见[#24实施记录](evidence/issue-24/continuation-2026-11-04.md)和[逐项覆盖](evidence/issue-24/acceptance-matrix-2026-11-05.md)；用户于2026-11-05确认干净系统离线测试通过，#24本期验收完成。下文保留规格／发布阶段的计划与状态记录。

设计确认：2026-11-03；架构核对：2026-11-04（America/Vancouver）。依据：Q1–Q10 已确认设计及原 #24 的 R01–R10、V01–V06、T01–T12、自动触发表、稳定性矩阵和 F01–F05。本文规定完整验收目标；部分实现已进入工作区，进度见文末及[架构核对](evidence/issue-24/architecture-reconciliation-2026-11-04.md)。已有实现、历史测量与设计确认不能代替全工单验收。本轮产品验收目标为 Windows x64。

## Problem Statement

学习者应能直接开始或继续复习，不必手动刷新队列，也不应在批量建卡、合成或清理时失去查词、播放、评分和编辑能力。S0 冻结基线包含逐项 progress／规格／资产查询和全库文件检查。当前工作区已改成批量读取及短事务，仍检查全部符合条件的听音文件；有界窗口、持久修订及真正增量补位尚未完成。增加自动触发仍会放大剩余全量成本，线程数不能替代查询／校验范围优化。

S0 的规格资产模型缺少每词明确的当前内容版本。schema 12 已增加不可变实例、选定当前／上一版和恢复登记；执行中的新意图仍有回归缺口，完整共享归属／版本 GC 及容量闭环尚未完成。相同规格的非确定输出必须与规格键分开；旧缓存清理与新版本模型需要统一保护，避免误删、无限保留历史或把未完成的新需求报告为成功。

原工单的已提交批量任务、当前作答保护、独立排程、额度、撤销和错误可见性继续保留。需要改变的是查询与校验范围、明确的音频选择及保留规则，而不是复习业务含义。

## Solution

队列先从一致的短数据库快照批量读取候选、设置、额度、当天位置及选定音频元数据，再对近期使用窗口安全读取完整音频并校验。播放器解码与播放同一份已校验字节；听音卡本次成功播放之前不能评分。窗口外数据库就绪状态只代表候选，界面分别显示待生成、待校验、处理中、等待到期、额度用尽与真实失败。

每份生词记录至多选定一个当前单词音频版本。全局声线偏好影响后续生成，现有选定内容只有明确更换或符合修复规则时才变化；失败保留旧当前版本及学习数据。当前音频随生词永久保留，每词最多一份上一版，成功替换后保护 14 天；最后生词归属解除后保护 7 天；临时查询音频及未来例句从生成完成起 7 天。读取、播放、下载与恢复不延长绝对有效期。无归属音频采用 200 MiB 软目标，不能为达标破坏当前绑定或有效保护期。

自动更新、两阶段建卡和应用拥有的持久语音任务继续工作。近期与本日缺失音频优先，用户已提交的远期批量需求仍继续完成；不因日常刷新创建全库新任务。改动按可独立验证的批次实施，先补基线与回归、消除重复查询，再接版本迁移、窗口校验及生命周期闭环，性能参数由同期实测决定。

## User Stories

1. As a learner, I want the review queue to update when I enter the page, so that I can start without a refresh button.
2. As a learner, I want the queue to update after returning from the background or sleep, so that I see current eligibility.
3. As a learner, I want ratings, undo, skips, enrollment and pause changes to update the queue after success, so that it reflects committed actions.
4. As a learner, I want audio publication and relevant cleanup to update availability, so that newly usable cards can appear.
5. As a learner, I want due-time and learning-day changes to wake the queue automatically, so that waiting does not require polling.
6. As a learner, I want an idle or hidden review page to avoid repeated database and audio scans, so that it wastes no additional work.
7. As a learner, I want duplicate events to be merged, so that batch activity does not overwhelm the application.
8. As a learner, I want targeted retry after a queue failure, so that the actual failure remains understandable and recoverable.
9. As a learner, I want a bounded recent card window, so that large collections do not require loading every audio file before review.
10. As a learner, I want unverified candidates to be identified as pending verification, so that database readiness is not presented as playable audio.
11. As a learner, I want a missing or damaged undisplayed listening card to be repaired or bypassed safely, so that other eligible cards can be reached.
12. As a learner, I want an all-damaged collection to return bounded progress or failure, so that it does not freeze while scanning the entire library.
13. As a learner, I want playback to use the exact verified bytes, so that a later file replacement cannot change what is played.
14. As a learner, I want listening ratings blocked until this presentation has played successfully, so that failed playback cannot consume progress.
15. As a learner, I want my current question, answer and flipped state preserved during background updates, so that I do not lose an ongoing response.
16. As a learner, I want a valid already-played presentation to remain usable during unrelated publication, so that background work does not interrupt my rating.
17. As a learner, I want real pause, time, quota or progress conflicts checked when rating, so that a stale queue cannot authorize an invalid action.
18. As a learner, I want recognition and listening cards to retain separate identities and schedules, so that one card does not defer the other.
19. As a learner, I want new cards without progress or a due time to remain discoverable, so that first review is not lost during query optimization.
20. As a learner, I want due within-day learning cards available even when both daily limits are zero, so that learning repetitions remain independent of quotas.
21. As a learner, I want existing cross-day priority, quota accounting, mixed order and saved positions preserved, so that optimization does not change review rules.
22. As a learner, I want undo and lost-response retries to retain their original operation identity, so that progress and quota are not applied twice.
23. As a learner, I want current skips and pauses preserved across refreshes, so that background updates do not undo my choices.
24. As a learner, I want local 04:00 learning days and absolute due times respected, so that travel or daylight saving does not rewrite history.
25. As a learner, I want to select only the current notebook page, so that batch enrollment has a clear scope.
26. As a learner, I want unsubmitted selection cleared after changing page, search, filter, sort or page size, so that stale selections are not submitted.
27. As a learner, I want submitted IDs and operation identity fixed, so that later navigation cannot change an in-progress request.
28. As a learner, I want committed cards reported before audio generation finishes, so that recognition review can start immediately.
29. As a learner, I want repeated enrollment to preserve notes, card identities, progress and intentional pause, so that retry is safe.
30. As a learner, I want per-word and transaction failures distinguished, so that uncommitted work is not counted as success.
31. As a learner, I want queued audio tasks to continue after navigation and recover after restart, so that page lifetime does not own batch work.
32. As a learner, I want to stop remaining generation without deleting cards or completed audio, so that cancellation preserves completed work.
33. As a learner, I want lookup, editing, rating and cached playback usable during batch generation, so that background work does not block learning.
34. As a learner, I want an interaction slot and deduplicated requests, so that a click does not wait for the entire batch or start duplicate synthesis.
35. As a learner, I want audio generation to use CPU with bounded concurrency, so that resource use follows the confirmed platform choice.
36. As a learner, I want current and today's missing audio prioritized, so that background work prepares what I can use next.
37. As a learner, I want explicitly submitted distant batch demands to finish fairly, so that priority changes do not silently abandon my request.
38. As a learner, I want no new future-library demand created by every refresh, so that automatic review does not trigger unlimited synthesis.
39. As a learner, I want one selected audio version for each saved word, so that its familiar pronunciation stays stable.
40. As a learner, I want preference changes to affect later generation without replacing saved audio automatically, so that an existing learning cue stays intact.
41. As a learner, I want explicit replacement to switch only after verified publication and database commit, so that a failed change preserves old audio.
42. As a learner, I want a late old request unable to replace a newer selection, so that my latest intention wins.
43. As a learner, I want normal and slow playback to reuse the same content, so that playback speed creates no duplicate version.
44. As a learner, I want valid temporary query audio reused when saving a word, so that ownership can become durable without needless regeneration.
45. As a learner, I want deleting cards to keep a saved word's selected audio, so that audio ownership remains separate from review membership.
46. As a learner, I want shared selected versions protected while any word still refers to them, so that one removed reference cannot delete another word's audio.
47. As a learner, I want one previous version protected for 14 days after successful replacement, so that the old content remains recoverable for a limited time.
48. As a learner, I want the last detached word version protected for seven days, so that future deletion can retain a recovery window.
49. As a learner, I want temporary query audio to expire seven days after generation completes, so that repeated use does not make it permanent.
50. As a learner, I want expired temporary audio rejected even before physical cleanup runs, so that expiry is independent of maintenance timing.
51. As a learner, I want capacity cleanup to respect permanent ownership and protection windows, so that disk targets cannot delete protected content.
52. As a learner, I want logical, allocated, permanent, protected and reclaimable sizes distinguished, so that reported usage is meaningful.
53. As a learner, I want unknown files preserved and reported, so that cleanup does not guess ownership from a name.
54. As a learner, I want cleanup failures visible and retryable, so that permission or disk errors are not reported as success.
55. As a learner, I want old-format usable audio retained during upgrade, so that migration does not change its sound or destroy my data.
56. As a learner, I want old-format registration to run once outside the hot queue path, so that ordinary review does not repeat migration writes.
57. As a learner, I want damaged audio with a known specification repaired using that specification, so that repair does not silently adopt a new voice or model.
58. As a learner, I want unknown legacy repair to disclose possible differences and use current preferences only under that exception, so that its origin is represented honestly.
59. As a learner, I want unsupported known specifications reported explicitly, so that unavailable capability cannot be simulated.
60. As a learner, I want publication and cleanup interrupted by a crash to recover consistently, so that files and selections cannot disagree silently.
61. As a learner, I want retries and recovery to preserve attempt budgets and cancellation, so that restart does not cause infinite automatic work.
62. As a learner, I want missing or damaged content distinguished from permissions, unsafe paths and codec limitations, so that repair addresses the real cause.
63. As a learner, I want no text sent to an unconfigured or unapproved online service, so that local failure does not trigger unexpected external requests.
64. As a learner, I want sentence preparation bounded by actual review demand, so that future examples do not pre-generate an entire seven-day horizon.
65. As a learner, I want prefetch to preserve scheduling, quota, saved order and playback status, so that preparation cannot count as review.
66. As a learner, I want application shutdown to preserve unsaved-edit protection and stop its owned workers, so that exiting loses no confirmed work and leaves no orphan process.
67. As a learner, I want portable movement and restart to preserve my data and verify audio safely, so that moving the application does not break learning.
68. As a learner, I want large-collection and concurrent performance measured on real boundaries, so that reported gains reflect the application I use.
69. As a learner, I want cold startup, warm lookup, synthesis, decoding and rating latency reported separately, so that one fast number cannot hide a slow interaction.
70. As a learner, I want incomplete platform and future integration checks stated explicitly, so that Windows optimization is not confused with full release readiness.

## Implementation Decisions

### 已确认变更及保留的范围

| 主题 | 本规格取代的旧边界 | 新的权威行为 |
| --- | --- | --- |
| 窗口校验，Q3／ADR 0007 | 每次队列刷新完整检查全库未暂停听音文件 | 数据库候选与近期严格窗口分离；窗口外 readiness 可过时，不能作为播放许可。 |
| 选定版本，Q2／Q5／ADR 0008 | 按最近可读规格／输出隐式选择，归属保护全部规格历史内容 | 每词至多一个选定当前内容；偏好只影响以后生成，替换以最新有效意图及真实完成提交为准。 |
| 保留，Q7／Q8 | 旧“生成语音全部临时”或访问续七天，以及未细化的永久规格保护 | 选定当前永久、每词上一版 14 天、最后归属解除 7 天、临时完成起 7 天；无归属 200 MiB 软目标，保护优先。 |
| 自动发现，Q6 | 日常刷新发现全库单词缺失并生成，或将七天例句候选全部合成 | 优先近期及本日按额度可出卡需求；远期只承接已明确提交的批量需求。 |
| 旧格式，Q4／Q9 | 删除 legacy 支持或在日常队列读取中反复登记 | 保留非破坏兼容、一次后台登记；未知坏 legacy 的修复例外须提示，已知规格严格沿用。 |

原工单的本页选择、短事务两阶段建卡、自动触发、应用任务生命周期、CPU、重试、V/T/F 与稳定性要求继续有效。候选合成槽数、页面容量、事务规模与延迟目标不因本规格成为无条件性能承诺。尚未接入的删词、恢复上一版等产品入口不在本期擅自添加；保留关系与事务协议必须可测试，未来入口必须沿同一协议。

### 业务、查询及持久顺序

1. 复用共享 FSRS 核心、注入 SQLite 的存储边界、桌面安全文件与合成适配及 RuntimeAdapter。纯核心不引入文件、SQLite、Tauri 或网络。现有识义／听音卡独立排程、同词不暂缓、暂停、会话跳过、一级撤销及幂等操作不变。
2. 从一个短读快照批量获取设置、已用额度、卡片集合、原始进度、当天位置和选定版本；复用批量连接及位置 map，消除逐卡 progress、逐词 key 和逐资产查询。缺 progress 行及空 due 的新卡必须包含，不能用内连接或无条件到期过滤丢弃。
3. 保留跨日学习优先、长期复习剩余额度、新卡忽略开关及日内学习不限额。默认新 20、复习 50；同词两卡分别计数，只在成功评分计额度，撤销返还原影响。两额度零仍可出到期日内听音卡。
4. 长期准入沿用当前时区的到期学习日、既有 FNV 与卡片身份；混排采用过滤及额度裁剪后的实际类别数量。当天位置只代表当天曾准入记录，暂时过滤不丢锚点。分页必须保留完整类别／排序键、唯一身份、空值分支和快照修订，不能先任意截断再混排。
5. 日切、首次缺位及必要重编号用独立短写事务；条件确认快照修订后提交。稳定热刷新不写位置、不登记 legacy、不获取无必要立即写事务。重编号保留即时唯一约束安全策略。文件 I/O、合成及解码不持用户库写事务或长读事务。
6. 影响资格、集合、设置、选择、readiness 或位置的真实业务提交递增持久队列修订；幂等 no-op 不递增。一次失效校验及其位置更新可作为一个条件 delta 提交，不以自身写回造成无限重算。竞争时保留有效不可变字节、有限重读；超预算返回真实处理中状态。

### 窗口、界面及播放

7. 服务维护 metadata 候选缓冲、已验证小窗口及摘要。卡片列表长度不代表总准入数量；分别返回准入计数、待音频计数、有界目标词样例、待校验标识、下一唤醒、学习日／时区／队列修订。待音频计数为到期且未暂停／未跳过的非就绪听音卡，不等于拥有剩余额度的数量；窗口外未知坏文件不能冒充已确定全库结果。
8. 工程起点是 metadata 50、展示窗口 4（当前加后三张）、单次最多 8 次实际文件 probe、2 校验 worker、压缩字节缓存 32 MiB、等待 75 ms、待音频样例最多 100 个。参数可按先验预算和实测调整；单文件 10,000,000 字节上限继续保留。等待上限不是取消 OS 阻塞读的保证，固定 worker、在途与后继工作必须有界。
9. 文件校验用本次安全目录能力句柄、nofollow、实际句柄身份、完整有界读取及 Ogg／长度／SHA-256 检查，拒绝链接、重解析与越界。已验证缓存按不可变内容版本／hash 绑定，播放器使用同一份字节，正常／慢速只改变播放 rate（慢速 0.6）。压缩、IPC 拷贝、PCM、Blob 分别计量；预解码启用前另定时长及 PCM 上限。
10. 未展示坏卡条件标记 suspect／missing、持久去重修复并补位；probe 或等待预算耗尽时返回“检查／修复进行中”并分块继续，不能扫描完全部坏文件才响应。识义卡不增加音频门槛。权限、危险路径、模型／codec 未准入及磁盘满不当作可反复合成解决的坏文件。
11. 当前 presentation 绑定卡片、进度修订、选定内容版本／hash、请求代次与播放结果。后台刷新不换题、不自动重播、不清空翻面；合法已播放内存版本可完成当前作答。真实暂停、时间、额度、进度或播放失效仍阻评分；无关队列修订不直接拒绝有效评分。播放回执不是密码学听觉证明，恶意受控 renderer 不在此证明范围内。
12. 所有原自动触发保留：进入／重入、窗口激活／后台返回／休眠恢复；评分／撤销／跳过／暂停恢复；已提交单个、批量及启用的收藏自动加入；相关发布／替换／重试结果及实际缓存变化；额度设置；下一分钟到期、04:00 换日及时间／时区重核验。后端判定资格，定时器不是评分许可。
13. 同一上下文一个查询在途，通知合并、有最大等待，期间新变化可尾随补查；迟到响应不复活旧状态。普通音频事件合并起点 100–250 ms，当前需要的就绪可立即处理。稳定空闲新增队列 I/O 为零；隐藏页不持续查询，恢复时重核验。卸载释放订阅、定时器和播放等待，应用任务继续；失败只提供定向重试，禁止无限自动重读。

### 合成身份、内容版本及选择

14. 合成语义身份包括完整 NFC 原文、语境、语言、实际引擎／模型及前处理版本、声线、参数和编码，用规范结构计算 SHA-256；保留大小写、重音和撇号，不使用搜索归一键替代文本。归属不入生成键；内容 hash 与规格键分开，相同规格输出不保证确定。
15. 持久模型区分完整合成规格、不可变内容发布实例、每词选定绑定及修订／目标 generation、每词上一版与期限、发布 intent、版本级 GC intent。内容校验状态（未校验／已校验／可疑／缺失／无效）与 GC 状态（live／marked／deleted）分开；任务 queued／running／retry_wait／ready／failed／cancelled 不能代替文件校验或平台解码能力。
16. 每个文件发布实例有不可复用的版本身份与受控 locator。仅可读且无 GC intent 的 live 实例允许幂等复用；旧 marked／deleted 实例不复活，即使同 key／hash 再生成也使用新实例。新版本不得与旧迟到删除共享物理身份。
17. 用户新选择意图递增 desired generation，原 operation 的幂等重试不递增。执行身份、需求身份和选择身份分开；同规格有效订阅可合并执行，取消一个需求不取消其他订阅。已取消／结束执行不能承接新意图时，后继需求持久排队并最终执行，不能只丢迟到结果却遗漏最新需求。
18. 发布协议为持久 intent、事务外安全临时写入／同步／不可变名称发布／同字节回读验证、完成事务登记版本并条件切换选择／上一版、提交后通知。rename 不等于跨文件及 DB 原子提交。实际加载资源与持久完整规格执行前匹配，不能跨升级用新引擎输出冒用旧键。
19. 完成时核对目标词、最新 generation、有效执行、未取消及期望旧选择。迟到旧结果可登记为未绑定内容但不覆盖新选择；失败／取消／DB 失败保留旧当前。成功切换才启动上一版 14 天，同版本幂等结果不创造“上一版等于当前”。
20. 已知新格式音频修复使用原完整规格，不静默采用全局新偏好；规格不支持则明确失败并要求升级。可用 legacy 原样保留；未知规格的缺失／损坏 legacy 可按当前偏好重生成，须提示原规格未知、输出可能不同，未知字段保持 unknown。

### 保留、兼容与清理

21. 任一有效生词选定绑定永久保护共享版本；删卡、暂停、归档不解除生词音频归属。临时音频收藏时，仅有效 live 内容可事务化转为选定音频；过期或 marked 内容不能靠晚收藏复活，生词提交可正常完成并登记缺失需求。
22. UTC Unix 秒存储绝对期限，七天为 604,800 秒、十四天为 1,209,600 秒，期限采用安全运算并校验溢出。临时期限从生成完成起算；上一版从新选择成功切换事务起算；孤儿从最后当前归属解除事务起算。多种有效保护取较晚期限，expires_at／protect_until 等于 now 时视为已到期。未知 legacy 原生成时间保持未知，不伪造完成时间；先建立迁移／当前保护再决定可回收候选。
23. 每词最多一份上一版；再次替换释放更早版本的该词上一版保护，其他词当前／上一版及孤儿保护仍有效。将来删词入口必须同事务解除绑定、登记最后归属宽限并删除业务记录；本期不新增删除交互。上一版恢复须明确操作，不由 GC 或“最近可用”自动切换当前。
24. 无当前归属版本逻辑占用的软目标为 209,715,200 字节；另列实际分配、当前永久、保护期、可回收及 tmp／intent 占用。先清到期／释放的多余版本，再对无保护临时内容按 LRU；仍超标仅报告。last_used 可合并记录但不改 expires_at、不在热队列只读事务写入；保护期与永久内容不为容量提前牺牲。
25. GC 每批最多 100 项，短事务核查保护／intent 并条件标记具体 version／mark generation，事务外安全删除，短事务条件完成。读占用或权限失败保留 mark／原因，缺文件可完成删除；迟到清理不能改后来新版本。发布、取得归属、旧 pending-GC 和兼容读取统一使用一个版本保护仲裁，不能保留两套删除权威。
26. 结构迁移增量保留旧 JSON、身份、历史及路径；失败回滚且不提高 schema，高于识别版本报错不重建。音频登记独立分块且可恢复，选定初始化前严格读一次，选择旧流程真正可用的首选内容；legacy 用同一份已校验字节复制成受管理不可变兼容实例，完成事务前保留原文件。尚未初始化的旧 owner 享迁移保护，旧 marked 状态先恢复，不允许直接绑定。
27. 常规队列读取不再次遍历旧变体／登记旧 hash。后台迁移优先近期词并让出执行权，听音显示真实处理中，识义可继续；全库登记不阻塞进入应用。来源无法确认的文件保留报告，不仅凭文件名删除。
28. 启动恢复所有未完成发布及 GC intent，包含 failed／cancelled 任务残留；使用与正常完成相同的选择守卫。取消／过时代次只能登记未绑定内容并结束 intent，不能切换或假成功；重复恢复按原 operation 幂等，预算不复位。发布与清理统一协调，内存播放不需持整段文件 lease，未来流式播放另设计。
29. 关键恢复后低优先分块对账，起点每块 200 项及每 6 小时维护；大小／mtime／文件 ID 是线索而非完整性证明，同大小／时间替换由使用窗口 SHA 检查发现。portable 移动／换卷按内容验证；未知文件不删除，维护可暂停恢复，不要求全扫结束才能学习。物理删除不代替访问时拒用过期临时内容。

### 批量、任务及有条件优化

30. 本页容量 20／50／100 为比较候选，不设最低二十词或重复叠加一百词产品限制。提交固定 ID 快照及 operation，入口校验正 JS 安全整数、数量及去重。单连接短事务分段，每词 savepoint 保留部分失败归属，chunk 提交失败不报告其中成功。建卡与最小持久音频需求同事务，分别报告已建卡、待生成、可用、失败／取消，识义卡不等整批音频。
31. 任务由应用主管拥有，切页不取消；停止剩余只停止 queued／retry_wait，正常已开始项安全完成。缓存先于排队、同规格去重、P0 保留交互容量、总并发与投递有界；查询、评分、缓存读取及网络等待不占整批生成锁。退出有界停止并回收本次进程树，保留未保存编辑保护、完成数据及可恢复需求。
32. 仅使用已准入本地 CPU 引擎与模型，8 步、48 kbit/s Ogg Opus 为现有基线。现状 2 后台加 1 交互槽为对照起点，保留原 V02 后台 1／2／4 对照并比较总并发 1／2／3；先冻结交互／资源预算，再选择参数。旧独立模型吞吐不能证明产品 P0 延迟、原生播放或内存。
33. 优先级为点击播放／显式重试、近期窗口缺失、本日按额度可出卡缺失（包括不限额日内学习）、已明确提交的其他批量。后台有老化／公平调度，初始可比较每四个窗口任务服务一个本日／批量任务。用户已提交任务不缩成四词窗口；日常不主动生成全七天或全远期词。未来例句只按真实需求预取，当前加后三张为窗口候选，不模拟新卡排程，失败不增评分门槛。
34. 自动修复按失效版本／原因／需求 generation 持久去重；最多四次尝试，退避 2／8／20 秒，首次 claim 起 120 秒 deadline，单次执行 60 秒、启动 30 秒。刷新、自动退避和幂等重试不重置预算；显式终态重试可建立新执行预算。忙碌、坏内容、模型、输入、权限、磁盘及 codec 能力错误分别处理，终态不随刷新自动重启。
35. 本期没有在线配置或准入时绝不外发文本，不默认双引擎竞速或循环兜底。未来真实在线需 Retry-After、鉴权／硬配额分类、有限退避及任务／引擎／整批预算，不能解决本地磁盘发布失败；长期凭据不进前端或日志。
36. 先实现批量快照与单队列协调，再按测量决定进度／额度投影、索引、小读池、WAL 或写协调。投影保留完整原始状态为权威，在评分、撤销、回填及未来同步应用同事务维护，先独立差分再启用。新增 WAL／多连接 checkpoint 前验证 SQLite 已升级到受影响竞态的修复版本或明确回补，记录实际 version/source_id；统一配置并回读 foreign_keys、busy_timeout、journal_mode 与 synchronous，学习数据维持 FULL。
37. 增量缓存带队列修订及前后 delta，评分成功后有限更新／补充，不把固定全表 SQL 每次重扫叫增量。日切、时区／设置、暂停、集合、跳过、时钟／休眠、撤销及修订不匹配使缓存失效；音频事件只影响有关词／窗口。差分不通过可明确回退 DB-only 并报告该批未验收，不无声回退同时宣称完成。
38. 先满足不变量再按收益加深，S0 基线／红回归、S1 批量读取／短快照、S2 版本／兼容及统一 GC 仲裁、S3 有界窗口／同字节播放／摘要、S4 条件投影／索引／连接、S5 增量／优先级、S6 宽限／容量／对账／生命周期。S2 就必须切换删除权威，不能等 S6 才阻止旧 GC 误删。十个子票和十三条直接阻塞关系已发布，见[工单图](issue-24-ticket-plan.md)；S0 #25 已关闭，其余按实际实现和验收继续。

## Testing Decisions

### 已确认的测试边界

沿用 Q10 已确认的测试基线，无新增测试边界需要决定。优先从公开行为及持久结果观察：共享核心验证时间／额度／排程规则；真实隔离 SQLite 和 Windows 文件系统验证业务事务、版本、迁移、发布与 GC；RuntimeAdapter 的受控 IPC／事件 E2E 验证界面、自动更新与生命周期；固定 Windows x64 二进制验证真实 IPC、合成、解码／播放、退出及重启。核心规则不变时复用既有原生／实际 WASM 固定样例，新改核心才重做对应一致性检查。模拟浏览器不证明真实存储或播放。

复用已有 review／speech-work 存储测试、安全缓存测试、queue／audio-recovery／automatic-queue 浏览器测试，以及 product／scale／lifecycle／recovery／targeted-retry 原生检查。断言用户可见行为、身份、事务结果、调用／资源边界及失败原因，不为每个内部函数增加镜像测试；时钟、事件、worker 和故障用确定性屏障，不依靠任意 sleep 碰竞态。

独立参考结果从全量原始权威数据计算，不调用候选分页、增量缓存或投影构造期望。固定 seed 的随机评分／撤销／暂停／额度／时区／音频操作逐次检查业务状态，异步校验收敛后比较完整可见顺序、计数和唤醒；未知全坏场景先断言安全、有界和不准评分，不要求八次 probe 同时确定全库损坏数。

### 原开发前门槛 V01–V06

| ID | 必须交付的改前依据及进入条件 |
| --- | --- |
| V01 | 用隔离数据复现现有慢取卡、导航／串行／锁问题及真实支持状态；固定事件序列、队列／评分／查词／缓存／交互基线；新增行为用可控阻塞建立真实红回归。 |
| V02 | 固定模型／8 步／48 kbit/s／CPU，比较原后台 1／2／4、现状及总并发 1／2／3，测冷／热、请求身份、声线／语言、有限非静音、完整解码、实际 P0／吞吐／CPU／内存；实施前冻结预算，不事后放宽。 |
| V03 | 真实 SQLite 短事务／savepoint、唯一约束、旧进度／暂停、非法 ID、提交失败／重试及并发笔记／评分；页面容量、事务规模、合成并发分别定界。若换库／BLOB／WAL，另验锁、checkpoint 和恢复。 |
| V04 | 本规格绝对 UTC 秒、起算／等于边界、语义／内容身份、临时转持久、多引用、选定／上一版／删词保护、容量、legacy 与非破坏迁移；真实 DB／FS 竞争验证，未完成保护前不清理旧文件。 |
| V05 | 需求／执行／选择身份、条件提交／发布、去重／优先级、取消／超时／预算、重启及故障断点；缓存安全与 GC 共享权威，无孤儿或无限重启。 |
| V06 | 真实例句、在线、浏览器及同步分别满足许可、法语质量、凭据／费用、最低 codec／系统、稳定身份／冲突与恢复；未来项按 F01–F05 明确 deferred。 |

### 原场景与稳定性矩阵 T01–T12

| ID | 必须保留的场景和断言 |
| --- | --- |
| T01 | 全部自动触发在空／非空队列生效；恢复挂载／焦点／可见性合并、末尾事件／逆序响应／持续事件最终收敛、单查询在途、空闲及隐藏无队列 I/O、读取失败定向重试。 |
| T02 | 到期前／恰好／后、同刻多卡、分钟优先、04:00、零／用尽／增减额度、忽略开关、DST／换区／时钟跳变／休眠；双额度零日内听音仍出卡，不提前评分、不重复唤醒或重置已用额度。 |
| T03 | 题面／播放／翻面／评分在途遭遇加入、暂停、额度／时间、语音完成／清理；当前作答保持、真实失效禁评分、不重播；提交丢响应原 operation 重试，撤销不覆盖后来独立编辑。 |
| T04 | 页容量候选下 0／1／本页全部／末页不足，翻页／搜索／筛选／排序／容量清选择，逆序响应不恢复；提交 ID 快照、非法／超界载荷、任务积压有界。 |
| T05 | 混合新词／已有卡／暂停／可用音频，每词失败／chunk 失败／丢响应／重试，双卡身份与空新 due，真实分阶段计数，识义不等音频；满语音积压不得回滚应已提交的双卡。 |
| T06 | 生成／编码／发布／短事务同时真实查词、笔记、评分、缓存播放、缺音频点击／重试；同键后台已执行、P0 突发、连续多页；导航可用、去重、保留名额／投递有界，真实交互与批量时间分报。 |
| T07 | 切页／挂载、queued／running／retry_wait 取消、正常退出、超时／崩溃／重启、发布未提交；应用任务继续、无卸载回写，不删卡／完成文件、预算不复位，真实核验，无孤儿／半文件，保留未保存编辑保护。 |
| T08 | 临时转生词、生成／GC 同时收藏、无卡生词、多引用／解除、暂停／归档、规格变更及 0.6 倍；有效复用、选定保护、不续 TTL、共享不误删、精确到期拒用；未来例句不永久化。 |
| T09 | 旧数据升级／失败回滚、损坏／超大／缺文件、目录／文件链接及重解析、读后替换、发布／归属／GC 竞争、磁盘满／权限／非法 IPC；保留旧数据及可用内容，同字节安全，错误可见，播放失败禁评分。 |
| T10 | 当前加后三张、空 due、新／到期／七天边界、暂停／额度／跳过、部分例句及编辑／重排；缺词音频不只从合格听音队列发现，预取不改排程／额度／位置／播放状态，旧请求不覆盖。本期受控需求契约，真实例句另验。 |
| T11 | 模型／输入／worker／发布／codec 分类错误、忙碌、定向重试、未配置无外发／无竞速；精确次数／退避／预算，刷新不重启 FAILED，不循环兜底；实际在线 429／Retry-After／鉴权／配额另走 F02。 |
| T12 | 100／1k／10k 后端／多页累计，冷／热、空闲／持续通知、前后台／挂载／卸载、分段长时；请求／任务／通知／内存／监听／模型／音频增长有界，报告退化；fixture 或替身吞吐不冒充真实模型。 |

以上共同保留原稳定性矩阵的自动触发、时间额度、当前作答、请求竞争、故障恢复、音频目录、生命周期及批量交互八类边界。专项 T24 的 50 个用例补充而不替代这些断言；F01–F05 不计入本期已通过数。

### 专项红回归与崩溃验证

优先十二项是 T24-Q01（空 progress 新卡）、Q04（双额度零日内听音）、Q08（全坏有界）、Q10（热刷新只读）、A01（同字节校验／播放）、A03（播放成功前禁评分）、C01（当前作答／有效评分）、C04（事务资格／修订）、W06（发布与提交间崩溃）、G01（删卡不解除音频）、G04（旧 GC 不误删新归属／发布）、L03（未知坏 legacy 修复例外）。先得到真实失败再修复；原行为已经满足时保留通过基线，不伪造 red。其余队列／播放／修订／合成／GC／legacy／性能／平台专项继续逐项执行。

用真实文件和两连接屏障分别在 intent 已提交、tmp 未／已同步、rename 后、回读后、DB complete 后、GC mark 后、delete 后中断并重启。包括 failed／cancelled 的 pending、重复恢复、同规格后继需求、同 hash 新发布、共享 owner 与临时到期竞争；回读业务、文件、版本、选择、期限及 intent，不能只看退出码。进程 kill 证明进程崩溃恢复，不能冒充物理掉电持久性。

### 性能与交付协议

使用空库及 100／1k／10k／50k 词，明确每词两卡（10k 词为 20k 卡）、无 progress、新卡空 due、分钟／跨日／长期／未来、暂停／跳过、额度和大量评分历史；覆盖平滑、批量导入、停学积压与日切集中到期。音频包括 0／50／100% 缺失、metadata-ready 全坏、错 hash／截断／坏头／超限／重解析及真实短音频／64 KiB 合法音频。8 KiB Ogg 头 fixture 仅测 I/O，不能证明解码／播放／听音评分。

先预编译并固定二进制 hash、输入 seed、schema、参数、模型／资源、硬件／文件系统、WebView2、RTSS 及 OS cache 条件；性能不与编译或其他压力实验竞争。同期比较原调用、单查询优化和自动触发／合并；每稳态 case 至少 100 次尝试，保留成功样本、失败／超时和分位数算法，报告 p50／p95／max、失败率及命中率，失败不删除。首次／热缓存／物理冷盘分别标注，不可证明的 cold 不称物理冷盘。

分报数据库打开／锁等待、设置额度／候选 SQL及行数、JSON／投影解析、排序／位置写、file open／read／hash、摘要／校验／解码／IPC／UI、worker 排队／加载／推理／编码／发布、GC／对账。逻辑读取不冒充物理磁盘，累计并行 worker time 不相加当 wall time。额外测空闲、事件风暴、连续评分 delta／refill、后台合成的 P0／评分及资源增长。

| 指标 | 验收目标或确定性约束 |
| --- | --- |
| 查询结构 | 无逐卡／词／资产 N+1；热刷新无非必要写事务／legacy 登记；不把每次全表读取称为增量。 |
| 窗口与 payload | 按冻结 W／K／B／字节／等待预算有界，metadata 全缺失文件 probe 为零，未知全坏返回进度；候选／缺失 ID 不全量发前端。 |
| 自动触发 | 稳定空闲新增队列 I/O 零；一个在途，有界合并及持续事件最终收敛；当前 presentation 保持。 |
| 健康热缓存 10k 词／20k 卡 | DB 快照＋窗口可展示首卡 p95≤100 ms 为待实测目标；合成、迁移、首次解码与冷盘另列。 |
| 已预读有效下一卡 | 评分成功返回后的 UI 展示 p95≤20 ms 为待实测目标；点击到持久评分／下一卡的端到端另报。 |
| 全坏／50k／合成争用 | 结构、安全、等待及公平约束先验收；报告延迟、累计维护、CPU／内存及相对交互退化，不套用未证实的常数时间／吞吐保证。 |

按 README 对实际改动运行检查：文档阶段做 diff、链接、规格／用例／规则覆盖及 GitHub 回读；以后前端执行 lint、typecheck、build、适用 E2E，Rust 执行格式、locked 编译及相关测试。后端先 `pnpm rust:test:build`，再 `pnpm test:rust`，执行硬 60 秒；每项原生同限，长时／大批分片保存检查点，不用后台 helper 或放宽预算规避限制。逐项记录命令、断言、passed／failed／skipped／blocked／deferred 和原因，停本次进程／服务，保留无关进程与用户数据。就绪结论变化时另更新验证记录。

## Out of Scope

- 本期不新增真实例句引擎、在线翻译／TTS、账户、OAuth、BFF、Drive／OneDrive 同步或网页持久化。R06 仅包含可测试的需求筛选／预取契约，R07 仅本地任务／重试及无在线配置行为；不引入假服务证明未来完成。
- 不扩大到 macOS、Windows ARM64、macOS ARM64 的实机验收，也不宣布 M3／M4／M6 或四目标完整版／Lite 正式发行就绪。2026-11-04 用户决定 RTSS 兼容性不纳入 #24 验收，不再检查；已知崩溃历史保留，不表示兼容性已修复。干净系统禁网仍须实测。
- 不重算旧评分／到期历史、不强覆盖学习数据、不降级重建数据库、不删除来源未知文件，不擅自添加删词／恢复上一版交互或破坏迁移。真正破坏性的后续动作须先说明影响并获明确授权。
- 不复制不兼容上游源码，不承诺恒定十至二十 MB 音频包、未缓存 100 ms 出声、全平台吞吐或全库 O(1) 查询。SQLite/WAL、缓存投影和全局写协调只在证据及版本门槛支持时采用。

| 条件 ID | 后续里程碑与仍须完成的验收 |
| --- | --- |
| F01 | M3，deferred：真实长／短法语例句、质量／用途准入、实际需求／编辑失效／七天期限、有界预取、真实解码且不加评分门槛。 |
| F02 | M4，deferred：真实在线配置／鉴权／翻译／TTS、429／Retry-After／网络／5xx／鉴权／配额、费用／来源、预算／CPU 隔离；长期凭据不进前端／日志。 |
| F03 | M6，deferred：实际 Rust／WASM 评分／撤销／恢复、有效缓存优先、缺失按需生成、有界预取、不改进度／额度／顺序／播放标记，默认无 Drive 音频读写。 |
| F04 | M6，deferred：目标 Android／Chrome 与 iOS／Safari 真实 Ogg Opus／自动播放／法语系统声线／离线、配额／回收／persist／私密模式及七天边界；系统语音不冒充持久编码文件。Safari 18.4 支持仍受对应最低系统约束。 |
| F05 | M4／M6 可选，deferred：用户开启后仅共享有效持久单词资产、不含临时例句；稳定身份／hash／清单／偏移／规格／中断／重复／冲突／授权恢复，默认无上传，不误删其他文件。 |

本地 14／7 天和 200 MiB 协议不证明跨端引用、离线设备确认及远端删除安全；网页缓存可被宿主／用户回收。未来服务需独立准入与真实证据，模拟 transport、Web Speech 调用返回或格式探测均不等于完成实际集成／播放。

## Further Notes

### 需求与验收追踪

| 原需求 | 本规格中的交付范围 | 开发前／开发后验证 |
| --- | --- | --- |
| R01 | 自动更新、窗口／摘要、当前作答、单查询与时间资格 | V01／V05；T01–T03及全部自动触发／稳定性行；专项 Q／C／P。 |
| R02 | 本页选择、固定快照与两阶段提交 | V01／V03；T04／T05／T12，专项不替代本页验收。 |
| R03 | CPU 主管、交互隔离、去重／公平／容量 | V01／V02／V05；T06／T07／T12，真实在线另 F02。 |
| R04 | 幂等双卡、持久需求、取消／重启／发布恢复 | V03／V05；T05／T07／T09／T11及专项 W／L。 |
| R05 | 选定版本、类型／引用、绝对期限、兼容、GC | V04／V05；T08／T09及专项 A／G／L；F04／F05 deferred。 |
| R06 | 本期可测试需求／预取契约，不接真实例句 | V04／V05／V06；T10，真实例句 F01、网页 F03。 |
| R07 | 本地分类／有界重试、未配置不外发 | V02／V05／V06；T07／T11，真实在线 F02／F04 deferred。 |
| R08 | 网页需求及缓存保留为后续范围 | V06；F02／F03／F04 deferred。 |
| R09 | 可选稳定身份单词音频共享保留为后续范围 | V04／V06；F05 deferred。 |
| R10 | 有界开销、同期性能、持续负载及生命周期 | V01／V02／V03／V05；T01／T06／T07／T12及专项 P／S。 |

### #24 全工单验收清单

原 14 项保留其范围与未完成状态，音频相关旧规则按本规格已确认变更细化；设计／文档发布不勾选产品验收。

- [ ] AC01：保留隔离数据、多规模／缓存／硬件记录、方案收益／复杂度／并发／失效比较及可重复前后对照，明确已测、失败、未测。
- [ ] AC02：全部自动触发可用，正常流程无常驻刷新，等待自动更新，读取／音频失败可定向重试，三语及适用 UI／原生流程通过。
- [ ] AC03：当前作答／播放／评分稳定；双卡身份、独立排程、额度、暂停、跳过、持久顺序及撤销保留；过期／丢响应不重复评分。
- [ ] AC04：单查询、事件合并／最大等待、持续流最终可见性、空闲零 I/O、隐藏停止查询及生命周期释放具可复核断言。
- [ ] AC05：安全目录、批量短事务／两阶段及有界生成通过失败／取消／并发／交互；窗口完整读取／同字节播放符合本规格，独立实验不替代产品。
- [ ] AC06：原稳定性八类及专项逐项记录命令／断言／结果／原因，真实原生／SQLite／播放分别有证据，未测平台单列。
- [ ] AC07：同期单次／累计性能、空闲／事件风暴、资源增长及交互退化齐备，参数按充分证据接受，p95 目标有真实样本或明确未达状态。
- [ ] AC08：实际改动按 README 完成前端／Rust／适用 UI及原生检查，后端预编译后 60 秒执行，大批／长时分片，结论变化维护验证记录。
- [ ] AC09：本期 V01–V05及改前依据完成，V02／V03先冻结预算；V06未来项有里程碑，十二红回归及独立 oracle 通过。
- [ ] AC10：R02–R04本页选择、提交快照、幂等双卡、应用任务、CPU／交互容量、最小需求／取消／重启通过 T04–T09；生成失败不丢已提交卡。
- [ ] AC11：R05选定当前永久、每词上一版14天、最后归属解除7天、临时完成起7天、200MiB保护优先及非破坏 legacy 迁移通过真实 DB／FS、共享／GC竞争；不误续期、误删或改变学习数据。
- [ ] AC12：本地错误／重试通过 T07／T11，忙碌不当故障、无配置不请求、预算／次数精确，真实在线另 F02。
- [ ] AC13：R06本期需求契约通过 T10；F01–F05逐项 deferred／里程碑／待补证，模拟不冒充集成。
- [ ] AC14：R01–R10全部可追溯，T01–T12与原稳定性矩阵完整保留并记录状态；本期 Windows优化完成不等于未来里程碑或正式发行完成。

### 来源与当前状态

- 正式跟踪沿用 [GitHub #24](https://github.com/zardonc/lexique/issues/24)，保留原标题、OPEN 状态及 ready-for-agent；本次整合替代其较早方案边界，不新建重复父票，不关闭工单。
- 上位规格为 [总规格 #1](https://github.com/zardonc/lexique/issues/1)和 [M2 #12](https://github.com/zardonc/lexique/issues/12)，既有 Windows M2 收尾为 [#22](https://github.com/zardonc/lexique/issues/22)；复习核心规则及后续 M3／M4／M6范围保持。
- 项目内来源为 `docs/issue-24-design.md`、`docs/issue-24-optimization-plan.md`、`docs/issue-24-test-plan.md`、`GLOSSARY.md`、accepted ADR 0007／0008及旧 GitHub 需求复核。这些现有工作区材料的 Q1–Q10确认已纳入本文；原副本和历史证据保留。
- 历史万词全到期的三个请求约 1,684.9／768.3／691.7 ms 是特定固定二进制、RTSS-off 的 I/O fixture 样本，不是混合真实播放 p95；历史 98.4%文件 inspect 计时不能重解释为当前 SQL N+1成本。新规格目标均待同期实测。
- 已有 #24实现与修复仍在工作区；完整独立最终规范／规格复审存在缺口，其他硬件／平台、混合历史／冷盘／长时、干净系统禁网及未来集成仍须分别补证；RTSS兼容性已按用户决定排除。本文不宣称这些问题已验收。
- 当前已有十个子票，#25／S0 已关闭；#26 的批量快照及部分 #27／#28 选定版本／修复已在工作区。schema 12、后台兼容登记和迁移状态已接入，执行中的新 regeneration 仍有明确红回归。#29 窗口、#30 有条件数据库优化、#31 增量、#32 需求优先级、#33 完整版本生命周期和 #34 综合验收继续按[工单图](issue-24-ticket-plan.md)推进；代码存在不等于子票或父票已验收。
