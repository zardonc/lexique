# 法语双声线替代模型的准入补查

调查日期：2026-10-26。范围：解决 #14 默认资源不能带非商业限制的问题，保留用户确认的“法语女声／男声 × 正常／慢速”。本文取代[前次调查](piper-distribution-research.md)中的候选选择结论；旧调查保留为历史来源及引擎交付参考。

## 人工验收结果：不采用 MLS

2026-10-26 用户试听反馈：生成样本不是正确法语读音，全部为混乱人声且音质不佳。**MLS 的本轮音质验收失败，撤回默认候选选择，不进入产品包。** 下文许可证据与运行记录保留，不代表音质准入。WAV、时长、禁网及播放调用检查没有识别这项声学失败；不得再把这些技术检查写成发音正确的证据。

## 原选型结论（已撤回）

**曾选择 Piper `fr_FR-mls-medium` 的两名说话人作为替代验证组合，现因试听失败撤回：男声 `speaker_id=0`，女声 `speaker_id=1`。** 同一份 125 说话人模型覆盖两种声线，速度由推理参数控制，无需四份权重。该模型的许可与来源证据达到继续下载、试听及准备随包分发材料的工程准入门槛；最终包仍须通过实际音质、禁网运行、体积和许可材料验收，不能据本文宣布 #14 或正式发行完成。

模型固定为 HF 快照 `5041a7a49ae5b3a0f30e6837769eb03fdbb231bd`：

