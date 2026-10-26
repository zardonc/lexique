# Piper 法语双声线的随包分发调查

后续[准入补查](replacement-admission.md)已选用从零训练的 MLS medium 双说话人，并补充 Tom 作者明确的权重及人格许可。本文保留最初 SIWIS／Tom 调查的历史判断；当前选型以补查为准。

调查日期：2026-10-26。对象是已试听的 `piper-tts 1.8.0`、SIWIS medium、Tom medium，
不是对所有 Piper 声线的统一授权判断。本次只读调查及本文档，不改变产品许可、打包或准入状态。

## 结论与当前准入

**可以设计合规分发方案，但现有证据尚不足以批准两套声线同时进入 Lexique 默认包。**
主要阻塞是女声 SIWIS 的 Lessac 基础权重授权链；男声 Tom 已追溯到作者 AGPL 仓库中的同一权重，
仍须落实其首选修改形式、内容权利与来源交付。GPL／AGPL 不等于禁止商业分发，
也不能用项目根目录的 MIT 许可覆盖第三方组件。[GPL 条款 §4–6](https://raw.githubusercontent.com/OHF-Voice/piper1-gpl/v1.8.0/COPYING)、
[Tom 上游 AGPL 条款](https://git.bksp.space/Tjiho/French-tts-model-piper/src/commit/1b6757eb2aedec36a80704a0805f2262dabb7a40/LICENSE)。

| 部分 | 已确认事实 | 项目处理 |
| --- | --- | --- |
| Piper 1.8.0 引擎 | 包声明 `GPL-3.0-or-later`；不是旧 Piper 的 MIT | 可按 GPL 义务设计交付，不把整个包标为仅 MIT |
| SIWIS medium 女声 | SIWIS 数据 CC BY 4.0，但该模型由 Lessac medium 微调；Lessac 数据页是限制商业用途的研究协议 | **保持 blocked**，需要明确覆盖该权重及基础模型的额外授权证据，或更换来源清楚的女声 |
| Tom medium 男声 | 已测试文件与作者 `model 3/next.onnx` 字节哈希相同；作者模型仓库 AGPLv3 | 有较清楚的分发路径，完成下述模型来源／源形式核对与交付材料后再准入 |
| Lexique 自有代码 | 当前产品已选择 MIT；设置偏好不等于已经链接 Piper | 保留自有源码 MIT；最终组合的许可证边界另按实际集成方式审查 |

上表依据分别见下文固定来源。这里的 `blocked` 是本项目证据门槛，
不是宣判模型违法，也不表示训练数据协议必然自动成为所有模型权重的许可证。

## 固定文件与证据方法

两模型与配置使用 HF 快照 `5041a7a49ae5b3a0f30e6837769eb03fdbb231bd`，
完整四文件 SHA-256 见[本地清单](piper-models.json)。本次通过只读 HTTP 读取了该快照
[根 README](https://huggingface.co/rhasspy/piper-voices/raw/5041a7a49ae5b3a0f30e6837769eb03fdbb231bd/README.md)、
[SIWIS 卡](https://huggingface.co/rhasspy/piper-voices/raw/5041a7a49ae5b3a0f30e6837769eb03fdbb231bd/fr/fr_FR/siwis/medium/MODEL_CARD)、
[Tom 卡](https://huggingface.co/rhasspy/piper-voices/raw/5041a7a49ae5b3a0f30e6837769eb03fdbb231bd/fr/fr_FR/tom/medium/MODEL_CARD)、
[Lessac 卡](https://huggingface.co/rhasspy/piper-voices/raw/5041a7a49ae5b3a0f30e6837769eb03fdbb231bd/en/en_US/lessac/medium/MODEL_CARD)。
根 README 只有 `license: mit` 元数据，未逐一说明每套权重／配置的许可和权利链；
同快照根 `LICENSE` 请求返回 404。这不能证明不存在其他许可，但不足以消除模型卡的差异。

Tom 的作者仓库 `main` 本次解析为提交 `1b6757eb2aedec36a80704a0805f2262dabb7a40`。
下载三个作者 ONNX 后仅在内存中计算 SHA-256，未重新合成，也未写入产品资源：

| 作者文件 | 字节 | SHA-256 |
| --- | ---: | --- |
| `model 1/tom1.onnx` | 63,511,038 | `09776bd15cdb383f76f83f285059518ec8772c32dddd8e2548027941d0d61237` |
| `model 2/tom2.onnx` | 63,511,038 | `281e872f8d091deb945526f9b1dcd52f7230cf3df25f3a2d8a3e2a2978d1ca0b` |
| `model 3/next.onnx` | 63,511,038 | `bf65074ccdeeeeaa832e75edb1c0a513c01c9a972bdf085ff8a6e71ea234fd41` |

最后一项与 HF `fr_FR-tom-medium.onnx` 完全匹配。可复验的作者
[固定权重地址](https://git.bksp.space/Tjiho/French-tts-model-piper/raw/commit/1b6757eb2aedec36a80704a0805f2262dabb7a40/model%203/next.onnx)
与同提交 [LICENSE](https://git.bksp.space/Tjiho/French-tts-model-piper/src/commit/1b6757eb2aedec36a80704a0805f2262dabb7a40/LICENSE)
把目标文件对应到实际含模型的 AGPL 仓库，比只依赖 HF 总标签更可靠。
本次未对作者配置与 HF 配置做逐字节对应；两者须作为各自版本文件保留，不能假定相同。

## 女声：SIWIS 的开放数据不等于完整权重链已清楚

SIWIS 固定模型卡明确列出 CC BY 4.0 数据，并写明由美国英语 Lessac medium 微调。
SIWIS 数据发布者提供的[许可附件](https://datashare.ed.ac.uk/server/api/core/bitstreams/d7204860-2dc9-48e6-bb9a-604a3a49d9ce/content)
确实是 CC BY 4.0，数据作者为 Junichi Yamagishi、Pierre-Edouard Honnet、Philip Garner、Alexandros Lazaridis，
题名与 DOI 见[爱丁堡大学数据页](https://datashare.ed.ac.uk/handle/10283/2353)。

**新查明的具体风险**：Lessac 固定模型卡链接的
[Blizzard 2013 原始研究协议](https://www.cstr.ed.ac.uk/projects/blizzard/2013/lessac_blizzard2013/license.html)
§1.1 排除商业语音产品开发等用途，§3.2 是不可转让、不可再许可的研究用途许可，
§7 限制不符合协议的转交。该页不是 CC BY 数据许可。

因此，不能从“SIWIS 法语数据 CC BY 4.0”推导“这份微调权重可不受限制地随商业兼容默认包分发”。
协议是否约束某个下游模型、训练者是否另有授权、最终权重是否另获有效许可，
本次公开材料没有充分回答。**需要的是具体权重与基础模型的权利证据，不是再补一条署名。**
不能通过仅删除 Lessac 署名、重新导出 ONNX、量化或换推理引擎消除这个证据缺口。

后续有两条可行路线：取得权利人／模型发布者可核验的明确授权，覆盖目标权重、微调基础及再分发；
或改用／重新训练来源清楚且没有该受限基础权重的法语女声。本次未联系任何作者，
也未接受研究协议、注册、下载受限训练原始数据或替用户作任何法律承诺。

如果最终权重有效采用 CC BY 4.0，交付应保留已给出的作者、版权、许可和免责信息、来源链接，
标记本次及先前修改，并给出许可全文或链接；不得暗示作者背书。
这只是该许可适用时的义务，不是对当前权重许可的补授予。
[CC BY 4.0 §2–3](https://creativecommons.org/licenses/by/4.0/legalcode.en)。

## 男声：Tom 已定位原模型，但模型许可与数据许可必须分开

HF 固定卡在 Dataset 项写 `AGPLv3` 并指向 Tjiho 模型仓库。作者仓库实际含三套 ONNX；
[README](https://git.bksp.space/Tjiho/French-tts-model-piper/src/commit/1b6757eb2aedec36a80704a0805f2262dabb7a40/README.md)
说明第三套继续训练第二套，第二套仅使用 Baudelaire 数据；同提交的许可为 AGPLv3，
其末尾含 Tjiho 2023 的版权示例和“或以后版本”措辞。该示例标题为 `frozone`，
不是针对 `next.onnx` 的单独文件头；分发时应保留原文，不自行改造成权利人新声明。

**模型与数据许可并不相同**：作者关联的
[Baudelaire 数据仓库](https://git.bksp.space/Tjiho/baudelaire-sentences)和
[newspaper 数据仓库](https://git.bksp.space/Tjiho/newspaper-sentences)
均说是作者本人录音，但两者实际 LICENSE 是 ODbL，而非 AGPL：
[Baudelaire LICENSE](https://git.bksp.space/Tjiho/baudelaire-sentences/src/branch/master/LICENSE)、
[newspaper LICENSE](https://git.bksp.space/Tjiho/newspaper-sentences/src/branch/main/LICENSE)。
所以不能把 HF Dataset 行直接当成音频、文字和模型的一揽子授权。

ODbL §2.4 不覆盖单条内容的独立版权等全部权利；其 §4 还区分公开使用数据库、衍生数据库与 Produced Work 的义务。
本次没有找到单独覆盖全部录音／文本的统一内容许可说明，也没有将模型或生成语音武断归类成某一种数据库衍生物。
需要补明这些权利及适用范围，不能仅靠“训练者自己读的”推导完整内容许可。
[ODbL 1.0 原文](https://opendatacommons.org/licenses/odbl/1-0/)。

AGPL 模型的来源义务还取决于其作者首选的修改形式。AGPL §1 将源形式定义为首选修改形式；
若该 ONNX 是非源形式，则仅交付同一 ONNX 和网页链接不当然充分。
应确认作者用于修改该版本的材料，例如可继续训练的 checkpoint、配置与必要脚本是否是该模型的对应源，
并按适用范围提供；**不是在此断言所有训练数据或所有历史 checkpoint 一律必须交付**。
公开仓库只有 ONNX／配置和简短训练说明，本次未证明其已经满足该模型的全部对应源要求。
[上游 AGPL §1、§4–6](https://git.bksp.space/Tjiho/French-tts-model-piper/src/commit/1b6757eb2aedec36a80704a0805f2262dabb7a40/LICENSE)。

网络源码提供的额外条件见 AGPL §13，针对修改版本支持用户远程交互等情形；
它不是“用户在离线电脑上播放一句话就自动公开所有学习数据”的规则。
不能据此推导用户输入、笔记或每个输出音频都自动被改为 AGPL。
[同一 AGPL §2、§13](https://git.bksp.space/Tjiho/French-tts-model-piper/src/commit/1b6757eb2aedec36a80704a0805f2262dabb7a40/LICENSE)。

## 引擎：GPL 有可落实的分发路径

Piper `v1.8.0` 的 [setup.py](https://raw.githubusercontent.com/OHF-Voice/piper1-gpl/v1.8.0/setup.py)
声明 `GPL-3.0-or-later`，并打包 GPL `COPYING` 与 g2pW 的 Apache-2.0 许可。
[CMakeLists](https://raw.githubusercontent.com/OHF-Voice/piper1-gpl/v1.8.0/CMakeLists.txt)
固定 eSpeak NG 提交 `724808c`，以静态方式构建到桥接模块；eSpeak NG 本身声明 GPLv3 或以后，
另列 Windows getopt 的 BSD-2-Clause 部分。
[eSpeak 固定 README](https://raw.githubusercontent.com/espeak-ng/espeak-ng/724808c/README.md)。

本机安装包 `piper_tts-1.8.0.dist-info/METADATA` 与上述声明一致；完整 wheel 还包含不用于法语的资源。
不调用某文件不等于分发该文件时可以删除其许可。附带的 ONNX Runtime、NumPy 等库仍须逐项保留各自声明；
本轮仅核对了安装元数据，尚未完成目标四平台二进制的完整 SBOM／随包声明审计。
本机比较虚拟环境不是产品发行包。

建议的工程方向是可独立运行、可替换的 Piper 工具，Lexique 仅通过普通文本／文件接口调用，
并将引擎、模型、声明分别放置。**这是便于清楚记录许可边界的设计建议，不是“子进程必定规避 GPL”的结论。**
判断依据应落在实际组合关系：GPL §1 将紧密数据通信、控制流及专门依赖的子程序列入对应源讨论，
§5 则区分形成更大程序的组合与独立作品聚合；这里不能仅凭进程边界得出豁免结论。
[GPL §1、§5](https://raw.githubusercontent.com/OHF-Voice/piper1-gpl/v1.8.0/COPYING)。
GNU GPL FAQ 页面本轮抓取超时，未将其作为已核实的解释来源。

若最终构成受 GPL 约束的组合程序，不能只给一个 MIT 声明；Lexique 自有文件的 MIT 授权可保留，
但组合物的分发仍需满足适用 copyleft 条件。若是独立作品聚合，GPL §5 的聚合条款不自动覆盖其他独立部分。
实际接口尚未实现，当前不替未存在的集成作最终许可定性。
[GPL §5](https://raw.githubusercontent.com/OHF-Voice/piper1-gpl/v1.8.0/COPYING)。

对最终 GPL 引擎的可执行分发，应保留版权、许可和免责原文，标明补丁／日期，提供匹配二进制的
Piper、嵌入 eSpeak 及适用组件对应源、构建／安装脚本与版本清单。
在线发布可采用 GPL §6(d)：在二进制下载处提供等价、无额外费用的对应源获取及清楚说明；
不要求源码一定塞入同一 ZIP，但分发者仍须保证来源可用，不能只有一个模糊的项目首页。
这里建议每次发布同时发布固定源码归档和重建说明，避免依赖上游链接永远存在。
[GPL §1、§4–6](https://raw.githubusercontent.com/OHF-Voice/piper1-gpl/v1.8.0/COPYING)。

## 可执行的发行门槛

在不联系作者、不改变现有源码许可的前提下，现在可以完成：

1. 保留当前固定文件清单，保存模型卡、协议原文、版权、上游提交和变更记录；把 Tom 对应到作者第三套模型，不能混用三套配置。
2. 准备独立的 `licenses/`、组件清单、`THIRD_PARTY_NOTICES` 和固定 `sources/` 发布材料；对应源包含实际构建补丁及脚本，验收可重建性。
3. 设计引擎与模型的明确边界，再依据实际接口审查聚合／组合范围；不要先链接进核心再假定 MIT 不受影响。
4. 继续本地技术验证，但保持“试听／实验”和“默认分发准入”分开；不把用户首次下载作为规避许可或满足首次离线要求的替代。

两声线进入默认包前仍须闭合：

- **SIWIS**：取得足以说明该精确权重及 Lessac 基础允许目标商业兼容再分发的证据，或者替换女声；CC BY 署名不能单独解决。
- **Tom**：记录模型／配置的适用授权、首选修改形式和对应源交付范围，补清数据内容权利；AGPL 模型与 ODbL 数据分别处理。
- **整包**：按实际平台文件做依赖与许可清单，公开匹配的来源包；同时完成既有包体、首次离线、人工听感与四平台门槛。法律证据通过不代替技术验收。

当前 `scripts/package_portable.py` 只打包已有 Lexique 与词典许可，不含 TTS；
故不能把本文当作已完成的 TTS 发布包或所有声明已随包的证明。
本次未修改现有交付、联系权利人、做司法辖区法律意见或重跑运行测试；以上“可执行方向”须在实际发行前按目标工件落实。
