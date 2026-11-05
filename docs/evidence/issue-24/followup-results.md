# #24 万词取卡与 WebView2 关闭跟进验证

2026-11-03。对应 [#24](https://github.com/zardonc/lexique/issues/24)。用户选择审查全部 #24 已有提交及本次修复，固定基点 `eae82eab0364db59a64f2295fefd0fa1213bc04a`；工作区起始 HEAD 为 `c9d8bf8`。[此前桌面报告](desktop-results.md)的时间、未定位关闭及审查结论保留为历史证据；本报告记录后续修复。用户已有同步规划及研究文档修改保留，未提交、推送或关闭工单。

## 修复结果

一万词全到期的本机原生取卡已通过本轮 2,000 ms 回归预算。同一旧版在 RTSS 关闭时仍明显超标，因此慢取卡与 RTSS 崩溃是两个独立问题。偶发 WebView2 关闭已定位为注入的 `RTSSHooks64.dll` 访问异常；**尚无经过验证的 RTSS 开启兼容性修复**。

- 队列检查采用最多 16 个 scoped 线程，各自持有 SQLite 连接，共用本次更新的安全缓存目录。按清单顺序收集结果，保留跳过、未来到期唤醒和音频错误。完整读取及校验音频，legacy 元数据在读取结束后分段提交。
- 缓存验证 nofollow 打开的实际句柄，移除重复路径 metadata 查询；按验证后且不超过 10 MB 的长度预分配读取缓冲。完整长度、读取上限、Ogg 头、重解析点及访问时间保护保持。进度、音频 key 和 legacy 登记复用 prepared statements。
- 满后台语音积压时提交双卡，语音任务记为 failed 并报告真实容量错误；识义卡可用，显式 P0 重试仍可执行。未知词等真正的建卡失败继续拒绝，不能返回空卡模拟成功。
- 腐坏 legacy／ready 缓存错误保持可见，不再由 `?` 终止 supervisor；等待者收到实际读取失败，其他请求仍可处理。
- Windows 记录 WebView2 ProcessFailed 的 kind／reason／exit code，以及 CloseRequested／finish_close，不记录页面 URL 或学习内容。直接依赖引用既有锁定版本 `webview2-com=0.39.1`、`windows-core=0.62.2`，Cargo.lock 未增加新版本。

`main.rs` 与起始 HEAD 一致；所有 RTSS 环境变量／bootstrap 试验及临时性能打印均已移除。学习数据只用新建隔离 fixture，没有复制或改写用户学习库。

## 同条件性能比较

Windows x64，沿用此前 Ryzen 9 8945HS／32 GB RAM／NTFS NVMe 环境；RTSS 真正关闭。测量与编译、模型压力实验串行分开。8 KiB 文件只有 Ogg 头，是 I/O fixture，**不是可解码音频或物理冷盘证据**。首次请求与后续请求分列，未清空系统缓存，不把三个样本称为尾延迟。

| 一万词状态／二进制 | 首次 ms | 后续 ms | 结果 |
| --- | ---: | --- | --- |
| 全到期，起始 HEAD 旧版 | 15,821.2 | 7,478.0；4,561.1 | 三次均超 2,000 ms，预期 failed |
| 全到期，最终修复版 | 1,684.9 | 768.3；691.7 | 每次 20 卡，passed |
| 全未到期，最终修复版 | 260.1 | 236.5；229.2 | 每次 0 卡并提供未来唤醒，passed |

旧版 hash 为 `f5ab46d516021d46af5082f6c84672c2f94218403afe0bd907599cf75076bed1`。这是工作开始时的桌面实现，不是审查基点的二进制。旧版 due 检查点及日志为 `target/issue-24/followup-baseline-rtss-off-20261103/checkpoint-due.json`、`target/issue-24/followup-20261102/baseline-rtss-off.log`；退出 1 来自预算断言，未来状态未执行。较早 RTSS 开启时约 15.2／16.3 秒的复现不作本表同条件对照。

最终 100／1,000 词全到期首次分别 56.7／237.8 ms，后续分别 17.3–18.1／74.8–78.0 ms；全未到期首次 15.2／43.1 ms，后续 12.8–16.0／39.9–41.7 ms。[精简结果](followup-results.json)保存逐次数据及原始路径。

失败候选保留：并行读取时提交 legacy 元数据需 2,684.6 ms，已移除；prepared statements + 16／32 线程仍需 2,106.4／2,063.1 ms，超预算。最终选择 16 + 缓冲预分配，探索首次 1,653.8 ms 后另做固定二进制矩阵，没有用最快探索值替换最终验收值。

## WebView2 原因及被否定的绕过

多个本次 fixture 转储一致记录 `0xC0000005`，异常指令位于 `RTSSHooks64.dll + 0x1490af`。安装的 RTSS 为 7.3.5.28314。原生事件为 `ProcessFailed kind=0 reason=0 exit=-1073741819`，随后页面／CDP 关闭，与主动 stop-request 区分。

代表转储在 `target/issue-24/followup-parallel16-10k/Lexique-data/webview/EBWebView/EBWebView/Crashpad/reports/ced8e129-07bd-42d3-ada4-e12afa24a9e5.dmp`，只本地保留，不发布转储内存。失败事件亦见 `target/issue-24/followup-inherited-lifecycle-20261103/lifecycle.jsonl`。

测试过 `RTSSHooksProfileOverride=EnableHooking,0`：在 main 中设置、进程创建时继承及 bootstrap 重启。启动／部分播放曾通过，但生命周期仍在同一模块崩溃，均为 failed mitigation，全部撤除。日志“restarting without RTSS injection”只是已删除试验的意图，不能证明实际没有注入。

用户明确允许临时退出并恢复 RTSS，随后自行关闭。RTSS 关闭时最终矩阵全部 passed，15 次启动首请求、三轮真实合成／完整解码／播放结束及退出恢复均无捕获的 ProcessFailed。验证结束已从原路径正常恢复 RTSS，观察到 PID 13940，未改设置。开启后长期稳定性仍未验证，不能承诺本次代码已解决 RTSS 注入兼容性。

## 回归与最终二进制

最终通过 Tauri 重建并复制固定候选 `target/issue-24/followup-20261102/desktop-candidate.exe`，SHA-256 为 `57334356d84addf966e3a639d0fdb7cea38d936d96171f6f0f15fb46de8ed77b`。原生结果目录统一为 `target/issue-24/followup-pinned-<case>-20261103/`；命令日志在 `target/issue-24/followup-20261102/native-pinned-<case>.log`。

改前腐坏 legacy fixture 的 `followup-corrupt-red/lifecycle.jsonl` 记录 `Speech supervisor stopped: Invalid cached speech audio`；修改后 corrupt-retry 显式重试及另一 preview 成功，正常退出无 worker。满积压建卡回滚由聚焦回归暴露；测试 `full_speech_backlog_preserves_committed_cards_and_allows_interactive_retry` 及最终 single-errors 确认双卡、真实语音失败、识义可用和 P0 容量，重复 operation 不重建卡片。

| 检查／命令 | 最终结果；日志均在 followup-20261102 下 |
| --- | --- |
| `pnpm lint` | passed，最后脚本修改后再执行；lint-followup-final.log |
| `pnpm typecheck` | passed；typecheck-final.log |
| `pnpm desktop:build --no-bundle`（包含前端 build） | passed；build-pinned-final.log |
| `pnpm rust:fmt`、`pnpm rust:check` | passed，workspace check 为 locked；rust-check-final.log |
| `pnpm rust:test:build` 后 `pnpm test:rust` | 94 passed，speech_work 17 项；rust-test-build-final.log／rust-tests-final.log |
| 预编译 release speech_cache 后 `pnpm test:speech:cache` | 8 passed；speech-cache-final.log |
| 预编译 release desktop lib 后执行测试二进制 | 2 passed，含持有 BEGIN IMMEDIATE 时的并行只读检查；desktop-test-build-final.log／desktop-lib-final.log |
| `pnpm test:e2e` | 87 passed，55.9 秒，模拟 IPC；e2e-final.log |
| `LEXIQUE_TEST_EXECUTABLE=<固定候选>` + `pnpm test:speech:cleanup` | 4 passed，17.370 秒；cleanup-pinned-final.log |
| startup／disconnect | 15 次首请求通过；注入 page.close 可检测，不冒充 RTSS 崩溃注入 |
| product-1／2／3 | 每轮六段完整解码，有限／非静音／单声道；真实播放至 ended、并发查词／笔记／评分、重启保留，退出 worker=0 |
| lifecycle／recovery | 真 worker kill、retry／cancel、发布 gap；旧 schema／legacy、GC gap、笔记／暂停／双卡保留 |
| corrupt-retry／single-errors／targeted-retry | 腐坏缓存不杀 supervisor；未知词拒绝；满积压双卡与失败原因保留；仅请求词重试 |
| scale-100／1000／10000 | 原生 IPC／SQLite／完整文件检查；全部 due 样本符合 2,000 ms |
| 差异及跟进文档检查 | git diff --check、相对链接／JSON 校验 passed |

初次编译与执行分开，后端、cache、desktop lib、每项原生执行仍保持硬 60 秒上限；清理检查附加外层 60 秒限制。没有放宽预算。desktop lib 编译保留四个 test-only dead-code warnings，未扩大范围清理。

三轮真实主流程建卡提交 13.1／12.5／10.9 ms，实际播放开始 36.8／41.3／39.0 ms；小样本实播不代表一万词真实合成吞吐或未缓存出声保证。自己的 WebView2 使用 disable-gpu 和允许自动播放参数；CDP 直接调用不当作未加参数的手工 UI 验收。

最终矩阵使用 Tauri 生产构建后固定的候选二进制，执行身份由每份原生结果登记。完整 headless E2E 独立记录。

原生日志中主动 finally 清理的 `Owned desktop exited: 1` 不等于测试失败，以脚本退出码及业务断言为准。结束已核查本次 native／worker／headless browser／测试命令无残留，RTSS 恢复，无关进程保留。

## 复现

从根目录先运行静态检查与编译，Cargo 测试编译后再用 Tauri 重建并复制候选，重新记录 hash。资源使用已准入旧包的资源部分；以下在 apps/desktop 执行，目标必须是不存在的新目录，每项串行运行：

```powershell
$package = '../../target/issue-22-final-evidence/Lexique-0.1.0-windows-x64-portable-speech'
$candidate = '../../target/issue-24/followup-20261102/desktop-candidate.exe'
node e2e/native-scale-24.mjs $package $candidate ../../target/issue-24/new-followup-scale 10000 2000
node e2e/native-startup-24.mjs $package $candidate ../../target/issue-24/new-followup-startup
node e2e/native-startup-24.mjs $package $candidate ../../target/issue-24/new-followup-disconnect disconnect
node e2e/native-product-24.mjs $package $candidate ../../target/issue-24/new-followup-product
node e2e/native-lifecycle-24.mjs $package $candidate ../../target/issue-24/new-followup-lifecycle
node e2e/native-recovery-24.mjs $package $candidate ../../target/issue-24/new-followup-recovery
node e2e/native-corrupt-retry-24.mjs $package $candidate ../../target/issue-24/new-followup-corrupt
node e2e/native-single-errors-24.mjs $package $candidate ../../target/issue-24/new-followup-single
node e2e/native-targeted-retry-24.mjs $package $candidate ../../target/issue-24/new-followup-targeted
```

最终矩阵 RTSS-off 的环境边界必须如实记录；命令本身不会关闭 RTSS 或改设置。性能计时不得与编译／模型压力竞争。

## Standards

按用户指定 code-review 技能审查全部 #24 提交，另加本次修复。规范首轮、两路最终代理及 fresh-context 重试均受模型 token rate limit 中断，**不存在完成的本轮独立最终规范审查**，不能把历史 0 hard／0 possible 当作当前结论。

人工补查本次线程／连接所有权、缓存安全、错误传播、Windows 诊断及已有文档保护，不形成全基点范围的独立计数；工具约束通过不冒充 Fowler 异味审查通过。

## Spec

初轮确认两项规格缺陷：满语音积压回滚双卡；腐坏 legacy 缓存停止 supervisor。两项已修复并通过聚焦／原生回归。两路最终规格代理及 fresh-context 重试同样被 token rate limit 中断，**独立最终规格审查未完成**，不报告 Spec pass 或未解决数 0。

本机 10k due 已改善、RTSS 异常已定位，#24 更广验收仍未测：RTSS 开启长期兼容、其他硬件／平台、64 KiB 规模缓存、大量历史评分混合、物理冷盘、长时休眠／资源增长、新二进制干净系统断网及 F01–F05。保留逐项边界，不关闭工单、不扩大发行结论。

审查统计：Standards 最终计数未知，最坏剩余项未定；Spec 已确认 2 项并修复，最终计数未知，最坏剩余项未定。
