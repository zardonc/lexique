# 两份 ONNX 语音调研报告的复核

日期：2026-10-27。输入为用户提供的 `C:\Users\inthe\IdeaProjects\Notes-sur-l-IA\Projects\FR-dictionary\Requirements-ONNX.md`，其中分别称为 Research 1 和 Research 2。本文只审阅公开资料与本项目已有证据；没有下载、运行或更换模型。公开仓库可变，发行前须固定版本并复核。许可证描述是技术准入分析，不代替最终法律审查。

## 结论

**目前没有充分理由更换已试听通过的 Supertonic 3 int8 候选。** 用户已确认其法语与英语样本、男女声、正常生成后 0.6 倍播放和完整样本，以及 Opus 24 / 48 kbit/s 的听感；见[验证记录](../../verification.md)和[试听记录](supertonic-trial.md)。这些证明的是特定候选、样本和本机体验，并不证明产品已接入或正式发行已准入。两份新报告都未测试替代方案，不能证明新模型质量至少相当；特别是 Research 2 的推荐之一法语 MLS 声线已被用户实际试听否决。

**如果完整首装包测量后超出目标，首先考虑研究并征得授权后比较更小的具体转换版或 Supertonic 2；现阶段不宜预判替换。** 项目目标是按架构计算包含应用、默认词典、变位、模型、运行库和必要离线依赖的完整压缩下载量，而非只看权重；默认核心资源不能改为首次启动下载。超出 200 MB 预期上限须提供明细并由用户决定，功能正确优先，详见[项目准入门槛](../../development-readiness.md)和[产品约束](../../product.md)。

## 候选及证据边界

