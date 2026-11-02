# #24 桌面实施与验证

2026-11-02；本轮基准 `6850ffb095a69084b02ee40a702355f660c3bd97`。按用户选择完成 V01–V05 后继续桌面实施；[开发前结果](preflight-results.md)与当前产品证据分别记录。既有未提交规格／研究文档保留，不属于本次修改。

当前行为：队列由提交事件、到期／04:00 唤醒、页面进入和前台恢复自动更新；50 ms 合并、200 ms 最大等待、单查询在途、隐藏暂停、稳定空闲零新增队列 I/O。后台事件保持当前题面、翻面及播放；下一次取卡应用新队列。生词本只选择本页，容量 20／50／100，提交快照不会被换页改变。建卡每 100 词短事务、逐词 savepoint，立即提供识义卡；语音交给应用级持久 supervisor。

语音使用既有 CPU Supertonic、单常驻模型、2 推理线程、关闭空转、后台最多 2／P0 保留 1；SQLite 保存去重需求、声线快照、尝试、取消、预算和发布 intent。最多四次尝试，2／8／20 秒退避，执行上限 60 秒且不超过原 120 秒总预算；页面离开不取消，退出停止 worker，重启核验恢复。容量为 10,000 后台需求、32 交互需求、64 消息、32 等待订阅／每键 8；满额仍不阻断已缓存读取。

Schema 11 增量加入元数据与最小任务表，不替换学习库或存 BLOB。完整语义 SHA-256 和内容 SHA-256 分开；新文件为不可变内容版本，发布后核验再提交。临时单词／例句从生成完成的 UTC Unix **秒**算七天，恰好过期拒用，播放和访问不续期。有效期内收藏保护单词，例句不永久化；过期后收藏不能复活旧临时文件。旧声线／模型及最后归属未知策略保守保留，legacy 只登记已验证 hash，不猜来源或生成时间。笔记时间继续使用毫秒，收藏的音频归属使用独立秒钟。GC 分段标记后删除，恢复 pending 标记，缓存锁最多等待 2 秒，失败留诊断并保留学习功能。

## 检查结果

| 命令／边界 | 结果 |
| --- | --- |
| `pnpm lint`、`pnpm typecheck`、`pnpm build` | passed |
| `cargo fmt --all --check`、`cargo check --locked --workspace` | passed |
| `pnpm rust:test:build` 后 `pnpm test:rust` | 最终 93 passed；首次迁移 fixture 残留新表失败已修复；硬限 60 秒 |
| `pnpm test:e2e` | 最终 87 passed；Playwright 使用任务已有 headless shell，模拟 IPC |
| speech_work 产品存储聚焦 | 最终 16 passed；跨 chunk 声线／取消／同 operation 竞争、32 需求容量原子回滚、完成音频重新核验、终态保留、GC 恢复、重试预算及 100／1k／10k |
| `pnpm test:speech:cache`（先预编译） | 8 passed；Windows nofollow／junction、并发发布、失败保留、不可变版本／定向 GC |
| resident 私有 framing Rust 单元 | 1 passed；超界／未知字段／响应身份／截断 |
| `scripts/test_speech_protocol.py` | 3 passed；乱序响应归属、故障可见、严格 framing |
| `pnpm test:speech:cleanup` + 最新 release 路径 | 4 passed；锁竞争有界诊断和重启重试、目录失败、junction 不跟随、legacy 保留／管理 tmp 清理 |
| `pnpm desktop:build --no-bundle` + native-product/lifecycle/recovery-24 | passed；真实 Windows x64 IPC、生成／完整解码／播放结束、杀 worker／取消／恢复、schema10→11 legacy、GC 崩溃标记恢复 |

原生 release SHA-256：`1026d941075a32bcfaf9ec526cb907faf7a7f0307252b98bdcf4dfba679a4838`。原生测量串行执行，不与浏览器或编译压力试验重叠。六词两卡建卡提交 10.9 ms；音频完成前 7 张可用卡；冷 preview 5097.2 ms、热 P0 1067.7 ms、缓存 preview 5.8 ms；同时查词 19.6 ms、笔记 19.1 ms、评分 37 ms。缓存读 6.8–8.4 ms，真实播放开始 36.3 ms；六词完整解码为单声道、有限且非静音，实际播放到 ended。单常驻 worker，正常退出无残留，重启缓存命中不加载模型。只是一台设备的代表样本，不虚报尾延迟，也不把建卡反馈延迟当未缓存出声保证。

