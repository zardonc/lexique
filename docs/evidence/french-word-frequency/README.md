# 生词本自动词频标色：来源与统计标准

访问与核验日期：2026-11-08。状态：来源调查与产品标准建议；尚未接入或完成随包验收。

建议第一版固定采用 **Lexique 3.83 的字幕词元频率**，按标准词头合计各词性，再分为高频／中频／低频／未知。它是有作者、语料说明和公开文献的心理语言学统计资源；标色阈值是本产品明确采用的标准，不是法国官方教学标准或 CEFR 等级。

## 来源、版本与许可

- Lexique 由 Boris New、Christophe Pallier 等研究者维护；[3.83 数据说明](https://chrplr.github.io/openlexicon/datasets-info/Lexique383/README-Lexique.html)列出作者、相关论文及约 140,000 个法语项目，包含屈折形式与同形词性记录，不能解释为 140,000 个独立词元。[版本元数据](https://raw.githubusercontent.com/chrplr/openlexicon/master/datasets-info/_json/Lexique383.json)明确 `name=lexique383`、TSV 文件及原始体积 25,850,780 字节；2019-06-04 是该元数据的时间戳。
- [Lexique 3 手册](https://www.lexique.org/_documentation/Manuel_Lexique.3.2.pdf)署名研究单位包括 LPNC／CNRS UMR 5105／Université Savoie Mont Blanc 与认知神经影像研究单位／INSERM／CEA。手册描述字幕语料约 9,474 部电影或剧集季、约 5,000 万词，小说语料为 1950–2000 年的 218 部文本、约 1,470 万词；[2007 年原作者论文](https://raw.githubusercontent.com/chrplr/openlexicon/master/datasets-info/Lexique382/New.Brysbaert.Veronis.Pallier.2007.APU.pdf)以约 5,200 万词描述字幕语料。保留各自口径，不把这些近似数写成实测精确计数。
- [3.83 数据说明](https://chrplr.github.io/openlexicon/datasets-info/Lexique383/README-Lexique.html)单独声明数据为 CC BY-SA 4.0；[原作者主页](https://www.lexique.org/?page_id=790&lang=en)也明确链接该许可。[许可原文](https://creativecommons.org/licenses/by-sa/4.0/legalcode.en) §2(a)、§3、§4 允许复制、转换、抽取及再分发，没有非商业限制；须保留署名、来源、许可与修改说明，并按适用条款对衍生数据实行相同方式共享。应用／转换器源码的 MIT 与数据许可分别管理，符合 [ADR 0002](../../adr/0002-source-and-resource-licensing.md) 的资源政策。
- 原作者已于 [2026-04-27 发布 Lexique 4](https://www.lexique.org/?p=953&lang=en)，采用更大的字幕语料。本建议有意锁定已核实字段与分组语义的 3.83，不能称为最新版本；改用 4 需重新确认字段、映射与阈值结果。
- [FLELex 官方下载页](https://cental.uclouvain.be/cefrlex/flelex/download/)当前声明 CC BY-NC-SA 4.0，含非商业限制，不能依现行 ADR 0002 纳入默认包。

## 指标与字段

以下列名由[原始 `Lexique383.tsv`](https://www.lexique.org/databases/Lexique383/Lexique383.tsv)前 8,192 字节直接核实；定义依据[手册 §5.1](https://www.lexique.org/_documentation/Manuel_Lexique.3.2.pdf)。所有频率的单位均为次／每百万词。

| 字段 | 含义与用途 |
| --- | --- |
| `ortho` | 收录的表面词形；同一拼写可能有不同词元／词性行 |
| `lemme` | 词元，如动词不定式；用于关联标准词头 |
| `cgram` | 该行的词性，包含 `NOM`、`ADV`、`VER`、`AUX` 等 |
| `freqlemfilms2` | 字幕语料的词元／词性频率，已汇总相应屈折形式；选作标色依据 |
| `freqlemlivres` | 小说语料的词元／词性频率；作为另列参考，不与字幕值混合 |
| `freqfilms2`、`freqlivres` | 相应语料中的表面词形／词性频率；不用来替代词元指标 |
| `islem` | 该表面形式是否是词元；不能据此过滤整份词元索引 |
| `nbhomogr`、`cgramortho` | 同形项目数量及该拼写的词性集合；不代替 `lemme + cgram` 分组 |

`freqlemfilms2` 自 3.40 起基于更新字幕语料及子语料频率平均，不能把它称为小说与字幕的合并原始计数。[手册 §2.2、§5.1](https://www.lexique.org/_documentation/Manuel_Lexique.3.2.pdf)。

## 标准词头合计规则

[原作者 2007 论文的 “Comparison with written corpora” 段](https://raw.githubusercontent.com/chrplr/openlexicon/master/datasets-info/Lexique382/New.Brysbaert.Veronis.Pallier.2007.APU.pdf)分别定义 `play[noun] + plays[noun]` 与 `play[verb] + plays[verb] + played[verb]` 的词元频率。原始 3.83 有同词元、不同词性且频率不同的实值，证明选定字段尚未跨词性合计：

| `ortho` | `lemme` | `cgram` | `freqlemfilms2` |
| --- | --- | --- | ---: |
| a priori | a priori | ADV | 1.04 |
| a priori | a priori | NOM | 0.41 |
| abaissa | abaisser | VER | 4.93 |
| abaisser | abaisser | VER | 4.93 |

为适配[同标准词头一份生词记录](../../../GLOSSARY.md)，本产品定义 `F(词头) = Σ 各不同 cgram 的一份 freqlemfilms2`。转换时先按 `(NFC(lemme), cgram)` 分组，组内各屈折行必须给出一致、有限且非负的值，每组只取一次，再合计属于同一词头的全部词性。`a priori = 1.04 + 0.41 = 1.45` 次／百万词；`abaisser` 仍为 `4.93`，不把重复屈折行相加。该跨词性合计是产品派生指标，保留原始分组与来源以便查看。

不只取 `islem=1`：手册明确有些词元只有屈折形式出现，没有独立原形行。缺值、无效值、组内冲突或尚未可靠确定词元时为“未知”，保留可用候选；选中文本或 OCR 的 `a` 不能自动把名词 `a` 与动词词元 `avoir` 合并。

查找资源时可使用 NFC／小写搜索键，已保存标准词头身份保持不变，重音保留。原始小样已有 `a jeun → à jeun` 的明示词元映射和 `a l'instar` 的直撇号；只接受来源明确的变体关联，不自行去重音、替换撇号、拆合多词表达或猜测屈折原形。来源没有相应映射时为“未知”。

## 本产品分档建议

| 自动标签 | 合计字幕词元频率 `F`（次／百万词） | 颜色语义 |
| --- | --- | --- |
| 高频 | `F ≥ 100` | 绿 |
| 中频 | `10 ≤ F < 100` | 蓝 |
| 低频 | `0 ≤ F < 10` | 橙 |
| 未知 | 缺失、无效、冲突或词元未确定 | 中性灰 |

标签始终显示文字，颜色按现有主题落实对比度；`a priori` 属低频。来源记录的 `0.00` 是有效的低频值，可能涉及四舍五入或仅在另一语料出现；无匹配不是零。100／10 的切点及颜色均为产品选择，不宣称原作者给出了这组三档，也不推导 A1–C2、必学等级、学习者熟练度或复习优先级。

## 限制与接入门槛

- 字幕包含翻译作品和题材偏差，小说反映书面语；两者都不是当代自发口语的完整抽样。固定 3.83 的缺词、专有名词与多词表达覆盖不能由约 140,000 项的总数保证。[手册 §2–§5](https://www.lexique.org/_documentation/Manuel_Lexique.3.2.pdf)、[2007 论文](https://raw.githubusercontent.com/chrplr/openlexicon/master/datasets-info/Lexique382/New.Brysbaert.Veronis.Pallier.2007.APU.pdf)。
- 3.83 README 引用的同目录许可文件链接当前返回 404；本次依据版本声明、原作者主页与可访问的 CC 法律原文确认许可。接入前须固定完整合法来源、保留版本／署名／许可原文，完成全量分组一致性、重音／撇号／同形与无匹配验证，并备齐转换及再分发通知，按[开发准备清单](../../development-readiness.md)验收。
- 本次只请求原始 TSV 的 `bytes=0-8191`，服务器返回 `206` 及 `Content-Range: bytes 0-8191/25850780`；没有下载完整词表或 ZIP，没有转换、运行应用或核验最终包。前端／Rust／原生检查未运行；本文件仅有来源、小样与文档检查证据。
- 文档核验：`git diff --no-index --check -- NUL docs/evidence/french-word-frequency/README.md` 无空白诊断（新增文件差异退出 1）；PowerShell 回读、空白及三条本地链接检查 PASS；原始小样列名、20 个 `abaisser/VER` 行的一致值与 `a priori` 合计自检 PASS。