| 候选 | 核对与判断 |
| --- | --- |
| 已试听的 Supertonic 3 int8 | [固定资源与测量](replacement-options-after-mls.md)：模型七文件共 145,295,768 字节，模型压缩包 128,774,318 字节，均不含完整应用和依赖。Windows x64 禁网生成及用户试听已有记录，不能外推至四架构或完整首装包。上游[模型卡](https://huggingface.co/Supertone/supertonic-3/tree/3cadd1ee6394adea1bd021217a0e650ede09a323)区分模型 OpenRAIL-M 和代码 MIT；分发须逐项落实许可全文、用途限制、修改、通知及合成内容标识，参见[上游许可](https://huggingface.co/Supertone/supertonic-3/blob/3cadd1ee6394adea1bd021217a0e650ede09a323/LICENSE)。这条路径并非无限制 MIT 模型，是否符合最终产品政策仍须落实。 |
| Supertonic 2 int8 | [上游模型卡](https://huggingface.co/Supertone/supertonic-2)明确支持法语和英语，模型为 OpenRAIL-M；Research 1 引用的[维护者资源清单](https://huggingface.co/csukuangfj2/sherpa-onnx-supertonic-tts-int8-2026-03-06/tree/main)提出约 96.4 MB 模型文件。前期[记录](replacement-options-after-mls.md)给出旧发布包 84,692,981 字节，但当时未确认内部版本与声线映射。模型文件和压缩包不能直接比较；本轮也未验证该量化包与上游版本的一一对应、听感、所有运行依赖或完整首装包。它可作为减小体积的候选，不是已有质量合格的替代品。 |
| askurios8 Supertonic 3 int8 | [发布者模型卡](https://huggingface.co/askurios8/supertonic-3-int8)称其为非官方转换，四个图约 105 MB，另需配置及声线；保留 OpenRAIL-M。发布者提供的声学指标仅覆盖韩语和英语，不能证明法语和本项目声线听感，也不证明与当前 sherpa 资源格式直接兼容。转换前后文件构成、运行性能、完整压缩包和依赖需单独核验；即使同为 Supertonic 3，也不能直接把原先用户试听结论迁移过去。 |
| Piper MLS / LibriTTS-R | [MLS 模型卡](https://huggingface.co/rhasspy/piper-voices/blob/main/fr/fr_FR/mls/medium/MODEL_CARD)将 CC-BY-4.0 写在 Dataset 下，不能由此推定权重本身已明确采用该许可。更重要的是，本项目真实法语 MLS 样本被用户反馈为混乱、音质不佳并撤回，见[筛查记录](replacement-options-after-mls.md)。Research 2 的 48–56 MB 是未在 Lexique 实测的量化预测，不能据此覆盖失败听感或证明发行准入。 |
| Piper UPMC / SIWIS / Gilles | [UPMC 卡](https://huggingface.co/rhasspy/piper-voices/blob/main/fr/fr_FR/upmc/medium/MODEL_CARD)显示两说话人、数据 CC-BY-SA 4.0，且由 Lessac 模型微调；[SIWIS 卡](https://huggingface.co/rhasspy/piper-voices/blob/main/fr/fr_FR/siwis/medium/MODEL_CARD)也记载 Lessac 微调；[Gilles 卡](https://huggingface.co/rhasspy/piper-voices/blob/main/fr/fr_FR/gilles/low/MODEL_CARD)记载 Ryan 微调。[Lessac 模型卡](https://huggingface.co/rhasspy/piper-voices/blob/main/en/en_US/lessac/medium/MODEL_CARD)单独指向其数据许可。数据集、基础权重、微调权重及运行依赖是不同许可问题，不能依据当前语料 CC 标记就断言闭源分发无其他条件；UPMC 亦不能仅凭另一报告的相反断言直接认定禁止商业分发。须锁定具体权重和完整来源链后评估。 |
| Mimic 3 | [官方 README](https://github.com/MycroftAI/mimic3/blob/master/README.md)明确写代码为 AGPL-3.0，并标明项目不再维护；其快速开始还列出 `libespeak-ng1`。Research 2 的“主仓库 Apache-2.0、默认无 GPL 依赖”与官方材料不符。具体声线资源另需查其许可；未经构建和质量验证的 36–48 MB 不能作为已满足条件的结果。 |
| Kokoro-82M | [官方声线说明](https://huggingface.co/hexgrad/Kokoro-82M/blob/main/VOICES.md)列出法语仅一女声，并提醒极短输入可能较弱；英语声线混合作法不是官方法语男声的质量保证。历史用户反馈该女声部分单词偏快、不够清晰，见[比较记录](comparison.md)。它不自然满足本项目已确认的法语男女声。 |
| Kitten Nano | Research 1 的 Nano 0.8 与 Research 2 的 v0.1 不是同一模型版本，不能互换其体积或多语结论；单独使用的法语支持未被这两份材料充分证明。与另一个法语模型组合还需单独核对两模型、前端和依赖的实际分发许可与质量。 |

## Research 2 中需要修正的推断

1. “权重许可证完全取决于训练数据”与“CC 数据集配 MIT 运行时即可直接断定权重无限制闭源分发”证据不足。必须区分数据、基础权重、最终权重、转换、运行时与前端，核对每条实际授权链；不能把开源标签当作对所有资产的重新授权。[Piper 模型卡示例](https://huggingface.co/rhasspy/piper-voices/blob/main/fr/fr_FR/mls/medium/MODEL_CARD)显示 CC 标注的层级是 Dataset。
2. “词典现有权威 IPA、几十行映射即可替代 G2P”未经项目证明。当前项目产品文档只确认有限的离线词典能力，并未保证所有英法单词、变位及例句都有可供 TTS 使用的 IPA；模型音素表、重音、标点、连读、未知词和语境均需验证。[架构](../../architecture.md)也明确合成与播放尚未接入。`gruut` [官方说明](https://github.com/rhasspy/gruut)包含词典和猜测发音的模型，代码 MIT 不自动回答各语言词表、模型及完整依赖的分发条件。
3. 首次查询法语时静默下载语言包与[默认离线首装约束](../../product.md)冲突；不能将它作为现有产品方案的包体优化。
4. 给动态 INT8 推算固定压缩比、短词正确率、毫秒延迟或自定义 IPA 实现难度，在没有同版本实际测量与听感测试时均属于假设，不能用作选型验收依据。量化质量可能依组件而异，[askurios8 作者](https://huggingface.co/askurios8/supertonic-3-int8)自己也记录过不同声码器量化策略的输出差异；这不是对当前候选的比较测试。

## 后续决策条件

- 保留 Supertonic 3 为当前**候选**，先把模型许可、合成内容告知、运行时及第三方转换来源链落实，再测量 Windows 首装所需完整压缩包和安装占用；按既定门槛逐平台验证离线使用。仅已有模型压缩量不能推断最终首装量。
- 若完整包体或条款成为实际障碍，再对固定版本的 Supertonic 2、较小的 Supertonic 3 转换版进行同条件评估：法语／英语、男女声、短词和例句、`y` 字母名、以原速生成并在播放时 0.6 倍变速、性能、依赖与许可。任何新资源都需要独立听感验收和版本哈希。
- 用户复听后发现 Opus 24 kbit/s 在同一句中存在音量波动，决定新生成语音默认使用 48 kbit/s；这是生成**后音频**的格式选择，尚未接入产品。不能把样本 WAV 压缩比例当成模型权重或完整安装包可缩小同样比例；现有测试不证明其他平台播放兼容性。

未执行模型下载、模型运行、音频生成、替换或产品集成；本研究不声明四平台兼容、最终发行许可或最终包体合格。
