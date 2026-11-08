# #5 Windows 法语语音：对应源码交付

2026-11-07。范围为独立 Windows x64 包：Piper 前端 1.8.0、其静态 eSpeak 桥接，以及添加 Sherpa 元数据的 `fr_FR-tom-medium`。软件源码可用固定的完整仓库归档交付；Tom 的本次修改有完整转换输入和脚本。模型作者的首选修改形式仍有一个明确的上游证据缺口。

## 软件：固定归档与构建材料

**已核实事实：** Piper 的 `setup.py` 声明 `GPL-3.0-or-later`；`CMakeLists.txt` 将 eSpeak 和 ucd-tools 静态链接到 `espeakbridge.pyd`，并复制生成的 `espeak-ng-data`。这些材料属于桥接二进制及音素数据的源集。[Piper setup.py][piper-setup] · [Piper CMake][piper-cmake]

| 材料 | 固定身份与获取地址 | 保留的构建／许可材料 |
| --- | --- | --- |
| Piper 完整源码 | `639388b6317fc4731e91d53da42aea68fd4166ff`；[完整归档][piper-archive] | `src/piper/espeakbridge.c`、Python 源码、`CMakeLists.txt`、`setup.py`、`setup.cfg`、`pyproject.toml`、`MANIFEST.in`、`script/`、`.github/workflows/wheels.yml`、`COPYING`、`licenses/` 和文件内通知 |
| eSpeak NG 完整源码 | `724808c5a83f9ef95fdd0db886ba7ba537ff224a`；[完整归档][espeak-archive]；源码版本 `1.52.0.1` | C／C++ 源码、`dictsource/`、`phsource/`、数据源、全部 CMake 构建／数据生成脚本；`COPYING`、`COPYING.UCD`、`COPYING.BSD2`、`COPYING.APACHE` 及文件内通知 |
| ucd-tools | 已内嵌在上述 eSpeak 归档的 `src/ucd-tools/`；Git tree `fed138f1950b9eaad574dff05f2d6cfbbc9a2fed` | 六个被编译的 C 源文件、头文件、数据、`CMakeLists.txt`、`COPYING`、`COPYING.UCD`、`AUTHORS`；无独立子模块下载 |
| Sonic 构建配套 | `fbf75c3d6d846bad3bb3d456cbc5d07d9fd8c104`；[完整归档][sonic-archive] | `sonic.c`、`sonic.h` 与 Apache-2.0 `LICENSE`；用于满足上游配置阶段的源码获取 |

eSpeak 的[完整固定源树][espeak-tree]没有 gitlink；ucd-tools 是普通目录。Piper 的 eSpeak pin 虽写为短 SHA `724808c`，其对应完整提交如上。[eSpeak 根 CMake][espeak-cmake]给出 `1.52.0.1`；[ucd-tools CMake][ucd-cmake]明确编译其六个 C 文件。ucd-tools 代码为 GPLv3 或以后，Unicode 数据使用原样保留的 Unicode 许可；[README][ucd-readme]及 [COPYING.UCD][unicode-license]要求保留对应版权、许可和免责通知。

Piper 传入 `BUILD_SHARED_LIBS=OFF`，以及 `USE_ASYNC/MBROLA/LIBSONIC/LIBPCAUDIO/KLATT/SPEECHPLAYER=OFF`、`EXTRA_cmn/ru=ON`。eSpeak 的 [deps.cmake][espeak-deps]在主机没有 Sonic 库时仍抓取上述 Sonic 提交；[链接脚本][espeak-link]以 `HAVE_LIBSONIC AND USE_LIBSONIC` 控制是否链接。准备可离线使用的构建配套时应保留 Sonic 源码，并提供本地源目录覆盖或实际使用的补丁。[Sonic 源文件][sonic-source]声明 Apache-2.0。

**实际发行物缺陷：** 官方 [PyPI 1.8.0 元数据][pypi]所列 `piper_tts-1.8.0.tar.gz` 为 24,276,117 字节，SHA-256 `830588aded347df579c91a32703e0fc2a3685d84f1e3533b14f2de69135d4904`。取得并验证该[官方 sdist][piper-sdist]后，`tar -tf` 的 85 项目录中没有 `espeakbridge.c`、`CMakeLists.txt` 或 `script/`。它单独不能提供桥接的对应源码；交付采用上表的完整 Piper 归档。

