# #24 正式规格整合与发布记录

日期：2026-11-03（America/Vancouver）。阶段：`to-spec`，仅规格与文档发布。

## 交付与线上回读

- [本地正式规格](../../issue-24-spec.md)已发布到现有 [GitHub #24](https://github.com/zardonc/lexique/issues/24)，没有新建重复父票或子票。
- GitHub `updatedAt` 为 `2026-11-04T00:38:29Z`（Vancouver 2026-11-03 16:38:29）；发布后正文与本地 UTF-8 文档逐字一致，原标题、OPEN 状态及原 `ready-for-agent` 标签保留。
- 本地规格 SHA-256：`a7bd7786648ba55e1af720d554d8c977bf493b1a94e8f3d952e3e9083ac0aab0`。
- 发布前保存原正文／元数据，写入前重新读取并确认正文、标题和更新时间未变化。前后 JSON 位于忽略的 `target/issue-24-spec-publication-3b8c171e64bb4a0faf847e39b9b3c271/`；历史工单内容可从该快照复核，项目设计／研究及原测量文件保留。
- [设计入口](../../issue-24-design.md)增加正式规格与本记录链接；先前“只读复核／未修改 GitHub”的说明保留为设计阶段历史。

## 整合结果

正式规格有七个模板章节、70 条连续编号的用户故事、38 项实施决策及 14 项未勾选验收。明确三项主要变化：数据库候选加近期严格窗口／同字节播放；每词选定当前版本及可恢复的条件发布；当前永久、上一版 14 天、最后归属解除 7 天、临时生成完成起 7 天及保护优先的 200 MiB 软目标。UTC 秒沿用当前存储时间单位，不续绝对 TTL。

原 R01–R10、V01–V06、T01–T12、全部自动触发与稳定性八类边界及 F01–F05 保留，专项 50 用例与十二红回归继续作为补充。沿用 Q10 已确认的测试边界：共享核心、真实 SQLite／Windows 文件系统、RuntimeAdapter UI 和固定 Windows x64 原生检查。

## 命令与检查结果

| 命令／检查 | 结果及证明范围 |
| --- | --- |
| `gh issue view 24 --json number,title,body,labels,comments,state,url,updatedAt` | passed，发布前原文保存及新鲜度检查，发布后完整回读。 |
| `gh issue edit 24 --body-file docs/issue-24-spec.md --add-label ready-for-agent` | passed，更新现有工单正文，保留标题／状态／标签。 |
| PowerShell 规格结构与覆盖检查 | passed，七章节顺序、70 条故事模板／连续编号、14 条未勾选验收、R10／V6／T12／F5行覆盖、实施决策无文件路径或代码片段。 |
| 既有 `validate-docs.ps1` | passed，十份原设计文档、50 专项用例、十二红回归、Q1–Q9对应、87 个本地链接及 GitHub R／V／T／F覆盖。该脚本不自动包含新规格，另行检查新文件。 |
| 新规格／入口／本记录的 UTF-8、空白、标题／本地链接检查 | passed，文档可读，链接均可解析到现有文件。 |
| `git diff --check` 与新文件无索引差异检查 | passed；既有其他文件仅有 CRLF转LF提示，无新增空白错误。 |
| 产品 lint／typecheck／build／Rust／E2E／原生／性能 | skipped，本次未改产品代码，未新增或执行产品检查。既有结果不作为新规格通过。 |

本次只新增规格／发布记录并修改设计入口，没有提交或推送既有工作区、修改依赖／schema、操作用户库／音频或关闭 #24。没有扩大准备就绪声明。下一阶段为 `to-tickets` 的粒度和阻塞关系确认，之后从 S0 基线及回归开始；未完成的独立最终复审、RTSS 开启兼容性与更广平台／性能证据继续保留。
