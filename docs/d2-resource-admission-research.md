# D2 默认词典与分级来源调查

调查日期：2026-11-09；资源契约、扩展质量、来源链、替代定义边界、完整表达分级、XML 字段、印刷对照、新增法语分级来源及自定义 MDX/MDD 筛选复核：2026-11-10（America/Vancouver）。基线：`ea2c95ec099388642c2aef5e68a3159cc86f4479`。
依据 [#9 的 D2 切片](lookup-enhancement-slices.md#d2有来源的完整核心摘要)、[产品基线](product.md)及 [ADR 0002](adr/0002-source-and-resource-licensing.md)。
FreeDict 法英补充 MDX 与自选链接目录接入验证：2026-11-11（UTC），见[产品接入证据](evidence/d2-mdx/README.md)。

## 结论

默认词典以**法语维基词典的原始结构化文字**作为主候选：原创正文 CC BY-SA 4.0 允许转换和商业再分发。固定原 27 查询及按 Lexique 3.83 字幕词频独立选取的另 30 查询，已完成 **57 查询／56 个词头的来源审核摘要**；完整法语定义及所选词头 IPA 为 **57／57**，匹配英语译词为 **49／57**、中文为 **30／57**，CEFR 均未知。摘要保留完整定义及限定，译词按源页含义与语法记录选择。[法语版原始数据](https://kaikki.org/frwiktionary/rawdata.html)、[法语维基词典版权约定，修订 38903168](https://fr.wiktionary.org/w/index.php?title=Convention%3ADroit_d%E2%80%99auteur&oldid=38903168)。

两组的 **143 个法语记录／744 个义项**均与单词 JSONL 完整对象相同；官方 dump 已恢复 56 个词头页及四项明确重定向。全部记录完成词头 IPA 来源核对，139 条有明确值、四条保持未知。扩展审核确认首记录选择、词形指向、跨记录翻译错置及例句误作定义的问题，并保存八项来源支持的词形关系。56 页含 15 项导入通知；12 个 DAF8 标记页的所选定义已完成原作及修订核对，**6 项支持 CC BY-SA 4.0 贡献者文字重用，5 项 DAF8 表达的目标再分发依据未闭合，1 项 TLFi 表达的公开条款明确排除商业利用**。详见[扩展质量](#独立词频扩展30-查询)及[文字许可](#文字提取器和媒体许可)。

**XMLittré 为六个待处理词提供了明确采用 CC BY-SA 3.0 的历史定义来源。**固定 README 与完整 XML 头部均覆盖带标记数据，允许按条件转换及商业再分发；六词七项原文跨度、核心义及语法功能已作有界核对，`cote` 的范围较窄，`avoir` 分成物质与非物质／状态义。19 份所需字母文件支持 **51／57 查询、50／56 标准词头**的大小写或源标题显式对应；发音为历史拼读，匹配条目没有明确 IPA、译词或 CEFR 字段。七项起句的印刷可视对照已完成 **3／7**，`avoir` 两项与 `pouvoir` 一项的原文及限定一致。详见[替代定义与字段覆盖](#xmlittré有明确许可的历史替代定义)。

审核摘要适合继续采用**独立 SQLite 中的结构化 JSON**：保留完整定义、语法／义项限定、译词语言与传统写法，以及逐词来源引用。56 条摘要／四项明确重定向已通过 57 查询及资源／许可／压缩包回读；四文件研究包共 **336,420 字节**，tar.gz 为 **35,891 字节**。固定生产者代码及其声明的 dump／词头身份支持独立推导官方修订引用；提取输出的重现另行验证。该包用于研究契约，现有应用仍读取 TEI XML，未标记为已准入默认资源。见[资源契约与来源通知](#资源契约与来源通知)。

FreeDict 法英 0.4.1 可继续承担已有的部分英语译义；法中 2025.11.23 保留既有固定小样角色。两者都有常用查询缺项和义项质量限制，不能单独支撑完整 D2。WikDict 已在 2026-06 修正生成器的旧 3.0 许可模板，并有可固定的 4.0 新构建路线；公共 TEI 仍保留 2025 年的 3.0 头部，现有 FreeDict 包的底层快照仍需单独闭合。[官方当前下载](https://freedict.org/downloads/)、[既有英语准入](evidence/issue-42/README.md)、[法中固定来源](../resources/freedict-fra-zho-2025.11.23/provenance.json)、[生产者许可修正](https://github.com/karlb/wikdict-gen/commit/d6eb584643df2dca0a5429d26847612431b48f7d)。

本轮候选仍未同时满足完整 A1–B2 法语分级依据和目标再分发权利。FLELex 有 NC 限制；MIT 标签候选是句子难度；CEFR-J x28 转译英语等级，法语授权未闭合；Vocabcraft 明示使用词频启发式；DIALANG 是能力测试。PolyLexFLE 完整固定数据含 4,532 行，主等级列有 817 个有效等级值，但对本轮 57 查询的标准词头精确覆盖为 **0／57**；法语表达方法有有限验证。原始 Lexique-Grammaire 表格已建立按 LGPLLR 修改及商业随包的条件路线，PolyLexFLE 新增数据与 LGPL-2.1／LGPLLR 的组合授权仍未闭合。新增 Mauril FEC1–FEC3 提供真实法语词／表达在语境中的分级评估方法，未找到固定语料发布及目标数据授权；Eaquals 2015 为非穷尽课程清单，LoadWords 的方法未公开且条款限制目标使用。**默认词典主候选可继续逐项准入；CEFR 缺项保持未知。**详见[新增分级来源](#新增法语分级来源)及各来源条款。

MDX 分发范围为**仅选择 1–2 份法语相关词典随包，其余候选只内置来源／下载页链接供用户自选**。当前 Windows x64 开发包已接入一份 FreeDict 法英 0.4.1：明确 GPL-2.0-or-later，**273,758 B、8,505 条记录**的无媒体 MDX，CLI 完整读取、逐字节重建、原生查询／来源保存和通知随包通过。其原内容为 **2018-08-11**，适合作为有缺项的法英补充；正文较旧，不能承担完整 D2 摘要。较新的 WikDict 法英／法汉 2026-06-23 包也已转换并完整读取，但部分 DAF8 表达的目标再分发依据仍未闭合，暂保留候选。详见[法语随包范围](#法语随包范围与用户自选链接)。

用户自选资源中，**法汉 LarousseFC 与英英 OALD10 可优先做正文试读**：公开目录分别显示约 7.9 MB 与 39.08 MiB 的 MDX。Collins／Larousse 2024 网页包的单 MDX 大小未知；英汉社区包包含混合译文，OALDPE 含 Google 补译，朗文 Amazon 英英原包为 154.8 MiB、超过上限。Tatoeba 官方提供周更 TSV，可靠现成 MDX 未确认；同近反义优先现行 Larousse 网页，2004 MDX 为旧备用。八类选择及未知值见[资源列表](#自定义-mdxmdd-词典与相关资源)。设置已接入八类、12 项来源／下载页链接；其他候选未随包或自动下载，完整 D2／CEFR 的准入状态继续保留。

## 候选比较

以下许可与发布页在 2026-11-09–10 UTC 核对；数量沿用其注明的统计口径。

| 来源／固定版本 | 内容与字段 | 转换及随包许可 | D2 判断 |
| --- | --- | --- | --- |
| [FreeDict fra-eng 0.4.1](https://download.freedict.org/dictionaries/fra-eng/0.4.1/) | 法语词头、IPA、词性、英语短译词；8,505 TEI 记录；没有 `def` | GPL-2.0-or-later；保留通知、许可和对应源数据／修改 | 已有部分英语补充继续使用；定义、例句和覆盖不足 |
| [FreeDict fra-zho 2025.11.23](https://download.freedict.org/dictionaries/fra-zho/2025.11.23/) | 中文译词，部分法语定义；10,947 TEI 记录；无例句引用 | 包声明 CC BY-SA 3.0；当前上游 4.0，底层快照未固定 | 固定小样保持；扩大准入需解决版本链与质量 |
| [Kaikki 法语来源版](https://kaikki.org/frwiktionary/rawdata.html)，frwiktionary 2026-10-01 | 法语定义／词性／IPA、义项关联、中英译词、例句、屈折关系 | 原创词典文字 CC BY-SA 4.0；提取器代码 MIT；引用／媒体分别核查 | 首选主词典小样；尚未完成默认全量准入 |
| [Kaikki 英语来源版的 French](https://kaikki.org/dictionary/French/index.html)，enwiktionary 2026-10-01 | 法语条目、英语解释；统计包含屈折词形、词性与义项 | 原创正文 CC BY-SA 4.0；提取器 MIT；引用／媒体各自授权 | 可评估英语补充；不能与法语来源版混称 |
| [XMLittré](https://api.bitbucket.org/2.0/repositories/Mytskine/xmlittre-data/src/b44d144d12a153799e2c8f357ea91b049f8d83e2/README.md)，2017-11-16 | 历史释义及语法／拼读；源词头对应 51／57 查询；无明确 IPA、译词或 CEFR 字段 | 带标记数据 CC BY-SA 3.0，署名、通知、修改说明及相同方式共享 | 六词七项定义跨度及核心义已核对；印刷一致性部分通过，完整准入待验收 |
| [FLELex TT／CRF／TT-Beacco](https://cental.uclouvain.be/cefrlex/flelex/download/) | TT 14,236、CRF 17,871 词元／词性条目；Beacco 增加推断等级列 | CC BY-NC-SA 4.0 | 不符合默认资源政策 |
| [vekkt/french_CEFR](https://huggingface.co/datasets/vekkt/french_CEFR/tree/d6affd3ff35c09dc378e32070ca77db32783bb19)，2022-12-13 | `sentence`、`difficulty`、`label`，A1–C2 句子分类 | 卡片只声明 MIT；没有语料权利和标注来源说明 | 不能准入为 CEFR 词表 |
| [TUFS CEFR-J x28 法语样本](https://www.tufs.ac.jp/education/program/sgu/cefr-j_28.html) | 法语／英语／日语／CEFR 列；已取得 A1、A2 各一页 | 商业使用授权明确针对英文 1.6 词表；法语转译稿未确认同等授权 | 同一法语词可因英文拼写获得不同等级；完整法语分级和权利均未闭合 |
| [Vocabcraft 法语词表](https://vocabcraft.com/decks/french)，本轮网页 | 3,407 词；OpenSubtitles 词频排序、自定 CEFR 启发式 | 具体 deck 声明 CC BY-SA 4.0；底层语料权利及固定发布仍需核对 | 可核对词频用途，不能准入为可靠 CEFR 等级 |
| [DIALANG 官方项目](https://wp.lancs.ac.uk/ltrg/projects/dialang-2-0/) | 法语词汇及其他技能测试，以 CEFR 报告受测者能力；旧系统停用 | 未取得可转换、商业再分发的词汇分级数据授权 | 没有可准入的固定词元／义项等级导出 |
| [PolyLexFLE](https://github.com/amaliatodirascu/PolylexFLE/tree/024b1bf63ad2fff8e4499370f3398ea687944d3f)，2024-02-17 | 完整 TSV 为 4,532 行／4,526 个不同表达；主等级列有 817 个有效值；论文统计口径另列 | 原始 LG 表格明确 LGPLLR；仓库 LGPL-2.1，新增层及组合授权未闭合 | 本轮标准词头精确覆盖 0／57；部分表达候选，保留字段异常及等级上下文 |
| [PolyLexFLE 2026 扩展](https://aclanthology.org/2026.readi-1.14/) | 约 2,700 个法语表达，以最低教材出现级标注 | 所查论文／生产者资料未提供可固定的新数据及资源许可 | 方法可评估，数据尚未准入 |
| [Mauril FEC1–FEC3](https://aclanthology.org/2023.bea-1.43/)，2023 | 法语词／表达在语境中的能力评估集；FEC1 为 914 例、FEC2 为 473 项、FEC3 每个 A1–B2 等级 48 项 | 论文 CC BY 4.0；固定语料发布及教材／新增数据的随包权利未建立 | 可参考法语语境方法；不能据此采用孤立词头等级 |
| [Eaquals/CIEP Inventaire](https://www.eaquals.org/wp-content/uploads/Inventaire_ONLINE_full.pdf)，2015 | A1–C1 课程内容、主题及示例，非完整词元／词性分级表 | 官方条款要求复制、分发和修改前取得书面许可 | 示范清单中的单词不自动继承章节等级 |
| [LoadWords 法语页](https://loadwords.com/en/vocabulary/fr/a1/1)，本轮网页 | 按级别列词／词性／IPA／含义；未公开可重现分级方法 | 有限浏览许可；条款限制出版、商业化及数据提取 | 没有可准入的固定完整导出及方法依据 |
| [LanGeek 法语 A1–B2](https://help.langeek.co/cefr-french-vocabulary-pdf) | 网页声明词性、IPA、释义及课程等级 | 2022-05-27 条款限制转换、再分发和商业使用 | 不符合默认资源政策 |

## FreeDict：全源字段与实际质量

官方当前下载仍列出法英 0.4.1 和法中 2025.11.23。本轮读取仓库已固定的完整源归档，按 TEI 标签统计，并用断言核对记录数；没有执行上游转换代码。[法英原始源包](https://download.freedict.org/dictionaries/fra-eng/0.4.1/freedict-fra-eng-0.4.1.src.tar.xz)、[法中原始源包](https://download.freedict.org/dictionaries/fra-zho/2025.11.23/freedict-fra-zho-2025.11.23.src.tar.xz)。

| 全源指标 | 法英 0.4.1 | 法中 2025.11.23 |
| --- | ---: | ---: |
| TEI `entry` 记录 | 8,505 | 10,947 |
| 不同原始 `orth` 词头 | 8,417 | 10,777 |
| 含词性 `pos` 的记录 | 8,231 | 10,528 |
| 含读音 `pron` 的记录 | 8,293 | 10,593 |
| 含翻译引用的记录 | 8,504 | 10,947 |
| 含法语定义 `def` 的记录 | 0 | 3,629（33.2%） |
| 含例句引用的记录 | 1（共 3 个引用节点） | 0 |
| 固定 27 个查询的精确命中 | 21／27 | 18／27 |

记录数、不同词头和字段存在是不同指标；本表不是编辑正确率。TEI 原始词头也与旧 Dictd 的索引键不同，不能将本表的 8,417／10,777 与[历史 Dictd 调查](resource-feasibility.md)的 8,249／10,589 混为增长量。翻译节点未逐条携带 `xml:lang`，翻译语言来自资源元数据。

27 个查询预先覆盖常用词、重音、撇号、同形与短语，**不是 CEFR 基准**。法英缺 `œil`、`cote`、`procrastiner`、`algorithme`、`retentir` 和弯撇号 `aujourd’hui`；法中缺 ASCII 撇号 `aujourd'hui`、`cote`、`côté`、`s'il vous plaît`、`se souvenir`、`procrastiner`、`retentir`、`retenir`、`taire`。精确词头匹配结果不代替搜索归一规则或已验收的动词反查。

具体质量限制来自固定源包：法英 `cœur` 没有读音；`retenir` 含错置的 `thunder` 译义；`taire` 的译词偏向隐匿／隐藏，已有[英语资源说明](../resources/freedict-fra-eng-0.4.1/README.md)保留其代词用法修正边界。法中 `école` 的定义是简短的学习场所解释，`être` 保留三种语境；`cœur` 首条法语定义是情感／道德含义，却伴随“心脏／心”的译词。首条义项不能保证就是学习摘要所需的常用义。

法英包的 GPL-2.0-or-later 是数据许可，项目代码继续 MIT；既有通知、完整源归档、AUTHORS／COPYING 和转换来源路径可以复用。#42 的部分变位英语准入不等于默认词典覆盖准入。[法英许可原文](../resources/freedict-fra-eng-0.4.1/COPYING)、[已有资源证据](evidence/issue-42/README.md)。

法中 TEI 标题为 French-Chinese FreeDict+WikDict，维护／发布者 Karl Bartel，发布日期 2025-11-23，包内明确 CC BY-SA 3.0 Unported。`sourceDesc` 仅指 WikDict 自动创建、Wiktionary 经 DBnary 提供；没有 Wiktionary dump 日期、源页修订或提取日期。`revisionDesc` 的 2017／2018 年记录是生成器变更，不能当作底层词典文字的日期。[固定包来源](../resources/freedict-fra-zho-2025.11.23/provenance.json)、[原始许可](../resources/freedict-fra-zho-2025.11.23/COPYING)。

当前 [WikDict 下载页](https://www.wikdict.com/page/download)的许可链接和 [DBnary 数据集页](https://kaiko.getalp.org/about-dbnary/dataset/)均指向 CC BY-SA 4.0。生产者的[许可修正提交 `d6eb5846`](https://github.com/karlb/wikdict-gen/commit/d6eb584643df2dca0a5429d26847612431b48f7d)（2026-06-22）明确把 TEI／KOReader 的 3.0 字符串及 URL 改为 4.0，并引用 Wiktionary 在 2023 年 6 月的许可变更。固定现行生成器 [`21d067f6`](https://github.com/karlb/wikdict-gen/commit/21d067f61604c01da6285b13853b57c528bb379a)的[TEI 模板](https://github.com/karlb/wikdict-gen/blob/21d067f61604c01da6285b13853b57c528bb379a/src/tei.py#L95)已经声明 4.0。

| 当前发布／输入 | 本轮实测 | 来源链意义 |
| --- | --- | --- |
| [WikDict 推荐法中 TEI](https://download.wikdict.com/dictionaries/tei/recommended/fra-zho.tei) | HTTP 206，前 16 KiB；全文件 5,269,009 字节；最后修改 2025-11-21；edition `2025.11.21`，头部仍为 3.0 | 公共 TEI 尚未随生成器更新；与 FreeDict `2025.11.23` 制品分别记录 |
| [WikDict SQLite `2_2026-06/fr-zh.sqlite3`](https://download.wikdict.com/dictionaries/sqlite/2_2026-06/fr-zh.sqlite3) | HTTP 200；3,248,128 字节；最后修改 2026-06-23；只读 schema 有译词表，没有许可／dump 元数据表 | 可固定的新发布候选，仍缺 release-to-input 清单 |
| [DBnary `fr_dbnary_ontolex_20260601.ttl.bz2`](https://kaiko.getalp.org/static/ontolex/fr/fr_dbnary_ontolex_20260601.ttl.bz2) | HEAD 200；114,720,609 字节；最后修改 2026-06-02 | 可明确指定的新构建输入；不是证明上述 SQLite 使用了它 |

新法中资源可以固定生成器和[日期命名的 DBnary 输入](https://kaiko.getalp.org/static/ontolex/fr/)，另存实际使用的语言版、推断输入、来源页和许可材料。默认 [Makefile](https://github.com/karlb/wikdict-gen/blob/21d067f61604c01da6285b13853b57c528bb379a/Makefile#L54)读取可变 `latest/`，发布目录使用构建时年月；两者都不能还原旧包的底层快照。现有 FreeDict 固定小样保持原状态，新 4.0 构建和它的历史来源清单分别验证。

## 法语维基词典／Kaikki：推荐路线

法语版原始数据页固定的是 **frwiktionary dump 2026-10-01**，网页注明提取于 2026-11-02，Wiktextract 提交 `19f9a3006a405fb7bfb40193dca41c7117a009a5`、Wikitextprocessor 提交 `6f96ddf5f3466707f1869fe6618390fd77dfd109`。原始全语种 JSONL 的 HTTP 总量为 6,865,138,291 字节，gzip 为 736,596,796 字节；两者是提取输入体积。[原始数据与版本](https://kaikki.org/frwiktionary/rawdata.html)、[提取器提交](https://github.com/tatuylonen/wiktextract/commit/19f9a3006a405fb7bfb40193dca41c7117a009a5)、[处理器提交](https://github.com/tatuylonen/wikitextprocessor/commit/6f96ddf5f3466707f1869fe6618390fd77dfd109)。

英语版的 [French 页面](https://kaikki.org/dictionary/French/index.html)来自 enwiktionary，正文解释语言是英语；页面注明提取于 2026-10-30。法语条目语言不等于来源版语言。英语版原创正文同样可按 CC BY-SA 4.0 使用，外部引用仍有独立条件；两个版的来源、义项与许可记录各自保存，字段覆盖分别统计。[英语维基词典版权条款，修订 87545419](https://en.wiktionary.org/w/index.php?title=Wiktionary%3ACopyrights&oldid=87545419)。

### 实际有界原始小样

本轮从[法语版原始页](https://kaikki.org/frwiktionary/rawdata.html)的直接链接读取前 8 MiB：

- 精确 URL：`https://kaikki.org/frwiktionary/raw-wiktextract-data.jsonl`。
- HTTP 206，`Content-Range: bytes 0-8388607/6865138291`；`Last-Modified: Mon, 02 Nov 2026 00:47:30 GMT`。
- 完整 JSONL 行占 8,375,536 字节，共 2,033 个全语种记录；按 `lang_code=fr` 得 413 个记录、206 个不同词形。
- 这 413 个记录中：释义 413、IPA 410、例句 351、任意语言译词 269、词形表 `forms` 313；65 个含 `form_of`，没有顶层 CEFR 字段。英语译词出现在 254 个记录，中文 `zh`／`cmn` 译词出现在 146 个记录，两种目标语言的覆盖分别统计。

这是文件开头的有偏样本，413／206 包含屈折记录，不代表完整词元覆盖；字段存在不代表适合 A1–B2 学习者。样本出现 `accueil`、`lire` 等词；固定查询中只出现 `chat`、`école`、`travail`、`procrastiner` 四个词头的八条记录。27 项的完整结构化覆盖在下节单独统计。

### 固定 27 查询：完整原始页区块

从[原始 gzip](https://kaikki.org/frwiktionary/raw-wiktextract-data.jsonl.gz)按 HTTP Range 续读并保留压缩缓存，使用 `If-Match` 将各段固定在同一 ETag `"6ae7ddda-2be7933c"`；文件最后修改为 2026-11-02 00:48:26 GMT。缓存共 192,937,984 字节（184 MiB），处理到第 859,612 行、1,270,733,265 个解压 JSONL 字节时已取得所有目标区块；抽出的 44 条法语记录为 1,768,344 字节。

完整性依据是生产者按页取得 `page_data`，逐条写完该列表后才处理下一页；法语记录的 `word` 来自页标题，明确重定向则输出独立 `title`／`redirect`。扫描读取目标词头之后的下一页记录才标记区块完成，并保留该边界及行号。[按页写入](https://github.com/tatuylonen/wiktextract/blob/19f9a3006a405fb7bfb40193dca41c7117a009a5/src/wiktextract/wiktionary.py#L810-L833)、[标题及重定向](https://github.com/tatuylonen/wiktextract/blob/19f9a3006a405fb7bfb40193dca41c7117a009a5/src/wiktextract/wiktionary.py#L57-L70)、[法语词头](https://github.com/tatuylonen/wiktextract/blob/19f9a3006a405fb7bfb40193dca41c7117a009a5/src/wiktextract/extractor/fr/page.py#L250-L271)。

原始流同时保留 `aujourd'hui → aujourd’hui` 与 `s'il vous plaît → s’il vous plaît` 两条明确重定向。26 个词头的记录数量、顺序及完整解析 JSON 对象逐项等于下述单词文件；359 个义项、中英译词及缺项统计均相同。

### 结构化字段与语义核对

本轮通过[法语版词头目录](https://kaikki.org/frwiktionary/Fran%C3%A7ais/words/index.html)逐级读取实际链接，再下载对应单词 JSONL。页面说明这些数据经过后处理、消歧及外部数据合并；本节以它们作为原始记录的独立对照。网页文件的 HTTP 修改时间均为 2026-11-02。[后处理说明](https://kaikki.org/frwiktionary/Fran%C3%A7ais/index.html)。

同时固定了 27 个查询页及额外短语目标页的当前 Wiktionary 修订，作为语义对照；这些现行修订与 JSONL 的 2026-10-01 dump 分别记录。ASCII [aujourd'hui，修订 22836985](https://fr.wiktionary.org/w/index.php?title=aujourd%27hui&oldid=22836985)明确指向 [aujourd’hui，修订 39793601](https://fr.wiktionary.org/w/index.php?title=aujourd%E2%80%99hui&oldid=39793601)；[s'il vous plaît，修订 20202679](https://fr.wiktionary.org/w/index.php?title=s%27il%20vous%20pla%C3%AEt&oldid=20202679)明确指向 [s’il vous plaît，修订 39904474](https://fr.wiktionary.org/w/index.php?title=s%E2%80%99il%20vous%20pla%C3%AEt&oldid=39904474)。它们是来源中的具体重定向，未扩展为通用的撇号归并规则；查询原文和原页修订保留。

27 个查询对应 **26 个来源词头、44 个法语记录、359 个义项**。每个查询均有定义、词性、至少一项 IPA、英语译词及例句；中文译词覆盖 **19／27 查询、18／26 词头**。下表记录／义项是对应 JSONL 的节点数，英／中列是译词节点数；它们不代表不同词元数、全部义项的双语覆盖或完整英文／中文解释。两种 aujourd’hui 查询共享同一份来源数据。

| 查询（链接为实读 JSONL） | 记录／义项 | 词性 | 英／中译词节点 | 人工核对摘要 |
| --- | ---: | --- | ---: | --- |
| [être](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/%C3%AA/%C3%AAt/%C3%AAtre.jsonl) | 2／16 | noun／verb | 6／3 | 系词／位置／助动词有文字标签；存在类名词另存 |
| [avoir](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/a/av/avoir.jsonl) | 2／11 | noun／verb | 17／1 | 中文“有”标注所有关系；状态、助动词与资产名词另核 |
| [aller](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/a/al/aller.jsonl) | 2／22 | noun／verb | 7／1 | 去／go 对移动义；近将来与往程名词分开 |
| [faire](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/f/fa/faire.jsonl) | 2／61 | noun／verb | 16／4 | 61 个义项；制作、烹饪及其他用法逐义保留 |
| [manger](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/m/ma/manger.jsonl) | 2／13 | noun／verb | 4／3 | 吃／eat 编号 1；食物名词及比喻义分开 |
| [boire](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/b/bo/boire.jsonl) | 3／13 | noun／verb | 1／2 | 饮用、饮食名词与旧河道同形；中英译词无编号 |
| [parler](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/p/pa/parler.jsonl) | 2／19 | noun／verb | 7／10 | 讲话／speak 对编号 1；语言变体名词独立 |
| [apprendre](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/a/ap/apprendre.jsonl) | 1／8 | verb | 6／5 | 学习／learn 对编号 1；获取知识与形成习惯分开 |
| [école](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/%C3%A9/%C3%A9c/%C3%A9cole.jsonl) | 2／13 | noun／verb | 5／1 | 学校／school 对编号 1；另有 5 条 écoler 词形义 |
| [chat](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/c/ch/chat.jsonl) | 3／17 | noun | 4／2 | 动物、攻城器械、在线聊天三组，读音按组保留 |
| [maison](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/m/ma/maison.jsonl) | 2／16 | adj／noun | 11／2 | 房子／house 对义 1，家／home 对义 2；形容词另存 |
| [travail](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/t/tr/travail.jsonl) | 2／14 | noun | 9／1 | 劳动与牲畜固定架是两个名词记录 |
| [aujourd'hui](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/a/au/aujourd%E2%80%99hui.jsonl) | 2／4 | adv／noun | 7／5 | 显式重定向至弯撇号词头；副词与名词分开 |
| [aujourd’hui](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/a/au/aujourd%E2%80%99hui.jsonl) | 2／4 | adv／noun | 7／5 | 与上一查询共享来源；不重复计算来源词头 |
| [œil](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/%C5%93/%C5%93i/%C5%93il.jsonl) | 2／21 | noun | 2／2 | eye／glyph 及 IPA 抽取有错配，见下文 |
| [cœur](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/c/c%C5%93/c%C5%93ur.jsonl) | 1／26 | noun | 14／4 | heart／心脏对应器官标签；爱、勇气等义另核 |
| [cote](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/c/co/cote.jsonl) | 2／17 | noun／verb | 7／0 | 档案编号义与 5 条 coter 词形义分开；中文缺失 |
| [côte](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/c/c%C3%B4/c%C3%B4te.jsonl) | 1／11 | noun | 19／0 | rib／slope 有文字标签无编号；中文缺失 |
| [côté](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/c/c%C3%B4/c%C3%B4t%C3%A9.jsonl) | 1／10 | noun | 11／0 | side 的编号区分躯干部位／整侧；中文缺失 |
| [bonjour](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/b/bo/bonjour.jsonl) | 2／7 | intj／noun | 8／4 | hello／你好对应义 1；早晨／午后限定保留 |
| [s'il vous plaît](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/s/s%E2%80%99/s%E2%80%99il%20vous%20pla%C3%AEt.jsonl) | 2／6 | intj／phrase | 1／1 | 显式重定向；礼貌短语与委婉咒语分开 |
| [se souvenir](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/s/se/se%20souvenir.jsonl) | 1／5 | verb | 1／0 | 首义偏恩惠／报复；remember 标签无编号，中文缺失 |
| [procrastiner](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/p/pr/procrastiner.jsonl) | 1／1 | verb | 1／0 | 延期义与 procrastinate 对编号 1；中文缺失 |
| [algorithme](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/a/al/algorithme.jsonl) | 1／3 | noun | 1／1 | algorithm／算法无编号；专门定义不提供 CEFR |
| [retentir](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/r/re/retentir.jsonl) | 1／3 | verb | 4／0 | 鸣响义有 ring out 等无编号译词；中文缺失 |
| [retenir](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/r/re/retenir.jsonl) | 1／18 | verb | 4／0 | 记忆义为 4，现有英语仅标注停留义；中文缺失 |
| [taire](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/t/ta/taire.jsonl) | 1／4 | verb | 12／0 | 第一义是不说／略过；英语文字标签无编号，中文缺失 |

**数值义项关联也需语义核查。**按 26 个不同词头统计，237 条中英译词中，95 条给出范围内的正整数 `sense_index`，142 条缺少该编号；207 条提供文字 `sense` 标签，标签可以与编号并存。编号局限于所属记录，不能跨词性／同形记录拼接；本轮没有越界编号，但实际有含义错配：

- [œil 的结构化数据](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/%C5%93/%C5%93i/%C5%93il.jsonl)把 `eye` 与 `glyph` 都置于解剖名词记录、义项 1、“Organe permettant de voir”标签下，另一个印刷名词记录没有英语译词。[dump 日期之前的固定源页，修订 39820476](https://fr.wiktionary.org/w/index.php?title=%C5%93il&oldid=39820476)（2026-08-20）已明确写 `eye (1), glyph (2)`。结构化原始记录未保留这一行内区分；不能据范围内编号把 glyph 当成“眼睛”的译义，也不能机械改为该记录的义项 2。
- [retenir，修订 39885585](https://fr.wiktionary.org/w/index.php?title=retenir&oldid=39885585)的义项 4 是“Se souvenir, garder en mémoire”。本小样的四个英语译词全部标注“Arrêter, maintenir, faire demeurer”，对应停留义；英语字段非空仍不能补出记忆义的来源译文。
- [se souvenir，修订 39983392](https://fr.wiktionary.org/w/index.php?title=se%20souvenir&oldid=39983392)首义偏向记恩或记仇，后续含怨恨、照顾、回到记忆等义；英语块只以“garder en mémoire”标记 remember。默认取首义会得到过窄的学习摘要。
- [chat](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/c/ch/chat.jsonl)的动物、攻城器械与网络聊天，以及 [boire](https://kaikki.org/frwiktionary/Fran%C3%A7ais/meaning/b/bo/boire.jsonl)的饮用与旧河道，是独立同形记录。多项 IPA 还包含地区和整句发音记录，不能平铺为一条词头读音。

固定提取器只从翻译框标题及编号复制 `sense`／`sense_index`，忽略纯文字子节点，因此不会保留行内 `(1)`／`(2)`。[模板说明](https://fr.wiktionary.org/w/index.php?title=Mod%C3%A8le%3Atrad-d%C3%A9but%2FDocumentation&oldid=37396900)确认翻译框的第二参数是对应定义编号；[提取实现](https://github.com/tatuylonen/wiktextract/blob/19f9a3006a405fb7bfb40193dca41c7117a009a5/src/wiktextract/extractor/fr/translation.py#L14-L51)与[写入译词](https://github.com/tatuylonen/wiktextract/blob/19f9a3006a405fb7bfb40193dca41c7117a009a5/src/wiktextract/extractor/fr/translation.py#L86-L148)使用这一框级信息。[上游测试](https://github.com/tatuylonen/wiktextract/blob/19f9a3006a405fb7bfb40193dca41c7117a009a5/tests/test_fr_translation.py#L144-L188)还在只有一个定义时保留编号 12，证明编号没有按义项数量验证。

**IPA 非空也不等于正确的词头读音。**两个 `œil` 名词原始记录均带 /e/、/ɛ/，以及包含 `un œil`／`des yeux` 的多词发音。[修订 39820476 的发音说明](https://fr.wiktionary.org/w/index.php?title=%C5%93il&oldid=39820476)已把 /e/、/ɛ/ 用作规则说明，并明确 `œil` 是例外；它们不是该词的有效变体。[发音提取代码](https://github.com/tatuylonen/wiktextract/blob/19f9a3006a405fb7bfb40193dca41c7117a009a5/src/wiktextract/extractor/fr/pronunciation.py#L33-L47)将三级发音节的结果复制到同语言的各记录，[对应测试](https://github.com/tatuylonen/wiktextract/blob/19f9a3006a405fb7bfb40193dca41c7117a009a5/tests/test_fr_pronunciation.py#L26-L88)固定了跨词性复制行为。摘要音标须核对原页用途，保留可靠的单词与地区信息。

### 来源审核后的 27 查询摘要

固定 26 个来源页在 `2026-10-01T00:00:00Z` 当时或之前的最后可用修订，逐条核对主要义项、翻译框／行内限定和所属词性记录的词头读音。下表链接指向用于语义核对的独立源页修订；记录号在同一词头的法语记录中从 1 开始，义项号在所属记录中从 1 开始。它们标识本次固定小样。每个来源词头选择一个主要义项，完整 44 个原始记录及 359 个义项另行保留。

审核后的完整法语定义及词头 IPA 覆盖 **27／27 查询**，与所选义项相符的英语译词为 **26／27**、中文为 **19／27**；按 26 个来源词头分别为 26、26、25、18。两种 aujourd’hui 查询共用一行；ASCII s'il vous plaît 依明确重定向使用弯撇号行。全部 26 个所选义项的 `cefr` 为 `null`。中文列保留来源已有的繁体写法；译词是所选义项的翻译，不是完整的双语解释。

| 词头／来源修订 | 记录／义项／词性 | 完整法语定义及限定 | 词头 IPA | 英语／中文 |
| --- | --- | --- | --- | --- |
| [être · 39860125](https://fr.wiktionary.org/w/index.php?title=%C3%AAtre&oldid=39860125) | 1／1／verb | Définir un état, une caractéristique du sujet. | /ɛtʁ/ | be／是 |
| [avoir · 39879166](https://fr.wiktionary.org/w/index.php?title=avoir&oldid=39879166) | 1／1／verb | Être en relation possessive, soit concrète ou abstraite, soit permanente ou occasionnelle, dont le possesseur est le sujet et le possédé est le complément d’objet direct. | /a.vwaʁ/；/a.vwɑʁ/ | have／有 |
| [aller · 39881261](https://fr.wiktionary.org/w/index.php?title=aller&oldid=39881261) | 1／1／verb | Se déplacer jusqu'à un endroit. | /a.le/ | go／去 |
| [faire · 39908369](https://fr.wiktionary.org/w/index.php?title=faire&oldid=39908369) | 1／1／verb | Créer, produire, fabriquer, en parlant de toute œuvre matérielle. | /fɛʁ/ | make／制作（製作） |
| [manger · 39862155](https://fr.wiktionary.org/w/index.php?title=manger&oldid=39862155) | 1／1／verb | Mâcher et avaler un aliment dans le but de se nourrir. | /mɑ̃.ʒe/ | eat／吃 |
| [boire · 39862133](https://fr.wiktionary.org/w/index.php?title=boire&oldid=39862133) | 1／1／verb | Mettre un liquide dans sa bouche et l’avaler. | /bwaʁ/ | drink／喝 |
| [parler · 39838633](https://fr.wiktionary.org/w/index.php?title=parler&oldid=39838633) | 1／1／verb | User de la faculté du langage ; proférer, prononcer, articuler des mots. | /paʁ.le/ | speak／说话（說話） |
| [apprendre · 39773016](https://fr.wiktionary.org/w/index.php?title=apprendre&oldid=39773016) | 1／1／verb | Acquérir une connaissance ou un savoir-faire. | /a.pʁɑ̃dʁ/ | learn／学习（學習） |
| [école · 39867888](https://fr.wiktionary.org/w/index.php?title=%C3%A9cole&oldid=39867888) | 1／1／noun | Lieu dédié à l’apprentissage. | /e.kɔl/ | school／学校（學校） |
| [chat · 39898304](https://fr.wiktionary.org/w/index.php?title=chat&oldid=39898304) | 1／1／noun | Mammifère carnivore félin de taille moyenne, au museau court et arrondi, domestiqué, apprivoisé ou encore à l’état sauvage. （raw_tags: Félinologie） | /ʃa/；/ʃɑ/ | cat／猫（貓） |
| [maison · 39902276](https://fr.wiktionary.org/w/index.php?title=maison&oldid=39902276) | 1／1／noun | Bâtiment servant de logis, d’habitation, de demeure. （topics: construction） | /me.zɔ̃/；/mɛ.zɔ̃/ | house／房子 |
| [travail · 39877669](https://fr.wiktionary.org/w/index.php?title=travail&oldid=39877669) | 1／1／noun | Labeur, application à une tâche, effort soutenu pour faire quelque chose, en parlant de l’esprit comme du corps. | /tʁa.vaj/ | work／工作 |
| [aujourd’hui · 39793601](https://fr.wiktionary.org/w/index.php?title=aujourd%E2%80%99hui&oldid=39793601) | 1／1／adv | À la date présente, ce jour même. | /o.ʒuʁ.d‿ɥi/ | today／今天 |
| [œil · 39820476](https://fr.wiktionary.org/w/index.php?title=%C5%93il&oldid=39820476) | 1／1／noun | Organe de la vue. （topics: anatomy） | /œj/ | eye／眼睛 |
| [cœur · 39720190](https://fr.wiktionary.org/w/index.php?title=c%C5%93ur&oldid=39720190) | 1／1／noun | Organe musculaire, creux et pulsatile assurant la circulation sanguine dans le corps humain ou animal. （topics: anatomy） | /kœʁ/ | heart／心脏（心臟） |
| [cote · 39840652](https://fr.wiktionary.org/w/index.php?title=cote&oldid=39840652) | 1／1／noun | Code alphabétique, numéral ou alphanumérique dont on se sert pour classer les pièces d’un procès, d’un inventaire, pour identifier et ranger les livres dans une bibliothèque et les documents dans les archives, etc. | /kɔt/ | code；call number／未知 |
| [côte · 39813643](https://fr.wiktionary.org/w/index.php?title=c%C3%B4te&oldid=39813643) | 1／1／noun | Chacun des os qui forment la cage thoracique. （topics: anatomy） | /kot/ | rib／未知 |
| [côté · 39827549](https://fr.wiktionary.org/w/index.php?title=c%C3%B4t%C3%A9&oldid=39827549) | 1／6／noun | Face d'un objet plat. （tags: especially） | /ko.te/ | side／未知 |
| [bonjour · 39872057](https://fr.wiktionary.org/w/index.php?title=bonjour&oldid=39872057) | 1／1／intj | Formule de salutation utilisée lorsqu’on rencontre quelqu’un. （tags: polite） | /bɔ̃.ʒuʁ/ | hello／你好 |
| [s’il vous plaît · 39904474](https://fr.wiktionary.org/w/index.php?title=s%E2%80%99il%20vous%20pla%C3%AEt&oldid=39904474) | 1／1／phrase | Locution qui permet de demander quelque chose poliment, à quelqu’un que l’on vouvoie, ou à plusieurs personnes. （tags: polite） | /s‿il vu plɛ/ | please／请（請） |
| [se souvenir · 39909362](https://fr.wiktionary.org/w/index.php?title=se%20souvenir&oldid=39909362) | 1／4／verb | Revenir à la mémoire. | /sə suv.niʁ/ | remember／未知 |
| [procrastiner · 39802483](https://fr.wiktionary.org/w/index.php?title=procrastiner&oldid=39802483) | 1／1／verb | Reporter au lendemain, remettre à plus tard quelque chose | /pʁɔ.kʁas.ti.ne/ | procrastinate／未知 |
| [algorithme · 39392670](https://fr.wiktionary.org/w/index.php?title=algorithme&oldid=39392670) | 1／1／noun | Suite finie et non ambigüe d’opérations ou d’instructions permettant de résoudre un problème ou d’obtenir un résultat. （topics: mathematics） | /al.ɡɔ.ʁitm/ | algorithm／算法 |
| [retentir · 39796396](https://fr.wiktionary.org/w/index.php?title=retentir&oldid=39796396) | 1／1／verb | Rendre ou renvoyer un son éclatant. | /ʁə.tɑ̃.tiʁ/ | ring out／未知 |
| [retenir · 39885585](https://fr.wiktionary.org/w/index.php?title=retenir&oldid=39885585) | 1／4／verb | Se souvenir, garder en mémoire. | /ʁət.niʁ/；/ʁə.tə.niʁ/；/ʁtə.niʁ/ | 未知／未知 |
| [taire · 39681528](https://fr.wiktionary.org/w/index.php?title=taire&oldid=39681528) | 1／1／verb | Ne pas dire ; passer sous silence. | /tɛʁ/ | withhold／未知 |

所选定义完整保留原始 `glosses`，义项 `topics`／`tags`／`raw_tags` 与记录级语法限制分别保存；原始翻译节点及其文字标签、编号、罗马化和繁体写法均保留。最长定义是上表 `cote` 的分类编号义，正文 **214 个 Unicode 字符、224 个 UTF-8 字节**。研究 JSON 连同来源行和选择证据为 91,808 字节；不是成品资源体积。定义未按约 100 字节截断，最终 UI 的展开与超长内容行为在资源接入时验收。

关键选择依据：

- `œil` 保留解剖义和 `eye`／“眼睛”，排除同框内的 `glyph`；`cœur` 保留器官义。`côté` 选择义项 6 的平面物体一面及“En particulier”限定；`cote`／`côte`／`côté` 继续分别保存。
- `retenir` 选择记忆义 4，现有英语仅对应停留义，英语与中文均未知。`se souvenir` 使用所选修订的义项 4；源页第 17 行“Il se souvint plus tard que…”支持代词式记忆用法，与无编号的“garder en mémoire”／`remember` 框相符。记录保留 `pronominal` 及“Transitif avec le complément d’objet introduit par de”；后续义项的正式／非人称标签按其所属义项保存。
- `taire` 保留及物“不说／略过”义和 `withhold`；`be quiet` 对应其他语法用法。`chat` 使用动物记录，保留“Félinologie”及“Chat domestique”翻译标签；`s’il vous plaît` 使用礼貌请求记录，保留尊称／复数对象限定。
- 中文缺项仍为 `cote`、`côte`、`côté`、`se souvenir`、`procrastiner`、`retentir`、`retenir`、`taire`；未从其他义项或其他词头补写。

### 词头 IPA：记录级选择

全部 **44 个法语记录**已按原页各词性／同形记录的词头行核对，保留 **51 个记录级 IPA 值、34 个不同值**。其中本小样所选 26 个记录保留 **31 个 IPA 值**，如上表；第一项是源页词头行的首列值，其他明确列出的值一并保留。只将显示分隔符由反斜线改为斜线，IPA 本体、音节点、连读、空格及原有地区／用法标签保持原状。

[`chat` 的动物／攻城器械与网络聊天](https://fr.wiktionary.org/w/index.php?title=chat&oldid=39898304)分别使用 /ʃa/、/ʃɑ/ 与 /tʃat/；[`s’il vous plaît` 的礼貌短语及魁北克委婉咒语](https://fr.wiktionary.org/w/index.php?title=s%E2%80%99il%20vous%20pla%C3%AEt&oldid=39904474)分别为 /s‿il vu plɛ/ 与 /si vu plɛ/。[`aujourd’hui` 的副词／名词](https://fr.wiktionary.org/w/index.php?title=aujourd%E2%80%99hui&oldid=39793601)分别保留 /o.ʒuʁ.d‿ɥi/ 与 /o.ʒuʁ.dɥi/。这些记录边界决定摘要采用的读音。

来源用途确认排除 **36 个原始 sound 节点**：21 个整句／带冠词或其他短语片段、11 个属于其他同形记录的值、4 个 `œil` 发音规则的解释性反例。其他未选的地区／语音变体仍作为独立证据保留；未声明无效，也未推断缺失地区标签。

### 独立词频扩展：30 查询

复用[现有词频投影](../scripts/frequency_sample.py)的 `build_database`，从固定 Lexique 3.83 的 142,694 行建立词元／词性频率组；排除原 27 查询及其规范词头，按已知正频率词元的 `freqlemfilms2` 降序、二进制词头顺序打破同频，预先选取前 30 项。该指标按一致词性组只计一次，再对词元求和，表示影视字幕每百万词频率；它不提供 CEFR 或义项频率。

查询依次为 `je, de, ne, pas, le, la, tu, vous, il, et, un, à, l', qui, les, en, ça, tout, on, que, une, ce, d', pour, des, dire, pouvoir, vouloir, mais, me`。32 个截止日前源页包括 30 个查询页及两个额外目标页；[`l'`，修订 22132309](https://fr.wiktionary.org/w/index.php?oldid=22132309)明确指向 [`l’`，38868882](https://fr.wiktionary.org/w/index.php?oldid=38868882)，[`d'`，451741](https://fr.wiktionary.org/w/index.php?oldid=451741)明确指向 [`d’`，39770687](https://fr.wiktionary.org/w/index.php?oldid=39770687)。完整 API 响应共 974,404 字节，重定向按具体来源记录。

30 个网页单词 JSONL 均沿生产者目录的实际链接取得。离线扫描未改变的 192,937,984 字节 gzip 缓存，读取其中 141,557,760 压缩字节，取得全部 **99 条法语记录、385 个义项**及两项明确重定向；选出的原始 JSONL 为 1,715,763 字节。30 个完整词头区块的记录数量、顺序及完整解析对象均等于单词文件，没有追加原始 dump 下载。

| 同一 30 查询的字段存在 | Kaikki 法语原始记录 | FreeDict 法英 0.4.1 | FreeDict 法中 2025.11.23 |
| --- | ---: | ---: | ---: |
| 精确词头或来源明确重定向命中 | 30／30 | 27／30 | 15／30 |
| 任意法语 gloss／`def` | 30／30 | 0／30 | 11／30 |
| 任意原始 IPA／`pron` | 30／30 | 27／30 | 15／30 |
| 任意义项英语译词 | 27／30 | 27／30 | 不适用 |
| 任意义项中文译词 | 16／30 | 不适用 | 15／30 |
| 明确 CEFR 字段 | 0／30 | 未声明分级 | 未声明分级 |

FreeDict 两列读取各自完整固定 TEI，只做精确查询。法英缺 `ne, ça, d'`；法中缺 `de, ne, le, à, l', les, en, ça, on, une, ce, d', pour, des, pouvoir`。Kaikki 英语缺 `ne, l', d'`，中文缺 `de, ne, le, à, l', les, en, ça, on, une, d', pour, des, pouvoir`。这些是查询级字段存在，含任意词性／义项，不代表常用含义或词头读音正确。最长原始 gloss 是 `que` 的一个虚拟式用法说明，480 个 Unicode 字符／509 个 UTF-8 字节。

全部 **30 个扩展词头各完成一个来源支持的常用含义选择**，原先 13 项保持不变。完整 gloss、原始语法／义项标签、精确源行、IPA 和原始译词节点保存于 `reviewed-summary-sample.json`。所选法语定义／词头 IPA 为 **30／30**，匹配英语译词为 **23／30**、中文 **11／30**；最长所选定义为 `ce`，346 个 Unicode 字符／363 个 UTF-8 字节。记录／义项号沿用本固定小样的一基编号，不代表常用义频率排名。下表保留原先 13 项，其中英语为 7／13、中文为 2／13；字段存在统计与含义选择分别判断。

| 词头／固定来源修订 | 记录／义项／词性 | 来源支持的含义及限制 | 核对后的英／中译词 |
| --- | --- | --- | --- |
| [je · 39909570](https://fr.wiktionary.org/w/index.php?oldid=39909570) | 1／1／pron | 第一人称单数主语，兼阳／阴性；元音或哑音 h 前省略为 j’ | I／我 |
| [de · 39911252](https://fr.wiktionary.org/w/index.php?oldid=39911252) | 1／1／prep | 来源／起点，不变词；from 明确关联义项 1 | from／未知 |
| [ne · 39872602](https://fr.wiktionary.org/w/index.php?oldid=39872602) | 1／1／particle | 通常由 ne 与 pas／plus／jamais 等围绕动词构成的否定 | 未知／未知 |
| [le · 39770498](https://fr.wiktionary.org/w/index.php?oldid=39770498) | 1／1／article | 阳性单数定冠词，指语境中可确定的实体 | the／未知 |
| [la · 39823587](https://fr.wiktionary.org/w/index.php?oldid=39823587) | 1／1／article | le 的阴性单数定冠词形式 | the／未知 |
| [en · 39823527](https://fr.wiktionary.org/w/index.php?oldid=39823527) | 2／1／pron | De ça, de ceci, de cela；保留“Remplaçant un COI avec de”限制 | 未知／未知 |
| [que · 39808704](https://fr.wiktionary.org/w/index.php?oldid=39808704) | 5／1／pron | 非生物直接宾语的疑问代词，不变／中性 | what／什么（什麼） |
| [ce · 39770730](https://fr.wiktionary.org/w/index.php?oldid=39770730) | 2／1／pron | 此事／此情境；中性非人称指示代词，几乎总与 être 使用 | this；that／未知 |
| [l’ · 38868882](https://fr.wiktionary.org/w/index.php?oldid=38868882) | 1／1／article | le／la 在元音或哑音 h 前的定冠词省略形式 | 未知／未知 |
| [d’ · 39770687](https://fr.wiktionary.org/w/index.php?oldid=39770687) | 1／1／prep | de 在元音或哑音 h 前的介词省略形式 | 未知／未知 |
| [les · 39880984](https://fr.wiktionary.org/w/index.php?oldid=39880984) | 1／1／article | le／la 的复数定冠词形式，兼阳／阴性 | the／未知 |
| [une · 39772653](https://fr.wiktionary.org/w/index.php?oldid=39772653) | 4／1／article | un 的阴性单数不定冠词形式；此项正确指向 un | 未知／未知 |
| [des · 39770537](https://fr.wiktionary.org/w/index.php?oldid=39770537) | 1／1／article | un／une 的复数不定冠词形式，兼阳／阴性 | 未知／未知 |

其余 17 项也按固定原页选择；完整定义及限制保存在研究 JSON，表中只概括所选含义。

| 词头／固定来源修订 | 记录／义项／词性 | 来源支持的含义 | 核对后的英／中译词 |
| --- | --- | --- | --- |
| [pas · 39910982](https://fr.wiktionary.org/w/index.php?oldid=39910982) | 2／1／adv | 动词否定；文学语体与 ne 使用，口语常省 ne | not／不 |
| [tu · 39909638](https://fr.wiktionary.org/w/index.php?oldid=39909638) | 1／1／pron | 第二人称单数主语代词 | you／你 |
| [vous · 39871499](https://fr.wiktionary.org/w/index.php?oldid=39871499) | 1／1／pron | 对多人使用的第二人称复数主语代词 | you／你们（你們） |
| [il · 39770564](https://fr.wiktionary.org/w/index.php?oldid=39770564) | 1／1／pron | 第三人称阳性单数主语，指人、动物或事物 | he；it／他 |
| [et · 39723793](https://fr.wiktionary.org/w/index.php?oldid=39723793) | 1／1／conj | 在同一关系中连接多个概念 | and／和 |
| [un · 39752995](https://fr.wiktionary.org/w/index.php?oldid=39752995) | 3／1／article | 名词前的不定冠词，常有“多个中的一个”含义 | a；an／一个 |
| [à · 39774984](https://fr.wiktionary.org/w/index.php?oldid=39774984) | 2／2／prep | 地点补语中的移动方向，保留父级地点及子级方向 gloss | to／未知 |
| [qui · 39879378](https://fr.wiktionary.org/w/index.php?oldid=39879378) | 1／1／pron | 关系代词；Lequel, laquelle, lesquels, lesquelles | who；that／未知 |
| [ça · 39860132](https://fr.wiktionary.org/w/index.php?oldid=39860132) | 1／1／pron | 此事物／此情境 | 未知／未知 |
| [tout · 39886654](https://fr.wiktionary.org/w/index.php?oldid=39886654) | 1／1／adj | 所考虑事物的完整、全体或全部范围 | all／未知 |
| [on · 39840073](https://fr.wiktionary.org/w/index.php?oldid=39840073) | 1／1／pron | 第三人称单数，泛指未确定的人／多人 | one；you／未知 |
| [pour · 39879849](https://fr.wiktionary.org/w/index.php?oldid=39879849) | 1／1／prep | 原因、目的或面向的对象 | for／未知 |
| [dire · 39792727](https://fr.wiktionary.org/w/index.php?oldid=39792727) | 1／1／verb | 用语言表达 | say；tell／说；告诉 |
| [pouvoir · 39819957](https://fr.wiktionary.org/w/index.php?oldid=39819957) | 1／1／verb | 能够、有能力或处于可行状态 | can；be able to／未知 |
| [vouloir · 39905056](https://fr.wiktionary.org/w/index.php?oldid=39905056) | 1／1／verb | 有意图或意愿做某事 | want；be willing to／要 |
| [mais · 39875963](https://fr.wiktionary.org/w/index.php?oldid=39875963) | 1／1／conj | 表示对立、例外或差别 | but／可是；但是 |
| [me · 39891222](https://fr.wiktionary.org/w/index.php?oldid=39891222) | 1／1／pron | 第一人称单数直接宾语，je 的宾格 | me／未知 |

`il` 的 he／人或事物 it／“他”在原始记录 2，却由“Personne, animal ou chose”翻译框支持所选主语记录 1；非人称占位 it 不选。`on` 的 one／you 也从记录 2 的“Personne indéfinie”框恢复，保留原始选行文件内的记录序号、同词记录号及译词节点号。`vous` 复数摘要保留“你们／你們”，不采用同框裸“您”；`à` 的方向义不选静止 at／in，`me` 的宾格不选与格译词。`ça` 的其他记录只有精神分析 id／非人称 it，所选指示义保持未知；`pouvoir` 的能力义不选许可义及未分类框。44 个所选译词节点均与原始对象相等，关联判断另存来源证据。

来源及固定提取代码确认以下问题：

- **首记录并非所需词性。**`une` 第一记录是“Chose portant le numéro 1”名词，常用不定冠词在记录 4；`ce` 的指示限定词与代词、`que` 的连词与疑问／关系代词各有独立记录。Lexique 的词元／词性频率不能把来源义项顺序变成常用义排序。
- **词形标签不保证指向正确。**`l’` 的记录 1／2 分别指向 `Élision` 和 `pronoms personnels`，原页实际给出 le／la；`la`、`les` 的冠词以及 `d’` 的部分／不定冠词有词形标签但缺结构化目标。`des` 的 de + les 缩合记录只指向 de。[固定 `find_form_of_word`](https://github.com/tatuylonen/wiktextract/blob/19f9a3006a405fb7bfb40193dca41c7117a009a5/src/wiktextract/extractor/fr/gloss.py#L308-L327)只保留直接子节点的最后链接，未遍历斜体节点中的实际词头。
- **翻译归属可能跨记录错置。**`le` 的 COD 代词在记录 2，him／her／it 却位于记录 3 的中性表语代词，仍带第三人称宾格标签；原页两个翻译框置于后一个代词节之后，[提取器](https://github.com/tatuylonen/wiktextract/blob/19f9a3006a405fb7bfb40193dca41c7117a009a5/src/wiktextract/extractor/fr/translation.py#L148)写到 `page_data[-1]`。`une` 的 front page 属名词，`des` 的 of the 属 de + les 缩合，`en` 的多项英语来自“Traductions à trier suivant le sens”；它们不覆盖上表所选含义。
- **非空 gloss 可能是例句。**`que` 记录 4／义项 1 是“Les deux pays s’invectivent à coups de missiles et de bombardements…”；固定源页法语第 215–218 行只有 `#*` 例句，没有 `#` 定义。[节解析](https://github.com/tatuylonen/wiktextract/blob/19f9a3006a405fb7bfb40193dca41c7117a009a5/src/wiktextract/extractor/fr/page.py#L206-L210)接受以 `#` 开头的列表，[gloss 提取](https://github.com/tatuylonen/wiktextract/blob/19f9a3006a405fb7bfb40193dca41c7117a009a5/src/wiktextract/extractor/fr/gloss.py#L14-L24)把其中条目建立为义项。
- **IPA 的语境限定会丢失。**`je` 的 /ʒ/ 与 /ʃ/ 未保留原页的口语、有声／无声辅音环境，且复制到其他同形记录；`le` 的 /lø/ 未在裸 IPA 节点保留后附 -le 限定。[发音提取](https://github.com/tatuylonen/wiktextract/blob/19f9a3006a405fb7bfb40193dca41c7117a009a5/src/wiktextract/extractor/fr/pronunciation.py#L186-L196)在遇到 IPA 模板时保存当时标签，[三级发音节分配](https://github.com/tatuylonen/wiktextract/blob/19f9a3006a405fb7bfb40193dca41c7117a009a5/src/wiktextract/extractor/fr/pronunciation.py#L33-L45)再分发到同语言记录。

扩展 **99 条记录的词头 IPA 选择已全部完成**：95 条有明确值，共 100 个 IPA／38 个不同值；其余 `un` 名词记录 7、`en` 字母名记录 5、`on` 名词记录 4、`l’` 非规范代词记录 3 的词头行没有 IPA，保持 `null`，不沿用共享 sound。所选 30 个摘要共保留 33 个 IPA 值；`pas` 两记录、`les` 两记录及 `des` 不定冠词的五个双值词头行保持原顺序，没有推定地区排名。

全部 2,059 个原始 sound 节点保留于审计；100 个所选值均有完全相同的原始节点，重复节点号一并保留。来源确认排除 52 个节点：条件发音 4、后附发音 9、其他同形／共享记录 33、连读 6；如 `des` 的 /de.z‿/、/dɛ.z‿/ 不作为孤立词头读音。只更换外层显示分隔符，保留 IPA 本体、词头源行、音节及明确用途／地区条件；其他发音节变体、音频转写及尚未确定的节点继续保留为未选证据。

八项 `reviewed_form_relations` 另存原页支持的语法目标及条件：`la, les, l’, d’, une, des, ça, me`。`l’` 冠词指向 le／la，`d’` 介词指向 de，`des` 所选不定冠词指向 un／une；`ça` 为 cela 的缩写，`me` 为 je 的宾格。错误或缺失的原始 `form_of`／`alt_of` 保持不变，非所选记录的 de + les 缩合及代词省略继续独立保存，未推定应用身份归并规则。

### 字段与摘要质量

固定提取器的[法语版模型](https://github.com/tatuylonen/wiktextract/blob/19f9a3006a405fb7bfb40193dca41c7117a009a5/src/wiktextract/extractor/fr/models.py)定义：

- `word`、`lang_code`、`pos`／`pos_title`：词形、语言和词性；同词多词性保持独立义项。
- `senses[].glosses`、`tags`／`raw_tags`、`alt_of`／`form_of`：定义、用法／领域标签、异写与屈折指向。
- `sounds[].ipa`、`tags`：读音及地区／用法信息；同一词可有多种 IPA。
- `translations[]` 的 `lang_code`、`word`、`sense`、`sense_index`：译词、翻译框标签及编号；缺编号时默认 0，编号须独立核查。
- `senses[].examples[]` 的 `text`、`ref`、`translation`：例句及书籍／页码等引用；这里的 `translation` 是**例句的法语译文**，不承诺中文译句。

法语词典覆盖专业、旧义、地区用法、习语和屈折形式；通用定义也可能依赖另一个词。D2 小样应同时核验词元与 `form_of`、定义和译词的义项对应，以及常用义／用途标签。完整法语定义、英语解释和中文译词是三项不同的质量与覆盖目标。

该模型不输出源页修订、页 ID 或稳定义项 ID，内部 `pos_id` 也从 JSON 排除；`Sound` 没有可供摘要识别多词发音的词形／标题引用。资源版本、原词头及记录／义项位置只能标识本次小样，升级身份仍须冻结。[法语模型与排除的 `pos_id`](https://github.com/tatuylonen/wiktextract/blob/19f9a3006a405fb7bfb40193dca41c7117a009a5/src/wiktextract/extractor/fr/models.py#L157-L161)。56 个词头的独立官方 dump 引用已由声明的输入版本和标题身份闭合，详见[官方 dump 页引用](#官方-dump-页引用)。

### 文字、提取器和媒体许可

法语维基词典版权约定明确允许按 CC BY-SA 4.0 复制、分发和修改正文；该许可允许商业使用及格式转换，要求署名、许可链接、修改说明及适用的相同方式共享义务。[版权约定](https://fr.wiktionary.org/w/index.php?title=Convention%3ADroit_d%E2%80%99auteur&oldid=38903168)、[CC BY-SA 4.0 许可原文](https://creativecommons.org/licenses/by-sa/4.0/legalcode.en)。

Wiktextract 的固定 [LICENSE](https://github.com/tatuylonen/wiktextract/blob/19f9a3006a405fb7bfb40193dca41c7117a009a5/LICENSE)是 MIT，另指出部分测试文件属于 Wiktionary 内容许可。提取器 MIT 不覆盖词典文字；只读资源、来源通知和项目源码分别保留各自许可，符合 [ADR 0001](adr/0001-separate-bundled-resources-from-learning-data.md)及 [ADR 0002](adr/0002-source-and-resource-licensing.md)。

版权约定还明确标识外部文本、图像和声音可能保留其他权利。本轮小样的若干例句带书籍引用，`sounds` 也带 Commons 音频 URL；这些字段不提供逐文件授权链。主候选先采用原创词典定义、IPA、词性和可核验译词；例句按来源准入，词典录音／图片按文件描述许可另行准入。[版权约定中的外部材料条款](https://fr.wiktionary.org/wiki/Convention:Droit_d%E2%80%99auteur#Exemptions_exceptionnelles_et_obligations_concernant_les_autres_licences)。

全部 **56 个固定法语页的通知清单**已核对：14 页含 15 项明确导入通知。DAF8 的 12 页为 `être, avoir, aller, apprendre, école, aujourd’hui, cote, retentir, taire, ne, un, pouvoir`；CFC 的三项分别在 `œil` 的字体义、`cote` 的海拔义、`pas` 的制图名词义，均不属于所选摘要。`cote` 同时含两类通知。[`Import:DAF8`，22113837](https://fr.wiktionary.org/w/index.php?oldid=22113837)表示全部或部分文章来自原作；[`R:DAF8`，27621166](https://fr.wiktionary.org/w/index.php?oldid=27621166)只是普通文献引用。`de`、`le`、`en` 的参考文献因此不等同于导入声明；页级声明也不确定每个所选定义的作者。

**`ne` 的所选否定定义支持按 CC BY-SA 4.0 转换及随包再分发。**[修订 3386521](https://fr.wiktionary.org/w/index.php?oldid=3386521)仅有“Négation.”；Zorglub~frwiktionary 在 [3431284，2008-08-16 14:59:56 UTC](https://fr.wiktionary.org/w/index.php?oldid=3431284)加入“Première partie d’un morphème…”及 discordantiel／forclusif 说明。其直接子修订 [3431694，19:24:38 UTC](https://fr.wiktionary.org/w/index.php?oldid=3431694)由 Diligent 以“fusion DAF”保留该定义、加入后续 DAF8 文字并首次加入导入通知。[1935 年第八版原作 `ne`](https://www.dictionnaire-academie.fr/article/A8N0112)以“Particule négative”开头，没有所选现代形态学说明。此历史支持所选贡献者文字的授权路径；继续保留维基词典贡献者／文章历史署名、许可、选择／转换说明，以及“Dictionnaire de l’Académie française, huitième édition, 1932–1935”的**来源文章通知**。[固定版权约定，39851198](https://fr.wiktionary.org/w/index.php?oldid=39851198)、[Wikimedia §7](https://foundation.wikimedia.org/w/index.php?title=Policy%3ATerms_of_Use%2Fen&oldid=554852#7._Licensing_of_Content)。

另外 11 个 DAF8 标记页的所选定义已逐一核对。与上述 `ne` 合计，`être, aller, école, aujourd’hui, ne, un` 六项支持贡献者文字的 CC BY-SA 4.0 路线；`apprendre, cote, retentir, taire, pouvoir` 五项仍保留 DAF8 表达，`avoir` 则保留 TLFi 表达。判定范围为固定的所选定义。

| 词条／所选修订 | 原作与修订证据 | 所选定义重用判断 |
| --- | --- | --- |
| `être`，[39860125](https://fr.wiktionary.org/w/index.php?oldid=39860125) | [147](https://fr.wiktionary.org/w/index.php?oldid=147)加入“Définir un état, une caractéristique”；DAF 合并 [889037](https://fr.wiktionary.org/w/index.php?oldid=889037)保留它，[13958418](https://fr.wiktionary.org/w/index.php?oldid=13958418)加“du sujet”。[DAF8 原作](https://www.dictionnaire-academie.fr/article/A8E1965)以“Exister.”开头，没有所选整句。 | 支持 CC BY-SA 4.0 |
| `aller`，[39881261](https://fr.wiktionary.org/w/index.php?oldid=39881261) | 建页 [8890](https://fr.wiktionary.org/w/index.php?oldid=8890)已有“Se déplacer jusqu'à un endroit.”；DAF 合并 [1403106](https://fr.wiktionary.org/w/index.php?oldid=1403106)保留它。[DAF8 原作](https://www.dictionnaire-academie.fr/article/A8A0938)及 TLFi 对照均无该整句。 | 支持 CC BY-SA 4.0 |
| `école`，[39867888](https://fr.wiktionary.org/w/index.php?oldid=39867888) | 贡献者表述始见 [316](https://fr.wiktionary.org/w/index.php?oldid=316)；拼写整理后的“Lieu dédié à l’apprentissage.”已见 [852573](https://fr.wiktionary.org/w/index.php?oldid=852573)，由 DAF 合并 [887486](https://fr.wiktionary.org/w/index.php?oldid=887486)保留。[DAF8 原作](https://www.dictionnaire-academie.fr/article/A8E0196)定义为教授学科或技艺的机构。 | 支持 CC BY-SA 4.0 |
| `aujourd’hui`，[39793601](https://fr.wiktionary.org/w/index.php?oldid=39793601) | [12873](https://fr.wiktionary.org/w/index.php?oldid=12873)在 DAF 合并前加入“À la date présente, le jour même.”；[1396841](https://fr.wiktionary.org/w/index.php?oldid=1396841)保留它，[24574070](https://fr.wiktionary.org/w/index.php?oldid=24574070)只把“le”改为“ce”。[DAF8 原作](https://www.dictionnaire-academie.fr/article/A8A2339)为“Le jour où l’on est.”。 | 支持 CC BY-SA 4.0 |
| `un`，[39752995](https://fr.wiktionary.org/w/index.php?oldid=39752995) | [1091438](https://fr.wiktionary.org/w/index.php?oldid=1091438)将既有概念定义改写为所选两句；后来的 DAF 合并 [5381297](https://fr.wiktionary.org/w/index.php?oldid=5381297)保留它。[DAF8 原作](https://www.dictionnaire-academie.fr/article/A8U0020)的冠词段落与所选整段不同。 | 支持 CC BY-SA 4.0 |
| `apprendre`，[39773016](https://fr.wiktionary.org/w/index.php?oldid=39773016) | DaftBot 建页 [316729](https://fr.wiktionary.org/w/index.php?oldid=316729)导入[原作](https://www.dictionnaire-academie.fr/article/A8A1665)“Acquérir une connaissance.”；[1110123](https://fr.wiktionary.org/w/index.php?oldid=1110123)只追加“ou un savoir-faire”。 | DAF8 原表达再分发依据未闭合 |
| `cote`，[39840652](https://fr.wiktionary.org/w/index.php?oldid=39840652) | DaftBot 建页 [421317](https://fr.wiktionary.org/w/index.php?oldid=421317)与[原作](https://dictionnaire-academie.fr/article/A8C3092)一致；[3096178](https://fr.wiktionary.org/w/index.php?oldid=3096178)追加图书／档案用途，仍保留“dont on se sert pour classer les pièces d’un procès, d’un inventaire”的顺序表达。 | DAF8 原表达再分发依据未闭合 |
| `retentir`，[39796396](https://fr.wiktionary.org/w/index.php?oldid=39796396) | DaftBot 建页 [441530](https://fr.wiktionary.org/w/index.php?oldid=441530)导入[原作](https://www.dictionnaire-academie.fr/article/A8R1458)“Rendre, renvoyer un son éclatant.”；[26198465](https://fr.wiktionary.org/w/index.php?oldid=26198465)把逗号改为“ou”，原词汇均保留。 | DAF8 原表达再分发依据未闭合 |
| `taire`，[39681528](https://fr.wiktionary.org/w/index.php?oldid=39681528) | DaftBot 建页 [444490](https://fr.wiktionary.org/w/index.php?oldid=444490)导入[原作](https://www.dictionnaire-academie.fr/article/A8T0068)“Ne pas dire.”；[1401368](https://fr.wiktionary.org/w/index.php?oldid=1401368)追加“passer sous silence”，所选定义仍保留原句。 | DAF8 原表达再分发依据未闭合 |
| `pouvoir`，[39819957](https://fr.wiktionary.org/w/index.php?oldid=39819957) | [1037991](https://fr.wiktionary.org/w/index.php?oldid=1037991)在既有“Être capable…”义中加入[原作](https://www.dictionnaire-academie.fr/article/A8P2383)“Avoir la faculté, être en état de.”及例句；[26615278](https://fr.wiktionary.org/w/index.php?oldid=26615278)再加“être en mesure de”，仍保留两项原作表述。 | DAF8 原表达再分发依据未闭合 |
| `avoir`，[39879166](https://fr.wiktionary.org/w/index.php?oldid=39879166) | [9126711](https://fr.wiktionary.org/w/index.php?oldid=9126711)改用 [TLFi](https://www.cnrtl.fr/definition/avoir)的关系定义；[14795478](https://fr.wiktionary.org/w/index.php?oldid=14795478)加入 possessive 及主语／宾语角色，保留 concrete／abstract、permanent／occasional 成对表达。 | TLFi 公开条款排除商业利用 |

`avoir` 的所选原句虽不同于 [DAF8 原作](https://www.dictionnaire-academie.fr/article/A8A2534)，仍保留 [TLFi](https://www.cnrtl.fr/definition/avoir)的关系定义及具体／抽象、永久／偶然的成对表达。[ATILF 的 TLFi 资源页](https://www.atilf.fr/ressources/tlfi/)明确限定教学／研究查阅，“à l’exclusion de toute exploitation commerciale”；另允许附指定引用的科学出版摘录。[TLFi 原网站](http://atilf.atilf.fr/tlf.htm)重复此限制，官方 FAQ 将 CNRTL 说明为同一资源的另一界面。上述公开许可不覆盖离线词典转换及商业随包再分发；`avoir` 的当前 TLFi 表达不符合 [ADR 0002](adr/0002-source-and-resource-licensing.md)，需独立的目标用途授权或来源权利已核明的其他定义。

六项贡献者定义均保留维基词典文章／贡献者历史署名、CC BY-SA 4.0、选择／转换说明和 DAF8 来源文章通知。文字来源与再分发权利分别记录；贡献者修改及导入通知的加入日期均不能替代原作权利依据。

法国 [现行 L123-3](https://www.legifrance.gouv.fr/codes/article_lc/LEGIARTI000006278941)规定匿名／笔名／集体作品从出版次年起算 70 年，分期发布的各项分别起算；[1994 年的 L123-3](https://www.legifrance.gouv.fr/codes/article_lc/LEGIARTI000006278940/1994-05-11)为 50 年，且对 20 年内出版完结的分期集体作品，整体期限按最后一项的出版年计至第 50 年。两版完整条文现已保留，适用仍取决于作品分类及相关事实。

[1997 年法第 16 条的修正文本，WIPO Lex FR433](https://www.wipo.int/wipolex/fr/legislation/details/14244)完整原文确认：第二编从 **1995-07-01** 适用；此前已进入公有领域的作品，只有在该日仍于至少另一欧共体成员国受保护时才恢复权利。对 DAF8 尚未建立该具体作品的成员国保护状态及 **1996-01-01** 的法国保护状态。[17 USC 104A](https://www.govinfo.gov/content/pkg/USCODE-2024-title17/html/USCODE-2024-title17-chap1-sec104A.htm)的恢复条件要求作品未因保护期届满而在来源国进入公有领域；[304](https://www.govinfo.gov/content/pkg/USCODE-2024-title17/html/USCODE-2024-title17-chap3-sec304.htm)的适用期限不能代替这些事实。现有依据既未证明五项 DAF8 表达可按目标方式再分发，也未判定它们在美国必然受保护。

Académie 的[第八版序言](https://www.academie-francaise.fr/le-dictionnaire-les-neufs-prefaces/preface-de-la-huitieme-edition-1932-1935)说明词典委员会与学院会议持续参与编写；这不能确定外国法下的作者身份或作品分类。[WIPO 德国法律记录 1008](https://www.wipo.int/wipolex/fr/legislation/details/1008)收录截至 1996-07-19 的法律，列有作者定义、具名／匿名期限及欧盟／欧洲经济区国民待遇；其一般规则没有建立 DAF8 在 1995-07-01 的具体保护状态，也不是该截止日以前的法律版本证明。所查成员国路线仍未提供第 16 条恢复条件所需的作品事实。

Académie 现行[网站条款 §§5–6](https://www.dictionnaire-academie.fr/CGU)没有为旧版文字提供另一项公开再分发授权。五项 DAF8 保留表达继续等待作品及目标辖区的权利依据；`avoir` 单独记录为公开条款商业限制。CFC 原作权利未准入，所选三个摘要继续排除其明确导入义项。

### 固定页中的替代定义

六个固定页的全部相关定义已按词性、语法角色、核心义及可追溯文字来源核对；未建立适合替换当前摘要的贡献者定义。

| 固定页 | 最近的其他定义 | 核对结果 |
| --- | --- | --- |
| [`apprendre`，39773016](https://fr.wiktionary.org/w/index.php?oldid=39773016) | “Développer ses connaissances.” | 明示不及物用法，当前摘要为及物学习义 |
| [`cote`，39840652](https://fr.wiktionary.org/w/index.php?oldid=39840652) | 无第二个分类标记义 | 所选第 12 定义行是唯一对应义 |
| [`retentir`，39796396](https://fr.wiktionary.org/w/index.php?oldid=39796396) | “Faire ou produire un bruit éclatant.” | 含义接近，但建页 [441530](https://fr.wiktionary.org/w/index.php?oldid=441530)与 [DAF8 原作](https://www.dictionnaire-academie.fr/article/A8R1458)均已有该整句，未建立贡献者独立表述 |
| [`taire`，39681528](https://fr.wiktionary.org/w/index.php?oldid=39681528) | “Garder le silence, s’abstenir de parler.” | 代词式用法，不能替换及物的隐瞒／不说义 |
| [`pouvoir`，39819957](https://fr.wiktionary.org/w/index.php?oldid=39819957) | “Avoir la possibilité matérielle…” | 独立的现实可行性义，与所选一般能力义不同；名词义也不匹配 |
| [`avoir`，39879166](https://fr.wiktionary.org/w/index.php?oldid=39879166) | “Se procurer, obtenir quelque chose.”及标为口语的性义 | 前者是取得义，并与 DAF8 相同；后者改变用法及含义；唯一一般领有定义仍是所选 TLFi 表达 |

### XMLittré：有明确许可的历史替代定义

François Gannaz 的 XMLittré 固定于提交 `b44d144d12a153799e2c8f357ea91b049f8d83e2`（2017-11-16）。[固定 README](https://api.bitbucket.org/2.0/repositories/Mytskine/xmlittre-data/src/b44d144d12a153799e2c8f357ea91b049f8d83e2/README.md)及五份完整 XML 头部明确把 **全部带标记数据置于 CC BY-SA 3.0**；[原作者 FAQ](https://www.littre.org/faq#licence)区分公有领域纯文字与有许可的 XML／HTML。转换及商业随包再分发须按 [CC BY-SA 3.0 原文](https://creativecommons.org/licenses/by-sa/3.0/legalcode)保留作者／来源、许可与通知、修改说明及适用的相同方式共享条件。

[BnF FRBNF30824717](https://catalogue.bnf.fr/ark:/12148/cb30824717s)确认 Émile Littré（1801–1881）、巴黎 Hachette、1873–1874 年四卷本；[电子版说明](https://www.littre.org/faq#referencer)标识第二版及部分合并的 1878 年补编。本轮起句均来自主释义 `corps`。法国 [L123-1](https://www.legifrance.gouv.fr/codes/article_lc/LEGIARTI000006278937)的普通终身加 70 年经济权利期限对此历史原作已届满；[L121-1](https://www.legifrance.gouv.fr/codes/article_lc/LEGIARTI000006278891)的永久精神权利仍适用。美国[版权局 Circular 38B，第 2 页](https://www.copyright.gov/circs/circ38b.pdf)说明恢复权利仅覆盖普通期限剩余部分，1978 年前出版作品一般按首次出版加 95 年计算；1873–1874 年原作及 1878 年补编均已超过该期限上限。这支持法国／美国的历史原作路线，现代转录按 XML 的明确许可处理。

下表是固定 XML 主释义的**原文起句**，保留历史 `v. a.`／`v. n.`／`s. f.` 标签及源义项位置。

| 词／固定 XML 义项 | 原文起句 | 与当前核心义的关系 |
| --- | --- | --- |
| [`apprendre`](https://api.bitbucket.org/2.0/repositories/Mytskine/xmlittre-data/src/b44d144d12a153799e2c8f357ea91b049f8d83e2/a.xml)，variant 1，`v. a.` | Acquérir une connaissance, retenir dans sa mémoire. | 学习知识；原例也包括钢琴、阅读及舞蹈 |
| [`cote`](https://api.bitbucket.org/2.0/repositories/Mytskine/xmlittre-data/src/b44d144d12a153799e2c8f357ea91b049f8d83e2/c.xml)，variant 4，`s. f.` | Marque alphabétique ou numérale servant à classer chaque pièce d'un procès ou d'un dossier. | 诉讼／卷宗的分类标记，范围较当前图书／档案／清单义窄 |
| [`retentir`](https://api.bitbucket.org/2.0/repositories/Mytskine/xmlittre-data/src/b44d144d12a153799e2c8f357ea91b049f8d83e2/r.xml)，variant 1，`v. n.` | Rendre, renvoyer un son éclatant. | 同一核心回响义；当前摘要用“ou”连接 |
| [`taire`](https://api.bitbucket.org/2.0/repositories/Mytskine/xmlittre-data/src/b44d144d12a153799e2c8f357ea91b049f8d83e2/t.xml)，variant 1，`v. a.` | Ne pas dire, cacher. | 及物不说／隐瞒义，与 `se taire` 分开 |
| [`pouvoir`](https://api.bitbucket.org/2.0/repositories/Mytskine/xmlittre-data/src/b44d144d12a153799e2c8f357ea91b049f8d83e2/p.xml)，动词 entry 1／variant 1，`v. n.` | Avoir la faculté de, être en état de. | 一般能力义；保留原语法标签及不定式用法 |
| [`avoir`](https://api.bitbucket.org/2.0/repositories/Mytskine/xmlittre-data/src/b44d144d12a153799e2c8f357ea91b049f8d83e2/a.xml)，动词 entry 1／variants 1、3，`v. a.`；variant 3 原标 `Fig.` | Posséder un objet physique, posséder quelqu'un ou quelque chose dans un certain état. ／ Posséder une chose immatérielle, une qualité ; éprouver une sensation ou un sentiment ; être dans un état ; être âgé de ; être d'une dimension de. | 分别覆盖物质领有和非物质／状态义，改变当前关系定义的义项组织 |

#### 定义的原文边界

六词的七项起句已建立**精确源文本跨度**：固定 entry／variant／直接文本 XPath、原始 UTF-8 半开字节范围、解码后字符范围及完整直接文本均保存在候选 JSON，七项源字节切片与既有起句逐字相等。没有扩写或现代化法语正文。

`apprendre`、`cote` 及 `avoir` variants 1／3 的四个直接文本节点在定义后继续包含例文；`retentir`、`taire`、`pouvoir` 的对应节点在引用前仅余空白。因此不能直接采用整个首文本节点。后续 `cit`／`indent` 不在所选范围，`cote` 的另一个缩进义 “La couverture même.” 也独立于所选分类标记义。`avoir` variant 3 取自 `Fig.` 指示节点后的 tail，历史 `Fig.` 与语法标签分别保留。

五项起句可表达其注明的历史核心义；`pouvoir` 的 `de` 补语框及 `avoir` variant 3 的年龄／尺寸等框是完整的历史词典定义，学习者呈现仍需对应补语语境。源文有未选入定义的 “On ne peut être.... on ne peut faire.... il est impossible d'être.... de faire....”、“Il avait vingt ans.” 和 “Rue qui a 10 m. de large.”；三处均已保留准确源位置，未填写新的法语例文或现代助动词标签。

七项候选仅选择 XMLittré 法语跨度及其历史元数据，不携带跨来源译词。来源通知为 Émile Littré 原作、François Gannaz／littre.org XML 转录、固定提交及逐义引用，所选 XMLittré 贡献保留 CC BY-SA 3.0；完整许可及源通知随适用内容保留。

#### 有界字段覆盖与学习者质量

原 **57 查询／56 标准词头**保持冻结。已解析固定提交的全部 **19 份所需字母文件**，其中五份复用既有完整材料、14 份新增完整下载；覆盖统计保留重音、撇号、连字、空格和原始标题，仅明确区分以下对应方式。

| 来源对应方式 | 原查询 | 标准词头 |
| --- | --- | --- |
| 与完整源标题逐字相等 | 0／57 | 0／56 |
| 仅改变大小写后与完整标题相等 | 45／57 | 44／56 |
| 形式在组合源标题中明确列出 | 8／57 | 8／56 |
| 后两项并集 | **51／57** | **50／56** |

逐字相等为零源于大写标题。五个组合标题是 `CHAT, CHATTE`、`IL, au singulier, ILS, au pluriel`、`LE, LA, LES`、`TOUT, TOUTE`、`UN, UNE`；`la`、`les` 同时属于大小写对应，故计并集。未覆盖的原查询为 `aujourd’hui`、`s'il vous plaît`、`se souvenir`、`procrastiner`、`l'`、`d'`；`AUJOURD'HUI` 支持另一条 ASCII 撇号查询，`SOUVENIR` 条目存在但未列出 `SE SOUVENIR` 词头。没有据此新增拼写或代词式别名。

这些对应涉及 **69 个源条目**。`AVOIR`、`POUVOIR` 各有动词／名词条目，`ÊTRE` 有三个条目；词头对应并不完成词性及义项选择。

| 字段或结构 | 匹配样本的机械统计 | 可采用含义 |
| --- | --- | --- |
| `corps` 正文 | 69／69 条目、50／50 已覆盖标准词头 | 有些是交叉引用或混合文本，不能直接等同于独立定义 |
| 带标记的语法标签 | 条目头部 60／69；任一 `nature` 位置 64／69；46／50 已覆盖标准词头 | 部分语法位于义项中；保留历史原标签 |
| `prononciation` 头部 | 66／69 条目、49／50 已覆盖标准词头；`ce` 缺项 | 历史拼读及使用说明；明确 IPA 字段为零 |
| `variante` 节点 | 顶层 954 个；包含嵌套共 1,324 个 | 结构变体数量，不能作为已验证现代义项数量 |
| `cit` 引用节点 | 8,400 个，其中 `corps` 内 5,000 个 | 文学引文与所选定义起句分别保留来源 |
| 译词／CEFR 字段 | 明确字段均为零 | 不提供结构化中英译词或学习者等级 |

`apprendre` 的首个直接文本节点混有定义与钢琴、阅读、舞蹈等例文；`avoir` 的发音包含十七世纪及当时用法说明。历史 `v. n.` 也不能直接推成现代助动词标签。上述观察符合[固定 README](https://api.bitbucket.org/2.0/repositories/Mytskine/xmlittre-data/src/b44d144d12a153799e2c8f357ea91b049f8d83e2/README.md)对不完整／错误标记及不作现代化的说明。六词核心义的有界对应见前表；全样干净定义提取、现代发音适用性及全部引用权利另列于限制。

#### 印刷对照

七项起句均已按固定 XML 核对，其中 **3／7** 已与可读扫描作可视比对。Internet Archive 的[卷 I](https://archive.org/details/dictionnairedel01litt)与[卷 III](https://archive.org/details/dictionnairedel03litt)题名页均可见巴黎 Hachette、1873 年，分别为 A–C、I–P；两页未印可见的版次序号。BnF 与转录者提供的第二版说明不等同于题名页版次序号的观察。

| 固定 XML 义项 | 可见印刷卷／页 | 图像文件标识 | 原文比对 |
| --- | --- | --- | --- |
| `avoir` variants 1／3，两项起句 | 卷 I，第 269 页 | [0337](https://iiif.archive.org/image/iiif/3/dictionnairedel01litt%2Fdictionnairedel01litt_jp2.zip%2Fdictionnairedel01litt_jp2%2Fdictionnairedel01litt_0337.jp2/full/max/0/default.jpg) | 两项法语起句一致，variant 3 的 `Fig.` 可见 |
| `pouvoir` 动词 entry 1／variant 1 | 卷 III，第 1260 页 | [1268](https://iiif.archive.org/image/iiif/3/dictionnairedel03litt%2Fdictionnairedel03litt_jp2.zip%2Fdictionnairedel03litt_jp2%2Fdictionnairedel03litt_1268.jp2/full/2000,/0/default.jpg) | 法语起句一致 |

比对忽略排印空格及换行断字，未改写固定 XML 起句。表中页码来自可见印刷页头，图像文件标识单独保留；[BnF IIIF 文档](https://api.bnf.fr/fr/api-iiif-de-recuperation-des-images-de-gallica)及其[接口规格](https://api.bnf.fr/sites/default/files/2021-02/api-gallica-iiif_0.yml)同样区分数字视图与编者分页，Gallica 的 `f242` 等标识不能代替印刷页码。

`apprendre`、`cote` 的目标正文图像未成功读取，`retentir`、`taire` 所需的目标卷 IV 可读扫描未取得，四项保持未核对。[Gallica 查看页](https://gallica.bnf.fr/ark:/12148/bpt6k5406710m/f242)、六项图像请求及三份卷清单仍受 HTTP 403 访问限制；拒绝响应保留于证据目录。已完成三项的一致性不覆盖其余四项或全量转录。

## 独立 SQLite 核心小样

本轮分别把上述 26 个词头、44 个原始记录及单词文件对照投影为独立研究库，沿用现有资源的 `ordinal`／二进制词头索引与参数化查询方式。保留定义、词性、用法标签、词形关系、IPA、中英译词及每条来源 JSONL URL；排除引用例句、音频、图片。原始库另保留 dump／提取器版本、压缩输入 ETag、明确重定向和未知页修订。`record_json` 是实验格式，尚未接入现有 TEI 解析器或应用资源版本。

| 实测内容 | 原始投影／字节 | 单词文件对照／字节 |
| --- | ---: | ---: |
| 26 词头、44 记录的输入 JSONL | 1,768,344 | 1,768,388 |
| 核心字段 JSONL | 82,829 | 82,829 |
| 核心 JSONL gzip | 16,667 | 16,546 |
| SQLite | 131,072 | 131,072 |
| SQLite gzip | 19,631 | 19,729 |

两个库均通过完整行与来源 URL 回读、全部 27 参数化查询、缺项查询、`cote`／`côte`／`côté` 分离、SQLite 完整性检查，以及 `mode=ro`／`query_only` 下的写入拒绝；原始库还通过完整版本／来源元数据回读。研究库位于独立忽略目录，与用户学习库分开。

小样准备了 CC BY-SA 4.0 原文及通知草稿，列出来源版、所选输入、词条／贡献历史链接和过滤转换方式。上述体积只计字段及数据库；44 行有大量多义词，不能按词头数线性推算全量包体。译义及 IPA 抽取问题原样保留供核查；生产摘要需确定有来源依据的所选义项、译词和音标。

## 资源契约与来源通知

### 审核摘要的结构化资源

沿用现有独立资源、`ordinal`／二进制词头索引、参数化查询与只读 SQLite 边界，审核摘要保存为 `record_json`。以下保留原 27 查询的冻结小样：完整法语 `glosses`、记录级语法限制、义项 `tags`／`raw_tags`／`topics`、31 个词头 IPA 值及其地区／用法标签、完整所选中英翻译节点均保留；无译词使用空列表，CEFR 为 `null`。完整 44 条原始记录及审核源行作为独立研究证据保留。

| 小样结构 | 保存的内容 |
| --- | --- |
| `resource(provenance)` | 来源版、dump／提取日期、输入 URL／ETag、两个提取器提交、许可、资源版本和所选内容版本 |
| `entries(ordinal, key, record_json)` | 26 个词头各一项审核摘要及来源引用；`key` 使用二进制精确比较 |
| `aliases(alias, key, source_json)` | 两项原始流与官方 dump 均确认的重定向，另存重定向页引用并关联目标词头 |
| `core.sqlite3`、`provenance.json`、`NOTICE.md`、`COPYING` | 数据库、独立来源清单、通知草稿及完整 CC BY-SA 4.0 许可原文 |

小样的格式版本为 1，SQLite `application_id` 为 `0x4C584432`；所选内容版本为 `d2-reviewed-27-query-core-v1`。这些值标识研究契约。`ordinal`、记录位置及义项位置只在相应资源版本内有效；词头来源引用另存资源／内容版本和审核参考。

| 审核摘要小样实测 | 字节 |
| --- | ---: |
| 含审核源行的研究 JSON 输入 | 91,808 |
| 核心内容及逐词引用 JSONL | 46,431 |
| 核心 JSONL gzip | 5,140 |
| 26 条摘要／2 项重定向的 SQLite | 81,920 |
| SQLite gzip | 7,268 |
| 四文件资源包总量 | 104,622 |
| 四文件资源包 tar.gz | 14,773 |

四文件总量包括 SQLite 81,920、完整许可 20,138、通知 1,372 及来源清单 1,192 字节。回读逐项等于选定内容；全部 27 参数化查询、两个明确重定向、直接弯撇号短语查询、重音／大小写区别、注入式缺项查询、繁体写法和未知字段均通过。最长定义仍为 `cote` 的 214 字符／224 UTF-8 字节。只读写入被拒绝，缺资源没有生成新库，截断库报告数据库错误；完整许可、清单及压缩包逐文件回读通过。

### 两组合并的 57 查询资源契约

原 26 条及新增 30 条审核摘要合并为 **56 词头／四项明确重定向**，所选内容版本为 `d2-reviewed-57-query-core-v1`，格式版本及只读边界继续为 1。两份输入保持冻结，原始记录序号分别只在各自选行文件中有效；所选义项和同词记录位置按固定来源版本解释。新资源保留 **64 个所选 IPA 值**及完整用途／地区证据、原始译词节点、跨记录译词原位置、原始和审核词形关系，以及逐词官方引用、通知和文字权利状态。CEFR 均为 `null`，未将研究包标记为已准入资源。

| 合并研究小样实测 | 字节 |
| --- | ---: |
| 含完整审核源行的合并研究 JSON | 354,261 |
| 核心内容、关联证据及逐词引用 JSONL | 252,855 |
| 核心 JSONL gzip | 23,362 |
| 56 条摘要／四项重定向 SQLite | 311,296 |
| SQLite gzip | 27,720 |
| 四文件资源包总量 | 336,420 |
| 四文件资源包 tar.gz | 35,891 |

四文件为 SQLite 311,296、完整 CC BY-SA 4.0 许可 20,138、通知 3,534、来源清单 1,452 字节；通知含 DAF8／CFC 来源文章及原有声明，文字权利按具体来源范围解释。完整 57 查询的定义／IPA 为 57／57，英语为 49／57、中文为 30／57。最长所选定义 `ce` 为 346 字符／363 UTF-8 字节，原文和限定完整回读。

全部 57 参数化查询、四项来源重定向、重音／大小写区别、未知值、语法／义项限定、繁体写法、IPA 用途证据及通知均通过。`il`／`on` 翻译保持原始记录 2 的节点位置及关联证据；`l’` 的原始错误目标与审核后的 le／la 关系分别回读。只读拒写、缺资源不创建库、截断资源报错、完整许可／来源清单／tar.gz 逐文件回读均通过。该测量含小样审核证据，不能按 56 词头线性推算全量资源体积。

[现有词典查询器](../crates/dictionary/src/lib.rs)读取 `entry_xml`，其 [TEI 解析器](../crates/dictionary/src/entry.rs)只投影读音、词性／性别、定义与未分语言的译词；义项限定及完整译词节点无法通过已有返回类型保留。[前端契约](../apps/desktop/src/platform/dictionary.ts)同样没有译词语言、CEFR 或逐义引用字段；[已存兜底摘要](../crates/storage/src/vocabulary.rs)只保存读音、语法和单项译词。因此本小样确定的是可回读的数据契约，应用接入需在现有词典、IPC 与保存边界扩展相应字段及格式识别。

生词身份继续遵循 [ADR 0004](adr/0004-one-vocabulary-record-with-reference-digest-and-notes.md)：NFC 标准词头决定同词记录，保留重音、大小写和撇号区别；可靠重定向用于查词解析，保存身份的归并规则另按确认需求实施。收藏时冻结来源引用、资源／内容版本和兜底摘要。升级资源后，旧引用或旧位置无法解析时保留原摘要；独立笔记与复习状态继续独立保存。

### 官方 dump 页引用

本阶段再次读取 [Kaikki 原始页](https://kaikki.org/frwiktionary/rawdata.html)，仍指向 2026-10-01 dump、2026-11-02 提取；[Wikimedia 官方任务清单](https://dumps.wikimedia.org/frwiktionary/20261001/dumpstatus.json)的相应 multistream 任务为 `done`。依照[官方 multistream 格式](https://meta.wikimedia.org/wiki/Data_dumps/Dump_format#Multistream_dumps)，索引的 `file-offset:page-id:page-title` 给出可独立解压的 bzip2 流起点；本轮下载[首分区索引](https://dumps.wikimedia.org/frwiktionary/20261001/frwiktionary-20261001-pages-articles-multistream-index1.txt-p1p1500000.bz2) 8,058,092 字节，再按相邻起点截取[对应 XML 分区](https://dumps.wikimedia.org/frwiktionary/20261001/frwiktionary-20261001-pages-articles-multistream1.xml-p1p1500000.bz2)。

22 个 HTTP 206 范围共 3,579,735 压缩字节，恢复 26 个词头页及两项重定向；XML 分区全长为 271,186,128 字节。各范围使用同一 ETag `"6abe52a2-1029f8d0"`，后续请求带 `If-Match`；原始压缩流、范围响应及解出的目标 wikitext 均保留。26 个词头的 page ID、修订 ID 和完整 wikitext 与此前截止日前审核参考逐项一致；两个重定向目标也与原始 JSONL 一致。

扩展样本复用该索引及七个已有流，另取七个 HTTP 206 范围、1,194,689 字节；14 个相关流共 3,083,092 字节。新增 30 个词头及两项重定向的 **32／32 修订 ID、32／32 完整 wikitext** 均与截止日前的 API 审核参考一致。两组累计恢复 **56 个不同词头及四项明确重定向**；原始／单词文件对象对照及词头身份断言通过。

固定 [dump 解析器](https://github.com/tatuylonen/wikitextprocessor/blob/6f96ddf5f3466707f1869fe6618390fd77dfd109/src/wikitextprocessor/dumpparser.py#L45-L87)读取标题、正文及 model，未保留数字页／修订 ID；[法语页解析器](https://github.com/tatuylonen/wiktextract/blob/19f9a3006a405fb7bfb40193dca41c7117a009a5/src/wiktextract/extractor/fr/page.py#L225-L271)把 `page_title` 写为 `word` 再序列化。这条生产者声明版本／固定代码／直接标题链支持为全部 143 条法语记录推导对应的官方输入页修订引用。

逐词引用分别保存文章／贡献历史 URL、语义审核修订，以及独立官方 dump 的 page ID／修订／时间／永久链接。已有冻结小样的 `exact_raw_dump_revision` 保持 `null`，表示该值未从 Kaikki 自带字段读取；新引用可以保存已独立推导的声明输入修订及其推导依据。页级引用与本版本的记录／义项位置分别保存，输出重现是独立的转换验证。

小规模重现实验可先用一个完整词头、同 dump 的模板／模块等支持页、两项固定提交及新空数据库，再逐记录比较已保留 JSON。固定 [CLI](https://github.com/tatuylonen/wiktextract/blob/19f9a3006a405fb7bfb40193dca41c7117a009a5/src/wiktextract/wiktwords.py#L427-L459)在缓存已有页时跳过 dump 导入；[单页接口](https://github.com/tatuylonen/wiktextract/blob/19f9a3006a405fb7bfb40193dca41c7117a009a5/src/wiktextract/wiktwords.py#L46-L84)允许有界执行。支持页、缓存及运行环境影响输出重现；现有[准入要求](development-readiness.md#词典变位与分级数据)并未把全量上游重跑列为建立来源引用的前提。

### 署名、修改说明与许可范围

[Wikimedia 使用条款，修订 554852](https://foundation.wikimedia.org/w/index.php?title=Policy%3ATerms_of_Use%2Fen&oldid=554852)的 §7(b)(i) 和 §7(g)(i)明确接受以原文章 URL 署名，因为文章历史列出贡献者。每个重用词头保留其规范原页及贡献历史链接；单独的 Kaikki 下载页无法承担逐文章署名。该方式不要求取得确切 dump 修订，也不要求离线复制全部贡献者名单；包内仍保存完整许可与来源通知。

CC BY-SA 4.0 [§2(a)(4)](https://creativecommons.org/licenses/by-sa/4.0/legalcode.en#s2a4)规定必要技术格式转换本身不产生 Adapted Material；本小样完整法语定义是措辞未改的摘录。通知如实记录字段／义项选择、结构化表示和 IPA 显示分隔符变化。若后续实质改写或翻译受保护正文，分享时按 [§3(a)／§3(b)](https://creativecommons.org/licenses/by-sa/4.0/legalcode.en#s3)保留署名、修改说明及相应 ShareAlike 许可；已有附加来源／修改通知随适用内容保留。

资源通知草稿的核心文字为：

```text
Source text: Wiktionnaire (French Wiktionary), by its contributors.
Each reused article and its contributor history are retained in the source citation.
Source: frwiktionary 2026-10-01; Kaikki extraction 2026-11-02.
Resource version: frwiktionary-2026-10-01-extracted-2026-11-02.
Selected-content version: d2-reviewed-27-query-core-v1.
Dictionary text and resource-data contributions: CC BY-SA 4.0.
https://creativecommons.org/licenses/by-sa/4.0/ — full license and disclaimer: COPYING.
Changes: fields and senses selected; structured SQLite representation;
French definition wording unchanged; IPA display delimiters changed.
```

引用中的署名、文章／历史链接、许可、修改说明及冻结版本随保存的来源摘要保留；含来源文字的共享导出也携带对应通知，资源被移除时引用仍可解析这些值。独立编写的笔记与学习状态保留自身权利；应用及转换代码继续 MIT。这复用[现有 Lexique 资源通知](../resources/lexique-3.83/README.md)和 [ADR 0002](adr/0002-source-and-resource-licensing.md)的数据／代码许可安排。

### 六词代用释义的来源关联

六个待处理词均有固定维基词典动词／名词词头行支持所选 IPA。采用 XMLittré 法语定义时，可独立保留这些词头 IPA 的来源；译词则须重新核对义项对应，不能只按同词拼接。

| 固定译词框 | XMLittré 代用义的边界 |
| --- | --- |
| [`cote`](https://fr.wiktionary.org/w/index.php?title=cote&oldid=39840652) 的分类框包括诉讼、清单、图书馆及档案，英语为 `code`／`call number` | 所选 XML 义仅为诉讼／卷宗分类标记，未建立图书馆专用 `call number` 的对应 |
| [`avoir`](https://fr.wiktionary.org/w/index.php?title=avoir&oldid=39879166) 的 `Posséder` 框提供 `have`／`有` | 不能给包含年龄、尺寸及状态的 variant 3 每个分句统一套用这两个译词 |

固定六词记录还含 `definition_wikitext`，部分译词框标签重复旧法语定义。只替换 `definition.glosses` 会在其他字段继续保留待核权利的表达；随包候选须同时确定可再分发的证据内容与引用范围。原记录及十项译词保持在冻结研究材料中。

来源通知分别保留 XMLittré 标记文字的 **CC BY-SA 3.0** 与符合准入范围的维基词典 IPA／译词的 **CC BY-SA 4.0**，并记录各自固定词头／词性／义项及作者、许可和修改说明。[3.0 §3／§4(a)](https://creativecommons.org/licenses/by-sa/3.0/legalcode)允许必要格式转换并保留作品许可；[§4(b)](https://creativecommons.org/licenses/by-sa/3.0/legalcode#s4b)的后续版本许可针对改编作品。转换成 JSON／SQLite 本身不构成把原有 3.0 文字统一改标 4.0 的依据；[4.0 §2(a)(4)](https://creativecommons.org/licenses/by-sa/4.0/legalcode.en#s2a4)明确技术转换不产生 Adapted Material。上述单一维基词典通知草稿的范围因此不能直接覆盖新增 XML 来源。

## CEFR：可用语义与准入缺口

FLELex 的语料是按 CEFR 级别组织的法语外语教材阅读材料和简写读物，描述的是接受性词汇。TT 和 CRF 报告各级每百万词归一频率及总频率；某个词出现在 B2 材料中，不代表它只能属于 B2，也不是该级必背词单。[FLELex 原作者说明](https://cental.uclouvain.be/cefrlex/flelex/)。

TT 为 14,236 个词元／词性条目，不包含多词表达；CRF 为 17,871 个条目，包含多词表达。TT-Beacco 在分布信息上结合专家知识推断单个 CEFR 等级，是比频率列更接近本项目分级需求的版本，仍需评估推断误差和词义差异。[版本说明](https://cental.uclouvain.be/cefrlex/flelex/download/)、[Beacco 方法原论文](https://aclanthology.org/2020.readi-1.13/)。

三个官方下载均明确 CC BY-NC-SA 4.0。按当前资源政策，它们不能直接随默认包分发；可行的授权路线是取得覆盖目标转换和商业再分发的额外许可。公开查询和学术可用性不替代该许可。[官方下载及许可](https://cental.uclouvain.be/cefrlex/flelex/download/)、[ADR 0002](adr/0002-source-and-resource-licensing.md)。

非 NC 候选 `vekkt/french_CEFR` 已固定提交 `d6affd3ff35c09dc378e32070ca77db32783bb19`，最后修改于 2022-12-13。仓库只有 `.gitattributes`、README 和 train／val／test CSV；README 只有 `license: mit` 声明。实际字段是句子、A1–C2 难度和 0–5 标签；没有词元、词性、义项、分级判据或标注者说明。MIT 标签没有解决原始句子来源的权利链；句子等级也不能分配给其中每个词。[固定文件树](https://huggingface.co/datasets/vekkt/french_CEFR/tree/d6affd3ff35c09dc378e32070ca77db32783bb19)、[固定卡片](https://huggingface.co/datasets/vekkt/french_CEFR/blob/d6affd3ff35c09dc378e32070ca77db32783bb19/README.md)、[实际数据字段](https://huggingface.co/datasets/vekkt/french_CEFR)。

### CEFR-J x28：法语转译与英文授权

[TUFS 项目说明](https://www.tufs.ac.jp/education/program/sgu/cefr-j_28.html)明确将英文 CEFR-J Wordlist 多语言化。本轮取得的法语下载是 [A1](https://www.tufs.ac.jp/documents/collaboration/intlaffairs/sgu/French_WL_A1.pdf)与 [A2](https://www.tufs.ac.jp/documents/collaboration/intlaffairs/sgu/French_WL_A2.pdf)各一页，列为 French／English／Japanese／CEFR；尚未取得完整法语 A1–B2 数据。样本中的 `avion` 因英文 `airplane`／`aeroplane` 分别列为 A1／A2，另有屈折形式 `suis`。这些是转译后的行标签，不能直接归并成一个法语标准词头等级。

[CEFR-J 英文资源说明](https://cefr-j.org/download_eng)明确允许 English Wordlist Version 1.6（2020-03-24）在注明来源后用于研究及商业用途；该授权针对英文词表。[TUFS 网站政策](https://www.tufs.ac.jp/sitepolicy/)保留未另行授权内容的大学版权，x28 法语页面及两份样本没有明确的可转换、商业再分发许可，也没有独立法语判级验证。法语译稿的权利、完整固定版本及法语词元／词性或义项依据仍需核对。

### Vocabcraft 与 DIALANG

[Vocabcraft 法语 deck](https://vocabcraft.com/decks/french)列 3,407 词并声明 CC BY-SA 4.0，同时明确 CEFR 是自己的词频启发式，底层为 OpenSubtitles 2018。其 FrequencyWords 的[固定 README](https://github.com/hermitdave/FrequencyWords/blob/525f9b560de45753a5ea01069454e72e9aa541c6/README.md)区分代码 MIT 与内容 CC BY-SA 4.0，并可固定[法语频率文件](https://github.com/hermitdave/FrequencyWords/blob/525f9b560de45753a5ea01069454e72e9aa541c6/content/2018/fr/fr_50k.txt)。本轮没有闭合字幕语料权利或取得不可变的 deck 发布；生产者也没有提供可核查的法语词义标注、专家审定或独立 CEFR 验证。频率用途与 CEFR 分级用途分别判断。

[Lancaster 的 DIALANG 官方说明](https://wp.lancs.ac.uk/ltrg/projects/dialang-2-0/)确认它测试法语词汇等技能，并按 CEFR 报告受测者能力；旧系统已停用。公开材料没有固定的词元／义项等级导出及目标再分发授权，因此不能从测试等级推出词表等级。

Lexique 3.83 的词频和 Wiktionary 的词形／标签不能补出 CEFR。当前 D2 的词典准入和 CEFR 来源准入分别记录；A1–B2 分级缺项保持未知，默认分级资源门槛继续保留。FLELex 官网也未提供本轮可确认的非 NC 替代授权；采用它仍需覆盖固定版本、转换与商业再分发的额外许可。[分级准入清单](development-readiness.md#词典变位与分级数据)、[现行词频标准](word-frequency-colors.md)、[FLELex 当前许可](https://cental.uclouvain.be/cefrlex/flelex/download/)。

### PolyLexFLE：部分法语表达与许可链

[StarFLE 生产者资源页](https://www.starfle.fr/en/resources/databases)及 [2024 原论文](https://research.dial.uclouvain.be/server/api/core/bitstreams/2687f4c2-bc1d-4ab9-9915-7d08b248b8ad/content)给出法语多词表达方法：4,525 项中 1,186 项有等级，人工依据 Beacco，自动方法依据 857,737 个法语教材词次。推荐 FirstOcc 即最低教材出现级，属于法语表达的接受性语料指标。61 个表达／52 名学习者的检验未独立测定学习者 CEFR；FirstOcc 与专家等级的 Spearman 相关为 0.32，未达显著。方法有有限验证，尚不能支持完整而稳定的词元／义项等级。（论文 §§5–7。）

固定 [2024-02-17 提交](https://github.com/amaliatodirascu/PolylexFLE/tree/024b1bf63ad2fff8e4499370f3398ea687944d3f)的两个文件已完整取得，分别为 **432,021／418,446 字节**，均为 4,532 行。`PolylexFLE.csv` 实为无表头九列 TSV，含 4,526 个不同表达及六个重复表达行。第五列的主等级值如下；有效值共 **817 个，涉及 816 个不同表达**，其中 A1–B2 为 800 个值。[固定 TSV](https://raw.githubusercontent.com/amaliatodirascu/PolylexFLE/024b1bf63ad2fff8e4499370f3398ea687944d3f/PolylexFLE.csv)、[固定分号文件](https://raw.githubusercontent.com/amaliatodirascu/PolylexFLE/024b1bf63ad2fff8e4499370f3398ea687944d3f/all_data_merged1.csv)。

| 主等级列内容 | A1 | A2 | B1 | B2 | C1 | N/A | 非等级／错位 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 行数 | 241 | 98 | 173 | 288 | 17 | 3,713 | 2 |

九列宽度一致不代表字段都正确：第 **4527** 行 `tu manges le pain de` 的第五列是 `expression idiomatique`，N/A 位于后列；第 **4529** 行 `râper (du fromage` 的表达继续进入下一字段，主等级列为空。另有 **七行、11 个非空附加字段**，部分包含其他构式和等级，例如第 2314 行 `faire la vidange` 的主等级为 A2，后列另有 B2 及不同构式。重复表达有同词 B2／N/A 分列的情况；原行、构式及等级上下文均保留。

分号文件的列数分别为六列 4,518 行、七列 8 行、九列 4 行、五列 1 行、八列 1 行；4,518 个位置与 TSV 的前六列完全一致。其余 **14 行与六列投影不一致**，包括附加构式、尾部空字段、拆分或错位字段，不能仅按列数将它们全部判为损坏。第 4526 行只有五列；第 4527／4529 行的语义错位也见于 TSV。逐行差异及完整字段保存在 `audit-results.json`。

按既有 **57 查询／56 个标准词头**精确匹配完整 TSV 的表达列，覆盖为 **0／57**。`avoir bon cœur` 的 B1、`avoir fourbu` 的 A2 及 `avoir bon dos` 的 N/A 都是表达行标签，不能赋给单独的 `avoir`。论文的 4,525 项／1,186 项分级统计与本固定文件的行数及主等级统计不同；文件没有逐行标明等级来自专家还是哪种自动规则。完整数据解决了文件覆盖核对，仍不提供本轮词头的可采用 CEFR 等级。

#### 数据许可与条件再分发路线

原始 Lexique-Grammaire 的[官方资源页](https://infolingu.univ-eiffel.fr/DonneesLinguistiques/Lexiques-Grammaires/Visualisation.html)明确把所链接的语言资源置于 **LGPLLR**，包含法语固定表达表第二版。这是针对数据的授权；该页未标识 PolyLexFLE 固定行对应的原始 LG 发布版本。[LGPLLR 全文](https://infolingu.univ-eiffel.fr/DonneesLinguistiques/Lexiques-Grammaires/lgpllr.html)给出以下条件：

| 条款 | 转换及随包条件 |
| --- | --- |
| §1 | 保留版权／无担保通知及既有许可通知，随资源附完整许可；允许收取副本传递费用 |
| §2(a)–(c) | 修改后仍为语言资源，文件注明修改及日期；整个衍生资源向第三方按 LGPLLR 无偿许可 |
| §3 | 不含资源衍生内容的读取程序在独立状态下不受该资源许可覆盖；包含资源部分的组合包转入 §4 |
| §4 | 包条款允许用户自行修改及为调试修改而逆向工程；显著说明资源／许可、附全文，按条款保留运行时版权通知，并满足一种源形式安排 |
| §4(a) | 可选的离线路线：随包提供含全部修改的完整、机器可读且适合修改的对应资源形式，按 §§1–2 分发 |

§4(a) 对加密资源另要求对应程序及重建材料。对应可修改形式须包含分发中实际采用的修改。此条件路线适用于得到明确授权的 LG 资源，尚未建立 PolyLexFLE 全部组合数据的授权。

PolyLexFLE 仓库 [LICENSE](https://raw.githubusercontent.com/amaliatodirascu/PolylexFLE/024b1bf63ad2fff8e4499370f3398ea687944d3f/LICENSE)为标准 **LGPL-2.1**；[README](https://raw.githubusercontent.com/amaliatodirascu/PolylexFLE/024b1bf63ad2fff8e4499370f3398ea687944d3f/README.md)说明数据库及人工／自动 CEFR 标注源自 LG 表格，但未给出针对新增数据的许可及层间适用说明。LGPL-2.1 §0 可覆盖为与程序链接而准备的函数／数据集合，其出现本身不能确定整个固定数据符合适用范围。[2024 生产者论文](https://research.dial.uclouvain.be/server/api/core/bitstreams/2687f4c2-bc1d-4ab9-9915-7d08b248b8ad/content)的 PDF 第 21 页可视原文为 “with a GNU LGPL licence Creative Common license.”；两个标签均未划除。它支持数据库 GNU LGPL 发布意图，但没有明确 Creative Common 版本、LGPLLR／LGPL-2.1 的衔接授权或新增字段许可范围。

LGPLLR §2(c) 与 LGPL-2.1 §2(c) 各自要求相应衍生整体沿用本许可；近似名称不构成相互改许可的依据。生产者明确的新增层授权及可同时满足底层许可的分发方案尚未建立，因此固定 PolyLexFLE 仍保留为部分表达候选。

[READI 2026 扩展论文](https://aclanthology.org/2026.readi-1.14.pdf)报告约 800 个动词及 1,900 个名词表达，仍按最低教材出现级标注；表达类型标注一致性不等于 CEFR 效度。论文将学习者教学验证列为后续工作，并承诺公开数据，但所查正文、旧仓库及生产者材料未给出可固定的新数据或适用资源许可。（§§4–8。）[LanGeek 的 2022-05-27 版权指引](https://help.langeek.co/legal/copyright-guidelines/)及[服务条款](https://help.langeek.co/legal/terms-of-service/)则限制未经许可的复制、修改、分发和商业使用；免费法语课程／PDF 不包含本项目目标再分发授权。本轮没有新增可准入的完整 A1–B2 来源。

### 新增法语分级来源

[Mauril 的 FEC1–FEC3 论文](https://aclanthology.org/2023.bea-1.43.pdf)（Aleksandrova／Pouliot，2023，DOI `10.18653/v1/2023.bea-1.43`）§4.2 给出法语词／表达在语境中的评估集。初始材料含 Didier《Les référentiels》的 13,016 项 A1–B2 内容，经 Québec 用法筛选及重复／多义处理；FEC1 为 914 例，FEC2 为 473 项且含十项 C 级，FEC3 为每个 A1–B2 等级 48 项。FEC2 有受训母语语言学家的新增例文，FEC3 采用 Québec《Agir pour interagir》教学材料，作者区分理解能力与产出能力。它提供相关的法语语境设计依据；已查论文／发布记录未给出固定 FEC 语料下载或数据许可，论文 CC BY 4.0 不代替外部教材及新增语料的目标再分发权利。

[Eaquals/CIEP 2015 清单](https://www.eaquals.org/wp-content/uploads/Inventaire_ONLINE_full.pdf) §§3、5–6 基于专家对 CEFR、课程及参考书的共识，明确示例为非穷尽、集体直觉所得，不能按示例所在章节给孤立单词赋级。[官方条款](https://www.eaquals.org/copyright-disclaimer/)要求复制、分发及修改前取得明确书面许可。[LoadWords A1–B2](https://loadwords.com/en/vocabulary/fr/a1/1) 的四个等级首页可读，但[FAQ](https://loadwords.com/en/faq)未提供可核查的分级法语验证，[使用条款](https://loadwords.com/en/terms-of-use)限制出版、出售、商业化及数据提取。网页界面版本 `v8.0.0` 不构成词汇数据固定发布。

本轮新增来源没有形成可转换并商业随包的完整法语词元／义项数据；57 查询的新增 CEFR 精确覆盖未评估，缺项仍未知。

## 自定义 MDX/MDD 词典与相关资源

核实日期：2026-11-10（America/Vancouver）。范围为法英、法法、法汉、英汉、英英及低优先级的法语例句、法语同近反义、英语例句。内容版次、制包正文更新、样式更新和镜像文件时间分别记录；出版社的新书版次不自动证明存在对应 MDX。推荐依据为编纂机构、可核对的制作者发布及公开质量反馈；未取得各制包可比较的独立评分。

本报告基线中的 [MDX reader](../crates/dictionary/src/mdx.rs)限制单文件 **128 MiB、100 万条记录**，接受 UTF-8、UTF-16／UTF-16LE；[正文渲染](../apps/desktop/src/dictionary-html.tsx)保留有限 HTML 和内联样式，移除脚本、外部 CSS、图片及音视频。MDX 正文与 MDD 资源分别判断：**MDD 媒体、样式资源和词条跳转尚未接入**，参见 [D1／D3 验收](evidence/issue-9/README.md)及[独立格式矩阵](evidence/issue-3/README.md)。压缩合集的总大小不能代替解压后的单个 MDX 大小。下列三个公开来源的试制 MDX 已完整执行 CLI 读取；商业社区包仍未下载／解包，包头、压缩／加密方式和应用导入未确认。

下列商业词典及社区整合包作为合法获取并符合授权条件后的自定义词典候选；本轮没有确认其正文、图片和声音的目标转换／随包再分发授权。默认内置仍遵循 [ADR 0002](adr/0002-source-and-resource-licensing.md)，不因文件可下载或解析器可读取而获得内容授权。

### 法语随包范围与用户自选链接

完整版和 Lite 仅选择 **1–2 份法语相关 MDX** 随包，必要 MDD 配套随同一词典分别准入。当前 Windows x64 开发包提供 **1 份 FreeDict 法英 0.4.1 补充 MDX**，保留全部原始记录；新候选的权利和质量条件满足后再决定第二份。其他词典只提供可核对的制作方／下载页链接，用户自行获取后通过既有导入入口使用。设置已提供八类、12 项链接，标明格式与兼容限制；[接入验证](evidence/d2-mdx/README.md)不替代完整版／Lite 的 M5 正式发行验收。

FreeDict 原始 TEI 的 publication／revision 均为 **2018-08-11**；2019 年目录时间和 2026 年转换时间不改变内容版次。[原始源包](https://download.freedict.org/dictionaries/fra-eng/0.4.1/freedict-fra-eng-0.4.1.src.tar.xz)、[完整许可](../resources/freedict-fra-eng-0.4.1/COPYING)、[作者](../resources/freedict-fra-eng-0.4.1/AUTHORS)。该 MDX 保留同形记录，以 HTML 显示词头、原始发音、语法和完整义项文字，未改写源译词；完整源包、AUTHORS／COPYING 与修改通知一并保留。随包复用 #42 已有同源材料，减少重复源档案；[资源说明](../resources/freedict-fra-eng-mdx-0.4.1/README.md)提供具体重建路径。

| 试制资源／分发角色 | 原始发布与许可 | 实际 MDX 格式、大小及条数 | 验证与决定 |
| --- | --- | --- | --- |
| **FreeDict 法英 0.4.1；首份随包补充** | 内容 2018-08-11；GPL-2.0-or-later | `freedict-fra-eng-0.4.1.mdx` **273,758 B**；8,505 条、8,417 个键；2.0／UTF-8／zlib／未加密；无 MDD | 全部记录读取，88 条同形重复及 1,514,479 B 解码正文通过。逐字节重建、Windows x64 随包查询／来源保存、通知和重启保持通过；正文较旧，保留缺词和源译义问题。整机断网首装与 M5 另验。 |
| WikDict 法英；较新技术候选 | 官方 StarDict 构建 2026-06-23；IFO 声明 CC BY-SA 4.0；底层 dump 日期未知 | `wikdict-fr-en-2026.06.23.mdx` **5,137,488 B**；217,525 条，含 92,896 个展开别名；2.0／UTF-8／zlib／未加密；无 MDD | 全部记录、5,909 条重复及 38,556,468 B 解码正文通过；原作表达及逐词来源未闭合，保留未准入状态。 |
| WikDict 法汉；第二份优先考察候选 | 官方 StarDict 构建 2026-06-23；IFO 声明 CC BY-SA 4.0；底层 dump 日期未知 | `wikdict-fr-zh-2026.06.23.mdx` **558,264 B**；19,018 条，含 7,919 个展开别名；2.0／UTF-8／zlib／未加密；无 MDD | 全部记录、344 条重复及 3,484,013 B 解码正文通过；保留未准入状态，当前仅提供原格式来源链接。 |

WikDict [官方法英 ZIP](https://download.wikdict.com/dictionaries/stardict/wikdict-fr-en.zip)为 **4,839,120 B**，含 124,629 条正文；[法汉 ZIP](https://download.wikdict.com/dictionaries/stardict/wikdict-fr-zh.zip)为 **533,277 B**，含 11,099 条正文。日期来自实际 IFO，两个 ZIP 的许可链接均为 4.0；[官方下载页](https://www.wikdict.com/page/download)提供 StarDict／Kobo／SQLite／TEI，**没有官方 MDX**。URL 是可变下载入口，本轮固定保存下来的 ZIP 和响应元数据，未确认完整输入清单。

MDX 使用仓库已有夹具验证过的 [mdict-utils MIT 写入器](https://github.com/liuyug/mdict-utils/tree/64e15b99aca786dbf65e5a2274f85547f8029f2e)。StarDict 索引的字节布局参考固定 [PyGlossary reader](https://github.com/ilius/pyglossary/blob/19ed3a63df91b873af8226affe0d3ef96ab7e5e4/pyglossary/plugins/stardict/reader.py#L189)及[HTML 格式说明](https://github.com/ilius/pyglossary/blob/19ed3a63df91b873af8226affe0d3ef96ab7e5e4/doc/p/stardict.md#L73)；本轮只用标准库读取原始四成员，不引入新依赖。`.syn` 是备用键索引，其数量不等于语义同义词数；转换把别名展开为相同正文，不要求尚未接入的词条跳转。试制日期 **2026-11-10** 是格式转换日期。

较新的 4.0 包头没有闭合全部原作表达：法英包的 `apprendre`、`cote` 分类义、`pouvoir`、`taire`，以及法汉包的 `apprendre` 保留已核对的 DAF8 表达，沿用[既有文字许可](#文字提取器和媒体许可)的未闭合状态。[DAF8 apprendre](https://www.dictionnaire-academie.fr/article/A8A1665)、[cote](https://www.dictionnaire-academie.fr/article/A8C3092)、[pouvoir](https://www.dictionnaire-academie.fr/article/A8P2383)、[taire](https://www.dictionnaire-academie.fr/article/A8T0068)。正文未带逐词源页链接或导入通知；整体通知只列 Karl Bartel／WikDict、Wiktionary via DBnary、FreeDict。已知 CFC 制图义和 TLFi `avoir` 原句未在这些小样中确认，不能据此判定全包无其他导入文本。两份 WikDict 试制文件保留 `resource_admitted=false`。

以下为可内置的用户自选来源／下载页链接；每项版次、原格式与兼容性见后续类别表。StarDict／TSV 与仅有网页的资源须标明格式，不能作为可直接导入的 MDX。商业制包的随包授权未确认，页面可获取性也不等于已实测完整导入。

| 类别 | 用户自选资源 | 制作者／下载入口 | 当前获取格式 |
| --- | --- | --- | --- |
| 法英 | Collins 2024 网页包 | [Akira 发布与补链](https://forum.freemdict.com/t/topic/31704) | MDX 合集；单文件详情待解包 |
| 法法 | Larousse 2024 网页包 | [Akira 发布与补链](https://forum.freemdict.com/t/topic/31704) | MDX 合集；单文件详情待解包 |
| 法汉 | LarousseFC 重制包 | [wynick27 制作发布](https://forum.freemdict.com/t/topic/45012) | MDX／MDD／独立 CSS |
| 法汉备用 | WikDict 2026-06-23 | [官方法汉下载](https://download.wikdict.com/dictionaries/stardict/wikdict-fr-zh.zip) | StarDict，当前应用需转换后导入 |
| 英汉 | 朗文 6C；OALDPE | [6C 正文](https://forum.freemdict.com/t/topic/27460)、[夏日配套](https://forum.freemdict.com/t/topic/40024)；[OALDPE](https://forum.freemdict.com/t/topic/30466) | MDX／MDD；混合译源与样式依赖见下表 |
| 英英 | OALD10；LDOCE6 Dean 路线 | [OALD10 文件目录](https://downloads.freemdict.com/%E5%B0%9A%E6%9C%AA%E6%95%B4%E7%90%86/%E5%85%B1%E4%BA%AB2020.5.11/content/1_english/3_Oxford%20series/OALD10/)；[Dean 制作发布](https://forum.freemdict.com/t/topic/24705) | MDX／MDD；朗文原包超限，修订包大小未知 |
| 法语／英语例句 | Tatoeba 相应语言子集 | [官方导出](https://tatoeba.org/en/downloads) | TSV；可靠现成 MDX 未确认 |
| 法语同近反义 | Larousse 现行资源；2004 旧版备用 | [官方现行网页](https://www.larousse.fr/dictionnaires/synonymes)；[旧版 MDX 目录](https://downloads.freemdict.com/%E5%B0%9A%E6%9C%AA%E6%95%B4%E7%90%86/%E9%9B%86%E5%90%88/Other%20language%20Dictionaries/French/%E6%B3%95%E8%AF%AD/Le%20Grand%20Dictionnaire%20des%20Synonymes%20et%20Contraires%2C%202004/) | 现行网页没有已确认 MDX；旧版 MDX 见下表 |

### 法语—英语词典

| 资源 | 原内容与制包时间 | MDX/MDD 核实结果 | 适用建议 |
| --- | --- | --- | --- |
| [FreeDict fra-eng 0.4.1](https://download.freedict.org/dictionaries/fra-eng/0.4.1/)，随包补充 | 原内容 2018-08-11；本轮格式转换 2026-11-10。GPL-2.0-or-later，完整源数据／作者／修改通知已保留。 | 官方提供 TEI／StarDict 等，未提供官方 MDX；应用随包的 `freedict-fra-eng-0.4.1.mdx` 为 273,758 B、8,505 条，MDX 2.0／UTF-8／zlib／未加密、无 MDD，CLI 全部读取与原生查询通过。 | **首份随包补充**；内容较旧、有常用缺词及源译义问题。Windows x64 开发包已接入，来源保存／通知／重启保持通过；整机断网首装和正式发行另验。 |
| [Collins French-English / English-French online](https://www.collinsdictionary.com/dictionary/french-english) | 出版社现行网页词典；社区截取日期、固定书版未知。Akira 于 2024-09-10 发布。 | [实际镜像](https://cloud.freemdict.com/index.php/s/GZ2RKxWKRprG4dn)列 `Collins French-English (MDX).7z`，780.4 MB、September 2024。MDX 发布声明／目录标识已核；档内单 MDX 及 MDD 文件名、大小未解包核实。 | 法英主候选。需确认单 MDX 大小及正文依赖；780.4 MB 是压缩合集大小。 |

Collins 作为用户自选的网页词典制包候选，官方称提供超过 230,000 个现行法英／英法译项。[Akira 原发布帖](https://forum.freemdict.com/t/topic/31704)声明含原网站声音和图片，也明确无暇继续更新；[后续回复](https://forum.freemdict.com/t/topic/31704?page=2)提供镜像补链。补链或镜像 README 的 2025 年时间不构成新内容版本。FreeDict 的公开许可及读取结果不替代其内容质量和产品准入。

### 法语单语词典

| 资源 | 原内容与制包时间 | MDX/MDD 核实结果 | 适用建议 |
| --- | --- | --- | --- |
| [Larousse French-French online](https://www.larousse.fr/dictionnaires/francais) | 出版社现行网页词典，135,000 条定义、90,000 篇条目；社区截取日期未知。[原发布](https://forum.freemdict.com/t/topic/31704)为 2024-09-10。 | [实际镜像](https://cloud.freemdict.com/index.php/s/saWooeAkGDDb44M?dir=/Larousse)列 `Larousse French-French (MDX).7z`，1.4 GB、September 2024。档内单 MDX/MDD 名称、大小未知。 | 法语定义正文主候选；不是已证明对应 Petit Larousse 2024／2026 的印刷新版。需确认正文尺寸和实际阅读。 |

既有 Larousse 同名历史样本的解析结果可参考[格式矩阵](evidence/issue-3/README.md)，不替代这个 2024 包的检查。同批 Le Robert 包在原帖有缺词严重的用户反馈，未取得完整性证据。

### 法语—中文词典

| 资源 | 原内容与制包时间 | MDX/MDD 核实结果 | 适用建议 |
| --- | --- | --- | --- |
| [外研社《拉鲁斯法汉双解词典》](https://mlp.fltrp.com/edu/wys/bookstore/detail?id=976&mid=)重制包 | 官方目录出版日期 2012-11，法文蓝本为 1995 年词典，ISBN 978-7-5600-1580-4(02)；名称“24新”未证明 2024 内容修订。[制作者 wynick27](https://forum.freemdict.com/t/topic/45012)于 2026-09-09 发布，[9 月 11 日回复](https://forum.freemdict.com/t/topic/45012?page=2)称修复索引问题。 | [作者所链实际目录](https://cloud.freemdict.com/index.php/s/ey7mb8t3PnrsHXc)列 `LarousseFC.mdx` 7.9 MB、`LarousseFC.mdd` 9 KB、CSS 16 KB、PNG 1.5 MB。目录显示 September 11，原 title 为 `9/11/26, 4:23:52 AM`，时区未知。 | **近期法汉正文试读首选**；尺寸符合基线。底本较老，重音、同形条及附录需实际检查；MDD／独立 CSS 尚不能使用。 |
| [WikDict français–中文](https://download.wikdict.com/dictionaries/stardict/wikdict-fr-zh.zip)，2026-06-23 | 实际 IFO 构建日期 2026-06-23，声明 CC BY-SA 4.0；底层 dump 日期及完整输入链未确认。 | 官方 ZIP 为 StarDict 3.0.0、11,099 条正文及 7,919 个备用键；**没有官方 MDX**。本轮转换为 558,264 B、19,018 条的 MDX 2.0／UTF-8／zlib／未加密，无 MDD，独立 CLI 全部读取通过。 | **第二份法语随包考察候选**；格式转换可行，但已确认 `apprendre` 保留 DAF8 表达，原作再分发依据及来源关联未闭合，未准入。当前仅提供原格式来源链接。 |

出版社说明中文由十位法语教学专家、学者历经四年翻译。重制包采用 Casio 数据，补入合作 OCR 的四类附录；Casio 数据的具体版次未确认。原帖可追踪排序重打包及 `prononcé`、`feu` 的索引反馈／修复声明；9 月 14 日另发的 Yomitan 文件不作为 MDX 更新日。

### 英语—中文词典

| 资源 | 正式书版与社区正文 | MDX/MDD、维护时间与来源 | 适用建议 |
| --- | --- | --- | --- |
| [朗文当代高级英语辞典第 6 版英英·英汉双解](https://www.fltrp.com/c/2020-05-11/496989.shtml)的社区 6C 双解 4.0＋夏日清凉 V3.3 | 正式双解第 6 版在 [2019 年已上市](https://epaper.gmw.cn/zhdsb/html/2019-05/29/nw.D110000zhdsb_20190529_11-02.htm)，ISBN 9787521309096；官方商品页日期 2020-05-11 不是首版出版日期。[社区正文](https://forum.freemdict.com/t/topic/27460)混合朗文 6 英文、旧双解、朗文 5 译文及部分手工录入的第 6 版译文，首帖修订至 2024-03-31。 | 作者明确提供 MDX、沿用 Amazon MDD。[夏日版](https://forum.freemdict.com/t/topic/40024)更新到 2026-07-18 V3.3，提供 CSS／JS／MDD 并要求另配原 MDX 或 Karx 重定向版；这是样式／交互更新。实际 MDX/MDD 字节数未知。 | 英汉优先核验候选；**未核实完整官方第 6 版中文正文**。脚本折叠、entry 跳转和外部样式在基线中不可用，单 MDX 大小也待确认。 |
| [牛津高阶英汉双解词典第 10 版](https://www.cp.com.cn/book/ae049cb5-d.html)的 OALDPE 社区包 | 正式书版为 2023-04，ISBN 9787100220026；[OALDPE](https://forum.freemdict.com/t/topic/30466)日志至 2025-01-10，记录改用 OALD 2024.09 底层数据，所标数据更新时间为 2024-11-07。[底层作者](https://forum.freemdict.com/t/topic/22813)列出的中文来自官方 APP、个人审校／个人译、其他社区包及剩余 Google 补译。 | 作者要求 `oaldpe.mdx`＋`oaldpe.mdd`；另有 `.1.mdd` 单词音、`.2.mdd` 图片、`.3.mdd` 例句音。单文件字节数未知。2025-01 增加中文译源的样式区分。 | **有条件辅助候选**；中文含机器补译，未证明逐条对应官方 2023 纸本。译源标识依赖样式，当前完整阅读及来源区分未确认。 |

这两项的正式词典由成熟出版社编纂，但社区正文均有合并／补译；品牌和“完美版”标题不能证明整包中文质量。V3.3 的 2026 年日期也不能写成朗文正文更新到 2026 年。

### 英语单语词典

| 资源 | 内容版次与包版本 | MDX/MDD 核实结果 | 适用建议 |
| --- | --- | --- | --- |
| [Oxford Advanced Learner's Dictionary 10th](https://elt.oup.com/catalogue/items/global/dictionaries/oxford-advanced-learners-dictionary/9780194798488)，英英 | OUP 确认第 10 版；[OUP 2020-01-20 发行介绍](https://learningenglishwithoxford.oup.com/2020/01/20/language-change-eco-anxiety/)明确当月推出，纸本具体发行日未核实。索引 MDX 时间为 2020-11-02 16:18；制作者及后续修订未知。 | [实际文件索引](https://downloads.freemdict.com/%E5%B0%9A%E6%9C%AA%E6%95%B4%E7%90%86/%E5%85%B1%E4%BA%AB2020.5.11/content/1_english/3_Oxford%20series/OALD10/)列同名 MDX／MDD：MDX **40,975,963 B（39.08 MiB）**，MDD **1,845,936,118 B（1.72 GiB）**。 | **英英正文试读首选**。MDX 尺寸符合基线；包头、正文及原生导入未验。MDD 大小与 MDX 上限分开判断，媒体当前未接入。 |
| [Longman Dictionary of Contemporary English 6th](https://www.pearson.com/languages/en-us/educators/connected-english-learning-program/longman-english-dictionaries.html)，Dean 修订路线 | 英文第 6 版由 Pearson 于 2014 年出版，见 [2016 年专业书评及其出版方记录](https://api.crossref.org/works/10.1007/s40607-015-0023-6)。[Dean 帖](https://forum.freemdict.com/t/topic/24705)标题为 v2024.02.02，正文维护标记为 2025.12；修订包确切版本及文件大小未知。 | 作者所链 [Amazon 原包索引](https://downloads.freemdict.com/100G_Super_Big_Collection/%E8%8B%B1%E8%AF%AD/%E7%B3%BB%E5%88%97%E8%AF%8D%E5%85%B8/AMAZON%20PDAWIKI/STFU%E6%9C%97%E6%96%87%E5%85%A8%E7%B3%BB%E5%88%97%EF%BC%88%E5%85%B15%E9%83%A8%EF%BC%89-%E7%B2%BE%E4%BF%AE%E7%89%88%20By%20Amazon%2020160928/LongmanDictionaryOfContemporaryEnglish6thEnEn/)列 MDX **162,322,205 B（154.8 MiB）**、MDD **1,298,038,335 B（1.21 GiB）**，时间 2018-10-23。修订包大小不能沿用原包值。 | 第二候选；**已确认原包 MDX 超出 128 MiB，基线会拒绝**。需取得修订包的真实尺寸；维护日期不改变 2014 内容版次。 |

OUP 公开产品说明学习者级别 B2–C2、Oxford 3000／5000 和例证；Pearson 说明 LDOCE 面向高级学习者，专业书评称其为可靠的外语学习词典。产品中的英语 CEFR 信息不用于给法语词头赋级。

### 法语例句库（低优先级）

| 资源 | 更新与来源 | 格式与许可 | 适用建议 |
| --- | --- | --- | --- |
| [Tatoeba 法语子集／英法句对](https://tatoeba.org/en/downloads) | 官方每周六 06:30 UTC 更新导出，可按法语 `fra`、英语 `eng` 筛选并保留双向句子链接；本轮没有固定最新导出日期或句对数量。[OPUS 同源研究快照](https://opus.nlpl.eu/datasets/Tatoeba)现标 v2026-07-08。 | 官方为 tab 分隔文本及 `.tar.bz2` 档案，含句子、作者、修改时间及句子 ID 关系。**未确认可靠的现成 MDX/MDD 版**。文本默认 CC BY 2.0 FR，另有 CC0 子集；录音许可另核。 | 优先作为筛选后的结构化例句来源；当前不能直接以 MDX 导入。 |

[官方使用条款](https://tatoeba.org/en/terms_of_use)明确社区运作且不保证翻译经过专业校验；研究使用不证明逐句质量。复用须保留作者、来源及适用许可，详细导出中的 Username 字段可承担作者关联。

### 法语同义词、近义词及反义词（低优先级）

| 资源 | 版本与来源 | MDX/MDD 核实结果 | 适用建议 |
| --- | --- | --- | --- |
| [Larousse Dictionnaire des synonymes et contraires](https://www.larousse.fr/dictionnaires/synonymes)，现行官方资源 | 出版社专门栏目，称覆盖 160,000 个同义词、90,000 个反义词；固定版次／更新时间未公布。 | **未确认对应现行内容的现代 MDX/MDD**；2024 法单语包是否完整包含本栏目也未证实。 | 此类权威主资源；当前离线包有证据缺口。 |
| Le Grand Dictionnaire des Synonymes et Contraires，2004，旧版备用 | 目录标为 2004；原书具体版次、制作者和制包日未知。镜像文件时间 2020-11-02。[2024 社区讨论](https://forum.freemdict.com/t/topic/31053)提供按义项比较的有限实用反馈。 | [实际目录](https://downloads.freemdict.com/%E5%B0%9A%E6%9C%AA%E6%95%B4%E7%90%86/%E9%9B%86%E5%90%88/Other%20language%20Dictionaries/French/%E6%B3%95%E8%AF%AD/Le%20Grand%20Dictionnaire%20des%20Synonymes%20et%20Contraires%2C%202004/)列同名 MDX **1,934,040 B**，未列 MDD；不能据此断定 MDD 从未存在。 | 仅适合愿意接受较旧内容的备用选择；不满足较新原内容偏好。既有同名样本参考[格式矩阵](evidence/issue-3/README.md)，本轮未试读此目录文件。 |

### 英语例句库（低优先级）

| 资源 | 更新与来源 | MDX/MDD 核实结果 | 适用建议 |
| --- | --- | --- | --- |
| [Tatoeba 英语子集／英法句对](https://tatoeba.org/en/downloads) | 同一滚动语料的英语方向，官方周更；OPUS 快照 v2026-07-08 不作为 Tatoeba 官方导出版本。 | 官方 tab 分隔句子／详细作者信息／链接；可靠 MDX/MDD 版未确认。文本 CC BY 2.0 FR／CC0 子集。 | 低优先例句来源；需筛选及转换，保留原句 ID、作者和译文关系。 |

### 可获取性及兼容性缺口

[Larousse French-English / English-French online](https://www.larousse.fr/dictionnaires/francais-anglais)保留为法英扩展线索：官方称覆盖 250,000 个词与表达；同批社区包于 2024-09-10 发布，[实际分卷目录](https://cloud.freemdict.com/index.php/s/GZ2RKxWKRprG4dn?dir=/Larousse/Larousse%20French-English%20%28MDX%29)有 `.7z.001–.007`，总计 5.9 GB、September 2024。MDX 标识已核，截取日期、固定书版、单 MDX/MDD 名称与大小未解包核实，暂不列入两部法英优先项；合集体积不能用来判断单 MDX 是否超限。

[French Wiktionary – February 2026 发布页](https://forum.freemdict.com/t/topic/42269)标 MDX，并声称 1,988,298 条、无声图、仅测试 GoldenDict-ng；首帖时间为 2025-12-02，标题不能独立证明底层 dump／制包版本。[作者所链 pCloud](https://e.pcloud.link/publink/show?code=kZWooaZJ1Yg1CDSAyFSdCw6rKF0C71jXKvX)在本轮显示链接失效，当前文件名与大小未确认。声称条数超过研究基线的一百万上限；它不具备本轮可优先导入的证据。

“相对较新”在现有 MDX 生态有明确取舍：法汉包虽于 2026 年重制，底本较旧；2024 网页词典包的截取日未知；英汉较新维护包混合不同来源译文，样式修订不等于内容更新。上述推荐保留这些具体事实，不把所有类别标记为同时满足新内容、完整原版及当前应用完整可用。

## 下一步准入决定

同一固定 dump 的 **57 查询、56 个词头／143 个记录／744 个义项的原始对照、官方 60 页恢复、全部摘要及记录级词头 IPA 审核已完成**。原 27 查询的小样保持冻结，扩展 30 查询完成全部所选定义、译词、IPA、八项语法关系和通知范围审核；合并资源、57 查询及四文件压缩包均已回读。12 个 DAF8 标记页的所选定义现已逐一核对：六项贡献者定义支持 CC BY-SA 4.0，五项 DAF8 表达的目标再分发依据未闭合；`avoir` 的 TLFi 公开条款明确排除商业利用。

1. 按已核对的七项原文跨度，确定是否接受 XMLittré 的历史表达、`cote` 较窄范围及 `avoir` 的义项拆分；采用前落实学习者补语语境、印刷对照、逐字段译词关联和 CC BY-SA 3.0／4.0 通知范围，并检查证据载荷中旧表达的权利。若保留现有五项 DAF8 表达，仍需作品及目标辖区的再分发依据；保留 `avoir` 的 TLFi 表达则需独立的商业随包授权。引用例句及媒体按各自原作／文件准入，CFC 明确导入义项继续在所选摘要之外。
2. 在现有词典模块接入已准入的结构化资源与相应 IPC／摘要字段，保留完整定义、限定、来源引用和未知值；用实际 UI 验收超长定义、原生读取、缺资源／损坏、断网首启及通知可见性。固定发布版本及保存／导出引用，按 [ADR 0004](adr/0004-one-vocabulary-record-with-reference-digest-and-notes.md)保留旧摘要、独立笔记和生词身份；保存独立推导的官方输入引用，提取输出另做有界重现验证。
3. 再测量所选默认结构化资源的全量转换、来源归档及完整随包体积，按实际资源边界验收。MDX 先完成 FreeDict 法英补充候选的产品导入、通知和首启断网，再决定第二份法语资源；其他候选只接入用户自选来源／下载页链接。WikDict 2026-06-23 法英／法汉的实际 IFO 和试制 MDX 已核实，但原作表达权利与完整输入链仍未闭合，不能仅凭 4.0 头部准入。

CEFR 另需有法语词元／词性或义项依据、明确 A1–B2 方法且允许目标再分发的来源，或 FLELex 的额外授权。PolyLexFLE 的表达等级不赋给组成词，其 LGPLLR 条件路线也不代替新增层及组合授权；Mauril 的语境方法仍需固定语料及数据授权。词频启发式与翻译后的英语等级均保留其实际语义；缺项继续未知。

## 验证与限制

- 已完成：官方发布／许可页及固定生产者代码核对；完整 FreeDict TEI 字段统计及 27 项精确查询；专用法语版 8 MiB 原始前缀及固定 27 查询的完整原始页区块；原始／单词 JSON 对象逐项对照、dump 日期前源页及翻译／发音提取机制；26 个所选义项／译词及 44 个记录的词头 IPA 来源核对；两个独立 44 行 SQLite 投影与失败路径；法中公共 TEI／SQLite 元数据及日期命名输入；CEFR 数据卡、TUFS 法语 PDF／英文授权、Vocabcraft 与 DIALANG 原作者材料。
- 探查命令均通过：`uv run --no-project python -X utf8 target/d2-research/probe_freedict.py`；`uv run --no-project python -X utf8 target/d2-research/continuation/parent/probe_current_raw_prefix.py`；同目录的 `probe_queries_kaikki.py` 与 `audit_query_fields.py`。使用 Python 3.14.10 标准库，没有执行上游生成器或添加依赖。
- 原始查询及投影检查通过：`uv run --no-project --offline --python 3.14.10 python -X utf8 target/d2-research/continuation/parent/collect_raw_queries.py`；同目录的 `audit_raw_queries.py`。两者使用标准库断言检查页区块／重定向、字段对照、回读及失败路径。
- 摘要检查通过：`uv run --no-project --offline --python 3.14.10 python -X utf8 target/d2-research/summary-selection/build_reviewed_summaries.py`；断言核对 26 条定义源行、所选翻译框、记录级 IPA、27 个明确查询映射、重音区别、未知译词、语法／领域限定、繁体写法和 JSON 回读。IPA 独立 `inspect_ipa.py`／`select_ipa.py` 已核对全部 44 条记录、51 个值及确认排除节点；七项重点选择另有原页定义／译词／语法限定核对。
- 来源引用检查通过：`uv run --no-project --offline --python 3.14.10 python -X utf8 target/d2-research/resource-contract/source-reference-probe/recover_dump_references.py`；读取官方已完成任务及索引，恢复 28 个目标页，核对 26 个审核页的修订与完整 wikitext、两项重定向、每个范围及目标页完整性。另保留三个 HTTP 200 元数据页的 `probe_dump_sources.py`／完整日志。
- 扩展检查通过：`target/d2-research/expanded-quality/` 下的 `select_frequency_sample.py`、`audit_raw_cached_sample.py`、`compare_freedict_sample.py`、`recover_dump_sample.py`，使用 uv／Python 3.14.10 标准库；30 项独立选择、99 行／385 义项原始对象对照、两份完整 FreeDict 查询、32 个官方修订／wikitext 相等断言均通过，完整命令日志保留。
- 扩展语义检查通过：`source-agent/` 的 `fetch_source_pages.py`、`inspect_primary.py`、`inspect_kaikki.py`、`build_focus_evidence.py` 及 `verify_findings.py`；32 个截止日前源页、13 个精确源行／记录选择及译词节点、34 个本地引用和 32 个引用修订 ID 检查通过。生产者链的 `check_retained_title_linkage.py`／`check_expanded_title_linkage.py` 核对两组共享输入 URL／ETag、143 条记录／56 个不同词头及独立 dump 引用；23 项 HTTP 200 固定代码／元数据来源及代码行引用已核对。
- 资源契约检查通过：`uv run --no-project --offline --python 3.14.10 python -X utf8 target/d2-research/resource-contract/probe_reviewed_resource.py`；26 条摘要／两项重定向及完整来源清单回读、27 查询、限定／繁体／未知值、只读拒写、缺失／截断错误和四文件压缩包逐项断言通过。
- 完整扩展审核通过：`uv run --python 3.14.10 --no-project --offline target/d2-research/expanded-summary/source-agent/verify_expanded_reviews.py`；八组断言核对 99 记录／385 义项、32 官方引用、原 13 摘要保持、44 个所选译词原始节点及 `il`／`on` 跨记录来源、2,059 个 sound 节点、100 IPA 值／四个未知／52 排除、八项语法关系、四项通知及 69 本地证据链接。结果及完整命令日志已保留。
- 合并资源契约通过：`uv run --no-project --offline --python 3.14.10 python -X utf8 target/d2-research/expanded-summary/probe_combined_resource.py`；56 摘要／四重定向、57 查询、原始与审核关系、译词原位置、IPA 完整用途证据、14 页／15 通知、只读及缺失／截断错误、许可／清单／四文件 tar.gz 均逐项回读通过。实际字节数保存于 `combined-reviewed-core-probe/probe-results.json`。
- 导入文字检查通过：原 26 页的 `check_original_source_notices.py` 核对全部所选源行及修订，10 页／11 通知；`imported-text-rights/verification-results.json` 确认 `ne` 修订前后及原作对照、两组 56 页／15 通知、49 份完整 HTTP 响应及证据链接。追加 11 项定义的原作、贡献者编辑／父修订和通知范围已核对；六项贡献者文字、五项 DAF8 保留表达与一项 TLFi 表达分别记录。
- 所选定义权利检查通过：`uv run --no-project --offline --python 3.14.10 python -X utf8 target/d2-research/daf8-span-review/verify_selected_span_reviews.py`；11 项新审核的原句、固定修订、记录／义项位置、既有 `ne` 证明、56 页既有通知审核的 15 项清单、11 个报告修订链接和未变的准入标志均核对通过。主流程 51 份完整 HTTP 响应、1,014,716 字节及回读断言通过；三个独立审核分别核对 33／42／48 份完整响应。文字来源结果为六项支持、五项 DAF8 保留表达、一项 TLFi 保留表达；当前公开权利范围见下项。
- 公开条款与法律原文检查通过：`uv run --no-project --offline --python 3.14.10 python -X utf8 target/d2-research/rights-basis-continuation/parent/verify_rights_basis.py`；三个法律原文、七项准确引文、1995-07-01 条件及 12 项所选定义的当前权利分类核对通过。`tlfi/check_evidence.py` 独立核对八项来源、18 项引文、34 个证据链接及 `avoir` 固定修订／原句，通过。
- 成员国路线检查通过：`pwsh -NoProfile -File target/d2-research/rights-basis-continuation/daf8-territory/check-evidence.ps1`；22 项断言核对来源正文、版本、响应类型及未建立 DAF8 具体保护状态的结论，结果保存在 `check-evidence-result.json`。
- 固定页替代定义检查通过：`target/d2-research/definition-alternatives/daf8-five/verify_findings.py` 核对五个修订、49 个定义位置、四项实际候选及 `retentir` 的 DAF8 原句；`avoir/check.py` 核对七项动词定义、固定源行、完整响应相等及文字许可。两组均使用 uv／Python 3.14.10 标准库，未准入替代定义。
- XMLittré 检查通过：`pwsh -NoProfile -File target/d2-research/definition-alternatives/early-dictionary/self-check.ps1`；六词、七项原文起句、五份完整固定 XML 的字节数／词性／义项及许可头部均核对，结果与完整验证响应保存在同目录。
- 完整 XMLittré 字段检查通过：`uv run --no-project --no-python-downloads --python 3.14.10 python -X utf8 target/d2-research/xmlittre-field-quality/verify_quality.py`；57 查询／56 标准词头未变、19 份固定文件、三种对应方式及并集、69 条目、字段／结构／引用数量、七项原文起句及本地证据链接均核对，完整结果见 `verification.log`。
- XMLittré 定义边界检查通过：`uv run --no-project --offline --python 3.14.10 python -X utf8 target/d2-research/xmlittre-gloss-boundaries/verify_boundaries.py`；六个固定条目身份、七项原文／字节跨度、历史语法及 `Fig.`、三个未选入的补语语境位置、四份冻结输入的前后字节及原 `not_admitted`／质量标志均核对通过。
- Gallica 访问证据检查通过：`pwsh -NoProfile -File target/d2-research/xmlittre-print-fidelity/check-evidence.ps1`；六个视图、七项未变起句及九份完整 HTTP 403 HTML 响应核对，证据结果为 PASS；该访问检查本身不证明印刷一致性。
- 追加 CEFR 检查通过：`additional-cefr-sources/inspect_sources.py`／`validate_evidence.py` 使用 uv／Python 3.14.10 标准库，核对两份 65,536 字节前缀、586 个共同完整行、等级／字段及 17 份保留响应／证据链接；三项候选结论均保留固定来源及具体权利判断。
- 完整 PolyLexFLE 检查通过：`uv run --no-project --offline --python 3.14.10 python -X utf8 target/d2-research/polylexfle-complete-audit/audit_complete.py`；两份完整正文／固定元数据、既有前缀相等、4,532 行、817 个主等级值／816 个不同表达、两项错位、七行／11 个附加字段、重复表达上下文、4,518 个相同六列位置及 0／57 词头覆盖均核对。断言结果为 PASS，数据质量结果仍为 `issues_found`；原文件未修补或推断等级。
- PolyLexFLE 许可证据检查通过：`uv run --offline --no-project --python 3.14.10 python target/d2-research/polylexfle-license-chain/check.py` 核对完整官方数据通知、两套许可准确条款、固定提交、26／12 页生产者 PDF、2024 许可段落的渲染图、工具响应及本地链接，完整结果见 `check.log`；PDF 提取警告保留于 `pdf-read.log`。原始数据有条件路线，组合授权仍未闭合。
- 许可通知检查通过：`uv run --no-project --offline --python 3.14.10 target/d2-research/resource-contract/attribution-agent/fetch-primary.py`；八次 HTTP 200 下载保留完整 CC 原文、固定／现行 Wikimedia 554852 及法语版权约定 38903168，修订身份断言通过，17 个证据文件／本地引用解析通过。
- 代用释义多源检查通过：`uv run --no-project --offline --python 3.14.10 python -X utf8 target/d2-research/mixed-source-contract/inspect_fields_and_notices.py`；六个词头 IPA、十项原译词／框、两个义项对应边界、五项 CC 准确原文、固定引用／JSON 回读及未变的冻结输入均核对通过。
- 新增 CEFR 检查通过：`uv run --no-project --offline --python 3.14.10 python -X utf8 target/d2-research/cefr-primary-followup/fetch_primary.py` 及同目录 `inspect_evidence.py`；11 份完整 HTTP 200 正文／元数据、八份完整搜索批次、解析器断言、原文引文、49 个本地链接和 57／56 冻结查询／词头数量均核对。语料覆盖／转换未执行，新增数据未准入。
- 仓库检查：`pnpm test:docs` 通过，153 份文档、1,252 个本地链接，无问题；77 个本地链接按检查器规则跳过。`git diff --check` 通过。
- MDX/MDD 筛选：八类各 1–2 项已核对出版社／官方资源、制作者声明、实际目录和公开质量反馈；完整来源证据位于 `target/d2-research/mdx/` 的 `french-findings.md`／`english-findings.md`，法语随包来源与权利核对见 [french-bundle-findings.md](../target/d2-research/mdx/french-bundle-findings.md)。目录链接的资源对应关系已复核，记录位于同目录的 `catalog-link-check-2026-11-10.md`；发布页可读不代表所有网盘入口均可取包。商业社区词典仍未下载／解包，其包头及完整正文兼容性未知。
- MDX 转换与独立读取通过：`target/d2-research/mdx/french-bundle-proof/` 下的 `build_freedict_mdx.py`／`build_mdx.py` 内置断言核对源日期、成员与索引边界、全部记录、备用键及文件尺寸；三份 MDX 分别通过 `cargo run --locked --manifest-path tools/mdict-probe/Cargo.toml -- <MDX 路径> école être avoir cœur œil cote côte côté apprendre pouvoir retentir taire lexique-no-such-word-2026`。全记录解码、重复保留、重音区别和缺词查询结果分别保存于 [FreeDict CLI 结果](../target/d2-research/mdx/french-bundle-proof/native-freedict_en.json)、[WikDict 法英结果](../target/d2-research/mdx/french-bundle-proof/native-wikdict_en.json)、[WikDict 法汉结果](../target/d2-research/mdx/french-bundle-proof/native-wikdict_zh.json)及同名完整日志。实际条数／字节见随包资源表；FreeDict 的 1,514,479 B 解码正文与转换器预期相等。2026-11-11，FreeDict 的随包原生查询／来源保存、通知和八类链接目录接入通过，见[产品证据](evidence/d2-mdx/README.md)；WikDict、MDD／脚本样式、整机断网首装和正式发行未验收。
- 忽略目录中的复现材料：`target/d2-research/freedict-field-probe.json`；`target/d2-research/continuation/parent/` 下的探查脚本、完整日志、下载清单、28 个现行源页修订、`query-field-audit.json`、`raw-query-field-audit.json`、`current-raw-probe/`、`current-raw-queries/`、`current-raw-queries-256m/`、`sqlite-core-probe/` 与 `sqlite-raw-core-probe/`；许可链／CEFR 原始佐证保存在 `target/d2-research/continuation/rights-agent/`；提取机制及 dump 日期前源页证据见 `target/d2-research/raw-continuation/alignment-agent/sense-association-research.md`。本轮摘要材料位于 `target/d2-research/summary-selection/`，包括 26 个截止日前源页、`sense-choices.json`、构建脚本／完整日志、`reviewed-summary-sample.json`、`source-choice-check.md`，以及 `ipa-agent/` 下的全部记录选择、源行、排除节点和独立检查材料。
- 资源契约复现材料：`target/d2-research/resource-contract/` 下的 `probe_reviewed_resource.py`／完整日志、`reviewed-core-probe/` 的结果、核心 JSONL 及四文件资源／压缩包；`source-reference-probe/` 的官方格式／任务清单／版本页、恢复脚本／完整日志、`dump-pages/` 的首分区索引、22 个压缩范围／响应元数据、28 页 wikitext 及 `dump-page-references.json`；`attribution-agent/rights-and-notices.md`、八项原始法律材料及 `evidence-index.json`。这些研究材料全部位于忽略目录。
- 扩展及生产者链材料：`target/d2-research/expanded-quality/` 的选择／比较／恢复脚本及完整日志、`sample-selection.json`、`freedict-expanded-audit.json`、30 个 `parent-data/` JSONL 及 HTTP 清单、`raw-cache-probe/expanded-field-audit.json`／原始选行、`dump-reference-probe/` 的七个新增范围及 32 页引用；`source-agent/findings.md`／`common-meaning-selections.json`、API／渲染响应、固定 gloss 代码及来源通知；`target/d2-research/source-correspondence/producer-agent/` 的固定生产者材料、两组标题引用断言及完整日志。原有冻结小样和材料保持原状态，新材料全部位于忽略目录。
- 完整扩展及权利材料：`target/d2-research/expanded-summary/source-agent/` 的 30 摘要 JSON、99 记录 IPA JSON、译词框／源记录审计、八组断言脚本／`verification-results.json`／完整日志及单份 findings；同级 `original-source-notices.json`／检查脚本，以及 `probe_combined_resource.py`／完整通过日志、`combined-reviewed-core-probe/` 的合并输入、核心 JSONL、四文件包及实际计量。`target/d2-research/imported-text-rights/` 保存 `ne` 修订转换证据、原作／法律／模板响应、56 页通知及检查结果；`target/d2-research/additional-cefr-sources/` 保存三项候选、固定发布／条款／论文、两份数据前缀、17 响应清单与字段／链接检查。材料位于忽略目录，原冻结小样未修改。
- 所选文字权利材料：`target/d2-research/daf8-span-review/combined-selected-span-reviews.json`、`verification-results.json`、`verify_selected_span_reviews.py` 及完整通过日志；`parent/` 下的单份 findings、原作、两项修订追踪及 51 份 HTTP 响应；`apprendre-retentir-taire/`、`etre-avoir-aller/`、`un-pouvoir/` 各自的 findings、逐项 JSON、原作／修订正文、响应元数据与独立检查日志。当前公开授权及法律依据另存于 `target/d2-research/rights-basis-continuation/`：`parent/` 保留完整法律／浏览器响应、WIPO 修正原文、当前权利 JSON、检查结果及日志；`tlfi/` 保留官方资源条款、完整 HTTP 正文／元数据、18 项准确引文及独立断言。全部材料位于忽略目录，先前冻结的小样及研究包保持原状态。
- 替代来源与完整表达分级材料：`target/d2-research/definition-alternatives/` 的 `daf8-five/`、`avoir/`、`early-dictionary/` 保留逐词候选／义项／语法角色、固定页与 XML、原作者许可／版本、BnF／法律原文及断言日志；`target/d2-research/rights-basis-continuation/daf8-territory/` 保留成员国与网站条款材料及检查结果。`target/d2-research/polylexfle-complete-audit/` 保留下载清单、全文件审核脚本／结果／日志；两个完整正文及响应元数据在 `additional-cefr-sources/sources/`，既有两份前缀逐字节保留且与完整正文匹配。全部位于忽略目录。
- PolyLexFLE 许可链材料：`target/d2-research/polylexfle-license-chain/` 下的 `findings.md`、`evidence-index.json`、完整官方通知／许可／固定发布正文及元数据、两份生产者 PDF／提取文本、`derived/polylexfle-2024-page-21.png`、原始工具响应、检查脚本及完整日志。材料位于忽略目录，未取得新增数据的补充授权。
- 代用释义多源及新增 CEFR 材料：`target/d2-research/mixed-source-contract/` 下的 `findings.md`、`source-field-and-license-evidence.json`、检查脚本／完整日志；`target/d2-research/cefr-primary-followup/` 下的 `findings.md`、`evidence-index.md`、11 份完整主来源 HTTP 快照及元数据、完整搜索／提取响应、检查结果与日志。全部位于忽略目录。
- XMLittré 定义边界材料：`target/d2-research/xmlittre-gloss-boundaries/` 下的单份 `findings.md`、七项法语跨度及通知提案 `candidate.json`、七份原始 variant 片段、`learner-context-locations.json`、四份输入的原始前状态字节、检查脚本／完整通过日志。全部位于忽略目录；旧摘要及原准入标志保持原状态。
- 未扫描 Wiktionary／DBnary 全量输入或执行 WikDict 新构建；未执行 Kaikki 提取器、恢复支持页依赖闭包或重现其输出。全量 A1–B2／三语学习者覆盖、五项 DAF8 的目标辖区权利依据、`avoir` 的独立目标用途授权、CFC 原作与逐项媒体／例句授权、完整来源归档／随包体积、应用读取／保存／导出、原生 UI、断网首启及发行仍待验收。本轮未作短语版权独创性门槛判定；不能把未闭合的许可依据等同于每个短语均受版权保护。法国 L123-3 现行／历史完整条文及修正后的第 16 条、美国 104A／304 原文均已保留；DAF8 的具体成员国保护状态及来源国恢复条件仍未确认。CEFR 额外授权未取得，D2 及 CEFR 准入继续开放。
- XMLittré 已完成固定 57 查询的词头／机械字段覆盖、六词核心义及七项定义跨度核对；印刷可视对照为 3／7，版次序号未观察，其余四项及全量转录一致性、世界范围权利状态、全样干净定义提取、历史发音的现代适用性和全量引用权利仍未核对。字段证据位于 `target/d2-research/xmlittre-field-quality/`，Gallica 拒绝响应及离线检查位于 `target/d2-research/xmlittre-print-fidelity/`；可读题名页／正文图像及逐项比较位于 `target/d2-research/littre-accessible-print/`。采用候选会改变当前定义及义项组织；五项 DAF8 和一项 TLFi 的既有摘要未替换，资源准入标志未改变。