产品 100／1,000／10,000 词存储测量：100 词内部分段，10,000 词 100 个提交、整批约 3.016 s，最长分段 38.51 ms，满足预先冻结 100 ms。首轮最长 105.12 ms **failed**；改为每事务统计积压后复测通过，未放宽预算。SQLite 的规模数据为合成 fixture，不能当一万次真实模型吞吐；窗口／重试测试也是受控调度证据。

真实 native IPC 的同期规模对照如下（ms；第一轮与后续范围分别记录，8 KiB Ogg 头 fixture，只用于 I/O，不可解码，不是冷盘）：

| 生词数／状态 | 基准首次／后续 | 当前首次／后续 |
| --- | --- | --- |
| 100 到期 | 197.7 / 75.1–79.2 | 152.7 / 42.6–44.5 |
| 1,000 到期 | 1805.2 / 1108.3–1248.6 | 1407.9 / 392.6–673.9 |
| 1,000 全未到期 | 902.6 / 615.8–711.6 | 74.2 / 71.7–78.2 |
| 10,000 到期 | 19665.4 / 未测 | 14270.5 / 未测 |
| 10,000 全未到期 | 9896.3 / 未测 | 697.4 / 未测 |

初版 legacy 首次逐文件登记导致 1,000 词 6146.4 ms 的退化，**failed**；修正为文件检查结束后每 100 个元数据短事务登记，原样 hash 不写回，再测得到上表。10,000 基准四轮组合超过硬 60 秒，进程树终止；改成分段单样本，保留 due 检查点，未放宽硬限。当前 10,000 全到期仍需约 14.27 s，热缓存／大量历史评分／真实物理读取及长时资源增长缺证据；不能声称低端设备的快速取卡或所有性能门槛通过。空闲没有通知时不进行这种全量查询。

## V／T／R 覆盖与剩余边界

下表的 passed 指列出的断言已执行。部分／未测子场景不会因为行内其他测试通过而算通过。

| 场景／需求 | 已执行断言及证据 | 状态与未测 |
| --- | --- | --- |
| T01 / R01,R10 | automatic-queue：空闲零新增读取、500 在途事件只收尾补查、焦点／pageshow／visibility 合并、隐藏不查、remount 单在途；旧 review-queue 回归保留 | passed；持续极长事件流的资源计量未测 |
| T02 / R01 | 可控绝对 due 边界及 04:00 换日唤醒；存储额度／换日／时区及原有调度回归 | passed；原生长时休眠、DST、实时改系统时钟／时区的 UI 唤醒未测 |
| T03 / R01 | 旧后台响应不能恢复跳过卡；当前答案不替换；原有评分丢响应／撤销／播放失败／在途保护 | passed；全组合排列与真实系统时间前后跳未测 |
| T04 / R02 | notebook：选择本页、分页／搜索／排序／容量清选择、重复点击、导航保留提交快照；严格 IPC ID／数量／响应身份 | passed |
| T05 / R02,R04 | 真实 SQLite 双卡幂等、非法／未知 ID、单词 savepoint／commit 失败、重复／丢响应身份、既有暂停／进度；原生六词建卡不等语音 | passed |
| T06 / R03,R10 | 真实合成同时查词／笔记／评分／P0、热缓存读取／实播；同键去重、满后台积压仍保留 P0 | passed；大规模真实生成同期持续前台负载、P0 长突发资源采样未测 |
| T07 / R03,R04,R07 | UI 卸载／隐藏、跨页继续；真 worker kill→retry_wait、cancel、正常退出无 worker、重启恢复、真实发布 gap hash 恢复；总预算不复位 | passed；真实 worker 执行满 60 秒的超时注入与原生长时休眠未测 |
| T08 / R05 | 秒级恰好过期拒用、有效期内收藏复用／无复习卡保护、过期不复活、词／句归属、语义版本、慢速复用；真实收藏字节相同 | passed；全部多引用／GC 并发交错未穷举 |
| T09 / R04,R05 | schema10→11 真实旧 Ogg 精确读取、笔记／暂停／双卡保留；GC gap 恢复；Windows nofollow／junction／大小／坏头、原子替换失败保留 | passed；磁盘满、真实权限矩阵／文件 symlink 特权环境、部分文件替换竞态未测 |
| T10 / R06 | speech_demand：当前加后三卡、不回绕、同词去重、空 due／七天、暂停／跳过／额度／暂缓、部分／变更例句身份、不改排程 | passed 契约；真实句子合成 F01 deferred |
| T11 / R07 | Python/Rust framing 故障、永久／暂时错误、四次／退避／总预算、终态队列不自行重启、显式 P0 重试及32容量、未配置无在线调用 | passed 本地；真实在线分类／Retry-After／批次服务预算 F02 deferred |
| T12 / R02,R10 | 100／1k／10k 短事务及有界 claim、P0、取消；UI 500事件／隐藏／remount；代表真实音频独立验证 | 部分；native 规模 IPC 对照另记录，物理冷盘、大量历史评分、长时内存增长、所有硬件未测 |
| R08,R09 / V06,F02–F05 | 网页 TTS/BFF/IndexedDB、WASM 复习集成与可选 Drive 音频稳定身份／冲突 | deferred：M4／M6；未增加凭据、网络 fallback 或音频上传 |

