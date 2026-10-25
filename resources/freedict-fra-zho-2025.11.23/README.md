# FreeDict 法中开发小样

对应 [issue #2](https://github.com/zardonc/lexique/issues/2)。本目录保留一个可随包分发的开发资源小样所需源数据、许可和归属；不是正式发行的完整默认词典，不代表三语覆盖或桌面查词已完成。

## 来源与许可

- 源包：`freedict-fra-zho-2025.11.23.src.tar.xz`，来自 [FreeDict 官方发布](https://download.freedict.org/dictionaries/fra-zho/2025.11.23/freedict-fra-zho-2025.11.23.src.tar.xz)。版本、语言、原始 URL、转换说明和 SHA-512 见 [provenance.json](provenance.json)。
- 2026-10-24 已下载[官方 SHA-512](https://download.freedict.org/dictionaries/fra-zho/2025.11.23/freedict-fra-zho-2025.11.23.src.tar.xz.sha512)，与源包实算一致。原发布校验文件一并保留；这证明字节与发布校验一致，不是发布者数字签名。
- 原始 TEI 头部明确声明 CC BY-SA 3.0 Unported，维护者 / 出版者 Karl Bartel；FreeDict / WikDict 从 Wiktionary 经 DBnary 自动生成。完整源包和其中的许可正文原样保留；独立的 [COPYING](COPYING) 与包内字节一致，未运行源包里的 Makefile 或其他文件。
- 依据 [CC BY-SA 3.0 条款](https://creativecommons.org/licenses/by-sa/3.0/legalcode.en)第 3、4 节，可以转换和再分发，包括商业用途；必须保留署名、来源和许可 / 免责声明，标明转换，改编词典数据沿用 CC BY-SA 3.0，不附加限制或 DRM。许可没有 GPL 式“提供对应源码”的义务；本小样仍随载荷提供原始 TEI 源包和校验文件，方便复核、修改和重建。
- 项目转换 / 查询 / 测试代码保持 [MIT](../../LICENSE)。MIT 不覆盖上游词典或转换后的词典数据，也不赋予任何其他词典内容的授权。归属和转换说明随 `provenance.json` 进入载荷。

## 生成与查询

仅资源工具额外需要 Python 3.12+ 标准库（含 SQLite）；不增加应用运行依赖。所有输入已保存在仓库，以下步骤不下载任何内容。输出目录必须不存在；不会覆盖已有资源或学习数据。

在仓库根目录运行：

```sh
python -X utf8 scripts/dictionary_sample.py build target/dictionary-sample
python -X utf8 scripts/dictionary_sample.py query target/dictionary-sample/dictionary.sqlite3 "œil"
python -X utf8 scripts/dictionary_sample.py query target/dictionary-sample/dictionary.sqlite3 "aujourd’hui"
python -X utf8 -m unittest discover -s scripts -p test_dictionary_sample.py -v
```

也可运行 `pnpm resources:build target/dictionary-sample`、`pnpm resources:query <database> <key>`、`pnpm test:resources`。Windows 的 `python` 必须指向真实安装，不能仅有 Microsoft Store 的应用别名；也可用 Python 的绝对路径代替。

构建先验证固定源包 SHA-512、发布校验文件和原始许可，再逐条转换。归档只在内存中读取指定成员，不向文件系统解包。SQL 参数化；不规范化、去重、生成词形或执行词条 XML。公开 Python 查询入口是 `Dictionary(path)` / `lookup(key)`；连接以 `mode=ro` 打开，验证应用标识、版本、SQLite 完整性、归属、记录数量及固定的完整内容校验值。与用户学习数据库没有连接或迁移关系。

CLI 命中返回 JSON 列表及退出码 0；无结果返回 `[]` 及退出码 2；资源损坏、未知格式或不存在时保留异常上下文及退出码 1。Python `lookup` 用 `[]` 明确表示无结果，资源错误抛出异常。输入区分大小写、重音和撇号；本票不引入模糊搜索或字符折叠。

输出 `dictionary.sqlite3`、原始源包、`COPYING`、`provenance.json`、发布校验文件，以及包含上述五项的 `sample.tar.gz`。`report.json` 是包外测量报告，不计入安装载荷。gzip 时间戳 / 文件名和 tar 时间 / 用户 / 权限固定；相同 Python / SQLite / 压缩运行库环境下连续两次生成逐字节一致。跨 SQLite 或压缩库版本只承诺相同记录内容，不承诺相同物理文件哈希；报告保留工具版本。

## 小样结构与覆盖

SQLite 是本小样可用的结构化格式：10,947 条源记录 / 10,777 个不同原始键，重键用独立序号保存，索引支持参数化精确查询，读取不要求额外服务。表 `entries(ordinal INTEGER PRIMARY KEY, key TEXT COLLATE BINARY, entry_xml TEXT)` 保存源文件顺序中的序号、原始词头和原始 `<entry>...</entry>` 片段；`resource(provenance TEXT)` 保存归属 JSON。`application_id = 0x4C585244`，`user_version = 1` 是此小样格式版本，和用户库版本无关。

`entry_xml` 保留所有词义、译文、音标和语法标签及其层级，UTF-8 内容不做改写；片段继承的 TEI 命名空间原在源文件根元素上，完整 TEI 同时随源包保留。序号仅在该固定快照内有效，不是后续笔记的稳定身份。完整资源的最终 schema、搜索方式及 Rust 接入未在本票冻结。

| 检查 | 实测结果 |
| --- | --- |
| 重音、连字、中文 | `école` → 含“学校”；`œil` → 含“眼睛” |
| 撇号 | `aujourd’hui`、`Côte d’Ivoire` 命中；ASCII `aujourd'hui`、`Côte d'Ivoire` 不命中，原始弯撇号保留 |
| 同键多记录 / 多义 | `or` 保留 3 条、`Abu Dhabi` 保留 2 条；每条内部全部 sense 保留 |
| 已存在的复数形词头 | `vacances` 原样可查；这是独立词条，不代表一般屈折或变位覆盖 |
| 缺项 | `sommes`、`yeux`、`chevaux`、`données`、`gens` 无记录；`été` 只有季节名词义，不能充当 être 分词证据 |
| 样本语言与质量 | 法语词头 / 中文译文，含法语释义；没有法英 / 法法完整词典、CEFR 等级、媒体、TTS 或全量词形映射；未逐条人工评审译文质量 |

先前调查的 Dictd 包同为 10,947 条记录，但只有 10,589 个不同索引键；TEI 原始词头有 10,777 个。两者不是数据丢失的同一指标，不把 Dictd 规范化索引当成原始词头。测试直接逐条读取原始 TEI，对比公开查询的每个元素 / 属性 / 文本及记录顺序，并检查原资源未变、缺项、损坏、无效来源和拒绝覆盖。

## 本机测量

2026-10-24，Windows x64、Python 3.12.13、SQLite 3.50.4、转换版本 1；生成结果位于忽略目录 `.tooling/dictionary-admission/result/`。

| 项目 | 字节 |
| --- | ---: |
| 原始官方源包下载 | 464,676 |
| 源包中 TEI 原文 | 4,346,160 |
| 转换 SQLite | 5,062,656 |
| 完整载荷压缩下载 `sample.tar.gz` | 1,341,112 |
| 安装载荷逻辑字节合计（SQLite + 源包 + 许可 + 归属 + 发布校验） | 5,551,649 |

SQLite SHA-256：`11cdba7ed8fbdcd6f6a388586c11a181def517d840aae0baa6ef4a4ab5c55c51`。
压缩包 SHA-256：`797adc8e927b51f52a7d559547ec707eea6aa084c83c6560dad0d1e703745829`。
安装载荷按各文件长度相加，不含文件系统分配块、应用程序或系统运行依赖；压缩包是开发资源载荷，不是 Tauri 安装器。无需把源 TEI 全部解开后才可查询。

SQLite 小样准入通过仅限上述固定数据和随附义务。桌面集成、完整三语词典、四架构运行、正式安装包与首装禁网验收仍需后续工作。
