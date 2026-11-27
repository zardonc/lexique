# 自选导入 CEFR 分级与内置核心名词性别：实施前两阶段试跑报告

全量收尾的固定来源、187 条原始残差及七条新增例外见[复核表](noun-gender-residual-review.md)。
应用实现及检查见[当前验证记录](../verification.md#自选-cefr-与名词性别)；以下保留实施前试跑的数据。

日期：2026-11-27。基线：`origin/main`（`136c43f`，`core-162-v1`）。对应方案见 [实施前试跑方案](../cefr-option2-and-core-gender-research-plan.md) 与 [ADR 0012](../adr/0012-user-imported-cefr-grading-overlay.md)，验证脚本见 [cefr_gender_pilot.py](../../scripts/cefr_gender_pilot.py) 与 [test_cefr_gender_pilot.py](../../scripts/test_cefr_gender_pilot.py)。

## 1. 执行范围与合规确认

- **零 `CC BY-NC-SA 4.0` 数据入库：** 全量试跑在本地只读读取未跟踪样本 `FleLex_TT_Beacco.tsv`（`718,413 B`，`14,236` 行），仓库跟踪文件与本报告均未写入任何词条 `A1–C2` 等级表副本；产物校验器 `assert_zero_nc_levels_in_tracked_payload` 已做强断言拦截。
- **模型调用控制：** 阶段一全量干跑模型调用数为 **`0`**；阶段二 25 组试点中 **11 组**由确定性规则直接完成，仅 **14 组**残差项汇总为单份紧凑证据包（`6,228 B`），由当前主代理单次直接完成语义核对（**`0` 个子代理**）。
- **复核状态声明：** 阶段二残差判定当前状态为 `self_review_only`（`independent_human_review: pending`），尚未修改 `crates/`、`apps/` 或正式词典输入文件。

## 2. 阶段一：全量确定性干跑结果（`0` 次 LLM 调用）

### 2.1 运行耗时（Python 3.14 单进程实测，7 轮统计）

| 步骤 | 冷启动耗时 | 热运行中位数 | 说明 |
| --- | ---: | ---: | --- |
| 固定词典输入与 `Lexique 3.83` 加载 | `1,187.02 ms` | — | 读取 `curated-inputs.json`、`selected-source-excerpts.json`、`Lexique383.tsv.gz` |
| `FleLex_TT_Beacco.tsv` 结构校验 | `57.34 ms` | `57.55 ms` | 校验 `718,413 B`、`14,236` 唯一 `(word, tag)`，CRLF 与 LF 均通过 |
| 内置核心名词性别全量分流 | `20.46 ms` | `18.91 ms` | 覆盖全部 `4,062` 个已准入核心义项（含 `2,134` 个 `NOM`） |
| FLELex 词性映射与拼写别名关联 | `19.03 ms` | `20.33 ms` | 覆盖全部 `4,062` 个已准入核心义项及专业义／多义排除表 |

### 2.2 内置核心名词性别全量分流统计

| 分类队列 | 义项数 | 占 `2,134` 个 `NOM` 比例 | 说明 |
| --- | ---: | ---: | --- |
| 非名词与零字段注记（直接置 `null`） | `1,928`（另含 `25` 个注记组） | — | 非 `NOM` 义项严格保持 `gender = null` |
| 自动通过：固定维基词头模板 + `Lexique 3.83` 一致 | `1,669` | `78.21%` | 词头摘录含显式 `{{m}}`／`{{f}}` 且非同形异义冲突 |
| 自动通过：非同形异义 `Lexique 3.83` + 已复核 `use` 一致 | `278` | `13.03%` | 批次 `#99`–`#108` 中 `ipa` 摘录按列切片为 `{{pron\|...}}`，正式合入前可按固定 `oldid` 回填整行词头以统一源段回放 |
| **确定性自动通过合计** | **`1,947`** | **`91.24%`** | **`0` 次模型调用** |
| 残差队列 `R_conflict`（`{{mf}}` 或跨源不一致） | `91` | `4.26%` | 如 `enfant`、`élève`、`docteur`、`médecin`、`flic` |
| 残差队列 `R_homograph`（阴阳同形异义／双性词元） | `47` | `2.20%` | 如 `livre`、`tour`、`mode`、`voile`、`mort`、`espace`、`politique`、`somme`、`couple` |
| 残差队列 `R_missing_or_uncertain_span`（摘录切片或存疑标记） | `49` | `2.30%` | 含 `gens`（`{{mf ?\|fr}}`）及 `48` 条列切片名词；其中 **`18` 条**已从本地归档 `.wiki` 源页零网络回读整行，仅剩 **`30` 条**需按固定 `oldid` 补抓词头行 |
| **残差复核合计** | **`187`** | **`8.76%`** | 全库仅 `187` 条需进入紧凑表格复核 |

### 2.3 FLELex `TT-Beacco` 全量确定性对齐统计

- **结构校验器（7 项单测全过）：** 兼容 `\r\n` 与 `\n`；对空文件、超 `5 MiB`、8 列无等级表头（`FleLex_TT.csv`）、非法等级值、负频率及重复 `(word, tag)` 均返回明确错误。
- **覆盖与冲突统计（共 `4,062` 个已准入义项）：**
  - 多词性标签冲突数：**`0`**（21 类 `lexique_cgram` 映射至 11 类 TreeTagger 标签无任何同词跨标签等级冲突；`la/ART:def` 不误借 `PRO` 或音符 `NOM`；`être/AUX`、`avoir/AUX` 准确落入动词类）。
  - 拼写／省音／连字等价恢复数：**`17`**（如 `coeur` ↔ `cœur`、`aujourd'hui` ↔ `aujourd’hui`、`quelqu'un` ↔ `quelqu’un`）。
  - 自动抑制的专业／受限义项命中数：**`3`**（高尔夫专业义 `par/NOM` 行 128、心理／自我概念义 `moi/NOM` 行 83、现代俚语义 `ben/NOM` 行 1070 在原始 FLELex 中均有同名 `NOM` 等级行，经 `excluded_sense_ids` 准确拦截；另预置排除 `ça/NOM` 行 34 与 `pars/NOM` 行 3128，合计静态排除 `5` 个 `sense_id`；`blé/NOM` 行 2727 核实为日常“小麦”本义，保留匹配）。
  - 最终有效匹配义项：**`3,657 / 4,062`（`90.03%`）**；未命中保持空值：**`402`（`9.90%`）**；规则排除：**`3`（`0.07%`）**。

## 3. 阶段二：分层 25 组残差试点结果

### 3.1 载荷压缩与契约核验

- 25 组原始 `curated-inputs` JSON 体积：**`83,444 B`**
- 筛除 11 组确定性通过项后，14 组残差单行证据包体积：**`6,228 B`**（**体积压缩 `92.54%`**，缩小 `13.4` 倍）
- 契约与来源回放校验（`verify_pilot_decisions`）：**`25 / 25` 通过（`0` 项违规）**

### 3.2 25 组逐项核验结果摘要（不含 FLELex 等级值）

| 冻结行 | 词头／词性 | 分流队列与处理方式 | `gender` 判定 | CEFR 叠层资格 | 证据与边界说明 |
| ---: | --- | --- | --- | --- | --- |
| 252 | `maison/NOM` | 确定性自动通过 | `f` | 允许（命中） | 维基 L9 `{{f}}` + `Lexique 3.83` 词干复数 `maisons=f` 一致 |
| 89 | `temps/NOM` | 确定性自动通过 | `m` | 允许（命中） | 维基 L10 `{{m}}` + `Lexique 3.83` `m` 一致 |
| 261 | `eau/NOM` | 确定性自动通过 | `f` | 允许（命中） | 维基 L9 `{{f}}` + `Lexique 3.83` `f` 一致 |
| 107 | `monde/NOM` | 确定性自动通过 | `m` | 允许（命中） | 维基 L10 `{{m}}` + `Lexique 3.83` `m` 一致 |
| 164 | `enfant/NOM` | 残差 `R_conflict` | `mf` | 允许（命中） | 维基 L11 `{{mf}}`，所选义项明确阴阳同形 |
| 1581 | `élève/NOM` | 残差 `R_conflict` | `mf` | 允许（命中） | 维基 L9 `{{mf}}`，学习者义项阴阳同形 |
| 367 | `docteur/NOM` | 残差 `R_conflict` | `mf` | 允许（命中） | 维基 L9 `{{mf}}`；原拼写 `docteur` 可配 *un/une* |
| 566 | `médecin/NOM` | 残差 `R_conflict` | `mf` | 允许（命中） | 维基 L9 `{{mf}}`；原拼写 `médecin` 可配 *un/une* |
| 669 | `flic/NOM` | 残差 `R_conflict` | `m` | 允许（命中） | 维基词头标 `{{mf}}`，但 `core-162-v1` 的 `use` 明确限定所选阳性用法，按义项定为 `m` |
| 551 | `livre/NOM` | 残差 `R_homograph` | `m` | 允许（命中） | 所选为“书”（维基 L11 `{{m}}`），排除阴性“磅／镑” |
| 343 | `tour/NOM` | 残差 `R_homograph` | `f` | 允许（命中） | 所选为建筑“塔”（维基 L160 `{{f}}`），排除阳性“圈／轮” |
| 1044 | `poste/NOM` | 残差 `R_missing_span` | `m` | 允许（命中） | 所选为“职位”（`use` 明载阳性），L149 切片待补抓整行词头以回放 `{{m}}` |
| 1244 | `mode/NOM` | 残差 `R_homograph` | `f` | 允许（命中） | 所选为“时尚／潮流”（维基 L9 `{{f}}`），排除阳性“方式／语式” |
| 2005 | `voile/NOM` | 残差 `R_homograph` | `f` | 允许（命中） | 所选为“船帆”（维基 L123 `{{f}}`），排除阳性“面纱” |
| 253 | `gens/NOM` | 残差 `R_missing_span` | `null` | 允许（命中） | 维基 L10 `{{mf ?\|fr}}`，复数配合随语境变化，保持 `null` 并由 `usage` 说明 |
| 929 | `professeur/NOM` | 残差 `R_missing_span` | `mf` | 允许（命中） | 从归档固定源页 L8 精确回读 `'''professeur''' {{pron\|...}} {{mf}}` |
| 1101 | `camarade/NOM` | 残差 `R_missing_span` | `mf` | 允许（命中） | 从归档固定源页 L11 精确回读 `'''camarade''' {{pron\|...}} {{mf}}` |
| 1165 | `coupable/NOM` | 残差 `R_missing_span` | `mf` | 允许（命中） | 从归档固定源页 L87 精确回读 `'''coupable''' {{pron\|...}} {{mf}}` |
| 483 | `coeur/NOM` | 确定性自动通过 | `m` | 允许（别名命中） | 维基 `cœur` L12 `{{m}}`；FLELex 通过 `coeur` ↔ `cœur` 关联命中 |
| 274 | `aujourd'hui/ADV` | 非名词置 `null` | `null` | 允许（别名命中） | 副词 `gender = null`；撇号规范化正常命中 |
| 129 | `par/PRE` | 非名词置 `null` | `null` | 允许（命中） | 介词 `gender = null`；`PRP` 精确命中 |
| 128 | `par/NOM` | 确定性自动通过 | `m` | **抑制排除** | 高尔夫专业名词性别为 `m`（维基 L161 `{{m}}`），`excluded_sense_ids` 成功拦截同名 `NOM` 等级 |
| 899 | `ferme/ADJ` | 非名词置 `null` | `null` | 允许（命中） | 形容词 `gender = null`；与 `ferme/NOM` 独立匹配各自词性等级 |
| 900 | `ferme/NOM` | 确定性自动通过 | `f` | 允许（命中） | 维基 L139 `{{f}}`（农场）；生词本合并展示按 Q8 取两词性中更基础一级 |
| 1 | `être/AUX` | 非名词置 `null` | `null` | 允许（命中） | 助动词 `gender = null`；精确映射 `VER` 而不误取 `être/NOM` |

## 4. 试跑提前暴露并解决的 5 个关键边界问题

1. **词头 `{{mf}}` 与所选单性别义项不一致（`flic/NOM`）：** 若直接相信词头 `{{mf}}` 会误标已限定阳性义项的词条；将全部维基 `{{mf}}` 路由至 `R_conflict` 并与 `use` 核对，准确区分了真双性词（`enfant`、`docteur`、`professeur`）与单性别限定词（`flic`）。
2. **部分中间批次 `ipa.source_span` 存在列切片（`column_1_based > 1`）：** 批次 `#99`–`#108` 部分词条仅截取了 `{{pron|...}}` 子串，切掉了同行右侧的 `{{m}}`／`{{f}}`／`{{mf}}`。通过 `(line_1_based, column_1_based)` 精确子串校验回读整行词头，直接从本地归档 `.wiki` 恢复了 `professeur`、`camarade`、`coupable` 等 18 条，定位出剩余 30 条待按固定 `oldid` 补齐整行的残差项。
3. **`Lexique 3.83` 单数空 `genre` 语料现象（`maison`、`main`、`voiture` vs `doctoresse`）：** `Lexique 3.83` 因同位不变用法（如 *un plat maison*）将单数 `maison` 的 `genre` 置空，而复数 `maisons` 标 `f`；同时把异形阴性词 `doctoresse`、`professeure` 归在阳性词元下。限定仅在同词干规则复数（`{lemme, lemme+'s', lemme+'x'}`）回填词元性别后，`24` 个常规单性别名词全部恢复自动通过，且未引入异形派生污染。
4. **无词性限定的拼写关系项（`quelqu'un`）：** `selected-source-excerpts.json` 中个别拼写等价项省略了 `lexique_cgram`；同时查询 `(headword, pos)` 与 `(headword, None)` 后完整恢复 `17` 条省音／连字匹配。
5. **专业义与受限义项在 FLELex 中的真实同名碰撞（`par/NOM`、`moi/NOM`、`ben/NOM`）：** 实测确认 `FleLex_TT_Beacco.tsv` 确实含有 `par/NOM`、`moi/NOM`、`ben/NOM` 等级行；若无静态 `excluded_sense_ids` 排除表，高尔夫“标准杆”、哲学“自我”与现代俚语“ben”都会错误展示基础教材等级。

## 5. 进入正式实现前的收尾清单（待用户确认后启动）

1. **全量名词性别残差收尾（仅 `187` 条）：** 按固定 `oldid` 补齐剩余切片词头行，一次性产出 `187` 条残差判定表并经人工独立复核后写入 `curated-inputs.json`／核心词典构建产物。
2. **Rust 与前端实现（遵循 Q1–Q10 及 [ADR 0012](../adr/0012-user-imported-cefr-grading-overlay.md)）：**
   - 核心词典构建与 `CoreSense`／微型兜底摘要新增可选 `gender`（既有记录与 v1–v4 备份零改动；识义卡正面不显示性别）；
   - 设置页提供 UCLouvain 官方下载链接与本机 `FleLex_TT_Beacco.tsv` 导入／替换／移出及结构校验，运行时作为只读内存／本地缓存叠层在词典、生词本与卡片背面展示 `A1–C2`，与用户学习数据、导出和备份完全隔离。
