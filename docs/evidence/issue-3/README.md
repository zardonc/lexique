# 自定义 MDX/MDD 正文与媒体验证

2026-11-07，Windows x64 的独立探针通过 12 个真实 MDX、两个同名配套 MDD 及六组可公开的原创 MDX/MDD 夹具。真实 MDX 共解码 1,124,587 条物理记录，保留 43,024 条重复键记录；两个 MDD 分别完整读取 362 和 64 项媒体。原创配对实际验证“打开文件 → 查询正文 → 读取正文引用的 CSS、PNG、WAV”，三项媒体逐字节结果与原始文件一致。

探针复用原先 `mdictlib 0.2.8 + lzo` 的 MDX 1.2 实测边界，固定 `lzokay 2.0.1` 与 `serde_json 1.0.154`，使用独立 [Cargo workspace](../../../tools/mdict-probe/README.md) 和锁文件。解析器与 LZO 依赖为 MIT，JSON 依赖为 MIT OR Apache-2.0；原创夹具采用项目 MIT，独立写入器来源与复现方法见[夹具说明](../../../tools/mdict-probe/tests/fixtures/README.md)。

## 真实文件矩阵

用户指定的 [mdx.mdict.org 法语目录](https://mdx.mdict.org/按词典语种来分类/法语/) 与 [FreeMdict French 目录](https://downloads.freemdict.com/尚未整理/集合/Other%20language%20Dictionaries/French/) 均取得实际文件。完整来源 URL、原文件名、本地别名、字节数、SHA-256、头部版本／编码／日期、压缩标记、授权状态、查询结果和执行时间见 [samples.json](samples.json)。压缩列对应首个键块和全部正文块；首个键块的标记未被推断为全部键块的压缩方式。所有记录均实际解码并校验。

| 来源 | MDX 样本 | 版本／编码 | 压缩 | Encrypted | 物理记录 | 重复记录 | 结果 |
| --- | --- | --- | --- | --- | ---: | ---: | --- |
| mdx.mdict.org | Heliknight 词形变化，20070325 | 1.2 / UTF-8 | LZO | No | 18,085 | 0 | 通过 |
| mdx.mdict.org | 匿名英法／法英 | 1.2 / UTF-8 | LZO | No | 55,360 | 1,702 | 通过 |
| mdx.mdict.org | 拉鲁斯法语 | 2.0 / UTF-16 | zlib | 0 | 202,172 | 0 | 通过 |
| mdx.mdict.org | 罗贝特法法 | 2.0 / UTF-16 | zlib | 0 | 83,373 | 0 | 通过 |
| mdx.mdict.org | XMLittré | 2.0 / UTF-16 | zlib | 0 | 123,221 | 0 | 通过 |
| FreeMdict | 我爱法语 201909 | 2.0 / UTF-8 | zlib | 2 | 33,415 | 44 | 通过 |
| FreeMdict | Babylon 英法／法英，2016-10-17 | 2.0 / UTF-8 | zlib | 2 | 26,447 | 0 | 通过 |
| FreeMdict | Synonymes et Contraires，2004 | 2.0 / UTF-8 | zlib | 2 | 14,800 | 0 | 通过 |
| FreeMdict | 法语语音库 | 2.0 / UTF-8 | zlib | 2 | 121,311 | 4,350 | 通过 |
| FreeMdict | Académie，lingoes 090428 | 1.2 / UTF-16 | LZO | No | 31,210 | 16 | 通过 |
| FreeMdict | Académie，第八版 | 2.0 / UTF-16 | zlib | 2 | 31,216 | 0 | 通过 |
| FreeMdict | spell2 法语构词法 | 2.0 / UTF-8 | zlib | 2 | 383,977 | 36,912 | 通过 |

开始新增验证前，Heliknight 原文件的 SHA-256 核对为 `91ca54428d5cf5cce8d91bdaae50895136ae37880549ed4ceb9fed9492b4b817`。重新读取仍得到 18,085 条记录与 1,353,113 个 UTF-8 正文字节；`être`、`sommes`、`aujourd'hui`、`chat`、`amour` 命中，`œil`、`école` 未命中，与原实测一致。

| 同名配套 MDD | 版本／键编码 | 压缩 | 媒体项 | 解码字节 | 结果 |
| --- | --- | --- | ---: | ---: | --- |
| 拉鲁斯法语词典.mdd | 2.0 / UTF-16LE | zlib | 362 | 4,615,564 | 全部读取通过 |
| 罗贝特法法词典.mdd | 2.0 / UTF-16LE | zlib | 64 | 391,434 | 全部读取通过 |

两个真实配套 MDD 的结果是同名 MDX 查询与整库媒体读取；正文引用到特定媒体的合法、可公开验证由以下原创配对承担。

## 可公开配对与失败检查

独立 `mdict-utils` 写入器固定提交为 `64e15b99aca786dbf65e5a2274f85547f8029f2e`。每个 MDX 有十条记录，包括两条不同的 `double`、`être`、`œil`、直／弯撇号、`cote`／`côte`／`côté` 和原始重定向；每个 MDD 含同一正文引用的 CSS、PNG、WAV。哈希和参数见 [manifest.json](../../../tools/mdict-probe/tests/fixtures/manifest.json)。

| 配对 | MDX 版本／编码 | 键与正文压缩 | 索引加密 | 查询／配套媒体 |
| --- | --- | --- | --- | --- |
| v1-utf8-none | 1.2 / UTF-8 | 无 | 否 | 通过 |
| v1-utf16-none | 1.2 / UTF-16 | 无 | 否 | 通过 |
| v2-utf8-none | 2.0 / UTF-8 | 无 | 否 | 通过 |
| v2-utf8-zlib | 2.0 / UTF-8 | zlib | 否 | 通过，三项媒体完整字节对照 |
| v2-utf16-zlib | 2.0 / UTF-16 | zlib | 否 | 通过 |
| v2-utf8-encrypted-index | 2.0 / UTF-8 | zlib | 是，Encrypted=2 | 通过 |

九项公开回归检查覆盖六组格式、重复键、上述法语拼写、正文／媒体缺项、MDX 和 MDD 的截断头部／正文、错误头部／记录块校验和、超出 128 MiB 的文件、超出 16 MiB 的解压块声明及不支持的 v3 头部。失败返回非零状态且不输出部分成功报告。原始重定向文本已读取；未执行跳转链。

文件、记录数、块、单条内容、累计解码量及 parser 内存均有[明确上限](../../../tools/mdict-probe/README.md#read-limits)。两个文件均显式使用 `ChecksumPolicy::Verify`，HTML 不执行，MDD 键不被用作主机提取路径。

## 正式接入约束与 #9 的影响

| 暂存方向 | 本次证据与剩余决定 |
| --- | --- |
| D1 多 MDX／按需正文 | v1/v2、两种 Unicode 编码、LZO／zlib／无压缩、重复键和索引加密已有实际读取证据。标签展开、优先级、索引／缓存、逐词典错误隔离与加载／查询预算仍需产品规格。当前 CLI 为完整扫描验证，计时不能作为按需查询预算。 |
| D2 完整内置摘要 | 本次验证不准入新的默认词典、CEFR 或摘要字段；来源、语言、义项与质量继续分别验收。 |
| D3 词形归属 | Heliknight／spell2 能读取不等于可靠原形映射已验证。多匹配、异写、资源升级重绑定及同词记录归属继续保持现有边界。 |
| C1 重定向／媒体／内跳 | 配对媒体读取具备合法复现输入。跳转语义、循环／缺项边界和显示隔离仍需正式接入规格。物理 ordinal 只对同一未改变文件稳定。 |
| C2 例句摘录 | 提取、内容授权、来源引用与明确保存仍待规格；本次不自动保存词典原文或例句。 |
| C3 静态音频／在线缓存 | 原创 WAV 完整读取通过；第三方录音许可、音质、格式／播放及缓存规则须分别验证。 |

外部 HTML 必须与 Tauri IPC、原生文件系统及服务凭据隔离，脚本、事件和任意外链不能取得应用权限。HTML／CSS 的资源引用只能经受控虚拟词典资源解析，限制协议、资源类型、大小及网络请求；MDD 键保持容器内身份，不生成任意主机路径。图片和声音须验证类型并由受控字节载入，离开词条时释放播放与对象 URL。`@@@LINK`／词条内跳转须在明确词典上下文内处理缺项和循环；具体呈现方案留待正式适配。

## 验证命令

| 命令／边界 | 结果 |
| --- | --- |
| `pnpm mdict:test:build` → `pnpm test:mdict` | 通过，九项回归；执行硬限 60 秒 |
| `cargo fmt --manifest-path tools/mdict-probe/Cargo.toml --check` | 通过 |
| `cargo clippy --locked --manifest-path tools/mdict-probe/Cargo.toml --all-targets -- -D warnings` | 通过 |
| `cargo build --release --locked --manifest-path tools/mdict-probe/Cargo.toml` | 通过，Windows x64 原生 CLI |
| 上述 CLI 对真实 MDX 查询固定法语词与明确缺项，再完整读取两个同名 MDD | 14 个文件通过；先核对每个原文件哈希，单次执行上限 60 秒；输出见 samples.json |
| `pnpm lint`、`pnpm typecheck`、`pnpm build` | 通过 |
| `pnpm rust:test:build` | 通过，使用 uv 管理的 Python 3.14 与新鲜资源；旧生成资源完整保留 |
| `pnpm test:rust` | 143 项通过，保留执行硬限 60 秒 |
| `pnpm rust:fmt`、`pnpm rust:check` | 通过，含 Windows x64 桌面编译 |
| `pnpm test:docs`、`git diff --cached --check` | 通过，127 份文档无问题，暂存差异无空白错误 |
| 相对 `cb2fb9c` 的 Standards 审查 | 通过，无发现 |
| 相对 `cb2fb9c` 的 Spec 审查 | 通过，无缺失、范围扩张或错误实现 |

## 限制

第三方原文件和未经授权正文未进入仓库。Heliknight、匿名词典、Larousse／Robert、Académie 和 XMLittré 的转换／再分发许可链未确认；XMLittré 的随附说明称免费，但未给出可核验授权链。我爱法语明确保留权利并要求作者授权；Synonymes et Contraires 与法语语音库含个人／学术非商业用途限制。这些文件的本地技术结果不赋予默认内置或公开夹具许可，正文／图片／声音的权利须分别核查。

本次没有真实 v3、其他文字编码、加密关键字头／passcode 或 MDD 分卷的兼容性证据；v3 夹具仅证明不支持头部被拒绝。语音库 MDX 没有本次取得并匹配的录音 MDD，不作为声音配对通过。生产导入、多词典加载、HTML 安全渲染、实际媒体播放、其他 OS／架构或 M5 发行未验收；用户学习库未改动。