原有稳定性矩阵按上述 T01–T09 对应到自动触发、时间／额度、当前作答、请求竞争、故障、音频目录、生命周期、批量交互；没有取消旧测试或改变双卡／额度／暂停／撤销语义。全部组合、长时平台场景仍按未测保留，#24 桌面源码交付不等于 M3/M4/M6 或正式发行完成。

## 复核与原始产物

根目录命令见上表。原生脚本从 `apps/desktop` 执行，输入既有准入资源包、最新二进制和**不存在的新目录**，自己的 WebView2 CDP；每次硬 60 秒，结束清理本次进程：

```powershell
node e2e/native-product-24.mjs ../../target/issue-22-final-evidence/Lexique-0.1.0-windows-x64-portable-speech ../../target/release/lexique-desktop.exe ../../target/issue-24/product-native-new
node e2e/native-lifecycle-24.mjs ../../target/issue-22-final-evidence/Lexique-0.1.0-windows-x64-portable-speech ../../target/release/lexique-desktop.exe ../../target/issue-24/product-life-new
node e2e/native-recovery-24.mjs ../../target/issue-22-final-evidence/Lexique-0.1.0-windows-x64-portable-speech ../../target/release/lexique-desktop.exe ../../target/issue-24/product-recovery-new
```

原始结果：`target/issue-24/product-native-accepted-c/results.json`、`native-lifecycle-accepted/results.json`、`native-recovery-accepted-b/results.json`；四项清理 `cleanup-final.log`；规模 `speech-work-final-complete.log`、`scale-*-b/results.json`；完整 E2E `e2e-full.log`、最终后端 `rust-tests-accepted.log`。精简可追溯结果随仓库保存于 `native-scale-results.json`、`native-results.json` 和 `preflight-summary.json`。a/b 初轮 origin 为 null，重建 Tauri release 并等待有效 origin 后通过；最新二进制两次主流程和一次 recovery 的 WebView2 在首请求关闭，重试及后续生命周期／迁移通过，关闭原因未确定。首次 recovery 的 schema 字段命名断言错误修复后通过；先前 baseline harness/时钟 fixture 失败保留。测试运行错误不冒充产品成功。

当前未施加网络限制、未做干净系统断网或其他平台实机，也未新增人工法语音质验收。旧包的用户离线结果保留为历史证据，不能继承为这个新二进制的结论。测试服务、窗口和 worker 均在本次验证结束停止；用户数据和无关进程保留。

## 审查修复与最终验证

审查比较固定为 `git diff 6850ffb095a69084b02ee40a702355f660c3bd97...HEAD`。两个独立代理分别审查规范与 #24 规格；不混合排序。

规范首轮：1 项 P2 硬违反（单项建卡丢弃逐词错误），2 项 P3 判断性异味（重复订阅生命周期、hash 校验器名称不准确）。已传播单项错误；统一 native change、focus、pageshow、visibility 的订阅／异步注销／清理，各调用者保留自身合并和隐藏逻辑；校验器改为 `validate_hash`，移除无用文件名分配。

规格首轮：3 项 P2，分别为旧 ready 在文件失效后不重新核验、当前听音卡播放失败没有重新生成入口、缺失语音重试扫描全库。新需求将完成任务排队核验文件／TTL，取消和失败仍保留终态；当前卡可显式重新生成，保留答案状态，替换音频成功播放前禁止评分，卸载后不回写／播放；队列返回首个实际缺失词 ID，重试只提交该词 P0，不复活无关取消项。未增加未来服务集成。