- [模型卡](https://huggingface.co/rhasspy/piper-voices/raw/5041a7a49ae5b3a0f30e6837769eb03fdbb231bd/fr/fr_FR/mls/medium/MODEL_CARD)：法语、125 名说话人、22,050 Hz、medium，声明从零训练。
- [ONNX 权重](https://huggingface.co/rhasspy/piper-voices/resolve/5041a7a49ae5b3a0f30e6837769eb03fdbb231bd/fr/fr_FR/mls/medium/fr_FR-mls-medium.onnx)与[配套配置](https://huggingface.co/rhasspy/piper-voices/raw/5041a7a49ae5b3a0f30e6837769eb03fdbb231bd/fr/fr_FR/mls/medium/fr_FR-mls-medium.onnx.json)必须成对固定、记录下载文件 SHA-256。配置将原始 MLS 说话人 `1840` 映射到 `0`，`3698` 映射到 `1`。

## 准入依据与推论边界

三个来源共同支持本次判断：

1. Piper 官方[固定版声线说明](https://github.com/OHF-Voice/piper1-gpl/blob/74ac24e7e8e2e57a5e1fada5505c74e3d9c1843a/docs/VOICES.md)要求查阅逐声线 `MODEL_CARD` 的许可信息，并声明不对模型附加限制。因此“面向个人及研究”不是该页另加的模型非商业条款。
2. 上述特定 MLS 模型卡列 CC BY 4.0，并明确从零训练，没有 SIWIS medium 的 Lessac 基础模型，也没有若干 low 模型的 Ryan 基础模型链。
3. 原数据发布页 [OpenSLR SLR94](https://openslr.org/94/)确认 MLS 来自 LibriVox 有声书，数据许可为 CC BY 4.0；[Facebook 官方数据仓库](https://huggingface.co/datasets/facebook/multilingual_librispeech/tree/2e83e61823b4c47dcbcb1980bb88601274127609/data/mls_french)提供来源元数据。

**这里的模型许可判断是结合官方逐模型许可指引、特定模型卡及训练来源作出的合理解释。** 模型卡的许可字段位于 Dataset 节，未另写一句覆盖权重的独立授权文本；不能把数据 CC BY 4.0 单独当作任意下游权重的自动授权，也不能把仓库总 MIT 标签当作所有声线的许可。本次没有发现这份从零训练模型另带非商业条件或受限基础模型的证据，因此可按该逐模型声明的 CC BY 4.0 路径准入，无需为没有具体反证的可能性无限阻塞。

[CC BY 4.0 正文 §2–3](https://creativecommons.org/licenses/by/4.0/legalcode.en)允许复制、分发与改编，未排除商业用途；随包时保留已提供的作者、版权、来源、许可及免责信息，说明修改，不暗示背书。该许可不要求交付训练源码或全部训练数据；Piper 引擎及其依赖的源码交付义务须按各自 GPL 等许可另行落实，不能与模型许可混为一谈。人格、隐私及商标权也没有因此获得全面授权，不将界面声线描述成某个真实人物。

模型卡、CC BY 4.0 原始全文、固定版 Piper 指引及逐文件来源／哈希已保存到 [mls-licenses](mls-licenses/ATTRIBUTION.md)，署名包括 Michael Hansen／Piper 与 MLS 数据作者。实际下载的权重和配置哈希见[模型清单](mls-model.json)，二者未修改；选择说话人或改变推理时长不是重新训练权重。实际采用的 GPL 引擎版本、源码与构建交付材料仍见[前次调查](piper-distribution-research.md)，本次没有宣称该引擎的完整再分发义务已经落实。

## 男女声映射的原始证据

读取的是 Facebook 官方 MLS 数据仓库固定提交 `2e83e61823b4c47dcbcb1980bb88601274127609` 的 [法语 metainfo.txt](https://huggingface.co/datasets/facebook/multilingual_librispeech/resolve/2e83e61823b4c47dcbcb1980bb88601274127609/data/mls_french/metainfo.txt)，没有下载 61 GB 法语音频集。文件为 716,290 字节，SHA-256：

`1bb34041836b9fbc695efb8beb72a1865ae7781331adb2181e34d1734ade821b`

该文件表头包含 `SPEAKER | GENDER | PARTITION | MINUTES | BOOK ID | TITLE | CHAPTER`。按说话人 ID 聚合全部记录，结果如下；结合模型配置映射确定推理 ID，不凭声音猜测性别，也不把原始 ID 直接传成推理 ID。

| 界面选项 | 原 MLS ID | 元数据 GENDER | Piper speaker_id |
| --- | ---: | --- | ---: |
| 法语男声 | 1840 | M | 0 |
| 法语女声 | 3698 | F | 1 |
| 女声备用候选 | 123 | F | 2 |
| 女声备用候选 | 1474 | F | 3 |

“正常／慢速”只是同一说话人的速度预设，不声称支持情绪／语调控制。正常可从配置默认 `length_scale=1.0` 开始，慢速须在实际试听后固定；本调查未验证这两人的可懂度、`y` 的正确读音或播放效果。

## 其他候选的处理

| 候选 | 固定卡记录 | 决定 |
| --- | --- | --- |
| [SIWIS medium](https://huggingface.co/rhasspy/piper-voices/raw/5041a7a49ae5b3a0f30e6837769eb03fdbb231bd/fr/fr_FR/siwis/medium/MODEL_CARD) | 从 Lessac medium 微调 | 延续前次基础授权链缺口，不进默认包 |
| [UPMC medium](https://huggingface.co/rhasspy/piper-voices/raw/5041a7a49ae5b3a0f30e6837769eb03fdbb231bd/fr/fr_FR/upmc/medium/MODEL_CARD) | 两名说话人、数据 CC BY-SA 4.0，但从 Lessac medium 微调 | 不能靠换成此双人模型消除同一缺口 |
| [SIWIS low](https://huggingface.co/rhasspy/piper-voices/raw/5041a7a49ae5b3a0f30e6837769eb03fdbb231bd/fr/fr_FR/siwis/low/MODEL_CARD)、[MLS1840 low](https://huggingface.co/rhasspy/piper-voices/raw/5041a7a49ae5b3a0f30e6837769eb03fdbb231bd/fr/fr_FR/mls_1840/low/MODEL_CARD)、[Gilles low](https://huggingface.co/rhasspy/piper-voices/raw/5041a7a49ae5b3a0f30e6837769eb03fdbb231bd/fr/fr_FR/gilles/low/MODEL_CARD) | 均从 Ryan low 微调；[Ryan low 卡](https://huggingface.co/rhasspy/piper-voices/raw/5041a7a49ae5b3a0f30e6837769eb03fdbb231bd/en/en_US/ryan/low/MODEL_CARD)列数据 CC BY-NC-SA 4.0 | 无覆盖该基础链的额外授权，不作为商业兼容替代；这不等于判定所有训练产物必然自动继承数据协议 |
| Tom medium | 作者 GitHub [固定 README](https://github.com/tjiho/French-tts-model-piper/blob/a4601a0219fa8f8cf3f330bdd818dbb37ae287ce/README.md)明确将权重以 AGPL-3.0 发布，并另给声线人格权许可 | 存在可分发路径，但本次统一选择 MLS 两人，减少不同声线的交付条件 |

Tom 的新授权比前次 Forgejo 仓库说明更明确，不能继续笼统写成“权重授权不明”。作者还规定：不将声音表述成真实／已识别人物；不得在服务界面或向最终用户披露作者身份，但须保留权重文件的版权及许可声明；用权重或其生成音频训练的新模型须 AGPL-3.0。这些是作者声明的独立声线许可条件。README 对旧 Piper MIT 的描述不覆盖本项目 `piper-tts 1.8.0` 的 GPL；其“调用不扩展 AGPL”解释也不能替代实际引擎／程序组合审查。前次已验证 Tom 文件与作者权重的哈希对应，本次不重复下载该权重。

## 本次核验与待完成事项

已通过只读 HTTP 读取固定模型卡、配套配置、官方 Piper 指引、Tom 作者说明、OpenSLR 页面和 MLS 元数据；逐说话人聚合性别且计算元数据 SHA-256。未联系作者、未接受受限数据协议、未改产品许可、未重训或量化模型。

本文只确认替代模型的证据与准入路径。权重下载哈希、四组合试听、`y` 的文本／音素策略、禁网合成和播放、最终压缩包计量与第三方许可材料，分别由本轮运行验收和后续交付核查记录；全部通过前保留对应门槛。
