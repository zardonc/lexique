# 法语离线 TTS 的商业再分发替代候选

2026-11-07。目标是 Windows portable 完整版随包提供、CPU 离线生成法语单词和句子；完整版不受 200 MB 上限限制。默认女声、男女声选择及正常／慢速的产品要求见[产品基线](../../product.md)。

排序：**现有 Supertonic 3 保留首选；Kokoro 是权重许可最明确的女声回退；Pocket TTS 是有条件的替代引擎；MeloTTS 和当前 Piper Gilles／SIWIS 不作为本次默认替换建议。** 既有 Supertonic 3 已有男女声及 Windows CPU 试听／离线证据，见[#14 准入记录](../issue-14/README.md)。替换 Tom 对照模型与替换产品默认引擎是两个不同决定。

## Supertonic 3：现有产品首选

发布者在固定版本 `3cadd1ee6394adea1bd021217a0e650ede09a323` 的[模型许可](https://huggingface.co/Supertone/supertonic-3/resolve/3cadd1ee6394adea1bd021217a0e650ede09a323/LICENSE)明确采用 OpenRAIL-M：§2 允许复制、修改和分发，§4 允许在任何介质分发原模型及其衍生模型。许可允许商业用途，但 §5／Attachment A 的使用限制继续适用。该模型许可没有 AGPL 式对应源码交付义务；按其条款分发已有授权，不需要等待发布者回复。模型及声线资产、MIT 示例代码和 Apache-2.0 Sherpa 引擎的许可分别保留，原文及固定来源见[许可证据](../issue-14/supertonic-licenses/ATTRIBUTION.md)。

Lexique 的[现有 worker](../../../apps/desktop/src-tauri/speech_worker.py)已经通过 Sherpa 使用 Supertonic、CPU 和两线程。法语女声／男声、正常／慢速及人工试听见[#14](../issue-14/README.md)，完整变位表达和新鲜语音包的 Windows 离线验收见[#38](../issue-38/README.md)及[#49](../issue-49/README.md)。[固定 int8 模型归档](../issue-14/supertonic3-model.json)为 **128,774,318 字节（128.77 MB）**；这只是模型归档，运行库、编码器和完整应用另计。

[现有准备脚本](../../../scripts/prepare_speech.py)随语音资源复制完整模型许可与 `supertonic-licenses/` 通知；[portable 打包脚本](../../../scripts/package_portable.py)在 README 指向 `speech/docs` 并提示机器合成和模型使用限制。正式完整版和 Lite 模型资源的交付应以该模型许可约束使用／分发，满足 §4(a) 的具有约束力的下游使用限制和通知，并按 §4(b–d) 保留完整许可、修改声明及归属通知。原模型许可及 Attachment A 随交付保留，模型条款的适用范围与项目 MIT 代码分别说明。

## Kokoro-82M：女声回退

- **模型权重**：发布者的[固定模型卡](https://huggingface.co/hexgrad/Kokoro-82M/blob/f3ff3571791e39611d31c381e3a41a3af07b4987/README.md)直接说明权重为 Apache-2.0、允许生产部署。[Apache §2／§4](https://www.apache.org/licenses/LICENSE-2.0)允许源形式或对象形式再分发；保留许可、通知和修改说明，没有要求交付模型的首选修改形式或将 Lexique 源码改为同一许可。
- **声线与数据**：官方配套资产是 [ff_siwis.pt](https://huggingface.co/hexgrad/Kokoro-82M/blob/f3ff3571791e39611d31c381e3a41a3af07b4987/voices/ff_siwis.pt)。它位于声明 Apache-2.0 的模型仓库，但未见该文件独立的声线许可文本；不能把“权重明确 Apache”写成另一份单独的声线授权。发布者在模型卡中列明 SIWIS 原录音的 CC BY 4.0 来源；现有证据保留了这份署名和配套声线身份。这与 Piper SIWIS 的 Lessac 微调链不同。
- **产品与质量**：[官方声线表](https://huggingface.co/hexgrad/Kokoro-82M/blob/f3ff3571791e39611d31c381e3a41a3af07b4987/VOICES.md)仅列一份法语女声，没有法语男声，并明确指出少于 10–20 tokens 的短话语较弱。它适合女声回退评估，不能完整替代约定的男女声；通用 TTS 排名不能证明孤立法语词的质量。
- **CPU 与体积**：现有 [Kokoro 独立探针](../issue-14/README.md#许可与资源身份)已在 Windows x64 禁网生成并调用播放，包含孤立词回归；修正后的整套样本没有形成当前默认准入。复用的 [int8 发布文件](https://github.com/thewh1teagle/kokoro-onnx/releases/tag/model-files-v1.0)为 ONNX 92,361,271 字节、全部声线 BIN 28,214,398 字节，合计 **120.58 MB**，未含运行库和许可源码。官方原始 PTH 为 327,212,226 字节，单个 ff_siwis 为 523,425 字节。

模型 Apache 不覆盖运行前端。既有组合的 kokoro-onnx 包装为 MIT，ONNX Runtime 为 MIT；phonemizer／eSpeak 含 GPL，仍需相应许可、对应源码和构建材料。具体边界已记录于[既有许可证据](../issue-14/README.md#许可与资源身份)。

## Pocket TTS：公开的无克隆路线，条件尚未齐备

- **权重与公开条款**：官方 [pocket-tts-without-voice-cloning 固定仓库](https://huggingface.co/kyutai/pocket-tts-without-voice-cloning/tree/1e08e6a23401048648a9fdcfde2f89348215c2a7)声明 CC BY 4.0；该版本 API 返回 `gated: false`。这是官方法语配置列出的无声线克隆权重来源，取得它不需要接受完整克隆模型的访问门槛。[CC BY 4.0](https://creativecommons.org/licenses/by/4.0/)允许商业复制、改编和分享，要求署名和修改说明，没有模型对应源码义务。
- 同版[公开 README](https://huggingface.co/kyutai/pocket-tts-without-voice-cloning/blob/1e08e6a23401048648a9fdcfde2f89348215c2a7/README.md)另列禁止未经同意克隆／冒充、欺骗及有害非法用途等条款。它没有说明这些附加条款与 CC BY 的“不增加限制”规则如何协调，也未说明是否必须向下游传导成合同。因此有公开的商业分享许可依据，但具体产品许可包装仍有实质不确定性；没有接受访问条款或索取额外授权。
- **声线不是全都可商用**：[官方固定来源映射](https://github.com/kyutai-labs/pocket-tts/blob/28770c06f7deb1b65617834e60a6add7a4ff5b6d/pocket_tts/utils/utils.py)将 Estelle 指向 Kyutai 自有法语 `developpeuse-3.wav`；[声线库许可说明](https://huggingface.co/kyutai/tts-voices/blob/323332d33f997de8394f24a193e1a76df720e01a/README.md)将这些自有录音列为 CC0。Marius／Javert 来自 CC0 志愿者录音，但本次没有核实其原始语言和性别；Charles 来自英语 VCTK，许可 CC BY 4.0。Cosette 来自 Expresso、Jean 来自 EARS，两者的录音源是非商业许可。法语目录中的 embedding 文件并不证明该声线是法语母语，也不抹去录音源的限制。
- **CPU 与体积**：[固定法语配置](https://github.com/kyutai-labs/pocket-tts/blob/28770c06f7deb1b65617834e60a6add7a4ff5b6d/pocket_tts/config/french.yaml)和公开 README 提供 CPU／本地配置路径；[软件](https://github.com/kyutai-labs/pocket-tts/blob/28770c06f7deb1b65617834e60a6add7a4ff5b6d/LICENSE)为 MIT，[依赖配置](https://github.com/kyutai-labs/pocket-tts/blob/28770c06f7deb1b65617834e60a6add7a4ff5b6d/pyproject.toml)要求 PyTorch 2.5+ 等运行库。该固定版本法语权重 **219,029,196 字节**，Estelle embedding **7,571,256 字节**，两份 tokenizer 元数据合计 307,093 字节；这尚不是完整 Windows portable 包大小。无须把英语小模型的速度数据当作法语实测。

Pocket 有可公开取得的权重及明确的商业分享许可证，但本次仍没有合格的法语母语男女声组合、孤立词试听或 Windows portable 运行证据，故排在已有 Supertonic 3 和 Kokoro 回退之后。

## MeloTTS-French：MIT 声明较弱，Windows 路线较重

[官方权重仓库](https://huggingface.co/myshell-ai/MeloTTS-French/tree/1e9bf590262392d8bffb679b0a3b0c16b0f9fdaf)的模型元数据声明 MIT，但 README 的许可段只称“this library”采用 MIT，仓库没有单独的权重 LICENSE；训练录音来源和声线授权未在该卡中说明。因此可以记录发布者的模型 MIT 声明，不能声称声线权利链已核实，也不能只引用 [MIT 软件许可](https://github.com/myshell-ai/MeloTTS/blob/209145371cff8fc3bd60d7be902ea69cbdb7965a/LICENSE)替代它。

[官方用法](https://github.com/myshell-ai/MeloTTS/blob/209145371cff8fc3bd60d7be902ea69cbdb7965a/docs/quick_use.md)有 `TTS(language='FR', device='cpu')` 和速度参数；[模型配置](https://huggingface.co/myshell-ai/MeloTTS-French/blob/1e9bf590262392d8bffb679b0a3b0c16b0f9fdaf/config.json)只公布一个 `FR: 0` 声线，未说明性别。[安装说明](https://github.com/myshell-ai/MeloTTS/blob/209145371cff8fc3bd60d7be902ea69cbdb7965a/docs/install.md)推荐 Windows 使用 Docker，没有直接可复用的免安装产品交付。

法语 checkpoint 为 **207,860,748 字节**；[法语前端](https://github.com/myshell-ai/MeloTTS/blob/209145371cff8fc3bd60d7be902ea69cbdb7965a/melo/text/french_bert.py)另加载 [`dbmdz/bert-base-french-europeana-cased`](https://huggingface.co/dbmdz/bert-base-french-europeana-cased/tree/b895c3cf291f7bf4c15639078a6bee0b3e272c5b) 的 tokenizer／BERT，当前公开 PTH 为 **445,018,784 字节**。两个权重约 **652.88 MB**，还未含 PyTorch、其他资源和运行库。CPU 法语路径存在，但它没有解决本次声线与 portable 交付目标。

## 当前 Piper Gilles／SIWIS：基础权重授权链仍未闭合

官方声音仓库在 `e7229f7d33a10e145e109b886d661984201ac3c4` 仍只有既有七份法国法语模型卡。

- [Gilles low](https://huggingface.co/rhasspy/piper-voices/blob/e7229f7d33a10e145e109b886d661984201ac3c4/fr/fr_FR/gilles/low/MODEL_CARD)的法国法语数据是 CC0，但明确由 U.S. English Ryan low 微调；[Ryan low 卡](https://huggingface.co/rhasspy/piper-voices/blob/e7229f7d33a10e145e109b886d661984201ac3c4/en/en_US/ryan/low/MODEL_CARD)标明 RyanSpeech 为 CC BY-NC-SA 4.0、从零训练。法语数据的 CC0 未解释受限基础权重的商业再分发授权。
- [SIWIS medium](https://huggingface.co/rhasspy/piper-voices/blob/e7229f7d33a10e145e109b886d661984201ac3c4/fr/fr_FR/siwis/medium/MODEL_CARD)仍声明从 Lessac medium 微调；[UPMC medium](https://huggingface.co/rhasspy/piper-voices/blob/e7229f7d33a10e145e109b886d661984201ac3c4/fr/fr_FR/upmc/medium/MODEL_CARD)也是同一路径。SIWIS 的 CC BY 4.0 数据授权与 Lessac 的 Research Purposes 条款是不同对象，后者的完整核查见[既有许可证据](licenses/ATTRIBUTION.md#siwis-的独立核验)。没有发现清除这条基础权重缺口的新授权。

数据许可不自动等于派生模型许可，因此这里记录的是未闭合的授权链，而非断言所有派生权重必然继承数据的 NC。声音仓库的外层 MIT 元数据也不能独立解决该问题。`fr_FR-mls-medium` 保持用户已确认的音质否决；本次没有形成重新试听它的原因。

## 核验范围

本次 `gh api` 固定源码／发布元数据、HF 模型 API／固定模型卡读取和官方许可页面读取通过；没有下载新模型负载、生成新样本、改动应用或用户学习数据。体积均为公开文件逻辑字节或既有固定发布文件，完整应用压缩包／安装占用另计。Supertonic 的模型再分发许可明确，正式发行的下游条款交付仍待验收；#14 已接受的 wheel 构建来源和未独立复现校准限制沿用其既有记录。新候选的法语单词／句子、正常／慢速质量与 Windows portable 包尚未实测；ARM64、macOS 及干净机器测试按本次范围延期。
