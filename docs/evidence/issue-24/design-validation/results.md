# #24 文档写回与验证记录

日期：2026-11-03（America/Vancouver）。Q1–Q9 已逐项确认，Q10 已共同确认完整设计与测试基线。

## 写回范围

已将暂存的方案、测试计划、两份 accepted ADR、词汇表及验证材料写回项目，交付说明放在[设计入口](../../../issue-24-design.md)。项目根 README 与[#24 历史证据入口](../README.md)仅新增导航；原暂存副本、现有改动和历史测量保留。

GitHub 只读复核结论见[范围与覆盖对应](../github-design-review.md)。本次没有修改 GitHub、应用代码、依赖、学习数据或音频，没有创建提交或推送。

## 已执行检查

| 检查 | 命令或方法 | 状态与结果 |
| --- | --- | --- |
| GitHub 工单 | `gh issue view 24 --json number,title,body,labels,comments,state,url,updatedAt` | passed，退出码 0；OPEN、ready-for-agent、0 评论、14 项未勾选。 |
| GitHub 仓库 | `gh repo view --json nameWithOwner,url,defaultBranchRef` | passed，退出码 0；zardonc/lexique，默认 main。 |
| GitHub 范围与导言 | 独立代理只读核对原工单和写回新增内容 | passed；未发现重大遗漏、误记产品通过或新增未确认行为；Q3／Q6 细化与 R03 候选、上游完整验收范围表达一致。 |
| 文档、链接、确认与覆盖编号 | `& '.\docs\evidence\issue-24\design-validation\validate-docs.ps1'` | passed，退出码 0；10 份文档、87 个本地文件链接、50 个唯一用例、12 个优先回归、Q1–Q9 完整映射；Q10 及 accepted ADR 状态一致；R01–R10／V01–V06／T01–T12／F01–F05 编号完整。 |
| PowerShell 语法 | Parser.ParseFile 解析校验脚本 | 首次搬迁替换引入语法错误而 failed；修正字符串替换造成的重复尾段后 passed，退出码 0、错误 0；随后脚本实际执行通过。 |
| 本轮文档差异与空白 | 指定本轮文件执行 `git diff --check`；新增文件由显式文档校验覆盖 | passed，退出码 0，无诊断；本轮词汇表和两处新增导航差异已核对。 |
| 保留现有编辑 | 写回前后 SHA-256 对账；两份入口移除本轮新增段落后与原文比较 | passed；另外 27 个既有文件哈希一致，两份 README 原文完整，词汇表与已确认暂存内容一致；9 份暂存源文件哈希不变，本地分支和 HEAD 保持。 |

## 原暂存阶段与当前边界

原暂存阶段已通过 6 份文档、50 个唯一用例、12 个优先回归、Q1–Q9 映射及 14 个暂存内／36 个目标仓库文件链接检查。当时在线读取受网络与自动审批解码错误影响；本次只读 gh 请求已成功。原 no-index Git 检查退出码 1 且无诊断，不作为当时通过依据；本次在真实工作区重新检查。

[词汇表原快照](glossary-before.md)供差异核对，不代表当前领域规则。写回前当前词汇表与快照除末尾空行外一致，本轮用局部编辑合并 7 个音频术语及已废弃用语调整。

产品测试、迁移、原生播放、并发、崩溃恢复、离线与性能验收本次均未执行。100 ms 取卡、20 ms 已预读切卡及工程参数仍待实测；历史检查不替代新架构验收。没有启动服务或测试进程，没有扩大 docs/verification.md 的准备就绪结论。
