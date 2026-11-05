# M0 资源小样调查

日期：2026-10-24。来源为用户指定站点、项目默认资源政策及本次实际下载和解码。以下区分候选、局部验证和正式准入；尚未选定默认词典或 TTS。

## 词典下载与实际解析

| 来源与样本 | 下载字节 | 实测结果 | 许可与用途状态 |
| --- | ---: | --- | --- |
| [mdx.mdict.org 法语目录](https://mdx.mdict.org/按词典语种来分类/法语/)中的 Heliknight 法语词形变化词典，文件名版本日期 20070325 | 328,668 | MDX 1.2、UTF-8、HTML；头部校验通过。固定 `mdictlib 0.2.8`，启用 LZO 后逐条解码 18,085 条记录，18,085 个不同键，正文共 1,353,113 字节。`être`、`sommes`、`aujourd'hui`、`chat`、`amour` 命中；`œil`、`école` 未命中。 | 文件头没有查到明确转换、再分发授权。仅作本地兼容性调查；不得因此纳入默认包。命中 `sommes` 不代表已验证完整变位反查或覆盖。 |
| [FreeDict 法英 0.4.1](https://download.freedict.org/dictionaries/fra-eng/0.4.1/freedict-fra-eng-0.4.1.dictd.tar.xz) | 210,100 | Dictd 索引与正文全部解析：8,511 索引行，其中 6 行元数据、8,505 词条记录、8,249 个不同键；解压正文 394,629 字节。`être`、`école`、`chat`、`amour` 命中。 | 包内声明 GPL-2.0-or-later，附 GPL v2 正文。可继续评估默认资源，但须落实对应源数据、转换及再分发义务。未完成准入。 |
| [FreeDict 法中 2025.11.23](https://download.freedict.org/dictionaries/fra-zho/2025.11.23/freedict-fra-zho-2025.11.23.dictd.tar.xz) | 559,692 | Dictd 索引与正文全部解析：10,953 索引行，其中 6 行元数据、10,947 词条记录、10,589 个不同键；解压正文 1,056,352 字节。`être`、`œil`、`école`、`chat`、`amour` 命中。 | 包内声明 CC BY-SA 3.0 Unported，来源为 WikDict / Wiktionary / DBnary。可继续评估默认资源，需保留归属与适用共享义务。未完成准入。 |

“词条记录”不等于不同词头数量；同键记录不能在导入时静默去重。以上数据是小包探查结果，不能代表法中、法英、法法完整学习者覆盖或最终安装包体积。[FreeDict 官方下载目录](https://freedict.org/downloads/)提供更多格式及校验文件；正式准入仍应取得对应源包并对照发布校验值。

已读取用户指定的 [FreeMdict French 目录](https://downloads.freemdict.com/尚未整理/集合/Other%20language%20Dictionaries/French/)，以及其“法语”“法语111”子目录。它可作为获取自定义词典兼容性样本的线索；本次未下载这些子目录中的大词典、压缩全集或 MDD。两个聚合站都不能凭目录可访问性证明词典内容可随本项目发行。

另有 [XMLittré 目录](https://mdx.mdict.org/按词典语种来分类/法语/%5B其他语种%5D%20XMLittré%20法法词典/)和 [Kaikki / Wiktionary](https://kaikki.org/dictionary/French/index.html)可继续调查。XMLittré 的 MDX 标示约 48 MiB，本次仅检查目录；未核验该转换文件与原始上游的许可链。Kaikki 的后处理 JSONL 正在弃用，后续优先评估其官方原始数据路径。未将这些候选认定为已下载或准入。

### 下载身份

| 本地样本名 | SHA-256 |
| --- | --- |
| morphology.mdx | `91ca54428d5cf5cce8d91bdaae50895136ae37880549ed4ceb9fed9492b4b817` |
| freedict-fra-eng.dictd.tar.xz | `29779196b78e8863fad73392f8faed3d242aa4ae4230bd13b71cfb9fcbbff4bb` |
| freedict-fra-zho.dictd.tar.xz | `44127a41f8fdf55d308b9492e883ed190c513856556736790e363c0bf4528b40` |

下载源完整 URL、HTML 目录证据、包内许可、完整元数据、解析脚本、隔离的 Cargo 清单 / 锁文件与运行日志保存在忽略目录 `.tooling/resource-probe-20261024/`。原始第三方内容未加入版本控制。只给出本地 SHA-256，不声称已经对照官方 SHA-512。

## TTS 候选与任务边界

Sherpa-ONNX 是推理引擎，Kokoro 和 Pocket TTS 是模型 / 工具生态；应比较完整可运行组合，而不是把三者视为同一层的互斥组件。

| 组合 | 本次核查的具体风险 | 下一步实际验证 |
| --- | --- | --- |
| Sherpa-ONNX + Piper SIWIS，或能证明法语可用的其他兼容模型 | [Sherpa-ONNX v2.0.0](https://github.com/k2-fsa/sherpa-onnx/releases/tag/v2.0.0)移除内置 espeak-ng / piper-phonemize，旧示例不能直接证明新组合完整可用。[SIWIS 模型卡](https://huggingface.co/rhasspy/piper-voices/blob/main/fr/fr_FR/siwis/medium/MODEL_CARD)注明法语及数据集 CC BY 4.0；不能只按外层仓库 MIT 标签认定所有权重和配套文件许可。 | 固定引擎、模型、声线、文本前端和运行库，验证完全本地的文本到 WAV 及播放，测量全部依赖。 |
| Kokoro 法语声线 + 经验证的运行时 | [官方声线说明](https://huggingface.co/hexgrad/Kokoro-82M/blob/main/VOICES.md)列有法语，并提示短文本可能较弱。官方模型 Apache 标签不代替声线、G2P 和具体转换文件的分别核验。不能以多语言模型名称推断所选 ONNX 包含法语。 | 用单词、变位及短句测试法语发音；若通过 Sherpa-ONNX 运行，单独验证该版本的法语音素前端。 |
| Pocket TTS 法语 + 经验证的运行时 | [官方法语目录](https://huggingface.co/kyutai/pocket-tts/tree/main/languages/french)显示 CC BY 4.0 标签、约 219 MB 主权重及访问条件。该数值不是压缩发行包体积。[官方代码库](https://github.com/kyutai-labs/pocket-tts)声明支持法语，但其他语言的小模型不能直接作为法语证据。 | 先确认实际权重 / 声线获取与发行条件；需要登录并接受条款时交由用户处理，不绕过门槛。再实测法语组合及完整体积。 |

本次没有下载 TTS 权重、接受访问条件、生成或试听音频。比较任务可先推进无需新增授权的候选；Pocket 待授权不会阻塞其余组合，也不阻塞首条查词链路。进入 M3 前仍须有满足许可、法语质量、完整配套和目标架构要求的组合。

## 验证范围

用户确认的后续任务已作为总规格 #1 的原生子任务发布：[默认词典准入 #2](https://github.com/zardonc/lexique/issues/2)、[MDX/MDD #3](https://github.com/zardonc/lexique/issues/3)、[变位 #4](https://github.com/zardonc/lexique/issues/4)、[Sherpa-ONNX #5](https://github.com/zardonc/lexique/issues/5)、[Kokoro #6](https://github.com/zardonc/lexique/issues/6)、[Pocket #7](https://github.com/zardonc/lexique/issues/7)、[原生离线查词 #8](https://github.com/zardonc/lexique/issues/8)。仅 #2 阻塞 #8，三张 TTS 互不阻塞。任务发布不代表资源准入或功能实现完成。

- Dictd：仅在内存中读取归档成员，不解包执行；逐条检查索引范围并严格解码 UTF-8，区分元数据、记录和不同键，保留重复键。两个归档均成功。
- MDX：头部长度及校验和检查成功；独立 Rust 工具逐条读取正文，完成真实查询。文件 HTML 只按文本读取，没有在浏览器或原生窗口执行。
- 初次 Python 输出遇到终端 GBK 无法显示 `œ`，改为显式 UTF-8 后重跑成功；没有改写源数据来消除字符。
- 尚未验证：MDD、损坏输入、全部格式版本与压缩组合、SQLite 转换、应用内查询、完整词义质量、四目标运行或任何正式资源准入。

## M2 单词离线语音补查

2026-10-25：M2 访谈已将听音卡所需的单词离线发音提前，见[确认记录](m2-review-core.md)。本节仅核对当前官方代码、模型卡与依赖，不代表资源已准入或已实际生成声音。

- Sherpa 2 的旧 `data_dir` 路径已弃用，Piper 路径需要法语 lexicon 或外部音素输入；旧版 SIWIS 示例不足以证明新组合能完成法语文本到声音。[固定版本配置](https://raw.githubusercontent.com/k2-fsa/sherpa-onnx/v2.0.0/sherpa-onnx/csrc/offline-tts-vits-model-config.h)、[生成接口](https://raw.githubusercontent.com/k2-fsa/sherpa-onnx/v2.0.0/sherpa-onnx/c-api/c-api.h)
- SIWIS low 单权重约 28.1 MB，但模型卡说明从 Ryan-low 微调，其来源卡标注训练数据为 CC BY-NC-SA；SIWIS medium 约 63.2 MB，从 Lessac-medium 微调，后者指向限研究用途的来源许可。训练数据条款不能直接替代对最终权重授权的判断；目前证据不足以闭合这两条权重的目标再分发授权链，不能仅凭 SIWIS 数据集的 CC BY 标签认定准入。[SIWIS low](https://huggingface.co/rhasspy/piper-voices/blob/main/fr/fr_FR/siwis/low/MODEL_CARD)、[Ryan 来源](https://huggingface.co/rhasspy/piper-voices/blob/main/en/en_US/ryan/low/MODEL_CARD)、[SIWIS medium](https://huggingface.co/rhasspy/piper-voices/blob/main/fr/fr_FR/siwis/medium/MODEL_CARD)、[Lessac 来源](https://huggingface.co/rhasspy/piper-voices/blob/main/en/en_US/lessac/medium/MODEL_CARD)、[来源许可](https://www.cstr.ed.ac.uk/projects/blizzard/2013/lessac_blizzard2013/license.html)
- 维护中的 [OHF Piper](https://github.com/OHF-Voice/piper1-gpl) 提供含 eSpeak 的原生 C API，但采用 GPL-3.0，需分别核对代码集成与分发义务；替换运行时不会消除上述权重授权疑问。Windows ARM64 的完整构建仍须验证。[原生依赖与构建](https://github.com/OHF-Voice/piper1-gpl/blob/main/libpiper/README.md)
- Kokoro 仍需完整法语文本前端；已查 Sherpa 多语言包未证明包含可直接使用的法语 lexicon。该包模型文件约 326 MB，另有至少约 28.2 MB 声音文件；这些是文件体积，不是压缩分发包实测，也不覆盖全部依赖。短文本质量仍为单词卡的关键验证项。[包文件](https://huggingface.co/csukuangfj/kokoro-multi-lang-v1_0/tree/main)、[法语声音与短文本说明](https://huggingface.co/hexgrad/Kokoro-82M/blob/main/VOICES.md)
- Pocket 当前已有官方 Rust 独立运行时，不能沿用仅能通过 Python 的旧假设；法语权重仍约 219 MB，另需 tokenizer、声音与运行依赖，模型有访问条件。没有 Windows ARM64 的完整执行证据，也未测压缩下载总量。[官方运行时](https://github.com/kyutai-labs/pocket-tts)、[法语文件](https://huggingface.co/kyutai/pocket-tts/tree/main/languages/french)、[访问条件](https://huggingface.co/kyutai/pocket-tts/blob/main/README.md)

下一步先落实具体组合与版本的许可证据，再在 Windows x64 验证代表性单词、未知词 / 音素、真实播放、首启断网、重启缓存及失败重试，测量含前端、模型、原生库和许可文件的完整压缩包。词典式前端可能跳过未知部分，须检测部分生成而非仅判断是否返回音频。本轮未下载模型、接受访问条件、安装、编译或试听；没有候选被宣称满足完整分发要求。
