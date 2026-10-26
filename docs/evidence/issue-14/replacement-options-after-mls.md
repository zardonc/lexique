# MLS 试听失败后的法语 TTS 替代调查

日期：2026-10-26。用户反馈全部 MLS 样本混乱、难懂，**MLS 不再作为音质合格候选**。原准入材料仅保留历史证据，不构成继续选用它的理由。本文主体记录执行前的替代研究，调查代理当时未下载大权重。随后已完成资源下载及小批禁网生成，详见[执行记录](supertonic-trial.md)与[试听页](supertonic-samples/listen.html)；尚未宣称人工试听通过。

## 优先候选：Supertonic 3 的维护者 int8 包

推荐下一轮试听 **Supertonic 3 / F1 女声、M1 男声 × 正常／慢速**，通过 sherpa-onnx 维护者发布的量化包执行。该组合原生支持法语与英语，使用固定预置声线，不需要克隆任何人、购买声线或调用云服务。官方模型卡与维护者[集成文档](https://k2-fsa.github.io/sherpa/onnx/tts/supertonic.html)分别证明模型语言覆盖和离线推理接口；不能代替本项目试听。

| 项目 | 固定来源／事实 |
| --- | --- |
| 原模型 | [Supertone/supertonic-3](https://huggingface.co/Supertone/supertonic-3/tree/3cadd1ee6394adea1bd021217a0e650ede09a323)，快照 `3cadd1ee6394adea1bd021217a0e650ede09a323` |
| int8 发布包 | [sherpa-onnx-supertonic-3-tts-int8-2026-05-11.tar.bz2](https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/sherpa-onnx-supertonic-3-tts-int8-2026-05-11.tar.bz2)，128,774,318 字节（约 122.81 MiB） |
| 发布包 SHA-256 | GitHub asset digest：`82fa96f91c4ef8abaae3a14a3f4153facf88bed821d1f7331cec2700f432c427`；下载后仍须计算并核对 |
| 维护者文件镜像 | [csukuangfj2 固定快照](https://huggingface.co/csukuangfj2/sherpa-onnx-supertonic-3-tts-int8-2026-05-11/tree/cca5a0e6c96e1d2c720986bf7e75fcc81dee3ae4)，`cca5a0e6c96e1d2c720986bf7e75fcc81dee3ae4` |
| 模型必要文件总计 | 镜像 API 文件清单相加：145,295,768 字节（约 138.56 MiB）；不含运行库和许可材料 |
| 同版官方原始 ONNX | 四模型、两配置、F1／M1 两 JSON 合计 398,944,996 字节；本调查不下载此大包 |
| 速度 | 公开 API 支持 `speed`，较大值较快；建议先试听 `1.0` 与 `0.8`，最终预设以用户反馈固定 |

镜像不是随机声线仓库：sherpa 官方[发布工作流](https://github.com/k2-fsa/sherpa-onnx/blob/af2d8d81dbfa61c4cea4c568da78f3982edca005/.github/workflows/export-supertonic.yaml)直接发布到该 GitHub release 和该 HF 仓库。固定版[量化脚本](https://github.com/k2-fsa/sherpa-onnx/blob/af2d8d81dbfa61c4cea4c568da78f3982edca005/scripts/supertonic/run.sh)从 Supertone 官方 `supertonic-3` 获取模型，[说明](https://github.com/k2-fsa/sherpa-onnx/blob/af2d8d81dbfa61c4cea4c568da78f3982edca005/scripts/supertonic/README.md)描述动态／静态 int8 转换。工作流下载使用可变 main，**不能宣称整套量化参数由本次固定原模型快照精确重现**；现有发布包及镜像哈希固定的是实际交付对象。

### 说话人 ID：已作字节对应

Supertone 官方 SDK 的[固定声线说明](https://github.com/supertone-oss-archive/supertonic-py/blob/df0f9686dac7fbbde391b759e2ee5286a3737622/docs/voices.md)将 F1 描述为平稳偏低的女声、M1 描述为清晰活跃的男声。维护者[转换脚本](https://github.com/k2-fsa/sherpa-onnx/blob/af2d8d81dbfa61c4cea4c568da78f3982edca005/scripts/supertonic/generate_voices_bin.py)按 JSON 文件名排序并串接声线；本次进一步读取小文件在内存中实测，避免只凭排序推断：

- 固定镜像 `voice.bin` 为 517,168 字节，头部尺寸为 `(10,50,256,10,8,16)`。
- 原模型快照 [F1.json](https://huggingface.co/Supertone/supertonic-3/raw/3cadd1ee6394adea1bd021217a0e650ede09a323/voice_styles/F1.json) SHA-256 为 `bbdec6ee00231c2c742ad05483df5334cab3b52fda3ba38e6a07059c4563dbc2`，其两个风格张量转换 float32 后，完整匹配 `voice.bin` 的 **sid 0**。
- 原模型快照 [M1.json](https://huggingface.co/Supertone/supertonic-3/raw/3cadd1ee6394adea1bd021217a0e650ede09a323/voice_styles/M1.json) SHA-256 为 `e35604687f5d23694b8e91593a93eec0e4eca6c0b02bb8ed69139ab2ea6b0a5b`，同法完整匹配 **sid 5**。

因此可使用女声 `0`、男声 `5`，不要沿用 MLS 的 `0` 男／`1` 女映射。

### 运行文件与哈希

以下来自维护者固定 HF API 的 LFS SHA-256；`tts.json` 为 Git blob，另保留其 Git SHA-1。文件链接均可由固定镜像快照取得。

| 文件 | 字节 | 摘要 |
| --- | ---: | --- |
| `duration_predictor.int8.onnx` | 3,700,147 | SHA256 `c3eb91414d5ff8a7a239b7fe9e34e7e2bf8a8140d8375ffb14718b1c639325db` |
| `text_encoder.int8.onnx` | 36,416,150 | SHA256 `c7befd5ea8c3119769e8a6c1486c4edc6a3bc8365c67621c881bbb774b9902ff` |
| `vector_estimator.int8.onnx` | 78,400,833 | SHA256 `20cd86fa5c6effedfda0e7cffe5b0569ca401c440a0c3a1d72bf39286c0db3fd` |
| `vocoder.int8.onnx` | 25,991,073 | SHA256 `e923d60f53f95eb1ce235f1dc33ec56d9c057823c96fa6f8acf98f32b0da6152` |
| `tts.json` | 8,253 | Git SHA1 `28575b0793c92607a0f0a292550df9bd17709a79` |
| `unicode_indexer.bin` | 262,144 | SHA256 `8402ca48e5189a8950138580b0fff64db6f072f24ac07cd54ba8b2fbb9883b30` |
| `voice.bin` | 517,168 | SHA256 `67d5209b0ee8ce6c74105ffbe12fe6a7628aea3b4ba2fcb308a4a67938a93ce8` |

注意前两份文件与本次原模型快照对应 ONNX 的 LFS 哈希相同；不能因为文件名带 int8 就声称四模块都已量化。包的实际大小及字节哈希才是计量依据。

官方 [Python 示例](https://github.com/k2-fsa/sherpa-onnx/blob/af2d8d81dbfa61c4cea4c568da78f3982edca005/python-api-examples/supertonic-tts.py)通过 `OfflineTtsSupertonicModelConfig` 提供上述七文件路径，在 `GenerationConfig` 中设置 `sid`、`num_steps=8`、`speed` 及语言参数，调用 `tts.generate(text, gen_config)`。**本机 2.0.1 绑定的 extra getter 返回副本，必须完整赋值 `gen_config.extra = {"lang": "fr"}` 并读回校验；官方示例的下标修改未生效，已在执行回归中复现。** 样本需验证短词、例句和 `y`；该模型的字符前端不能直接沿用 Piper 专用音素输入。

## 许可：可商业分发，但不是模型 MIT

原模型[固定 README](https://huggingface.co/Supertone/supertonic-3/raw/3cadd1ee6394adea1bd021217a0e650ede09a323/README.md)明确区分：示例代码 MIT，配套模型为 **BigScience Open RAIL-M**。同快照[原始 LICENSE](https://huggingface.co/Supertone/supertonic-3/raw/3cadd1ee6394adea1bd021217a0e650ede09a323/LICENSE)为 15,007 字节，SHA-256 `0d944a9110fed9a9602d60e0423a272903e7bd21ab060490774efc77c2275e9f`。预置声线 JSON 在同一官方模型仓库配套发布，未发现单独的付费／非商业许可；不要将付费自定义 Voice Builder 的条件套到这些固定预置文件。

该许可 §2–4 授予复制、改编和再分发权，含销售权，没有研究专用／非商业门槛；量化与声线转二进制属于须说明的修改。§4–6 同时要求随模型提供许可全文、保留通知、标明修改，并向接收者传递具有约束力的用途限制。Attachment A 包含违法、伤害、冒充等限制及机器生成内容的明确标识要求。它不要求将 Lexique 自有代码改为同一许可，但**不适合只有 MIT 声明、没有模型使用条款的交付**。界面可用一句“由本地语音模型合成”说明语音来源，发行材料须纳入具体模型限制；最终产品文案及条款仍待落实。

**实际发现的许可包装缺口**：维护者镜像 `LICENSE` 为 1,070 字节的 Supertone **代码 MIT**，SHA-256 `0dfe0d0ba84416fe3879d9a34f4909d8d0137c78d1e95834177b0414ac096fa2`；上述官方工作流从代码仓库而非模型仓库取 LICENSE，解释了该文件来源。它不能重新授权上游模型。Lexique 若选择量化包，应保留已有 MIT 通知，额外补入原模型 OpenRAIL-M 全文、Supertone 版权／来源、维护者量化来源与修改说明、可执行用途限制，然后核对所有运行依赖许可。不能因包装遗漏直接宣布模型禁止分发，也不能忽略上游许可。

Supertone 代码仓库当前已迁移为 `supertone-oss-archive` 并归档；上述固定源码仍公开可读。这是后续维护成本，不能假定会持续收到上游修复。官方 HF 模型和演示仍由原发布机构标识，未采用随机人物克隆权重。

## 对比与回退

| 路线 | 本轮判断 |
| --- | --- |
| Supertonic 3 int8 | 首先试听；约 128.8 MB 压缩模型包，加运行库后重新计量；法语／英语、固定男女声、速度可选 |
| Supertonic 2 原始模型 | 官方固定 `75e6727618a02f323c720cba9478152d4bc16ca4` 的四 ONNX、两配置及 F1/M1 合计 263,940,729 字节；相同模型许可。先不下载，避免与 3 重复投入 |
| Supertonic 2 int8 | [维护者旧包](https://github.com/k2-fsa/sherpa-onnx/releases/download/tts-models/sherpa-onnx-supertonic-tts-int8-2026-03-06.tar.bz2)为 84,692,981 字节，GitHub digest `8c74359f63edd5045d47747f65331f0f6dbcbc91d7e898dd756d631295fe3259`；本轮未核对内部是否确实为 2、法语及声线映射，故不先选 |
| 当前官方 Piper 法语 | 本次 API 仍为 `5041a7a49ae5b3a0f30e6837769eb03fdbb231bd`，仅 Gilles low、MLS medium、MLS1840 low、SIWIS low/medium、Tom medium、UPMC medium，未发现新增可替代的官方 scratch 女声 |
| Tom 男声＋其他女声 | Tom 可保留用户已接受的音质及已查明的 AGPL／独立声线许可路径；仍缺另一份质量合格、无受限基础链的 Piper 女声，当前不如同一 Supertonic 模型统一试听 |

官方比较试听入口：[Supertonic 3 demo](https://huggingface.co/spaces/Supertone/supertonic-3)、[官方音频比较页](https://supertonic3.github.io/)、[Supertonic 2 demo](https://huggingface.co/spaces/Supertone/supertonic-2)。其中自定义声线示例与固定 F1/M1 不完全相同；不能用官网英文／日文示例替代用户对本地法语四组合的验收。

## 本次证据范围

已核验：官方模型和代码许可、固定版本与 API 文件体积、维护者量化／发布源码、量化包 asset digest、预置声线性别说明，以及小文件张量逐字节映射。调查阶段只下载约 1.1 MB 的声线元数据到内存。随后执行阶段已下载并校验压缩模型包，完成 26 个 WAV 生成／播放及本地识别对照，见[执行记录](supertonic-trial.md)。四组合音质、最终完整包体和正式产品条款仍待验收。
