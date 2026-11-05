# #24 优化测试计划

当前实现与验收结果见[#24实施记录](evidence/issue-24/continuation-2026-11-04.md)和[逐项覆盖](evidence/issue-24/acceptance-matrix-2026-11-05.md)；用户于2026-11-05确认干净系统离线测试通过，#24本期验收完成。下文保留规格／发布阶段的计划与状态记录。

设计确认：2026-11-03；状态核对：2026-11-04。Q10已共同确认的测试设计继续作为完整验收基线；部分存储、UI 和原生回归已执行，逐项结果以[专项证据](evidence/issue-24/README.md)及[架构核对](evidence/issue-24/architecture-reconciliation-2026-11-04.md)为准，不能由此认定全部 50 项或父票通过。
本轮验收平台为 Windows x64；macOS、ARM64 和网页端只记录后续约束。
2026-11-03 已通过 gh 只读复核在线 #24；本文依据当前仓库、已有证据和本轮 Q1–Q10 确认，未更改 GitHub。原工单范围和场景入口见 [GitHub 设计复核](evidence/issue-24/github-design-review.md)。

配套：[优化方案](issue-24-optimization-plan.md)、[产品基线](product.md)、[架构](architecture.md)、[路线](roadmap.md)、[验证记录](verification.md)。
历史基线见 [#24 跟进结果](evidence/issue-24/followup-results.md)；历史通过不代表本文新目标已通过。

## 测试边界与记法

- `T24-*` 是稳定用例 ID；表中共 50 个关键用例。红回归是其中必须先实现的 12 个，不另外重复计数。
- 这些是本轮优化专项用例；原工单 V01–V06、T01–T12、全部自动触发／稳定性矩阵及 F01–F05 继续保留。尤其本页选择、建卡事务及无在线配置不能仅凭队列／资产专项覆盖认定通过；逐项执行并记录状态。
- `U`：纯规则／有界调度检查；`DB`：真实 SQLite；`FS`：Windows 实际文件；`UI`：模拟 IPC 浏览器；`N`：固定二进制原生执行；`PERF`：原生计时。
- “复用并扩展”只说明已有测试位置可承载回归，不声称新断言已存在或通过。
- 以用户可见行为、事务结果和资源上限为断言；不为每个内部函数写镜像测试。
- 只使用新建隔离 fixture；不复制用户库、不发布学习内容、内存转储或私人路径。

## 可调工程参数

以下是测量起点，测试应能用更小边界触发分支；不是产品承诺，也不以固定常量代替行为检查。

| 项目 | 起点／不变量 |
| --- | --- |
| 元数据分页 | `B=50`；必须包含没有 progress 或 due 为空的新卡 |
| 严格校验窗口 | `W=4`，当前卡加后续三张；每次刷新严格 probe 最多 `K=8` |
| 请求校验预算 | 字节预算暂取 32 MiB、等待预算 75 ms、验证 worker 暂取 2；耗尽后有界返回并异步继续 |
| 合成并发 | 2 个后台加 1 个交互起点；CPU 合成，不使用 GPU |
| 合成重试 | 最多四次 attempt，退避 2／8／20 秒；从首次 claim 起 120 秒 deadline，单次执行 60 秒、启动 30 秒 |
| 音频保留 | 当前选定且仍归属生词永久；每词最多一份上一版，自成功切换保留 14 天 |
| 解除归属／临时音频 | 最后生词归属解除后 7 天；临时生成完成后 7 天，读取不续期 |
| 无归属容量 | 200 MiB 软目标；不提前淘汰保护期，不淘汰当前选定永久音频 |

## Fixture 与独立参考结果

| Fixture | 构成与用途 |
| --- | --- |
| `S` 规模 | 100／1,000／10,000／50,000 词，各词两卡；10,000 词是 20,000 卡，另含空库 |
| `M` 混合进度 | 无 progress、新卡 null due、分钟学习、跨日学习、长期复习、未来到期、暂停；entries 只含当日曾准入卡 |
| `D` 真实音频 | 已准入来源的可解码 Ogg Opus；覆盖真实短词样本、64 KiB 级合法音频，记录 SHA、时长、声道和来源 |
| `H` 文件 I/O | 8 KiB 仅 Ogg 头 fixture；只能用于文件开读／吞吐，不可作为可播放或可评分证据 |
| `E` 音频故障 | 0／50／100% 已知缺失；metadata-ready 但全损坏；截断、错 SHA、非法头、超限、重解析点、读中替换 |
| `R` 修订竞争 | 两 SQLite 连接、乱序 UI 响应、评分／撤销／设置／删除／发布／GC 的确定性屏障 |
| `J` 合成与恢复 | 满积压、真实 worker、可注入失败与发布断点、失败 pending、孤儿版本、资源缺失／旧规格 |
| `T` 时间 | 04:00 前后、IANA 时区、DST 前进／后退、旅行换区、设备时钟倒退；保留绝对 due |

参考实现独立从全量权威数据生成结果，不调用候选分页、增量缓存或归一化投影来算期望值。
使用固定 seed，读取原始 progress JSON、设置、暂停、当日 entries、选定音频及已知校验状态。

- 健康及已知缺失 fixture：比较完整可见顺序、分类数量、额度使用、结束原因与未来唤醒时间。
- 新／长期混排比例按音频、暂停、可用性过滤和额度截断后的实际类别数量计算。
- 长期准入按当前时区的 due 学习日及 FNV-1a(due_day 的 UTF-8 字节拼接 card_id 的 little-endian 字节)排序；沿用当前初值、乘数和 u64 wrapping，最终以 ID 打破平局。覆盖多个到期学习日和换区派生日变化，不能用简单 `due_at` 排序替代。
- 无 progress／null due 新卡必须纳入；不能把当日 entries 当作全库候选集合。
- metadata-ready 但损坏未知：先断言安全、有界、坏卡不评分；完成异步校验后比较可见序列最终收敛。
- 不要求最多八次 probe 的首次响应同时给出全库严格损坏计数；未扫描部分必须保持未知或验证进行中。
- 随机操作序列覆盖评分、撤销、暂停、恢复、限额、时区、音频发布及重新打开；逐次检查业务状态，收敛后检查顺序。

## 必须先实现的 12 个红回归

先在当前行为或未实现的新契约上得到真实失败，再实现优化；若原行为已满足，保留基线并说明，不伪造 red。

| 用例 | 必须锁住的回归 |
| --- | --- |
| T24-Q01 | 批量 LEFT JOIN 不丢没有 progress 的新卡 |
| T24-Q04 | 双额度零不能跳过日内听音；受额度阻断的新／跨日／长期候选不耗文件预算 |
| T24-Q08 | metadata-ready 全坏仍有界返回，不能无界补位 |
| T24-Q10 | 热刷新不写队列／legacy，不重复取得立即写事务 |
| T24-A01 | 安全打开并校验的同一份完整字节才进入解码／播放 |
| T24-A03 | 本次真实成功播放之前不能评分听音卡 |
| T24-C01 | 无关刷新／新音频发布不能丢当前作答或误拒有效评分 |
| T24-C04 | 缓存失效遗漏不能绕过评分事务的资格和卡片修订检查 |
| T24-W06 | 文件发布与 DB complete 之间崩溃可恢复且不假成功 |
| T24-G01 | 删卡仍保留在生词本单词的选定音频 |
| T24-G04 | GC 标记后新增归属／发布不能被旧清理任务误删 |
| T24-L03 | 原规格未知的坏 legacy 可按当前偏好修复，提示可能不同；不换可用 legacy |

## 规则追踪

| 确认 | 主要用例 |
| --- | --- |
| Q1 Windows x64，本轮不宣称其他平台 | T24-S02、T24-F01、T24-F02 |
| Q2 偏好影响以后生成，显式替换成功前保留旧音频 | T24-A06、T24-W04、T24-G02 |
| Q3 ready候选、窗口校验、未展示坏卡补位、本次播放后评分、当前作答保留 | T24-Q08、T24-Q09、T24-A01–A05、T24-C01 |
| Q4 legacy保留并移出队列热路径 | T24-Q10、T24-L01–L03 |
| Q5 每词选定音频永久，删卡不解除归属 | T24-G01、T24-G05 |
| Q6 自动修复近期／当日，远期需已有明确批量需求 | T24-W05 |
| Q7 上一版14天、解除归属／临时音频7天 | T24-G02、T24-G03 |
| Q8 无归属200MiB软目标，保护期不打断，未保护临时可提前LRU | T24-G06 |
| Q9 legacy 未知规格的修复例外，新格式遵守原 spec | T24-L03、T24-L04 |

## 队列、额度与分页（10）

| ID | 层级／Fixture | 关键断言 | 新增或复用位置 |
| --- | --- | --- | --- |
| T24-Q01 | DB／S+M | 新卡无 progress／null due 不丢；混合、空库与独立参考一致 | 扩展 `review.rs` |
| T24-Q02 | DB+UI／M | 分页类别失衡仍符合实际混排及FNV，无重复漏卡；准入数>B且B>W时UI使用独立计数，cards只含可展示窗口；窗口外坏音频保持待校验，切入未验证听音前禁评分 | 新增差分与窗口计数回归 |
| T24-Q03 | DB／M | 默认 20 新／50 复习；新默认占复习，跨日及长期占复习，日内重复不占；成功提交才计数 | 扩展 `review.rs` |
| T24-Q04 | DB+N／M+E | 双额度零，新／跨日／长期按 quota 阻断且不耗文件预算；日内学习识义／听音仍可出，听音必须校验，缺失不评分 | 新增桌面听音回归 |
| T24-Q05 | DB／M+R | 提高／降低限额保留已用次数、原位置；新准入按规则混排，重启后相同 | 扩展 `review.rs` |
| T24-Q06 | DB／T | 04:00、DST／换区正确；旧 due 绝对点不动，分钟未到期不提前，时钟倒退阻评分 | 扩展 `review.rs` |
| T24-Q07 | DB／M | 同词双卡独立且不暂缓；暂停／恢复、重复加入／音频重试不重置进度 | 扩展 `review.rs` |
| T24-Q08 | DB+FS+UI／S+E | 全 ready 全坏，每刷新 strict≤K，响应受等待／字节预算约束；显示验证／修复中并分块继续 | 新增，扩展音频恢复 UI |
| T24-Q09 | DB+UI／M+E | 未展示坏卡补位不扣额度；当前失败可重试／跳过而不评分；未来唤醒与错误均不吞 | 扩展队列恢复回归 |
| T24-Q10 | DB／M+J | 稳定热刷新只读；日切／首次准入单独事务；legacy 登记不在热路径，无逐卡 position 重写 | 新增 SQL／事务结构回归 |

## 严格音频校验与播放（8）

| ID | 层级／Fixture | 关键断言 | 新增或复用位置 |
| --- | --- | --- | --- |
| T24-A01 | FS+N／D+R | 当前＋窗口安全 nofollow 打开，完整读取、大小／Ogg 头／SHA 校验；同一 buffer 解码，换路径不换字节 | 扩展 `speech_cache.rs`／native product |
| T24-A02 | FS／E | 文件截断、错 SHA、超 10 MB、读中改变、重解析目录／文件均可见失败，不越权读取 | 扩展 `speech_cache.rs` |
| T24-A03 | UI+N／D+E | 解码失败、播放失败、取消、上一 presentation 成功均不允许本次听音评分；本次成功后才可评分 | 新增并扩展音频恢复 UI／native product |
| T24-A04 | UI／R | 预读乱序／取消和卡片切换按 request generation＋card＋version/hash 绑定；旧响应不串卡 | 新增窗口预读回归 |
| T24-A05 | N／D+H | D 完整解码、有限非静音并实际播放；H 即使文件 inspect 通过也不能取得播放／评分通过 | 扩展 native product |
| T24-A06 | DB+N／D+J | 偏好只影响以后生成；显式替换失败保留旧pointer，成功才切换；旧实例被GC后同hash重新发布用新locator，迟到旧删除不损坏新current | 新增版本替换及同hash重发布回归 |
| T24-A07 | FS／D+E | 小词窗口缓存与最大文件分开测；累计字节预算、单文件上限及取消释放都有效 | 新增内存边界回归 |
| T24-A08 | UI+N／D | 正面不泄露词头、翻面不自动播放；重播按当前 normal／0.6 偏好，失败可恢复 | 复用播放与 native product 回归 |

## 修订、事务与增量状态（6）

| ID | 层级／Fixture | 关键断言 | 新增或复用位置 |
| --- | --- | --- | --- |
| T24-C01 | DB+UI／R+D | 当前合法buffer/作答保留，无关queue修订/新发布不误拒；千次audio_ready风暴保持单查询在飞和有界合并，无idle轮询；card revision及事务资格有效才评分 | 新增 presentation 与事件风暴回归 |
| T24-C02 | DB／R | 评分／撤销与投影/revision同事务；读快照后另一连接改额度/暂停/selection或本次标suspect，旧位置计算不能提交；自身delta不无限重读，no-op不增revision；失败全回滚 | 扩展 `review.rs` 与快照屏障 |
| T24-C03 | DB／M+R | 归一化投影在评分／撤销／迁移及将来同步写点原子维护；与原始 JSON oracle 对账，不静默漏新卡 | 新增投影一致性回归；同步部分后续 |
| T24-C04 | DB／R | 另一连接暂停／删除／评分／改额度后旧缓存不能提交非法评分；revision 冲突可见，原 operation 重试幂等 | 扩展 `review.rs` |
| T24-C05 | DB／M+R | position间隙不足及接近i64上限安全重编；暂时过滤的锚点保留，UNIQUE无撞位/溢出；重编中提交失败不改序，重启及随后撤销恢复相对顺序 | 新增位置边界回归 |
| T24-C06 | DB／R | 若启用 WAL 小读池，先核对运行 SQLite≥3.51.3 或明确回补；写锁／checkpoint 竞争无破坏，FULL 保持 | 新增条件门槛与并发回归 |

## 合成、资源身份与发布（6）

| ID | 层级／Fixture | 关键断言 | 新增或复用位置 |
| --- | --- | --- | --- |
| T24-W01 | DB+N／J | 满后台积压仍保留已提交双卡和真实失败，交互容量可用；并发有上限、仅 CPU，退出无 owned worker | 扩展 `speech_work.rs`／native targeted retry |
| T24-W02 | DB+N／J+T | 四次attempt/退避/首次claim deadline跨重启保持；事件/幂等operation/同执行退避不重置预算，显式终态新需求可建新execute_generation；超时停止自身worker | 扩展 `speech_work.rs`／native lifecycle |
| T24-W03 | DB+N／J+R | cancel/完成不改已提交卡进度；同spec连续替换、取消后新需求与共享执行有最新订阅者时，最新有效意图最终完成，旧/取消执行不覆盖；重复operation不增选择代次 | 扩展 native lifecycle／targeted retry |
| T24-W04 | DB+FS／J | semantic key 含原 spec；不可变 version 与 content hash 分开；同 spec 两次不同输出均合法，旧资源不支持不冒充生成成功 | 新增，扩展 `speech_work.rs` |
| T24-W05 | DB／M+J | 近期/本日缺失优先，远期只承接既有明确批量需求，无全库自动合成；持续新窗口需求下已排队本日/批量按老化或加权得到执行，P0槽不被挤占 | 新增优先级／有界需求与公平回归 |
| T24-W06 | FS+DB+N／J | pending→tmp sync→rename→readback→DB complete各断点恢复；完整文件+已取消pending、同旧selection不同generation不切换；failed pending/孤儿登记，complete后重复恢复幂等 | 扩展 native recovery／lifecycle |

## 音频归属与 GC（7）

| ID | 层级／Fixture | 关键断言 | 新增或复用位置 |
| --- | --- | --- | --- |
| T24-G01 | DB+FS／M+D | 生词本每词一份选定当前音频永久保留；删两卡、暂停、读少或容量超目标均不解除生词归属 | 扩展 `speech_work.rs`／native recovery |
| T24-G02 | DB+FS／D+T | 显式切换成功后最多一份上一版保护 14 天；失败不启动切换期，多次替换不积累无限上一版 | 新增版本保留边界 |
| T24-G03 | DB+FS／T | 最后归属解除7天保护前不删、到期可回收；临时完成起7天且播放不续期，无容量压力时到期才清，有压力时可按Q8提前LRU | 新增分类期限回归 |
| T24-G04 | DB+FS／R | mark→delete→finish校验version/mark身份；新增归属不能复活已marked版本；迁移前v11 gc_pending及迟到旧删除不得误删新current，读占用失败重试 | 扩展 `speech_work.rs`／native recovery |
| T24-G05 | DB+FS／R | 共享字节的多个 owner／version 不因单一词删除被误删；已解除最后归属的重复事件不延后起点 | 新增最后归属竞争回归 |
| T24-G06 | DB+FS／S+D+T | 无归属>200MiB先到期/多余，再未保护临时LRU（可早于7天）；当前/上一版/删词宽限不被提前淘汰，受保护占用多时可超目标；实际分配与逻辑字节分列 | 新增软容量回归 |
| T24-G07 | FS+N／E+J | 删除失败、崩溃、文件占用及链接保留真实诊断、可重试；不删模型／词典／用户库，退出释放自身句柄 | 扩展 `speech_cache.rs`／native recovery |

## Legacy、迁移与恢复（5）

| ID | 层级／Fixture | 关键断言 | 新增或复用位置 |
| --- | --- | --- | --- |
| T24-L01 | DB+FS／J | 分块迁移幂等，选定初始化前旧owner相关内容有迁移保护；保留卡片/笔记/历史/暂停及实际旧字节，未知voice/model/gentime仍未知；保护建立后才交新GC | 扩展 `speech_work.rs`／native recovery |
| T24-L02 | DB+N／J+E | 可用 legacy 不自动替换；损坏登记不能停止 supervisor，其他请求及显式重试继续 | 复用并扩展 native recovery |
| T24-L03 | DB+UI+N／J+E | legacy 缺失／确实损坏且原 spec 未知，可按当前偏好自动修复；明确提示原规格未知／声音可能不同 | 新增 Q9 回归 |
| T24-L04 | DB+UI／J | 新格式修复使用原 spec；不支持／资源缺失可见失败并要求显式升级，偏好变化不偷偷降级或替换 | 新增旧 spec 回归 |
| T24-L05 | DB／J | 从 schema v11 非破坏升级；中断回滚、不升版本；更高 schema 拒绝降级重建；重开后投影／owner 对账一致 | 扩展存储迁移及 native recovery |

## 性能、隐私与平台（8）

| ID | 层级／Fixture | 关键断言 | 新增或复用位置 |
| --- | --- | --- | --- |
| T24-P01 | PERF／S+M+D | 100／1k／10k 健康热缓存，10k 的 DB＋窗口路径 p95≤100 ms 待测；首次请求及全量摘要独立计量 | 扩展 native scale |
| T24-P02 | PERF／S+M+D | 评分成功返回后已预读UI切卡p95≤20ms待测，端到端提交/出声另报；连续普通评分记录返回行/解析数及rebuild次数原因，不能每次固定一条全表SQL；刻意失效回退单列 | 新增增量结构及原生计时 |
| T24-P03 | PERF／S+E+H+D | 缺失0／50／100%及全 ready 全坏：每次有界请求 SQL 数不随全库词数增长、strict≤K、响应有限；分页对账／修复成本另列 | 扩展 native scale |
| T24-P04 | PERF+N／S+J | 50k 词、混合历史、积压、长时运行／休眠／重启的内存、句柄、worker 和 p95／p99先报告，不承诺目标 | 新增分片压力检查 |
| T24-S01 | FS+N／E+J | 音频／临时／日志只在 portable 所有权目录；诊断无词头、笔记、凭据、页面 URL 或外部路径泄露 | 扩展 native recovery 与日志检查 |
| T24-S02 | N／D+J | 固定 Windows x64 binary hash，禁网真实生成／播放／重启；ProcessFailed 可见。用户于 2026-11-04 将 RTSS 兼容性排除在验收外，不检查 | 复用 native product／lifecycle |
| T24-F01 | 后续／D+J | Windows ARM64、macOS x64／ARM64 的路径、编码／解码、安全打开及离线运行分别实测；编译不算实机通过 | 后续桌面平台计划 |
| T24-F02 | 后续／D+R | 网页端 codec、存储／配额、多标签与 WASM另验；Safari 18.4 Ogg Opus 需 macOS15.4／iOS18.4，不宣称 M6 | 后续浏览器计划 |

## 崩溃断点与性能执行协议

W06／G04 使用屏障逐个执行断点：intent 已提交、tmp 未同步／已同步、rename 后、readback 后、DB complete 后、GC mark 后、delete 后。
每个断点重启验证业务、文件与元数据，不只看退出码；新音频成功前保留旧版本。
加入失败 pending 恢复、重复恢复、孤儿版本和并发 owner；进程 kill 只证明进程崩溃恢复，不能证明掉电持久性。
SQLite 当前锁定 bundled 3.51.1 属 WAL-reset 受影响范围；若采用 WAL 读池，升级／明确回补是 C06 的前置门槛，不能靠压测碰巧没坏放行。

性能测量先预编译并固定二进制、输入 seed／schema／参数、硬件、NTFS／存储、WebView2、RTSS 状态及系统缓存条件。
每个稳态指标至少取得 100 个有效样本，保留分位数算法、原始值、错误／超时数和窗口命中率；禁止删失败样本。
首次启动、首次请求、应用热缓存、系统缓存及物理冷盘分别标注；没有可证明的冷盘条件就不称物理 cold。
历史 10k due 的 1,684.9／768.3／691.7 ms 是三个 I/O 样本，不是 p95；历史 audio 98.4% inspect 不含当前逐卡 SQL。
真实 256 样本约 8.8 KB／1.43 秒／48 kbit/s 仅为输入量级参考；不能用 H 的通过代替 D 的解码／播放。
记录 SQL／事务次数、strict 次数、读入字节、JSON 解析、排序、IPC 大小、DB＋窗口、摘要、解码、出声及评分成本。
全量摘要及未知坏文件对账独立计量；50k 压力先报告。编译和模型压力实验不与性能采样竞争。
后端及每次原生 execution 保持硬 60 秒；按 fixture／断点／采样批次分片，不放宽超时来完成矩阵。

## 执行与交付

既有承载位置：[storage review](../crates/storage/tests/review.rs)、[speech work](../crates/storage/tests/speech_work.rs)、[speech cache](../apps/desktop/src-tauri/tests/speech_cache.rs)、[队列 UI](../apps/desktop/e2e/review-queue.spec.ts)、[音频恢复 UI](../apps/desktop/e2e/queue-audio-recovery.spec.ts)。
原生脚本：[product](../apps/desktop/e2e/native-product-24.mjs)、[scale](../apps/desktop/e2e/native-scale-24.mjs)、[lifecycle](../apps/desktop/e2e/native-lifecycle-24.mjs)、[recovery](../apps/desktop/e2e/native-recovery-24.mjs)、[targeted retry](../apps/desktop/e2e/native-targeted-retry-24.mjs)。
撤销／管理存储回归目前在 `review.rs`，UI 撤销在 `review-queue.spec.ts`；不引用不存在的独立 undo 测试文件。

未来实施先读 `LOCAL-DEV-ENV.md`（若存在），使用 package／toolchain 固定版本；安装采用 `pnpm install --frozen-lockfile`。
顺序为红回归 → 规则／真实 DB → FS 安全与竞争 → 模拟 UI → 固定原生主流程／断点 → 分片性能。
按 [README 检查](../README.md)执行：前端修改需 `pnpm lint`、`pnpm typecheck`、`pnpm build` 和适用 E2E。
Rust 修改需 `pnpm rust:fmt`、`pnpm rust:check`、`pnpm rust:test:build`，预编译后再 `pnpm test:rust`；Cargo 使用 locked。
原生缓存测试也先编译，执行仍限 60 秒。编译耗时单列；编译不证明存储、离线、实播或发行就绪。
每项记录 ID、命令、binary／fixture hash、passed／failed／skipped、原因和证据路径；结束停止本次服务与进程，保留无关进程。

验收必须满足业务／安全断言、12 红回归、独立参考收敛及结构预算；性能目标需真实分位数证据。
性能不足、资源不支持及平台未跑均保持可见；有界“验证／修复中”属于合法状态，但不能无限等待且必须报告进度／失败。
就绪结论变化后更新 `docs/verification.md`，区分模拟 UI、真实 DB、原生执行、物理掉电与未来平台。
本轮只对新文档做 diff／链接及用例 ID 检查；上述产品、性能、编译与原生检查全部未执行。
