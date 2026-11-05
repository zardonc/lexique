# #4：Lefff 反查与表读取验证

2026-11-05，Windows x64，Python 3.14.10 / SQLite 3.53.1。
固定 Lefff 3.4 全量形态源，生成独立 SQLite，公开 CLI 已实际跑通
“词形 → 全部原形／标签展开 → 每个原形的源表 → 全部匹配标记”。
这是 M0 资源验证；M3 正式范围仍待用户确认。

## 复验

仅使用 Python 3.12+ 标准库，无网络、模型或学习数据库依赖。
Windows 按 LOCAL-DEV-ENV 使用 uv 管理的 Python；将该解释器放入 PATH 后可运行 pnpm 别名。
输出目录必须不存在，失败不覆盖已有资源。

```sh
uv run --python 3.14.10 python -X utf8 scripts/conjugation_sample.py build target/conjugation-recheck
pnpm conjugation:query target/conjugation-recheck/conjugation.sqlite3 sommes
pnpm conjugation:table target/conjugation-recheck/conjugation.sqlite3 falloir
pnpm test:conjugation
```

后面三个命令也可使用 `uv run --python 3.14.10 python -X utf8 scripts/conjugation_sample.py query|table ...`
和 `uv run --python 3.14.10 python -X utf8 -m unittest discover -s scripts -p test_conjugation_sample.py -v`。
CLI 缺项返回 `status: not_found`；资源错误返回非零退出码和 stderr，绝不伪装缺项。
查询是原始拼写的二进制精确匹配，SQL 参数化，不移除重音、不改大小写或撇号。

## 输入、期望与实际

[完整 CLI 实际输出](actual-results.json)保留查询、全部匹配和源表；
[测试](../../../scripts/test_conjugation_sample.py)使用独立写定的期望值。
`source_line` 指向原始 mlex 行；`raw_tag` 保留原标签。
同标签的明确多时态／多人称展开成多个 cell，同格多拼写不覆盖。
`matched` 在返回的所有表中逐格比较原始形式。

| 输入 | 期望与实际 |
| --- | --- |
| sommes | sommer P2s、S2s；être P1p；3 个匹配格 |
| suis | suivre P1s、P2s、Y2s；être P1s；4 个匹配格 |
| aime | aimer P1s、P3s、S1s、S3s、Y2s；5 格 |
| été | être K，性数未明，保持 null |
| aimer / finir | aimons / finissons 在 P1p；规则动词实际源表 |
| aller | vais P1s；不规则源表 |
| avoir / être | avons / sommes P1p；助动词自身源表 |
| enfuir / s'enfuir | enfuir 有词形；带代词原形缺项，不声称已支持代词式 |
| falloir | 9 个源记录；只有第三人称单数有限式及 W / K，不补人称 |
| pleuvoir | 16 个源记录，保留源中的第三人称复数，不擅自删格 |
| payer | P1s 同时保留 paie / paye |
| lexique-inexistant / avons aimé | 明确缺项，无回退造词 |

缺失文件、非 SQLite、截断库、删掉匹配记录、缺少／损坏源包、已有输出目录均有 CLI 回归。
源包读取前校验归档和解压后 SHA-256，SQLite 只读打开并检查格式、完整性、来源及总行数。

## 覆盖矩阵，供正式范围确认

以下为源中实际出现的 lemma 数／展开格数，包含源异常项，不是语言正确率。
空格只能解释为“源未提供”；不能区分真正缺陷与漏收。
读取所有已收录格已验证；本转换器不生成新的法语形式。

| 语式／时态 | 源 tag | 有数据的 lemma / 格数 | 路径与状态 |
| --- | --- | --- | --- |
| 直陈式现在 | P | 7,802 / 46,865 | 源读取已验证 |
| 直陈式未完成过去 | I | 7,795 / 46,669 | 源读取已验证 |
| 直陈式简单过去 | J | 7,784 / 46,591 | 源读取已验证 |
| 直陈式简单将来 | F | 7,799 / 46,861 | 源读取已验证 |
| 条件式现在 | C | 7,799 / 46,864 | 源读取已验证 |
| 虚拟式现在 | S | 7,798 / 46,839 | 源读取已验证 |
| 虚拟式未完成过去 | T | 7,783 / 46,587 | 源读取已验证 |
| 命令式现在 | Y | 7,772 / 23,353 | 仅明确人称；不补不适用格 |
| 不定式现在 | W | 7,820 / 7,821 | 源读取已验证 |
| 现在分词 | G | 7,782 / 7,788 | 无主语性数推断；副动词 en + G 未生成 |
| 过去分词 | K | 7,803 / 31,167 | Kms/Kmp/Kfs/Kfp 明确；Km 无数、K 无性数保持 null |
| 未标语式 | 空 tag | 8 格 | 原样保留，tense=null，不冒充可定位的标准格 |
| 直陈式复合过去、愈过去、先过去、先将来 | 无 | 无 | 可用 avoir/être 的 P/I/J/F + K 组合；尚未实现／验证 |
| 条件式过去第一／第二形式 | 无 | 无 | C/T + K 候选，正式范围待确认 |
| 虚拟式过去／愈过去 | 无 | 无 | S/T + K 候选，正式范围待确认 |
| 命令式过去 | 无 | 无 | Y + K 候选，适用人称和缺陷传播待确认 |
| 不定式过去／复合分词 | 无 | 无 | W/G + K 候选，尚未实现／验证 |
| 代词式／被动式／否定、倒装 | 无完整句法 | 无 | 缺 clitic、助动词选取、语境和配合规则；不能只靠形态表生成 |

