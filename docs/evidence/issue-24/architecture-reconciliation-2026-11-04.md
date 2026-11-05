# #24 文档、需求与当前架构核对

日期：2026-11-04（America/Vancouver）。核对当前工作区、GitHub #24／#25–#34、accepted ADR 0007／0008 与 M2 规则；原有改动和历史测量保留。这里只更新过时状态，不降低尚未实现的验收要求。

## 权威边界

- 业务规则沿 [M2](../../m2-review-core.md)、[ADR 0005](../../adr/0005-independent-progress-for-vocabulary-card-types.md)和 [ADR 0006](../../adr/0006-learning-days-and-absolute-review-times.md)：独立双卡、同词不暂缓、日内学习不限额度、04:00 学习日、幂等评分／撤销和已有历史不变。
- [ADR 0007](../../adr/0007-windowed-audio-availability.md)要求候选与严格窗口分开、同字节播放；[ADR 0008](../../adr/0008-selected-word-audio-content-versions.md)要求选定当前永久、上一版十四天、最后归属七天、临时完成起七天及保护优先的 200 MiB 软目标。
- Rust core 只计算排程；SQLite／选择／需求事务留在 storage，路径／安全读写／CPU 主管留在桌面平台层，IPC 经 RuntimeAdapter 校验。浏览器仍是明确预览，M3／M4／M6 的真实服务、同步和浏览器持久化仍 deferred。

## 已修正文档与实现缺口

| 主题 | 当前源码／状态 | 处理及仍需验收的要求 |
| --- | --- | --- |
| M2 规则 | `review.rs` 已提供评分、队列、撤销和管理，#22 已收尾；旧 readiness 仍描述日内学习受额度／同词暂缓。 | 修正 product／README／readiness 的状态和规则；#24 不重新设计排程、额度或双卡身份。 |
| 队列快照 | `review.rs` 批量取得 progress／位置；快照外调用可用性检查，必要写入短事务，`data_version` 至多三次重读。 | 修正“现有每卡 progress 查询／总是立即事务”；保留 S0 冻结原实现作为历史。持久 queue revision 和有限 delta 尚未接入，#26／#31 继续验收。 |
| 桌面校验 | `vocabulary.rs` 批读选定元数据，但 `inspect_queue_audio` 仍检查全部符合条件的听音卡，队列调用仍有兼容登记路径。 | 不把批量 SQL 误报为有界窗口。#29 必须实现 W4／K8、候选摘要、待校验和同字节播放，常规读取不能反复迁移。 |
| 选定内容 | schema 12 增加 `speech_versions`、`word_audio_selections`、`word_audio_imports`；平台复制／回读后条件绑定，偏好不隐式换声线，已知修复沿原规格。 | 修正“没有选定指针／未开始 schema”。#27／#28 仍须满足完整选择、执行、失败／取消和恢复守卫；sealed 不是永久的文件健康证明。 |
| 兼容升级 | 旧文件保留，后台按最近修改词每批 16 项登记；pending 版本按 locator 每批 100 项恢复，UI 返回 pending／failed 数。 | 保留增量迁移和真实进度，继续检查失败回滚、未知规格提示、终态任务残留及所有发布断点。 |
| 清理协议 | 规格临时资产已有生成完成起七天／六小时维护，独立 `v-` 实例不会被旧名称清理命中。 | 修正访问续七天／24 小时的“当前默认”描述。名称隔离只是兼容保护；统一版本 GC、共享引用、孤儿期限、200 MiB 和对账仍必须实现，不以无限保留代替完成。 |
| 共享内容 | 当前版本记录绑定创建词，selection trigger 限制同一词；尚无可验证的多词共享版本／last-owner detach 协议。 | 这是实现缺口，不是删除“任一归属保护”的理由。#33 需在非破坏增量结构及真实两连接／文件竞争下补齐，不新增删词／恢复上一版 UI。 |
| 工单顺序 | gh 回读 #25 CLOSED、#26–#34 OPEN；十子票和十三条直接阻塞关系已存在。 | 修正“下一步 to-tickets／首先 S0／未发布子票”；依图继续，代码已存在不自动关闭子票或勾选 AC01–AC14。 |
| 数据库优化 | 当前使用注入连接及完整原始进度；额外投影、WAL、小读池为条件项。 | #30 依据实际瓶颈决定，不能为符合草案强制加结构；若启用 WAL，仍须过 SQLite 修复版本／FULL 门槛。 |

## 本次恢复的反馈循环

前一聊天的最后工作停在已有回归 `a_new_regeneration_during_execution_gets_a_successor_before_selection`；2026-11-04 在原预编译二进制重跑，0.01 秒测试执行可重复失败：已经运行的旧执行把新 regeneration 需求报告为 `ready`，期望 `queued`，旧选定内容应保持到后继成功。这属于 #28 的需求／执行／选择身份缺口。

```powershell
$taskPython = uv python find 3.14
& $taskPython -X utf8 -c 'import sys,json;from pathlib import Path;sys.path.insert(0,"scripts");from measure_s0 import execute;print(json.dumps(execute(["target/debug/deps/selected_audio-9ba5b9567165b898.exe","a_new_regeneration_during_execution_gets_a_successor_before_selection","--nocapture"],Path("target/issue-24-continuation/successor-red.log"))))'
```

观察：退出码 101；断言 `left: "ready"`／`right: "queued"`；外层 wall time 47.3 ms。真实 SQLite、固定时钟、原任务状态和已选内容，无模型、网络或任意 sleep；沿现有 60 秒执行限制。后续实现和重跑结果另记在[本次修复记录](continuation-2026-11-04.md)。

## 交付边界

已同步[架构](../../architecture.md)、[产品](../../product.md)、[readiness](../../development-readiness.md)、[设计入口](../../issue-24-design.md)、[正式规格](../../issue-24-spec.md)、[优化方案](../../issue-24-optimization-plan.md)、[测试计划](../../issue-24-test-plan.md)、[工单图](../../issue-24-ticket-plan.md)、[路线](../../roadmap.md)与根 README。GitHub #24 正文按本地正式规格回写／回读；状态、标签、子票关系和未完成验收保留。

新窗口、增量队列、完整版本生命周期、真实 CPU 争用／多规模性能和 #34 综合验收继续执行。macOS／ARM64、真实浏览器／云服务和正式八种发行仍不由本机结果证明；RTSS 兼容性已按既有用户决定排除。
