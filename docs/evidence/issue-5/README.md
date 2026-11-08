# M0：Sherpa-ONNX 法语离线组合验证

2026-11-07，GitHub [#5](https://github.com/zardonc/lexique/issues/5)。Windows x64 上，Sherpa-ONNX 2.0.1 + Piper `fr_FR-tom-medium` + 外部 eSpeak NG 前端完成八条固定输入的禁网生成、WAV 回读与同步播放。用户确认八条法语样本正确，包含 école 的词尾 /l/；该固定组合通过本机独立试听验收。

[试听通过的八条样本](samples-tom-v2/listen.html) · [人工通过记录](samples-tom-v2/quality.json) · [逐项原始报告](samples-tom-v2/report.json) · [资源与哈希](resources.json) · [分别许可](licenses/ATTRIBUTION.md) · [完整测量](measurements.json)

## 组合与前端

Tom 权重按作者固定的 AGPL-3.0 声明和单独的声线使用条件记录来源；许可原文与署名随证据保留。优先检查的 SIWIS medium 由 Lessac medium 微调，其基础权重授权链尚未闭合，详见许可记录。

- 引擎：Sherpa-ONNX 2.0.1，源码 `af2d8d81dbfa61c4cea4c568da78f3982edca005`，Windows x64 wheel。
- 模型：HF 快照 `5041a7a49ae5b3a0f30e6837769eb03fdbb231bd` 的 `fr_FR-tom-medium`；单说话人，推理 `speaker_id=0`，44,100 Hz。
- 前端：piper-tts 1.8.0 的 `EspeakPhonemizer`，使用本地 `espeak-ng-data` 与静态链接的 `espeakbridge.pyd`，原生 API 报告 eSpeak NG 1.52.0.1，声线严格取模型的 `fr`。
- 运行环境：官方 CPython 3.14.10 嵌入包，CPU、2 线程、速度 1.0，模型原始 `noise_scale=0.667`、`noise_w=0.8`、`length_scale=1.0`。完整 Python 依赖逐项固定在资源清单。

[prepare.py](prepare.py)复用固定 Sherpa 上游的元数据格式和音素 ID 导出规则。添加元数据后，序列化 ONNX 图与原文件完全相同；配置不改，导出的 `tokens.txt` 对应原始 `phoneme_id_map`。在准备工具环境使用 ONNX 1.23.2 再次执行时，三个输出哈希相同。

[探针源码](../../../scripts/sherpa_probe.py)把原始文本直接交给外部前端，保留每句音素与 Unicode codepoints，拒绝模型中没有映射的音素。随后以 `GenerationConfig.phoneme_codepoints` 调用 `tts.generate("", config)`，由 Sherpa 添加 Piper 的 BOS / PAD / EOS。2.0.1 的 `data_dir` 已被忽略，本组合不设置它，也不使用旧版内置前端或静态单词 lexicon。

`GenerationConfig.silence_scale=1.0` 保留模型完整波形。Sherpa 默认值 0.2 会缩短低振幅片段，影响 école 的安静词尾；固定上游实现与相同音素 ID 的对照见[诊断证据](diagnosis.json)。当前 école 保留 35,072 个采样点（0.795283 秒），符合模型 256 采样点的输出步幅，用户确认词尾 /l/。统一输入不补句、不加标点，也不做增益归一化。

原 MLS 的[失败样本与人工否决](samples/quality.json)、[原资源](samples/resources.json)、[测量](samples/measurements.json)及[打包报告](samples/package-report.json)保留。Tom 的[部分通过记录](samples-tom/quality.json)单独保留；最终通过只适用于 `samples-tom-v2/quality.json` 固定的八个文件。[句子识别证据](diagnosis.json)复用缓存的 [Whisper tiny int8](asr-model.json)，仅用于回归诊断；短词音素由人工试听确认。

## Windows x64 结果

机器为 AMD Ryzen 9 8945HS，8 核 / 16 线程，内存 33,555,279,872 字节，Windows 11 Home 10.0.26300。两项网络探测在同一进程开始和结束均返回 WinError 10013；八个 WAV 均为 44,100 Hz、单声道、16-bit PCM、非空且非静音，八次 `PlaySound` 同步调用完成。

加载、首个 PCM 回调、逐项生成与播放耗时、进程峰值工作集及体积方法见原始报告和完整测量。首个 PCM 是从该条文本开始计时的 Sherpa 回调；VITS 回调不证明流式播放。进程新启动，OS 文件缓存未清除，不将它称为冷启动或性能承诺。

[v5 打包运行报告](package-report.json)记录固定 Tom 组合的禁网生成与播放：总加载 2.7853150 秒，其中模型加载 1.7569227 秒；首条输入到首个 PCM 为 0.1229675 秒；八条生成共 0.7057890 秒、音频共 8.7307029 秒；进程峰值工作集 280,813,568 字节。该报告来自被测包中的探针与运行库，运行日期和环境字段由该进程记录。

完整运行资源获取为 **183,405,998 字节（183.41 MB）**，包括模型来源文件、所有运行库和离线 VC 安装器。另加元数据准备所需的三个工具 wheel，完整来源获取为 **191,781,282 字节（191.78 MB）**。测试用 ASR 资源独立缓存，不进入语音运行包或上述获取总量；核心语音资源在运行前准备好。

最终 v5 ZIP 为 **170,694,849 字节（170.69 MB）**；安装逻辑体积 **279,742,747 字节（279.74 MB）**，共 1,996 个文件。运行获取物料原始展开量 298,998,698 字节，计入准备工具后为 321,382,595 字节。inventory、ZIP CRC、成员全集与各文件哈希校验通过。

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
```

脚本校验每份输入的字节数与 SHA-256，已有准备目录或输出不覆盖。Microsoft 来源固定到已签名安装器的下载地址；哈希不匹配时保留失败，不换用未核验版本。VC 安装器只随包保留，此处不执行。

[measure.py](measure.py)校验目录文件全集、inventory 字节数与哈希、ZIP CRC 和压缩成员身份，输出 ZIP 及同名 `.measurements.json`。四个参数依次为下载目录、被测包、该包的生成报告和新 ZIP 路径；日期、CPU 标识、逻辑处理器数、OS / Python、时间与内存均引用明确传入的报告。旧报告缺少环境字段时预检失败；复验须使用当前探针生成的报告。CPU 型号、物理核数与 RAM 由主机 `Get-CimInstance` 另行记录，换机器复验时一并记录。下载总量计入清单中的来源文件、运行归档和准备工具；仓库已保存的探针、输入与许可证据不重复计入获取总量，安装体积包含这些材料。

生成与播放使用 Codex 0.167.0-alpha.3.2 的 Windows AppContainer，配置仅作用于本次子进程；不改网卡、防火墙或其他进程。`--require-network-denied` 本身只检查外部限制，两个公网 IP 的超时或连接拒绝不算断网证据。

```powershell
codex sandbox -P issue5 `
  -c 'permissions.issue5.filesystem={":root"="read",":project_roots"="write"}' `
  -c 'permissions.issue5.network.enabled=false' -C $PWD -- `
  .tooling/issue-5/windows-x64-recheck/runtime/python.exe -B -X utf8 `
  .tooling/issue-5/windows-x64-recheck/scripts/sherpa_probe.py `
  .tooling/issue-5/windows-x64-recheck/assets `
  .tooling/issue-5/recordings-recheck --require-network-denied --play

.tooling/issue-5/windows-x64-recheck/runtime/python.exe -B -X utf8 `
  docs/evidence/issue-5/measure.py $materials .tooling/issue-5/windows-x64-recheck `
  .tooling/issue-5/recordings-recheck/report.json .tooling/issue-5/windows-x64-recheck.zip
```

输出保留 `report.json` 和八个 WAV；运行时失败保留部分报告与异常上下文，预检失败显示具体缺项或哈希错误。[CLI 回归](../../../scripts/test_sherpa_probe.py)覆盖缺资源、损坏资源、已有输出、改写统一文本及真实禁网生成与播放，并验证完整模型步幅和例句识别。两个音频回归均在修复前复现失败。ASR 只接受 `apprends` / `apprend` 的同音拼写差异，不据此证明短词音素正确。

完整五项检查需同时提供两个环境变量。下例复用本机 `.tooling/issue-14/asr-check`；其他环境须先按 `asr-model.json` 备齐固定文件，测试内校验身份：

```powershell
$env:LEXIQUE_SHERPA_ASSETS = (Resolve-Path .tooling/issue-5/windows-x64-recheck/assets).Path
$env:LEXIQUE_SHERPA_ASR_ASSETS = (Resolve-Path .tooling/issue-14/asr-check).Path
New-Item -ItemType Directory -Force .tooling/issue-5/tests-temp | Out-Null
$env:TEMP = (Resolve-Path .tooling/issue-5/tests-temp).Path
$env:TMP = $env:TEMP
codex sandbox -P issue5 `
  -c 'permissions.issue5.filesystem={":root"="read",":project_roots"="write"}' `
  -c 'permissions.issue5.network.enabled=false' -C $PWD -- `
  .tooling/issue-5/windows-x64-recheck/runtime/python.exe -B -X utf8 `
  .tooling/issue-5/windows-x64-recheck/scripts/test_sherpa_probe.py -v
```

测试子进程保留 60 秒硬限。所有生成、播放和测试进程均同步退出，没有本票常驻服务。

2026-11-07 检查结果：禁网 `scripts/test_sherpa_probe.py` 五项通过、无跳过（16.675 秒）；`pnpm typecheck`、`pnpm test:docs` 和 `git diff --cached --check` 通过；`prepare.py` 输出身份复验通过；`bundle.py` 与 `measure.py` 的实际包检查通过。测量环境字段与显式生成报告一致，缺环境的旧报告在创建 ZIP 前拒绝。最终八条样本人工试听通过。

## 四目标路线

固定引擎 v2.0.1。模型、原配置和音素表可共用；eSpeak 前端与原生运行库须按架构获取或构建。下表的路线不等于实机通过。

| 目标 | 获取 / 编译路线 | 实机状态 |
| --- | --- | --- |
| Windows x64 | 本票固定 CPython 嵌入包与 PyPI wheels；或官方 [MD Release C API shared 归档](https://github.com/k2-fsa/sherpa-onnx/releases/download/v2.0.1/sherpa-onnx-v2.0.1-win-x64-shared-MD-Release-lib.tar.bz2)。源码使用 VS 2022 C++ 工具、Windows SDK 和 `cmake -A x64`；Piper 固定源码会构建其外部 eSpeak | 固定 Tom 组合生成、回读、禁网与播放调用通过；八条人工试听通过 |
| Windows ARM64 | 官方 [ARM64 MD Release C API shared 归档](https://github.com/k2-fsa/sherpa-onnx/releases/download/v2.0.1/sherpa-onnx-v2.0.1-win-arm64-shared-MD-Release-lib.tar.bz2)；前端从固定 Piper / eSpeak 源码原生构建 ARM64，或评估 VS `cmake -A ARM64` 交叉目标与 host 工具，并准备对应 Python / VC 运行依赖。若缺相符 wheel，走源码与 C API 路线 | 获取路线已列，编译、链接、生成、播放未实机验证 |
| macOS x64 | 官方 [osx x64 shared 归档](https://github.com/k2-fsa/sherpa-onnx/releases/download/v2.0.1/sherpa-onnx-v2.0.1-osx-x64-shared-lib.tar.bz2)，或固定源码 CMake `CMAKE_OSX_ARCHITECTURES=x86_64`；匹配的 Python wheel / 源码构建 Piper 前端，播放可使用系统 `afplay` | 未实机验证 |
| macOS ARM64 | 官方 [osx arm64 shared 归档](https://github.com/k2-fsa/sherpa-onnx/releases/download/v2.0.1/sherpa-onnx-v2.0.1-osx-arm64-shared-lib.tar.bz2)，或固定源码 CMake `CMAKE_OSX_ARCHITECTURES=arm64`；匹配的 Python wheel / 源码构建 Piper 前端，播放可使用系统 `afplay` | 未实机验证 |

官方[引擎发布页](https://github.com/k2-fsa/sherpa-onnx/releases/tag/v2.0.1)、[Piper 固定构建文件](https://github.com/OHF-Voice/piper1-gpl/blob/639388b6317fc4731e91d53da42aea68fd4166ff/CMakeLists.txt)。本票 CLI 的播放和峰值内存测量使用 Windows API；另外三个目标需要各自的平台测量与禁网启动环境，按用户要求暂缓。

## 验收状态与限制

| #5 要求 | 结果 |
| --- | --- |
| 固定引擎、模型、声线、前端、运行库、文件身份与分别许可 | 清单、原始许可、Tom 作者声明与逐文件 inventory 已保存；公共再分发核验见限制 |
| Sherpa 2 完整法语前端 | 外部 eSpeak → codepoints → Sherpa 音素映射路径实测通过 |
| 八条统一输入 | `fr-common-v1` 原样保留并校验哈希；不拼句，不补标点 |
| 真正禁网生成、WAV 与播放 | 同进程前后四次 WSA 10013，八条有效音频与同步播放调用通过 |
| Windows 硬件、时间、内存、完整体积 | 已测量并保存方法；运行获取 183.41 MB，含准备工具 191.78 MB，ZIP 170.69 MB |
| 可回放样本和人工质量 | 最终八条通过，含 école 词尾 /l/；逐文件通过记录与原否决记录均保留 |
| 四目标获取 / 编译路线 | 路线已列；其余三目标按用户要求未执行 |
| 可重复步骤、失败与进程 / 资源隔离 | 步骤和 CLI 检查已保存；无常驻进程，模型与工具环境独立 |

人工通过适用于保存的最终八条样本；v5 打包运行的音频未单独试听。SIWIS 的 Lessac 基础权重授权链、完整二进制构建来源、GPL / AGPL 对应源码交付和公共再分发核验未闭合，详见许可记录；这里的元数据准备不代表对应源码交付完成。VC 安装器保留但未执行，干净机器前提及其他三目标未测，其他平台按用户要求暂缓。#5 未发布或关闭；本机独立验收不建立产品默认声线准入或正式发行完成的结论。
