# 自选导入 CEFR 分级与内置核心名词性别：调研策略及实施前试跑方案

更新：2026-11-27。基线：`main` 提交 `136c43f`（已合入 `core-162-v1` 全部 4,087 个核心词性组）。决策依据见 [ADR 0002](adr/0002-source-and-resource-licensing.md)、[ADR 0003](adr/0003-note-content-survives-dictionary-changes.md)、[ADR 0004](adr/0004-one-vocabulary-record-with-reference-digest-and-notes.md) 及 [ADR 0012](adr/0012-user-imported-cefr-grading-overlay.md)。

## 1. 背景与已确认边界

`core-162-v1` 已完成全部 **4,087 个冻结核心词性组**（**4,062** 个独立复核义项／**16,248** 个法语、IPA、英语、中文字段，以及 **25** 项零字段注记例外）。早期针对未完成四字段队列的规划地图 [#202](https://github.com/zardonc/lexique/issues/202) 及其子决策票（#203–#207）中关于剩余四字段补缺的部分已被 `core-162-v1` 取代，不再重复执行模型调研。

在此基线上，新增两项低模型消耗的数据与产品准备任务：

| 维度 | 已确认规则（Q1–Q10） |
| --- | --- |
| **内置核心名词性别** | 仅在 `NOM`（2,134 个已准入核心组）的每个 `CoreSense` 上绑定可选 `gender`：取值为 `m`（阳性）、`f`（阴性）、`mf`（阴阳同形／双性）或 `null`（非名词、零字段注记，或如 `gens` 等性数配合随语境变化并在 `usage` 说明的特殊名词）。保持 4/4 必填标准不变，禁止按词尾猜测。 |
| **性别来源与复核边界** | 以固定法语维基词典源段对当前词头拼写及所选义项的性别标注为主权威，`Lexique 3.83`（`genre`）仅作交叉核验信号。单性别非同形异义词在双源一致时由程序确定性通过（`0` 次模型调用），继承该条既有固定维基词典归属；仅同形异义词、双源冲突（如维基 `mf` 与旧版 Lexique 单一 `m`/`f`）及当前摘录行缺失项进入残差复核队列。 |
| **性别界面与旧数据兼容** | 在词典查询、生词本摘要／详情和复习卡片**背面**展示性别；**识义卡正面保持仅显示词头**，不泄露性别提示。新保存的 `CoreSense` 微型兜底摘要包含 `gender`（及非空时的可选来源引用）；既有已保存记录与 v1–v4 备份保持原状，不执行回填改写。 |
| **自选导入 CEFR（方案 2）** | 不向第三方发送商业授权请求，不在仓库或安装包内分发 `CC BY-NC-SA 4.0` 数据。设置页提供 UCLouvain 官方下载与许可页链接 `https://cental.uclouvain.be/cefrlex/flelex/download/`（标明目标文件 `FleLex_TT_Beacco.tsv` 及 `CC BY-NC-SA 4.0` 个人／非商业条款），通过系统浏览器打开；用户本地选择 `.tsv` 导入至 `Lexique-data/`，支持替换与移出。 |
| **CEFR 校验、匹配与隔离** | 导入采用严格结构校验（UTF-8、`<= 5 MiB`、兼容 CRLF/LF、精确 10 列表头 `word..level`、有效 `A1–C2` 值、有限非负频率及唯一 `(word, tag)`），不绑定脆弱的单文件字节哈希。随包仅内置一份 MIT 许可的词性映射、已核验拼写／省音／连字等价关系及 `excluded_sense_ids` 排除表（不含任何 FLELex 等级值），展示全部六级 `A1–C2`，未命中保持空值。CEFR 仅作本机只读展示叠层（核心词典查询、生词本列表／详情、闪卡背面；生词本行对核心摘要取最低基础级，对外部／MDX 记录仅在词头等级无歧义时展示；未导入时不渲染占位），绝不写入 `VocabularyRecord`、`微型兜底摘要`、导出文件或学习备份。 |

## 2. 最小化 LLM 消耗的调研策略

### 2.1 内置核心名词性别提取与分流

1. **阶段 1A：确定性双源提取与自动核验（`0` 次 LLM 调用）**
   - 读取已随仓库固定的 `resources/core-pilot-79/curated-inputs.json`、`selected-source-excerpts.json` 与 `resources/lexique-3.83/Lexique383.tsv.gz`。
   - 从每个 `NOM` 义项已捕获的固定维基词典词头／定义行解析显式语法性别模板（`{{m}}`、`{{f}}`、`{{mf}}`、`{{masculin}}`、`{{féminin}}`、`{{m et f}}` 等），并与 `Lexique 3.83` 同词头 `NOM` 的 `genre` 集合比对。
   - **自动放行条件（队列 A）：** 非同形多性别词头，固定维基词典摘录明确给出单一 `m` 或 `f`，且与 `Lexique 3.83` 一致（或 `Lexique 3.83` 缺失而维基词典摘录唯一明确且无多义性别冲突）。直接继承对应维基词典源引用。
   - **进入残差队列（队列 R）：**
     - `R_conflict`：维基词典标 `mf` 而 `Lexique 3.83` 标单一 `m`/`f`，或两者性别相反；
     - `R_homograph`：同一词头存在不同性别对应不同词义的已知或检测到的同形异义名词（如 `livre`、`tour`、`poste`、`mode`、`voile`、`manche`、`mémoire`、`page`、`somme`、`vase`、`critique`、`garde`、`ombre`、`physique`、`parallèle` 等）；
     - `R_missing_span`：已保存单行 `ipa`／`definition` 摘录未包含词头性别模板（如词头模板位于相邻行，或如 `gens` 标 `{{mf ?|fr}}`）。
2. **阶段 1B：精简表格残差复核（不使用子代理）**
   - 仅将队列 R（预计约 75–110 项，占 2,134 个名词不足 5%）投影为单行紧凑证据记录（`frozen_row | headword | sense_id | definition_fr | use | wikt_span | lexique_genre`），结合固定源页上下文按 Q6 规则一次性给出结构化性别判定与来源行坐标，交人工独立复核后合入。

### 2.2 FLELex `TT-Beacco` 对齐与多义安全规则集

1. **阶段 2A：结构校验器与确定性 `(lemma, POS)` 关联（`0` 次 LLM 调用）**
   - 校验器核对 10 列表头 `word, tag, freq_A1, freq_A2, freq_B1, freq_B2, freq_C1, freq_C2, freq_total, level`，兼容 `\r\n` 与 `\n`，拒绝超过 `5 MiB`、缺 `level` 列（如 8 列 `FleLex_TT.csv`）、非法等级、负值／NaN 频率或重复 `(word, tag)` 的输入。
   - 建立 `core-162-v1` 的 21 类 `lexique_cgram` 到 FLELex TreeTagger 标签（`NOM`、`VER`、`ADJ`、`ADV`、`INT`、`PRO`、`PRP`、`PRP:det`、`KON`、`DET:POS`、`DET:ART`）的确定性映射，并复用已审核的拼写／省音／连字等价关系（如 `aujourd'hui` ↔ `aujourd’hui`、`oeil` ↔ `œil`、`coeur` ↔ `cœur`）。
   - 自动排除 25 个零字段注记组及 3 个已批准专业义（`ça/NOM`、`par/NOM`、`pars/NOM`）。
2. **阶段 2B：多义与受限义项筛查（最小残差批处理）**
   - 对命中 FLELex 等级但 `use` 带有特定领域／语域限制或同词头存在多个义项的条目进行筛选，仅将存在“基础词元等级误标到次要／受限义项”风险的候选送入单次紧凑复核，产出只含 `sense_id` 的静态排除列表 `excluded_sense_ids`（MIT 许可，零条 FLELex 等级数据进入仓库）。

## 3. 实施前两阶段试跑设计（Pre-Implementation Test Run）

在修改任何 Rust 或前端应用代码之前，按以下两阶段完成试跑并提交实测报告：

### 阶段一：全量确定性干跑（`0` 次 LLM 调用）

- **范围：** 覆盖 `core-162-v1` 全部 **4,087** 个核心组（其中 **2,134** 个已准入 `NOM` 组做性别提取与分流；全部 **4,062** 个已准入组做本地只读 `FleLex_TT_Beacco.tsv` 匹配模拟）。
- **验证内容：**
  1. 校验器单元测试：正常 CRLF 与转换后 LF 通过；超限大小、8 列 `FleLex_TT.csv` 表头、非法等级、负频率、重复 `(word, tag)` 明确报错。
  2. 名词性别自动通过数、各残差子队列（`R_conflict`、`R_homograph`、`R_missing_span`）精确数量与代表样本。
  3. FLELex 各词性精确命中率、拼写别名恢复数量、自动排除的专业义／注记组，以及需检查多义适配性的候选规模。
  4. 纯 Python 确定性处理的冷／热执行耗时（毫秒级计量）。

### 阶段二：分层 25 组残差试点（`0` 子代理，最小语义载荷）

选取固定 **25 个代表性核心组** 走完完整“确定性分流 → 残差语义判定 → 契约与来源回读校验”闭环：

| 分层类别 | 组数 | 试点样本（来自 `core-162-v1`） | 检验目标 |
| --- | ---: | --- | --- |
| 1. 常规双源一致名词对照 | 4 | `maison/NOM`、`temps/NOM`、`eau/NOM`、`monde/NOM` | 确认 `0` 模型调用自动通过并继承固定维基归属 |
| 2. 双性／职业名词 `mf` vs 单一性别 | 5 | `enfant/NOM`、`élève/NOM`、`docteur/NOM`、`médecin/NOM`、`flic/NOM` | 检验 Q6 规则：同形双性保留 `mf`，来源定位可回读 |
| 3. 性别区分词义的同形异义名词 | 5 | `livre/NOM`、`tour/NOM`、`poste/NOM`、`mode/NOM`、`voile/NOM` | 检验性别严格绑定 `core-162-v1` 所选义项，绝不混用另一性别义 |
| 4. 摘录行缺失或特殊配合名词 | 4 | `gens/NOM`、`professeur/NOM`、`camarade/NOM`、`coupable/NOM` | 检验源页上下文补证，以及 `gens/NOM` 保持 `null` 并依赖 `usage` 说明 |
| 5. 拼写别名、多词性与专业义 FLELex 边界 | 4 | `coeur/NOM`、`aujourd'hui/ADV`、`par/PRE`、`par/NOM` | 检验连字／直弯撇号匹配恢复，以及 `par/NOM` 高尔夫专业义自动抑制等级 |
| 6. 多义词义项适配与非名词对照 | 3 | `ferme/ADJ`、`ferme/NOM`、`être/AUX` | 检验多词性独立映射、多义词等级适用性判断，以及非名词 `gender = null` |

### 试跑验收门槛

1. **零违规数据落地：** 试跑脚本和报告不向 Git 写入任何 `FleLex_TT_Beacco.tsv` 等级表副本；产物仅记录统计摘要、规则集与 25 组核验结果。
2. **契约与来源 100% 可回读：** 试点中所有非空 `gender` 均能回读固定源文件行与许可，非名词及 `gens/NOM` 为 `null`；专业义不出现误绑定的基础 CEFR 等级。
3. **成本与耗时透明记录：** 分别报告确定性脚本耗时、残差证据包字节数、模型推理次数（主代理直接单次完成，`0` 子代理）及自审状态（标明 `self_review_only`，合入正式资源前仍经人工复核）。
4. **停止边界：** 试跑完成并通过自动化测试与文档链接检查后立即停下，向用户汇报实测数据，获确认后再进入全量残差收尾与 Rust／React 实现。实测结果与五项边界发现见 [实施前两阶段试跑报告](evidence/cefr-gender-pilot-20261127.md)。
