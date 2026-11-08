# Sherpa / Piper 法语组合：来源与许可

2026-11-07。范围是 #5 的本地 Windows x64 验证组合。

## Tom 模型与声线

当前验证模型为 Piper `fr_FR-tom-medium`：法语、单说话人、44,100 Hz、推理 ID `0`。HF 快照固定为 `5041a7a49ae5b3a0f30e6837769eb03fdbb231bd`；[模型卡](TOM-MODEL_CARD)、原始权重与配置的来源和哈希见[清单](../resources.json)。

作者在 GitHub 固定提交 `a4601a0219fa8f8cf3f330bdd818dbb37ae287ce` 的[声明](TOM-README.md)明确将权重以 [AGPL-3.0](LICENSE-TOM-AGPL-3.0.txt) 发布，并另行授权声线使用。既有[来源核验](../../issue-14/piper-distribution-research.md)将这份 HF 权重对应到作者固定仓库的 `model 3/next.onnx`：63,511,038 字节，SHA-256 `bf65074ccdeeeeaa832e75edb1c0a513c01c9a972bdf085ff8a6e71ea234fd41`；本次缓存文件与该身份相符。

声线条件要求：不得把合成声音表述为真实或已识别人物，不在服务界面或向最终用户披露作者身份；文件版权、许可及来源声明仍须保留。使用权重或生成音频训练的新声线模型须 AGPL-3.0。试听页仅标示合成语音，不将其归属于真实人物。

本次添加 Sherpa 的九项 ONNX 元数据，序列化图与原文件完全相同；配置不改，不量化或重训。`tokens.txt` 从原 `phoneme_id_map` 导出，保留音素 ID。转换代码是 [prepare.py](../prepare.py)，原文件的固定获取地址和身份在清单中。修改版分发仍须落实 AGPL 的声明及对应源形式义务；本地验证未完成公共分发核验。作者 README 对旧 Piper 的 MIT 说明不覆盖这里使用的 piper-tts 1.8.0 GPL 前端。

## 保留的 MLS 否决证据

原 Piper `fr_FR-mls-medium` 的 [MODEL_CARD](MODEL_CARD)、[CC BY 4.0](CC-BY-4.0.txt)、[Piper 声线政策](PIPER-VOICES.md)及署名保留。发布者为 rhasspy/piper-voices，署名 Michael Hansen 与 Piper contributors；MLS 数据来自 LibriVox，由 Facebook AI 发布，见 [OpenSLR SLR94](https://openslr.org/94/)。

原模型和女声 source `3698` → 推理 ID `1` 的许可解释沿用 [#14 准入补查](../../issue-14/replacement-admission.md)。其原始文件身份见[保留清单](../samples/resources.json)，人工质量否决见[原记录](../samples/quality.json)。这些材料不代表 MLS 的质量准入。

## SIWIS 的独立核验

- [SIWIS medium 模型卡](siwis-MODEL_CARD.txt)明确从 U.S. English Lessac medium 微调。
- SIWIS 数据由 Junichi Yamagishi、Pierre-Edouard Honnet、Philip Garner、Alexandros Lazaridis 发布，[Edinburgh 数据页](https://doi.org/10.7488/ds/1705)的[实际终端用户许可](siwis-dataset-licence.txt)为 CC BY 4.0。
- 外层 Hugging Face README 的 `license: mit` 只是一项仓库标签；[固定 README](https://huggingface.co/rhasspy/piper-voices/raw/5041a7a49ae5b3a0f30e6837769eb03fdbb231bd/README.md)不建立训练数据或派生权重的完整权利链。
- [Lessac medium 模型卡](lessac-MODEL_CARD.txt)链接 Blizzard 2013 数据许可。[原发布条款](https://www.cstr.ed.ac.uk/projects/blizzard/2013/lessac_blizzard2013/license.html)要求实名、人工授予许可，限定 Research Purposes；该定义排除商业语音合成产品开发与商业化。

这些数据条款不能直接等同于派生模型条款，但尚无材料闭合 Lessac 基础权重到 SIWIS 的授权链。SIWIS 未达到本票的资源准入门槛；法语数据的 CC BY 4.0 和外层 MIT 标签不填补这项缺口。本次实际运行 Tom 是同一 Piper 家族的独立验证。

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
