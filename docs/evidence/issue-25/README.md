# #25 S0 隔离基线与预算

[工单 #25](https://github.com/zardonc/lexique/issues/25)，2026-11-03 开始执行。
交付现状观察程序、独立 oracle、故障／竞争控制、原生检查、冻结预算及结果。
本次没有更改产品查询、schema、音频选择、清理或 UI 行为。
[精简结果](results.json)、[输入与环境](manifest.json)、[预算](budgets.json)可供后续票直接引用。
原始测量包含此前尚未提交的 #24 实现；manifest 保留测量时的源文件 hash。
2026-11-04 复核修正：新增 [冻结源码补丁](frozen-source.patch)及 [输入清单](source-inputs.json)，保存测量所需的源码、构建配置和测试输入。
`--build-only` 自动从固定 `c9d8bf8` 导出源码，在新输出目录的 `source/` 中应用补丁，并核对全部 16 个文件 hash；不读取当前产品文件来构造观察器，也不改动工作区产品代码。
Cargo 使用独立的 `target/issue-25/cargo` 编译缓存；新 manifest 分别记录基点、补丁 hash 和运行工具的 checkout HEAD。历史统计与二进制 hash 保留，新构建的二进制另记 hash，不宣称不同构建路径的产物逐字节相同。
重跑仍须核对资源及所用二进制，不能用另一个输入或环境的结果代替历史基线。

## 当前结果

| 边界 | 结果 | 证据（仓库根目录下） |
| --- | --- | --- |
| SQLite／安全文件 | 18 个规模／状态 case，每个首次＋100 次热请求，共 1,800 次热请求，无失败，顺序／结束原因／唤醒与 oracle 一致 | `target/issue-25/matrix-v1/results.json`，各 case 的 `samples.json` |
| 空库 | 100 次热请求，无失败；p95 3.183 ms | `target/issue-25/empty-v1/summary.json` |
| 固定操作 | seed 250001，100 次评分、撤销、暂停、恢复、额度、换区、音频失去／恢复、重开，逐次对账 | `target/issue-25/differential-v1/operations.json` |
| 50k | 50,000 词／100,000 卡，混合、50% 缺失，输入已准备；执行未测 | `target/issue-25/pressure-50k/fixture.json` |
| 原生万词 | 五次独立首次请求＋100 次热 IPC；原始条件 p50 831.9、p95 1,025、max 1,294.8 ms，无失败 | `target/issue-24/s0-native-10k-{1..5}/s0-results.json` |
| 原生交互 | 三轮 idle／batch 实际查词、笔记、评分、缓存读取、P0、完整解码／播放；后台 running 已实际观察，正常退出无自身 worker | `target/issue-24/s0-interaction-v{4,5,6}/s0-results.json` |
| 模型与解码 | 原始 12 次 CPU 对照成功；145 份真实输出完整解码、有限非静音单声道；77,231 byte／13.147 秒长法语音频通过，H 被解码器拒绝 | `target/issue-25/resources-v1/results.json`、`target/issue-24/s0-decode-v1/s0-results.json` |

万词后端 p95（ms）：新卡健康 670.0162，缺失 50% 为 610.2842，全缺失为 524.4481；混合健康 582.3762；积压／50% 缺失 640.2136；全未来健康 264.2946。
这些使用 H（8 KiB Ogg 头）测文件 I/O，不能证明播放或听音评分。
独立每请求 probe 进程与原生长驻 WebView 的计时分别保留，不能直接相减当作 IPC 成本。

用户随后关闭 RTSS 并报告切到性能模式。本次只读查询确认无 RTSS 进程；`powercfg /getactivescheme` 返回 Balanced，Windows 电源模式与方案分开记录。
新条件的五轮万词、三轮交互及 12 次模型对照保留在 `s0-rtss-off-*`／`resources-rtss-off-v1`，结果汇总到 `results.json` 的 `rtss_off`，旧结果继续保留。
不改变设置，也不将这一组多因素对照解释为 RTSS 单因素因果实验。
新条件万词 IPC p50 713.1、p95 895、max 918.7 ms（100 warm，无失败），p95 比原组降低约 12.7%，仍未达到未来 100 ms 目标。
模型 solo／1bg／2bg／4bg P0 中位数为 596.941／741.604／886.605／1,287.785 ms；2bg 退化 1.485 通过冻结门槛，4bg 2.157 未通过。
原生 P0 三次 1,108.3／1,063.6／1,152.5 ms；批量查词中位数 14.5 ms（idle 11.5）通过相对预算，笔记 42.4／7.6、评分 66.7／7.5、缓存 27.7／7.1 ms 仍超过 1.5 倍。三次对照不足以决定生产并发策略，#32 继续负责产品隔离验收。

## 当前缺陷与明确红命令

| 断言 | 当前真实结果 | 改前验证负责票 |
| --- | --- | --- |
| T24-Q04：双额度零仍显示到期日内识义＋听音 | oracle `[1,2]`，后端及原生均 `[1]` | [#26](https://github.com/zardonc/lexique/issues/26) |
| T24-Q10：稳定热刷新只读 | 热刷新仍有 BEGIN／COMMIT、legacy 登记及逐准入卡 position 读取；已有 position 不一定重写 | #26 |
| 共享写锁 | 第二连接持有 writer 时约 5 秒后可见 `database locked` | [#30](https://github.com/zardonc/lexique/issues/30) |

正常 `--regressions` 以缺陷复现成功为通过。以下独立命令要求新契约，已执行并得到预期 exit 1；不加入默认红套件：

```powershell
python -X utf8 scripts/measure_s0.py target/issue-25/red-new --binary target/issue-25/build-v3/baseline_probe.exe --regressions --require-contract
```

原生等价 `gap-red` 同样提供显式失败入口；本次原生执行 `gap`，21 个响应均复现缺陷。
`s0-interaction-v1/v2` 因检查序列使用了尚未创建的听音卡失败，属 fixture 问题，后续修正通过；保留原始失败，不认作产品回归。
首个 probe 构建误用 host serde 工件失败，已改为 Cargo release opt-level=3 工件；失败日志保留在 `build-v1`。
导航、本页选择、自动更新和正常批量属于已有支持，最终浏览器／原生检查分层记录，旧负对照不能用来宣称它们现在仍失败。

## 测量与预算语义

`baseline_probe.rs` 观察冻结时的 desktop blocking body、真实 storage 和安全 cache；`measure_s0.py` 在隔离目录恢复固定基点及源码补丁，只有 clock／计时插入，产品源文件不改。
锚点不唯一时构建失败，manifest 保存原文件、生成文件、Cargo.lock 和 probe hash。
`build-v2` 是 18-case 原始测量；`build-v3` 补上 file-open／JSON-to-value 计时，万词 100 次补测为 p95 791.6501 ms。
观察器自身开销包含在后端计时，未测减去计时器开销后的推算值。

| 阶段 | 测量方法与边界 |
| --- | --- |
| DB 打开／初始化／锁 | 每连接 monotonic 计时；真实持锁失败单列；初始化内 PRAGMA／事务包含在累计值 |
| SQL／返回行 | `trace_v2` 记录 statement 分类、ROW 和 PROFILE；SQLite PROFILE 粒度有限，跨线程累计可超过端到端，不求和为总耗时 |
| JSON／排序／位置 | raw progress JSON parse、FNV／排序和 position 排序计时；位置 SQL 由 trace 单列，端到端含位置分配；v3 另计 to_value 和最终 encode |
| 文件 | nofollow handle 打开、完整读取、逻辑 bytes、SHA；v2 没有单独 file-open 计时，v3 已补；目录安全检查包含在总路径 |
| IPC／UI | 原生 `performance.now` 从 invoke 至返回；完整 decode、实际 play-start／ended；浏览器导航／自动更新为模拟 IPC；已预取切卡 20 ms 新目标未执行 |
| worker | 模型 trial 的 initialization、warmup、inference、encode、service、completion、P0、CPU／private／working-set；初始化合并 import／资源读取／加载，未拆每个模型文件 |
| 发布／排队 | 原生 batch／P0 端到端单列；补测 current Actor 的 committed enqueue→claim 与实际 publication 各阶段，详见下段；模型 finish 写输出属于实验，不冒充产品发布 |

规格审查后补上三个 current Actor 观察试验（`target/issue-25/worker-phases-final-{1..3}`）：直接使用当前 Actor、SQLite claim（背景上限 2）、原生 Job 管理、resident 协议、已准入 CPU 模型及安全发布代码，6 个真实任务／轮均 ready 后正常 shutdown。
仅把 AppHandle 事件接收端注入为计数器，wire fields、安全校验、调度／发布顺序保持当前源码；这是直接任务路径观察，与固定原生 IPC 结果分列。
记录真实 committed enqueue→claim、worker startup→ready、protocol slot／executor queue、资源 hash、模型加载、推理、编码，以及实际 publication intent→tmp write→sync→rename→safe readback→DB complete。
Rust 各发布阶段保留单次样本与累计，resident 以数字 request ID 写独立 phase log；跨任务重叠时间不相加为端到端。当前发布已有这些阶段，未来选择／GC 新协议断点仍未执行。
`build-v8` 是该补测观察器；编译中的 anchor／include doc comment／borrow 错误记录在 v4/v5/v7，修正后编译和执行通过。
完整检查后 `target/release/lexique-desktop.exe` 已被构建流程重建，hash 与实测固定二进制不同。使用保存的同 hash 副本 `target/issue-25/fixed-native/Lexique.exe` 做最终原生红测；21 个响应确实复现 Q04。
误用变化后的构建产物的两次导航失败保留在 `s0-gap-red-final/retry`，不作为 Q04 red 或固定基线性能证据，也不宣称当前构建产物原生就绪。

每 case 首次与热请求分开；新文件／系统热缓存、每请求新 probe、原生进程内热 IPC 均标明，未证明物理冷盘。
nearest-rank 分位数至少 100 次尝试，失败保留，timeout 记 60,000 ms；三次模型／事务／交互对照只报样本、中位数、最大值和预算，不报 p95。

预算在测量前冻结：CPU 2 threads、8 steps、48 kbit/s，背景 1／2／4，总并发 1／2／3（另有总 5 原背景 4 对照）；模型 P0 相对中位数 ≤1.5、max ≤3,000 ms，private ≤1.5 GiB、working set ≤1 GiB。
原始 solo／1bg／2bg／4bg P0 中位数为 655.875／801.615／1,161.283／1,477.916 ms；退化 1／1.222／1.771／2.253，2bg／4bg 未通过延迟预算，内存预算通过。
原生三次原始 P0 为 1,208.3／1,117.8／1,084.6 ms；部分笔记、评分、缓存相对 idle >1.5，完整样本／判定见 JSON。
新增事务字段只收录既有 V03 的 100 ms transaction／250 ms concurrent notes+rating 门槛，模型／交互门槛未事后放宽；保留各 run 的预算 hash。

页容量 20／50／100 由实际 UI 本页选择检查承载；真实 SQLite `preflight_24` 对候选 transaction 20／50／100 各三次测短事务、并发笔记和评分。
候选事务与产品固定 chunk=100 是两种边界，不能把候选通过当作产品分页／建卡协议优化已通过。
未来 B50／W4／K8／32 MiB／75 ms、万词 p95 ≤100 ms、预读 UI ≤20 ms 留给 #26–#34 验收，S0 未采用这些产品协议。

## 独立 oracle 与后续 fixture

`s0_baseline.py` 从原始 cards／progress JSON／ratings／settings／paused／queue entries 生成期望；音频健康由独立 fixture manifest 提供，未知损坏保持 unknown，不调用候选分页、缓存、投影。
Node ICU 独立计算 IANA 学习日／04:00 与 DST；new 无 progress／null due、跨日／分钟／长期、额度、暂停／跳过、音频过滤后比例、FNV 长期顺序及不可见 position 锚点都保留。
现有队列 API 没有类别／用量摘要；顺序／结束原因／唤醒作产品对账，oracle 的 classes／used 由 raw history 与规则检查独立验证。后续窗口 API 新摘要必须再直接对账。
固定随机序列不以 H 文件评分听音卡；recognition 评分走真实 storage，settings／音频变动只发生在合成 fixture。

`s0_faults.py` 提供时钟、可控制 worker、实际截断／错 hash／非法头／超限／缺文件及 deterministic rendezvous。
发布点：intent、tmp written／synced、rename、readback、DB complete；GC 点：mark、delete、finish。
这些是可注入的检查控制，尚未接入不存在的新选择／GC 协议，不宣称断点恢复已通过。
真实 writer 与 commit-hook／savepoint 故障复用 storage preflight；安全链接／同句柄字节复用 speech-cache 检查。

| 十二优先回归 | S0 当前状态／改前负责票 |
| --- | --- |
| Q01 | 原行为无 progress／null due 基线通过；新 LEFT JOIN 契约 #26 |
| Q04、Q10 | 当前缺陷已复现；#26 |
| Q08、A01、A03 | 安全文件／播放基线与未知故障输入已备；有界窗口／同 buffer presentation 新断言 #29 |
| C01 | 已有 UI 当前作答检查基线；新增量／事件契约 #31 |
| C04 | 现有 storage 资格检查基线；新 revision／缓存竞争屏障 #26 |
| W06、L03 | 发布／worker／legacy 错误控制已备；新恢复与修复契约 #28 |
| G01 | 原始归属输入已备；新选定永久保护 #27 |
| G04 | GC mark／delete／finish 屏障已备；新 mark 身份竞争 #33 |

全部 50 专项用例主负责：#26 Q01–Q07/Q10/C02/C04/C05；#27 G01/L01/L02/L05；#28 A06/W03/W04/W06/L03/L04；#29 Q08/Q09/A01–A05/A07/A08/P03；#30 C03/C06/P01；#31 C01/P02；#32 W01/W02/W05；#33 G02–G07；#34 P04/S01/S02/F01/F02。
此处专项 F01/F02 是平台项，与原工单 F01–F05 的未来集成区分。全部当前状态与原 R/V/T/AC 逐项记录在 [traceability.json](traceability.json)，没有把 S0 基线算作 #24 新协议完成。

## 重跑与最终检查

PowerShell 7，pnpm 11.25.0、Rust 1.99.1；Python 使用 `uv python find 3.14`，把该解释器目录放 PATH 首位。
资源使用已准入 Supertonic，模型 assets hash 已实际核对；不安装新模型或服务。
所有输出必须是新子目录，禁止复用／覆盖；只创建合成数据。
资源工具的直接 `--child 0/1/2/4` 和 `--large` 同样要求新输出目录，合成统一在硬限 60 秒的子进程中执行。
每个 case 的音频写入新的 `audio/` 子目录；结果 JSON 以 `status`／`result` 包装直接运行结果，并记录 `audio_directory: "audio"`。解码检查兼容历史平铺结果与此新布局。
源码基点须存在于本地 Git 历史；缺少、补丁损坏或 hash 不匹配均明确失败。编译依赖与模型的获取方式沿用已有准入说明。

```powershell
python -X utf8 scripts/measure_s0.py target/issue-25/build-new --build-only
python -X utf8 scripts/measure_s0.py target/issue-25/matrix-new --binary target/issue-25/build-new/baseline_probe.exe --matrix
python -X utf8 scripts/measure_s0.py target/issue-25/ops-new --binary target/issue-25/build-new/baseline_probe.exe --operations
python -X utf8 scripts/measure_s0_resources.py target/issue-24/followup-final-product-20261103/speech target/issue-25/resources-new
# 从 apps/desktop 执行；五次不同目的目录，每次 1 first +20 warm，合并后 100 warm。
node e2e/native-s0-baseline.mjs ../../target/issue-24/followup-final-product-20261103 ../../target/issue-25/fixed-native/Lexique.exe ../../target/issue-24/s0-native-new scale 10000 new 0
```

编译与执行分开；每个 backend／native 执行硬限 60 秒，规模／模型分片，probe 编译单独 1,200 秒。
原始 logs 留在 `target`，未提交数据库、合成音频或敏感诊断；tracked JSON 只保留统计、hash 和合成 fixture 路径。
最终命令／状态见 `results.json.checks` 和 `target/issue-25/final-checks-v1`：S0 规则、Python／Node syntax、Rust format；lint／typecheck／build；locked rust check／precompile／60s tests；cache／desktop unit；V03；完整 Python tooling、build-args 与 E2E。
最终 87 项 E2E、94 项 core/storage/dictionary、8 项安全缓存、2 项 desktop 单元通过；V03 20／50／100 各三次 transaction 为 4.2596–6.9414 ms，竞争笔记＋评分 14.3467–29.9288 ms，均通过原门槛。
Python 完整发现套件最终 54 项中 6 项因缺专用输入／环境跳过；其中四项原生清理随后指定固定二进制执行通过，剩余真实资源打包及禁网 CLI 两项未执行，不用语法／mock 代替。
初次直接 `node --test scripts/test-build-desktop.mjs` 缺 pnpm 环境而失败，按测试要求改用 `pnpm test:build-args` 两项通过；不修改产品修复错误调用。
审查后的 9 项 S0 回归、完整 Python、lint／syntax／format 复测通过，结果在 `followup_checks`；原失败保留。显式 backend／native contract red 均 exit 1，默认检查没有未归属红测。
浏览器优先查 Obscura；自动化使用仓库已有 bundled headless Playwright，未手动启动系统浏览器。

本轮不声明物理冷盘、禁网新包、掉电恢复、50k 长时、其他平台或发行就绪。后续票负责优化前红测与优化后同输入复测。
历史 [Standards／Spec 审查](review.md) 使用当时工作区作为输入。2026-11-04 按 `c9d8bf8 → 3c48180` 复核发现提交源码不能重建观察器、资源直接模式绕过输出保护；本次修复和复核记录另见 [提交复核](fix-review.md)，历史检查与失败证据继续保留。

2026-11-04 最终独立复核：干净修复快照 `006cfa81` 对固定基点 `c9d8bf8`，Standards 通过（0 项规范违反、1 项 P3 维护建议），Spec 通过（0 项可执行发现）。20 项 S0 回归和语法／格式检查重新通过；同输入的此前编译、后端、模型及原生证据经 hash／原始结果核对后复用。S0 可闭票；Q04 与超过预算的对照仍按后续票处理，完整历史矩阵、50k 执行、其他平台和正式发行未重验。