共 26 个有限式展开格缺少明确人称或数，不能当完整定位。
`K` 不直接展开成四个性数格：例如 été、fallu 的裸标签不能证明每种配合合法。
`Km` 的无数标签可能表示数同形，但本验证保留未知，正式展示须确认 tagset 语义。
复合时态还需语义／及物性控制助动词、前置直接宾语与代词配合，
以及应保留哪些异体、缺陷、文学时态的产品决定。这里没有冻结矩阵。

## 来源、许可与已知生成风险

[资源说明](../../../resources/lefff-3.4/README.md)、[固定来源与双哈希](../../../resources/lefff-3.4/provenance.json)
和[原始 LGPL-LR](../../../resources/lefff-3.4/LICENSE-LGPLLR)随仓库及构建输出保留。
原作者为 Benoît Sagot 与贡献者，引用 Sagot (2010), LREC。
镜像提交固定为 `ff38d3275e86567142506592d6e14ce679bf70e3`；gzip 解压后是原始文件，
不是 RosaeNLG 的衍生 JSON。生成 SQLite 是 2026-11-05 的数据转换，仍按 LGPL-LR，
附完整可编辑源、转换源码、修改日期、署名、许可及无担保声明，满足本验证包的第 1/2/4a 条路径。
代码与数据分别核查，无非商业限制；正式发行仍须保留这些材料。

参考并审阅同一固定提交的
[RosaeNLG 生成代码](https://github.com/RosaeNLG/rosaenlg/blob/ff38d3275e86567142506592d6e14ce679bf70e3/packages/french-verbs-lefff/src/create/createList.ts)
（文件头 Apache-2.0）和
[复合时态代码](https://github.com/RosaeNLG/rosaenlg/blob/ff38d3275e86567142506592d6e14ce679bf70e3/packages/french-verbs/src/index.ts)，
没有复制、运行或引入这些代码。其
[数据说明](https://github.com/RosaeNLG/rosaenlg/blob/ff38d3275e86567142506592d6e14ce679bf70e3/packages/french-verbs-lefff/README.md)
明确衍生数据继续 LGPL-LR；不能用外层代码许可替代。
该设计显示复合时态需要额外助动词／配合输入；按格赋值的转换可能覆盖 paie/paye 等异体，
裸 K 自动填四格还会把不明确性数当成完整数据。

原始数据包括 `_error`、`uw`、`uwSe`、être 的 `st` P3p，以及符号词形和空 tag。
本验证保留并暴露，未施加隐藏修正。M3 必须确定过滤／校订和来源可追踪策略。
此处全部 lemma／格数因此不能当作 7,826 个已审核教学动词。

## 实测体积

[原始计量](sizes.json)，SQLite 页分配量分别在建表、原形索引及反查索引后实测。

| 对象 | bytes |
| --- | --- |
| 原始全量 mlex | 15,677,049 |
| 源 gzip | 2,843,158 |
| 表及元数据 | 17,842,176 |
| 原形索引增量 | 7,311,360 |
| 反查索引增量 | 7,942,144 |
| SQLite 总计 | 33,095,680 |
| SQLite gzip | 9,799,124 |

342,771 个原始动词记录 → 397,413 个展开格，7,826 个原形、301,208 个不同原始形式。
总库约 31.56 MiB，反查索引约 7.57 MiB；附源是许可履约材料，单独计量。
数据库压缩量受 SQLite／gzip 版本影响；以上为本次实际执行，非四平台发行包计量。

## 验证范围

公开 CLI 的真实源构建、全部样例和失败路径已执行；测试命令与最终结果见下方记录。
未接 Rust、IPC、桌面 UI、打包或用户库，也未运行上游语言生成器。
其他平台执行、全词表语言学审校、复合时态生成及正式 M3 范围批准仍待后续。