[上游 wheel 工作流][piper-wheels]使用 `cibuildwheel==2.23.2`、`CIBW_BUILD=cp39-*`；`setup.cfg` 设置 `py_limited_api=cp39`。构建说明应保留这些 ABI 选择、Piper CMake 中的原生选项，以及实际使用的补丁、编译与安装命令。未修改的通用编译器、CMake、Python 构建工具本身是否需交付，按下述 §1 的工具／系统库排除条款判断。

## Tom：可交付的源集与本次修改

**已核实事实：** 作者固定 [README][tom-readme]将权重许可明确设为 AGPL-3.0，并把权重许可与声线使用授权分开。作者 [model 3 源树][tom-tree]包含 `next.onnx` 和 `next.onnx.json`。本项目原始 ONNX 的身份为 63,511,038 字节、SHA-256 `bf65074ccdeeeeaa832e75edb1c0a513c01c9a972bdf085ff8a6e71ea234fd41`，与既有[来源核验](https://github.com/zardonc/lexique/blob/9698653fb2a0eb8af5de56ba7d37265d3b01d169/docs/evidence/issue-14/piper-distribution-research.md)中的作者第三套模型相同；固定下载身份见 [resources.json](resources.json)。

交付以下输入即可重现本项目的 ONNX 元数据与 token 转换：

- [原始 ONNX][tom-onnx]，保留作者提交 `a4601a0219fa8f8cf3f330bdd818dbb37ae287ce` 和 HF 快照 `5041a7a49ae5b3a0f30e6837769eb03fdbb231bd` 的来源记录。
- [作者第三套配置][tom-author-config]（7,079 字节）和[实际 HF 配置][tom-hf-config]（4,959 字节）分别保留。两者的解析差异只有 `dataset`、`audio.quality` 和语言描述；音素 ID、推理设置、说话人映射、`piper_version` 与 44,100 Hz 一致。
- [prepare.py](prepare.py)、[resources.json](resources.json) 和[固定模型卡](licenses/TOM-MODEL_CARD)。脚本复制配置，从原 `phoneme_id_map` 生成 `tokens.txt`，添加九项 ONNX 元数据，并检查序列化 graph 不变。结果身份为 SHA-256 `576982a23a5d3336aa2edeaaccd4b670485d5f97fb9017bf8c018d75d75d6572`。
- 作者完整 [LICENSE][tom-license]、[README][tom-readme]、模型／配置来源以及日期为 2026-11-07 的修改说明：九项元数据与 token 导出，权重图未改、未量化、未重训。

**交付解释：** 这是作者公开的模型材料与本项目修改的完整输入／源脚本。按作者将格式转换视为修改的声明，准备版继续保留 AGPL 和修改通知。它证明本次转换可重现；作者首选源形式的问题集中列在文末。

## 许可条文与最小交付路线

**条文：** [GPL §1、§4–6][gpl]和 [AGPL §1、§4–6][tom-license]要求保留版权、许可、免责及适用附加条件；修改版本须显著标明修改及日期。对象形式的对应源码包括生成、安装、运行和修改该作品所需的源码及控制脚本；可自动再生的文件、满足定义的系统库和未修改的通用工具有排除条款。仅把包称为 MIT 不会改变这些第三方义务。

**交付解释：** 将匹配的 `sources.zip` 与运行 ZIP 一起提供，在运行包 README 中给出直接位置。来源包保留上表的固定源码归档、Tom 输入／通知，以及本项目的 `prepare.py`、`bundle.py`、`scripts/sherpa_probe.py`、`scripts/speech_probe.py`、匹配清单和构建／安装说明；实际采用的源目录覆盖或补丁一并交付。运行包保留全部 wheel 原有许可，包括 Piper 自带 g2pW Apache-2.0 通知，以及 eSpeak／ucd-tools 的 GPL 和 Unicode 通知。完整来源归档中的其他许可与版权原文保持不变。

按 GPL／AGPL §6(d)，从指定地点提供二进制时，在同一地点、同样方式提供等价的对应源码访问且无额外费用即可；源码不必进入运行 ZIP。源码可位于第三方服务器，但二进制旁须有清楚指引，分发者仍负责其可用性。此处选择同次交付固定来源包。若通过物理介质交付，则同介质附源可采用 §6(a)。本路线不采用 §6(b) 的三年书面承诺。

## 声线与试听界面

**作者条件：** [README][tom-readme]另行授权声线使用：不得将语音呈现为真实或已识别人物的声音，不向服务用户披露作者身份；文件及其来源的版权／许可通知仍须保留。从权重或生成音频训练、微调、重建的新声线模型须以 AGPL-3.0 发布。

当前运行包的试听页披露“以下语音为机器合成”，使用通用法语单声线标签。作者和精确模型身份留在许可／来源文件中。此组合只提供本机离线试听；[AGPL §13][tom-license]关于修改版本的远程交互源码入口，没有当前可触发的网络服务界面。对象形式的再分发仍适用 §4–6。

## 本地验证、模型再分发与作者回复

**已有授权：** [AGPL §2][agpl-official]允许运行原版，以及在不向他人交付时制作、运行和修改作品；[GNU FAQ][gnu-private]也明确允许不公开的内部修改。作者 [README][tom-readme]已经授予权重的 AGPL 许可和符合所列条件的免费声线使用授权。本票的本机离线验证不需要另获作者同意，也不以回信为完成条件。

**再分发条件：** AGPL 已经允许符合条款的交付；§1、§6 要求的是对应源码，不是作者签发的证明。作者说明是确认首选修改形式的一种证据，实际工作流、公开训练／导出材料也可提供证据。[GNU 对非软件作品的解释][gnu-nonsoftware]仍使用“首选修改形式”，没有按 ONNX、checkpoint 或数据集文件类型列出一律必须交付的清单。如果 ONNX 本身就是该作品的首选修改形式，§1 将同一作品视为其对应源码；如果不是，须交付实际所需的源形式和控制脚本。本次公开材料尚不能确定哪种情形成立，不能因此断言 ONNX 必定不足。

**无回复时的路径（交付判断）：** 保留已完成的本地验证；可提供本项目代码和复现说明，让使用者自行从上游取得模型，暂不把该权重纳入本项目的公开下载包。这条路径不声称上游的源形式问题已解决。以后若可信公开材料足以确定首选修改形式并取得对应源集，就可按既有许可交付，无需等待作者回信；不兼容声线条件的用途才需另谈授权。作者沉默既不撤销已有许可，也不补足缺失的源材料或授予例外。

## 未闭合事实与验证范围

- **模型首选源形式：** [作者固定完整树][tom-tree]只有三个 ONNX／配置对、README、LICENSE 和试听 WAV，没有 checkpoint、训练程序、导出配方或“ONNX 是首选修改形式”的声明。2026-11-07 查验的[当前 GitHub 树][tom-current-tree]与固定树一致；[固定 HF 目录][tom-hf-tree]提供 ONNX、配置、模型卡及试听子目录，模型卡的 Training 仅指回作者仓库。[GitHub releases][tom-releases]和[原作者站点 releases][tom-gitea-releases]均为空。作者 README 说明第三套是在第二套上继续训练 300 epochs，但未提供该训练的源形式。AGPL §1 的“preferred form … for making modifications”是这项缺口的依据；公开 ONNX 可修改与本次转换可重现，尚不能单独证明该精确作品的首选修改形式。闭合可由能够明确该事实的可信公开材料及对应源集完成，并不限定为作者回信。条文没有按文件类型要求所有训练数据或全部历史 checkpoint，不由此扩张交付范围。
- 未执行源码编译或安装；来源和脚本存在不等于已从源重建 Windows bridge。干净机器安装和其他三个平台实际执行按本次范围延期。

[piper-archive]: https://github.com/OHF-Voice/piper1-gpl/archive/639388b6317fc4731e91d53da42aea68fd4166ff.tar.gz
[piper-cmake]: https://raw.githubusercontent.com/OHF-Voice/piper1-gpl/639388b6317fc4731e91d53da42aea68fd4166ff/CMakeLists.txt
[piper-setup]: https://raw.githubusercontent.com/OHF-Voice/piper1-gpl/639388b6317fc4731e91d53da42aea68fd4166ff/setup.py
[piper-wheels]: https://raw.githubusercontent.com/OHF-Voice/piper1-gpl/639388b6317fc4731e91d53da42aea68fd4166ff/.github/workflows/wheels.yml
[piper-sdist]: https://files.pythonhosted.org/packages/02/cc/95b18b58d9c235d8e0bcece7321b7a7347873c14f9fea2c734b31ef04ff2/piper_tts-1.8.0.tar.gz
[pypi]: https://pypi.org/pypi/piper-tts/1.8.0/json
[gpl]: https://raw.githubusercontent.com/OHF-Voice/piper1-gpl/639388b6317fc4731e91d53da42aea68fd4166ff/COPYING
[agpl-official]: https://www.gnu.org/licenses/agpl-3.0.en.html
[gnu-private]: https://www.gnu.org/licenses/gpl-faq.en.html#GPLRequireSourcePostedPublic
[gnu-nonsoftware]: https://www.gnu.org/licenses/gpl-faq.en.html#GPLOtherThanSoftware
[espeak-archive]: https://github.com/espeak-ng/espeak-ng/archive/724808c5a83f9ef95fdd0db886ba7ba537ff224a.tar.gz
[espeak-tree]: https://api.github.com/repos/espeak-ng/espeak-ng/git/trees/724808c5a83f9ef95fdd0db886ba7ba537ff224a?recursive=1
[espeak-cmake]: https://raw.githubusercontent.com/espeak-ng/espeak-ng/724808c5a83f9ef95fdd0db886ba7ba537ff224a/CMakeLists.txt
[espeak-deps]: https://raw.githubusercontent.com/espeak-ng/espeak-ng/724808c5a83f9ef95fdd0db886ba7ba537ff224a/cmake/deps.cmake
[espeak-link]: https://raw.githubusercontent.com/espeak-ng/espeak-ng/724808c5a83f9ef95fdd0db886ba7ba537ff224a/src/libespeak-ng/CMakeLists.txt
[ucd-cmake]: https://raw.githubusercontent.com/espeak-ng/espeak-ng/724808c5a83f9ef95fdd0db886ba7ba537ff224a/src/ucd-tools/CMakeLists.txt
[ucd-readme]: https://raw.githubusercontent.com/espeak-ng/espeak-ng/724808c5a83f9ef95fdd0db886ba7ba537ff224a/src/ucd-tools/README.md
[unicode-license]: https://raw.githubusercontent.com/espeak-ng/espeak-ng/724808c5a83f9ef95fdd0db886ba7ba537ff224a/COPYING.UCD
[sonic-archive]: https://github.com/waywardgeek/sonic/archive/fbf75c3d6d846bad3bb3d456cbc5d07d9fd8c104.tar.gz
[sonic-source]: https://raw.githubusercontent.com/waywardgeek/sonic/fbf75c3d6d846bad3bb3d456cbc5d07d9fd8c104/sonic.c
[tom-tree]: https://api.github.com/repos/tjiho/French-tts-model-piper/git/trees/a4601a0219fa8f8cf3f330bdd818dbb37ae287ce?recursive=1
[tom-current-tree]: https://api.github.com/repos/tjiho/French-tts-model-piper/git/trees/HEAD?recursive=1
[tom-hf-tree]: https://huggingface.co/api/models/rhasspy/piper-voices/tree/5041a7a49ae5b3a0f30e6837769eb03fdbb231bd/fr/fr_FR/tom/medium
[tom-readme]: https://raw.githubusercontent.com/tjiho/French-tts-model-piper/a4601a0219fa8f8cf3f330bdd818dbb37ae287ce/README.md
[tom-license]: https://raw.githubusercontent.com/tjiho/French-tts-model-piper/a4601a0219fa8f8cf3f330bdd818dbb37ae287ce/LICENSE
[tom-onnx]: https://huggingface.co/rhasspy/piper-voices/resolve/5041a7a49ae5b3a0f30e6837769eb03fdbb231bd/fr/fr_FR/tom/medium/fr_FR-tom-medium.onnx
[tom-author-config]: https://raw.githubusercontent.com/tjiho/French-tts-model-piper/a4601a0219fa8f8cf3f330bdd818dbb37ae287ce/model%203/next.onnx.json
[tom-hf-config]: https://huggingface.co/rhasspy/piper-voices/raw/5041a7a49ae5b3a0f30e6837769eb03fdbb231bd/fr/fr_FR/tom/medium/fr_FR-tom-medium.onnx.json
[tom-releases]: https://api.github.com/repos/tjiho/French-tts-model-piper/releases?per_page=100
[tom-gitea-releases]: https://git.bksp.space/api/v1/repos/Tjiho/French-tts-model-piper/releases?limit=100