原生单项和 ready 两项先在首轮 `1026d941…` 二进制失败，日志为 `native-single-red.log`、`native-ready-red.log`；当前卡生成入口的浏览器红测为 `queue-audio-red.log`。随后修复并通过聚焦及完整回归。聚焦首轮的双 alert 选择器与 `pauseAt` 同时刻竞态属于测试错误，已分别改为按错误内容定位及从前一秒安装时钟；存储取消 fixture 最初还有其他活动批次引用，修正为取消全部引用再验证，未改变共享任务取消语义。定向原生 fixture 的 Unicode operation 被正确拒绝，改为合法 ASCII 身份后通过；失败日志保留。

最终 `pnpm lint`、`pnpm typecheck`、`pnpm build`、`cargo fmt --all --check`、`cargo check --locked --workspace` 通过；`pnpm rust:test:build` 后 `pnpm test:rust` 为 93 passed（speech_work 16），`pnpm test:e2e` 为 87 passed，预编译后 `pnpm test:speech:cache` 为 8 passed。最终日志为 `lint-review-final.log`、`rust-review-build-b.log`、`rust-review-tests-b.log`、`rust-review-check.log`、`e2e-review-full.log`、`cache-review-tests.log`。执行硬限仍为 60 秒，首次编译单独完成。

最后运行 `pnpm desktop:build --no-bundle`，得到新 release SHA-256 `33b34bb661ffb3de53dcaf73e715df6085352d434b15ecae4b172d75159000e3`。上文 `1026d941…` 的性能数字及规模对照保留为首轮证据；新二进制主流程六词建卡 9.5 ms、音频完成前 7 张卡可用，冷 preview 4927.1 ms、热 P0 1095.2 ms、缓存 5.6–6.9 ms、实际播放开始 39.2 ms；并发查词 14.7 ms、笔记 19.3 ms、评分 71.2 ms。六段真实音频完整解码并播放到 ended，重启缓存复用且正常退出无 worker。仍是一台设备的代表样本，不声称尾延迟或大规模实时生成达标。

新原生边界：`native-single-green` 确认未知词／满积压返回错误且无空卡成功；`native-ready-green` 删除自己 fixture 的真实音频后，新批次返回 queued，created_cards=0；`native-targeted-green-b` 确认缺失 ID、单词 P0、另一取消词保持取消及缺少必需 IPC 身份被拒绝。上述短用例最后采用 fixture 强制清理，不当作正常退出证据；正常退出与崩溃恢复由独立生命周期用例验证。各原生用例串行执行、使用独立目录、保留硬 60 秒上限。精简结果及新 hash 见 [最终修复结果](review-results.json)。

新 hash 下 `product-native-review`、`native-lifecycle-review`、`native-recovery-review` 全部通过：完整实播及重启、worker kill→retry_wait／取消／发布提交 gap 恢复、schema10→11 旧 Ogg 精确复用／笔记／暂停保留、GC gap 恢复、正常退出无残留 worker。此次三个独立复核首轮均通过；前述旧二进制 WebView2 偶发关闭仍未定位，不由本次通过抹除。源码交付继续保留其他平台、长时休眠／资源增长、全到期 10k 的明显等待、断网／干净系统和 F01–F05 边界。

对 `c4da7a8` 的规格复审剩余 0 项；规范复审原 1 hard／2 possible 已解决，另指出 1 项 P3 判断性 SQL 重复。已将 ready 与显式 retry 的重置合成一次 UPDATE：ready 总可重新核验，failed/cancelled 只在 retry 为真时重入。该整理后重新预编译并运行 93 项后端（日志 `rust-review-build-final.log`、`rust-review-tests-final.log`），格式和 workspace check 通过；前端代码未变，87 项完整 E2E 证据仍适用。最终二进制与六项原生复核单独保存，避免把上一个 hash 当最终运行证据。

最终 release SHA-256 为 `f5ab46d516021d46af5082f6c84672c2f94218403afe0bd907599cf75076bed1`，`native-single-errors-final`、`native-ready-final`、`native-targeted-retry-final`、`native-product-final`、`native-lifecycle-final`、`native-recovery-final` 六个新目录全部 passed；最终构建日志 `desktop-final-build.log`，workspace check 日志 `rust-review-check-final.log`。六项原生断言与上一轮相同，完整音频解码／播放结束、提交中断恢复、迁移与正常退出均在最终 hash 下重新执行。上轮 `33b34bb6…` 的原始快照可从提交 `c4da7a8` 恢复，当前 [review-results.json](review-results.json) 保存最终 hash 的结果。浏览器／SQLite／缓存／原生证据范围继续分开，未扩大未测场景。
