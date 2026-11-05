# #24 子票与阻塞关系发布记录

日期：2026-11-03（America/Vancouver）。用户确认10票／13边后，按 `to-tickets` 发布；本阶段仅文档与GitHub工单操作。

## 实际交付

- [父规格#24](https://github.com/zardonc/lexique/issues/24)新增10个原生子票[#25–#34的完整映射](../../issue-24-ticket-plan.md)，全部OPEN、ready-for-agent、未分配。
- 10份标题／正文已逐字对预发布UTF-8正文回读；共83条未勾选验收。现有本地正文移除草案声明与文字Blocked by节，按换行规范化与线上一致。
- 13条native blocked-by按数据库ID建立并逐票GET回读，方向、集合与已批准图完全相符；不使用issue number代替数据库ID。
- 父票标题、完整正文、OPEN、标签及评论逐项保持，`updatedAt`仍为`2026-11-04T00:38:29Z`。没有父票edit／comment／close操作，没有重复父票或子票。
- 当前唯一无阻塞子票为[#25／S0](https://github.com/zardonc/lexique/issues/25)。创建／ready-for-agent不表示已完成；未开始产品实现、分配、提交或推送。

## 已回读的直接阻塞

| 工单 | 阶段 | Blocked by |
| --- | --- | --- |
| [#25](https://github.com/zardonc/lexique/issues/25) | S0 | 无 |
| [#26](https://github.com/zardonc/lexique/issues/26) | S1 | #25 |
| [#27](https://github.com/zardonc/lexique/issues/27) | S2a | #25 |
| [#28](https://github.com/zardonc/lexique/issues/28) | S2b | #27 |
| [#29](https://github.com/zardonc/lexique/issues/29) | S3 | #26、#28 |
| [#30](https://github.com/zardonc/lexique/issues/30) | S4 | #29 |
| [#31](https://github.com/zardonc/lexique/issues/31) | S5a | #29 |
| [#32](https://github.com/zardonc/lexique/issues/32) | S5b | #29 |
| [#33](https://github.com/zardonc/lexique/issues/33) | S6a | #28 |
| [#34](https://github.com/zardonc/lexique/issues/34) | S6b | #30、#31、#32、#33 |

## 命令、结果及证据

| 命令／检查 | 结果 |
| --- | --- |
| `gh issue view 24 --json number,title,body,labels,comments,state,url,updatedAt` | passed，发布前确认本地正式规格／最新正文相同，保存前后快照，发布后父票保持。 |
| 每票 `gh issue create --title <approved-title> --body-file <prepared-utf8-body> --label ready-for-agent` | passed，10票；每次创建即时保存number／url，再读取数据库ID及正文，断点记录防重复创建。 |
| `gh api --method POST repos/zardonc/lexique/issues/24/sub_issues -F sub_issue_id=<database-id>` | passed，10个原生父子关系；GET集合等于本次10票。 |
| 每票 `gh api --method POST repos/zardonc/lexique/issues/<number>/dependencies/blocked_by -F issue_id=<blocker-database-id>` | passed，13边；逐票GET的实际依赖集合与批准图一致，#25为空。 |
| 10份REST正文／标题／标签／状态／分配及父票完整回读 | passed，各正文与预发布原文逐字相同，83条验收；所有子票未分配、父票保持。 |
| 本地正文／图／链接／UTF-8／diff检查 | passed，无缺失链接，13边无环／无传递重复，50专项各有主负责票；实际发布后本地正文规范化一致。 |
| 产品检查／实施 | not run，本阶段没有产品代码变更；没有扩大原生、性能或发行准备就绪声明。 |

第一票#25创建后，最初的父子POST误用`issue_id`，返回HTTP422并明确要求`sub_issue_id`；修正字段后只补关系并沿记录恢复，没有再次创建#25。长调用完成后PTY会话已关闭，poll返回unknown process id；依据持久manifest及完整线上回读确认所有创建／关系已经完成，没有盲目重发。

操作记录、预发布正文、每票原始／最终REST与依赖JSON、父票前后及子票集合保存在忽略的`target/issue-24-tickets-publication-43d8f663d151437981244293747bc98d/`。学习数据、音频、已有工作区、原设计／研究／证据均保留。子票完成后仍需#24全验收判定；本次不关闭父票。
