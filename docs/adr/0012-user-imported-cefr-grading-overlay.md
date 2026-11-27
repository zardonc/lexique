# 用户自选导入 CEFR 分级叠层并与内置资源及学习数据分离

2026-11-27。用户确认保留 [ADR 0002](0002-source-and-resource-licensing.md) 的源码 MIT 与默认随包资源不含非商业限制的边界，不向第三方申请额外商业授权，也不将含 `CC BY-NC-SA 4.0` 限制的 FLELex 数据随安装包或仓库分发。应用在设置中提供 UCLouvain 官方下载与许可说明页（`https://cental.uclouvain.be/cefrlex/flelex/download/`，目标文件 `FleLex_TT_Beacco.tsv`），由系统浏览器打开；用户自行下载后可将 `.tsv` 导入本机 `Lexique-data/`，并可随时替换或移出。

导入后的 `A1–C2` 等级仅作为本机只读展示叠层，配合随包 MIT 对齐与义项排除规则（不含任何 FLELex 等级值）在核心词典查询、生词本及闪卡背面显示；未导入时不展示等级占位，词频亦不换算为等级。该叠层不写入生词记录、微型兜底摘要、笔记内容交换导出或学习数据备份与同步，避免用户学习数据携带非商业共享限制。

同时，内置核心词典（`core-162-v1`）在保持既有四字段（法语定义、IPA、英语、中文）100% 必填标准不变的前提下，仅为名词（`NOM`）义项补充按义项归属的可选性别字段（`m`、`f`、`mf` 或 `null`），沿用固定 `CC BY-SA 4.0` 词典来源归属。新收藏的核心摘要保存该字段；既有已保存记录和历史备份按 [ADR 0003](0003-note-content-survives-dictionary-changes.md) 与 [ADR 0004](0004-one-vocabulary-record-with-reference-digest-and-notes.md) 保持原样，不作回写覆盖。
