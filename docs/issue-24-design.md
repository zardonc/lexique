# #24 已确认的优化方案与测试设计

日期：2026-11-03。Q1–Q9 已逐项确认，Q10 已共同确认完整设计与测试基线；本轮已将交付文档写回项目工作区。

2026-11-03 后续进入 `to-spec`：[正式规格](issue-24-spec.md)整合已确认的窗口校验、选定单词音频及版本／清理规则，保留原 #24 的 R01–R10、V01–V06、T01–T12、稳定性矩阵与 F01–F05。该规格沿用现有 #24 发布，具体结果见[规格发布记录](evidence/issue-24/spec-publication.md)。下面“只读复核／未修改 GitHub”的说明属于先前设计交付阶段；原设计与历史证据保留，正式规格不代表新增实现或验收通过。

2026-11-03，用户确认拆票后，已发布[10个子票与13条原生阻塞关系](issue-24-ticket-plan.md)，具体回读见[子票发布记录](evidence/issue-24/ticket-publication.md)。2026-11-04 回读确认 [#25／S0](https://github.com/zardonc/lexique/issues/25) 已关闭；批量快照和部分选定版本链路已实现，其余子票及父票仍 OPEN。当前状态及与旧说明的差异见[架构核对](evidence/issue-24/architecture-reconciliation-2026-11-04.md)。

- [完整优化方案](issue-24-optimization-plan.md)：现状、数据关系、非破坏迁移、批量查询、严格窗口、版本／GC、增量队列、优先级及 S0–S6。
- [专项测试计划](issue-24-test-plan.md)：50 个关键用例、12 个优先回归、独立 oracle、崩溃断点、并发与性能协议。
- [领域词汇表](../GLOSSARY.md)：新增 7 个单词语音术语，按当前 M2 规则移除已废弃的同词暂缓。
- [ADR 0007](adr/0007-windowed-audio-availability.md)和[ADR 0008](adr/0008-selected-word-audio-content-versions.md)：窗口校验与选定版本保护，状态 accepted。
- [GitHub 复核与覆盖对应](evidence/issue-24/github-design-review.md)：#24 的 R／V／T／F、原稳定性矩阵与本轮细化位置。
- [文档验证记录](evidence/issue-24/design-validation/results.md)和[校验脚本](evidence/issue-24/design-validation/validate-docs.ps1)：写回检查与可重复执行入口。

2026-11-03 已通过项目规定的 gh CLI 只读复核 GitHub #24；工单仍 OPEN，14 项验收均未勾选。工单的本页选择、两阶段建卡、全部自动触发和 T01–T12 继续验收；专项 50 个用例不替代全工单范围。F01–F05 仍沿 M3／M4／M6 准入。

2026-11-03 的设计发布仅完成文档与在线复核；上述只读／未实现描述限定于该阶段，不能代表当前工作区。原暂存副本和历史测量保留，实施结果按专项证据及[验证记录](verification.md)更新。

完整测试矩阵仍须逐项执行，性能目标仍须同期实测；不能把部分回归或 schema 已增加记为全工单／发行就绪。
