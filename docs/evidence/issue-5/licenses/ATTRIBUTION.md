# Sherpa / Piper MLS：来源与许可

2026-11-07。范围是 #5 的本地 Windows x64 验证组合。

## 模型与数据

Piper `fr_FR-mls-medium`，发布者 rhasspy/piper-voices；署名 Michael Hansen 与 Piper contributors。训练数据 MLS 来自 LibriVox，由 Facebook AI 发布；见 [OpenSLR SLR94](https://openslr.org/94/)。固定[模型卡](MODEL_CARD)声明法语、125 个说话人、22,050 Hz、从零训练、CC BY 4.0。

模型准入按[固定 Piper 声线政策](PIPER-VOICES.md)、模型卡和 MLS 数据许可共同解释为 CC BY 4.0。原始模型、配置、模型卡的来源和哈希见[清单](../resources.json)，许可全文见 [CC BY 4.0](CC-BY-4.0.txt)。这沿用 [#14 的模型准入依据](https://github.com/zardonc/lexique/blob/9698653fb2a0eb8af5de56ba7d37265d3b01d169/docs/evidence/issue-14/replacement-admission.md)，不改变其人工音质否决。

本次只给 ONNX 添加 Sherpa 的九项元数据；序列化图与原文件逐字节相同，权重不改、不量化、不重新训练。`tokens.txt` 从原 `phoneme_id_map` 导出，保留音素 ID。声线为 MLS 原始说话人 `3698`、女声、推理 ID `1`；该对应依据[固定 MLS metainfo](https://huggingface.co/datasets/facebook/multilingual_librispeech/resolve/2e83e61823b4c47dcbcb1980bb88601274127609/data/mls_french/metainfo.txt)。变更和源文件身份见清单；不暗示作者或发布者认可本项目。

## SIWIS 的独立核验

- [SIWIS medium 模型卡](siwis-MODEL_CARD.txt)明确从 U.S. English Lessac medium 微调。
- SIWIS 数据由 Junichi Yamagishi、Pierre-Edouard Honnet、Philip Garner、Alexandros Lazaridis 发布，[Edinburgh 数据页](https://doi.org/10.7488/ds/1705)的[实际终端用户许可](siwis-dataset-licence.txt)为 CC BY 4.0。
- 外层 Hugging Face README 的 `license: mit` 只是一项仓库标签；[固定 README](https://huggingface.co/rhasspy/piper-voices/raw/5041a7a49ae5b3a0f30e6837769eb03fdbb231bd/README.md)不建立训练数据或派生权重的完整权利链。
- [Lessac medium 模型卡](lessac-MODEL_CARD.txt)链接 Blizzard 2013 数据许可。[原发布条款](https://www.cstr.ed.ac.uk/projects/blizzard/2013/lessac_blizzard2013/license.html)要求实名、人工授予许可，限定 Research Purposes；该定义排除商业语音合成产品开发与商业化。

这些数据条款不能直接等同于派生模型条款，但尚无材料闭合 Lessac 基础权重到 SIWIS 的授权链。SIWIS 未达到本票的资源准入门槛；法语数据的 CC BY 4.0 和外层 MIT 标签不填补这项缺口。本次实际运行 MLS 是同一 Piper 家族的独立验证，不将其重新选为产品默认模型。

## 引擎、前端与运行库

| 构成 | 固定身份 | 分别许可与材料 |
| --- | --- | --- |
| Sherpa-ONNX | 2.0.1；源码 `af2d8d81dbfa61c4cea4c568da78f3982edca005` | [Apache-2.0](LICENSE-SHERPA-ONNX-Apache-2.0.txt)；两个 wheel 的来源、哈希与原始许可元数据随包保留 |
| Piper 外部前端 | piper-tts 1.8.0；源码 `639388b6317fc4731e91d53da42aea68fd4166ff` | [GPL-3.0](LICENSE-PIPER-GPL-3.0.txt)；wheel 自带 g2pW 的 Apache-2.0 通知亦保留 |
| eSpeak NG | 原生 API 自报 1.52.0.1；Piper CMake 固定 `724808c5a83f9ef95fdd0db886ba7ba537ff224a` | [GPL-3.0](LICENSE-ESPEAK-NG-GPL-3.0.txt)，并保留 ucd-tools 通知；静态链接于 `piper/espeakbridge.pyd`，数据随 wheel 提供 |
| CPython | 官方 3.14.10 Windows x64 嵌入包 | 包内 PSF 许可、[完整许可文档](CPYTHON-3.14.10-LICENSES.rst)及 [LibTomMath 通知](LICENSE-LIBTOMMATH-TCL-9.0.4.0.txt) |
| NumPy | 2.5.4 CPython 3.14 Windows x64 wheel | wheel 声明的 BSD、0BSD、MIT、Zlib、CC0 和内附第三方许可全部原样保留 |
| Python ONNX Runtime | 1.31.1 CPython 3.14 Windows x64 wheel | MIT 与 wheel 附带第三方通知；这是 Piper 包导入的运行库，Sherpa 的原生运行库单独随 sherpa-onnx-core 提供 |
| 其他 Python 依赖 | FlatBuffers 25.12.19、packaging 26.3、pathvalidate 3.3.1、protobuf 7.36.2 | 每个 wheel 的分别许可、原始 METADATA 和许可证文件随包保留；具体身份见清单 |
| 准备工具 | ONNX 1.23.2、ml-dtypes 0.6.0、typing-extensions 4.16.0，加上复用的 NumPy / protobuf | Apache-2.0 / PSF / 各自 wheel 通知；只用于元数据准备，独立工具环境，不进入运行包 |
| Windows VC 运行依赖 | 官方签名的 x64 redistributable 14.44.35211.0 | Microsoft 分发条款；安装器作为离线前提材料保留，本票未安装或执行它 |

源码引用：[Sherpa 的 Piper 示例](https://github.com/k2-fsa/sherpa-onnx/blob/af2d8d81dbfa61c4cea4c568da78f3982edca005/python-api-examples/test-offline-tts-piper-phonemize.py)、[元数据准备](https://github.com/k2-fsa/sherpa-onnx/blob/af2d8d81dbfa61c4cea4c568da78f3982edca005/scripts/piper/add_meta_data.py)、[Piper 前端](https://github.com/OHF-Voice/piper1-gpl/blob/639388b6317fc4731e91d53da42aea68fd4166ff/src/piper/phonemize_espeak.py)、[eSpeak 构建固定点](https://github.com/OHF-Voice/piper1-gpl/blob/639388b6317fc4731e91d53da42aea68fd4166ff/CMakeLists.txt)。

## 证据边界

原始许可文本的身份见 [sources.json](sources.json)，运行包的 `inventory.json` 固定每个实际文件的字节数和 SHA-256。引擎 wheel 与 #14 已接纳的字节相同，继续保留其[原生依赖与构建来源边界](https://github.com/zardonc/lexique/blob/9698653fb2a0eb8af5de56ba7d37265d3b01d169/docs/evidence/issue-14/supertonic-licenses/ATTRIBUTION.md)：源码提交自报和 wheel 哈希不是签名构建证明，附带候选依赖通知不证明全部编译成分。

本地下载、生成和试听不等于完成 GPL 对应源码交付、完整二进制依赖核验、Microsoft 再分发前提或正式产品许可验收。模型、前端、原生运行库与 Python 许可分别保留；仓库根 MIT 许可适用于本项目探针源码，不覆盖这些第三方资源。
