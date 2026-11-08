# M0：Sherpa-ONNX 法语离线组合验证

2026-11-07，GitHub [#5](https://github.com/zardonc/lexique/issues/5)。Windows x64 上，Sherpa-ONNX 2.0.1 + Piper `fr_FR-mls-medium` + 外部 eSpeak NG 前端完成八条固定输入的生成、WAV 回读与同步播放。用户试听判定“发音或音质失败”，该候选组合未通过法语质量验收；#5 保持开放。

[试听八条样本](samples/listen.html) · [人工否决记录](samples/quality.json) · [逐项原始报告](samples/report.json) · [资源与哈希](resources.json) · [分别许可](licenses/ATTRIBUTION.md) · [完整测量](measurements.json)

## 组合与前端

优先检查的 SIWIS medium 由 Lessac medium 微调。SIWIS 数据许可为 CC BY 4.0，外层仓库标签为 MIT，但 Lessac 基础权重的授权链尚未闭合；详见许可记录。本次选用声明从零训练的 MLS 作为同一 Piper 家族的验证组合。#14 曾否决另一 Piper/MLS 管线的听感，本次新样本的独立试听同样失败；产品默认声线保持既有选择。

- 引擎：Sherpa-ONNX 2.0.1，源码 `af2d8d81dbfa61c4cea4c568da78f3982edca005`，Windows x64 wheel。
- 模型：HF 快照 `5041a7a49ae5b3a0f30e6837769eb03fdbb231bd` 的 `fr_FR-mls-medium`；MLS source `3698` 女声 → 推理 `speaker_id=1`。
- 前端：piper-tts 1.8.0 的 `EspeakPhonemizer`，使用本地 `espeak-ng-data` 与静态链接的 `espeakbridge.pyd`，原生 API 报告 eSpeak NG 1.52.0.1，声线严格取模型的 `fr`。
- 运行环境：官方 CPython 3.14.10 嵌入包，CPU、2 线程、速度 1.0，模型原始 `noise_scale=0.333`、`noise_w=0.333`、`length_scale=1.0`。完整 Python 依赖逐项固定在资源清单。

[prepare.py](prepare.py)复用固定 Sherpa 上游的元数据格式和音素 ID 导出规则。添加元数据后，序列化 ONNX 图与原文件完全相同；配置不改，导出的 `tokens.txt` 对应原始 `phoneme_id_map`。在准备工具环境使用 ONNX 1.23.2 再次执行时，三个输出哈希相同。

[探针源码](../../../scripts/sherpa_probe.py)把原始文本直接交给外部前端，保留每句音素与 Unicode codepoints，拒绝模型中没有映射的音素。随后以 `GenerationConfig.phoneme_codepoints` 调用 `tts.generate("", config)`，由 Sherpa 添加 Piper 的 BOS / PAD / EOS。2.0.1 的 `data_dir` 已被忽略，本组合不设置它，也不使用旧版内置前端或静态单词 lexicon。

## Windows x64 结果

机器为 AMD Ryzen 9 8945HS，8 核 / 16 线程，内存 33,555,279,872 字节，Windows 11 Home 10.0.26300。两项网络探测在同一进程开始和结束均返回 WinError 10013；八个 WAV 均为 22,050 Hz、单声道、16-bit PCM、非空且非静音，八次 `PlaySound` 同步调用完成。

加载、首个 PCM 回调、逐项生成与播放耗时、进程峰值工作集及体积方法见原始报告和完整测量。首个 PCM 是从该条文本开始计时的 Sherpa 回调；VITS 回调不证明流式播放。进程新启动，OS 文件缓存未清除，不将它称为冷启动或性能承诺。

[打包目录的补充运行报告](package-report.json)记录相同固定组合的第二次禁网生成与播放：总加载 1.1032434 秒，其中模型加载 0.6938082 秒；首条输入到首个 PCM 为 0.1145799 秒；八条生成共 0.8802798 秒、音频共 16.8384127 秒；进程峰值工作集 299,474,944 字节。该次运行的音频未单独人工试听，用户否决对应 `samples/quality.json` 中固定的原始八条样本。最终 v2 包更新验收记录，推理代码与该次运行相同。

完整运行资源获取为 **196,630,776 字节（196.63 MB）**，包括模型来源文件、所有运行库和离线 VC 安装器。另加重建模型所需的三个工具 wheel，完整来源获取为 **205,006,060 字节（205.01 MB，超过 200 MB）**。运行 ZIP、原始展开量与实际安装逻辑大小分别测量，见测量文件；工具环境独立，核心资源在运行前准备好。

最终 v2 ZIP 为 **182,728,902 字节（182.73 MB）**；安装逻辑体积 **291,494,786 字节（291.49 MB）**，共 1,969 个文件。运行获取物料原始展开量 312,223,476 字节，计入准备工具后为 334,607,373 字节。inventory、ZIP CRC、成员全集与各文件哈希校验通过；初版包仍保留。

## 复验

先在仓库根目录使用 PowerShell 7、uv 和当前 Codex CLI。下载与准备阶段允许联网；原始大文件、工具环境和运行包仅放在忽略的 `.tooling/issue-5/`，不访问学习库。

```powershell
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true
$manifest = Get-Content docs/evidence/issue-5/resources.json -Raw | ConvertFrom-Json
$materials = '.tooling/issue-5/downloads'
$modelSource = '.tooling/issue-5/model-source'
New-Item -ItemType Directory -Force $materials, $modelSource | Out-Null
foreach ($item in @($manifest.downloads) + @($manifest.preparation_downloads)) {
  $path = Join-Path $materials $item.file
  Invoke-WebRequest $item.url -OutFile $path
  if ((Get-Item $path).Length -ne $item.bytes -or
      (Get-FileHash $path -Algorithm SHA256).Hash.ToLower() -ne $item.sha256) {
    throw "resource checksum mismatch: $($item.file)"
  }
}
foreach ($item in $manifest.source_files) {
  $path = Join-Path $modelSource $item.file
  Invoke-WebRequest $item.url -OutFile $path
  if ((Get-Item $path).Length -ne $item.bytes -or
      (Get-FileHash $path -Algorithm SHA256).Hash.ToLower() -ne $item.sha256) {
    throw "source resource checksum mismatch: $($item.file)"
  }
}
uv venv --python 3.14.10 .tooling/issue-5/prepare-recheck
uv pip install --python .tooling/issue-5/prepare-recheck/Scripts/python.exe `
  --no-index --find-links $materials onnx==1.23.2 ml-dtypes==0.6.0 `
  numpy==2.5.4 protobuf==7.36.2 typing-extensions==4.16.0
.tooling/issue-5/prepare-recheck/Scripts/python.exe -B -X utf8 `
  docs/evidence/issue-5/prepare.py $modelSource .tooling/issue-5/assets-recheck
.tooling/issue-5/prepare-recheck/Scripts/python.exe -B -X utf8 `
  docs/evidence/issue-5/bundle.py $materials .tooling/issue-5/assets-recheck `
  .tooling/issue-5/windows-x64-recheck
.tooling/issue-5/windows-x64-recheck/runtime/python.exe -B -X utf8 `
  docs/evidence/issue-5/measure.py $materials .tooling/issue-5/windows-x64-recheck `
  .tooling/issue-5/windows-x64-recheck.zip
```

脚本校验每份输入的字节数与 SHA-256，已有准备目录或输出不覆盖。Microsoft 来源固定到已签名安装器的下载地址；哈希不匹配时保留失败，不换用未核验版本。VC 安装器只随包保留，此处不执行。

[measure.py](measure.py)校验目录文件全集、inventory 字节数与哈希、ZIP CRC 和压缩成员身份，输出 ZIP 及同名 `.measurements.json`。体积由本次实际文件重新计算，时间与内存引用已保存的 Windows 原始运行报告。下载总量计入清单中的来源文件、运行归档和准备工具；仓库已保存的探针、输入与许可证据不重复计入获取总量，安装体积包含这些材料。此处的“从来源重建”指 ONNX 元数据准备，不代表 GPL 对应源码交付完成。

生成与播放使用 Codex 0.167.0-alpha.3.2 的 Windows AppContainer，配置仅作用于本次子进程；不改网卡、防火墙或其他进程。`--require-network-denied` 本身只检查外部限制，两个公网 IP 的超时或连接拒绝不算断网证据。

```powershell
codex sandbox -P issue5 `
  -c 'permissions.issue5.filesystem={":root"="read",":project_roots"="write"}' `
  -c 'permissions.issue5.network.enabled=false' -C $PWD -- `
  .tooling/issue-5/windows-x64-recheck/runtime/python.exe -B -X utf8 `
  scripts/sherpa_probe.py .tooling/issue-5/windows-x64-recheck/assets `
  .tooling/issue-5/recordings-recheck --require-network-denied --play
```

输出保留 `report.json` 和八个 WAV；运行时失败保留部分报告与异常上下文，预检失败显示具体缺项或哈希错误。缺资源、损坏资源、已有输出和改写统一文本均有 [CLI 回归](../../../scripts/test_sherpa_probe.py)。运行完整五项检查时，把 `LEXIQUE_SHERPA_ASSETS` 指向固定资源，TEMP / TMP 指向 `.tooling/issue-5/tests-temp`，使用同一禁网启动命令运行 `scripts/test_sherpa_probe.py`；子进程测试有 60 秒硬限。所有生成、播放和测试进程均同步退出，没有本票常驻服务。

2026-11-07 检查结果：禁网 `scripts/test_sherpa_probe.py` 五项通过、无跳过（21.978 秒）；`pnpm typecheck`、`pnpm test:docs` 和 `git diff --cached --check` 通过；`prepare.py` 输出身份复验通过；`bundle.py` 与 `measure.py` 的实际包检查通过。人工试听结果为失败。

## 四目标路线

固定引擎 v2.0.1。模型、原配置和音素表可共用；eSpeak 前端与原生运行库须按架构获取或构建。下表的路线不等于实机通过。

| 目标 | 获取 / 编译路线 | 实机状态 |
| --- | --- | --- |
| Windows x64 | 本票固定 CPython 嵌入包与 PyPI wheels；或官方 [MD Release C API shared 归档](https://github.com/k2-fsa/sherpa-onnx/releases/download/v2.0.1/sherpa-onnx-v2.0.1-win-x64-shared-MD-Release-lib.tar.bz2)。源码使用 VS 2022 C++ 工具、Windows SDK 和 `cmake -A x64`；Piper 固定源码会构建其外部 eSpeak | 生成、回读、禁网与播放调用通过；人工发音或音质验收失败 |
| Windows ARM64 | 官方 [ARM64 MD Release C API shared 归档](https://github.com/k2-fsa/sherpa-onnx/releases/download/v2.0.1/sherpa-onnx-v2.0.1-win-arm64-shared-MD-Release-lib.tar.bz2)；前端从固定 Piper / eSpeak 源码原生构建 ARM64，或评估 VS `cmake -A ARM64` 交叉目标与 host 工具，并准备对应 Python / VC 运行依赖。若缺相符 wheel，走源码与 C API 路线 | 获取路线已列，编译、链接、生成、播放未实机验证 |
| macOS x64 | 官方 [osx x64 shared 归档](https://github.com/k2-fsa/sherpa-onnx/releases/download/v2.0.1/sherpa-onnx-v2.0.1-osx-x64-shared-lib.tar.bz2)，或固定源码 CMake `CMAKE_OSX_ARCHITECTURES=x86_64`；匹配的 Python wheel / 源码构建 Piper 前端，播放可使用系统 `afplay` | 未实机验证 |
| macOS ARM64 | 官方 [osx arm64 shared 归档](https://github.com/k2-fsa/sherpa-onnx/releases/download/v2.0.1/sherpa-onnx-v2.0.1-osx-arm64-shared-lib.tar.bz2)，或固定源码 CMake `CMAKE_OSX_ARCHITECTURES=arm64`；匹配的 Python wheel / 源码构建 Piper 前端，播放可使用系统 `afplay` | 未实机验证 |

官方[引擎发布页](https://github.com/k2-fsa/sherpa-onnx/releases/tag/v2.0.1)、[Piper 固定构建文件](https://github.com/OHF-Voice/piper1-gpl/blob/639388b6317fc4731e91d53da42aea68fd4166ff/CMakeLists.txt)。本票 CLI 的播放和峰值内存测量使用 Windows API；另外三个目标需要各自的平台测量与禁网启动环境，按用户要求暂缓。

## 验收状态与限制

| #5 要求 | 结果 |
| --- | --- |
| 固定引擎、模型、声线、前端、运行库、文件身份与分别许可 | 部分完成：清单、原始许可与逐文件 inventory 已保存，完整二进制来源及再分发核验未闭合；SIWIS 基础权重授权链未闭合，采用 MLS 验证 |
| Sherpa 2 完整法语前端 | 外部 eSpeak → codepoints → Sherpa 音素映射路径实测通过 |
| 八条统一输入 | `fr-common-v1` 原样保留并校验哈希；不拼句，不补标点 |
| 真正禁网生成、WAV 与播放 | 同进程前后四次 WSA 10013，八条有效音频与同步播放调用通过 |
| Windows 硬件、时间、内存、完整体积 | 已测量并保存方法；从来源重建下载超过 200 MB 已报告 |
| 可回放样本和人工质量 | 八条已交付；用户判定发音或音质失败，原始 WAV 与逐文件否决记录保留 |
| 四目标获取 / 编译路线 | 路线已列；其余三目标按用户要求未执行 |
| 可重复步骤、失败与进程 / 资源隔离 | 步骤和 CLI 检查已保存；无常驻进程，模型与工具环境独立 |

用户否决未细分具体单词、错误类型或播放设备，不能据此推断逐词发音问题。MLS 候选的人工质量验收失败，SIWIS 的合法组合生成没有完成。完整二进制构建来源、GPL 对应源码交付、干净机器 VC 运行依赖和其他平台验收仍未闭合，详见许可记录。本票结果不能作为产品声线重新准入、四平台通过或正式发行完成的证据。
