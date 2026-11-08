# M0：Sherpa-ONNX 法语离线组合验证

2026-11-07，GitHub [#5](https://github.com/zardonc/lexique/issues/5)。Windows x64 上，Sherpa-ONNX 2.0.1 + Piper `fr_FR-tom-medium` + 外部 eSpeak NG 前端完成八条固定输入的禁网生成、WAV 回读与同步播放。用户确认八条法语样本正确，包含 école 的词尾 /l/；该固定组合通过本机独立试听验收。

[试听通过的八条样本](samples-tom-v2/listen.html) · [人工通过记录](samples-tom-v2/quality.json) · [逐项原始报告](samples-tom-v2/report.json) · [资源与哈希](resources.json) · [分别许可](licenses/ATTRIBUTION.md) · [完整测量](measurements.json) · [来源包测量](source-report.json) · [构建与转换](source-build.md)

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

[v7 打包运行报告](package-report.json)记录固定组合的禁网生成与播放：总加载 2.8055288 秒，其中模型加载 1.7887413 秒；首条输入到首个 PCM 为 0.1181481 秒；八条生成共 0.7723418 秒、音频共 8.9048526 秒；进程峰值工作集 281,075,712 字节。该报告来自被测包中的探针与运行库，运行日期和环境字段由该进程记录。

完整运行资源获取为 **183,405,998 字节（183.41 MB）**，包括原始模型材料、所有运行库和离线 VC 安装器；计入元数据准备工具为 **191,781,282 字节（191.78 MB）**。完整软件源码归档及作者配置另需 **63,654,526 字节**，去重后的总获取量为 **255,435,808 字节（255.44 MB）**，已报告超过 200 MB。原始 ONNX / HF 配置只在获取量中计一次。测试用 ASR 缓存独立，不进入语音包或上述总量；核心语音资源在运行前准备好。

| 交付物 | ZIP 字节 | 解压后逻辑字节 | 文件数 |
| --- | ---: | ---: | ---: |
| Windows x64 运行包 v7 | 170,714,070 | 279,806,634 | 2,003 |
| 独立来源包 v3 | 122,375,208 | 128,126,232 | 83 |
| 两包合计 | 293,089,278 | 407,932,866 | 2,086 |

来源包保留完整 Piper / eSpeak / ucd-tools / Sonic 源码与数据源、Sherpa / Eigen 归档、原始 ONNX、HF / 作者两份配置、许可与项目控制脚本；运行包 README 指向同次交付的来源 ZIP。[来源研究](source-delivery-research.md)记录对应源形式的解释。[原生导入记录](native-dependencies.json)覆盖 61 个 DLL / PYD；包外导入为 Windows 系统 API 与两项 VC 前提。运行获取物料展开为 298,998,698 字节，含准备工具为 321,382,595 字节，含源码材料且去重原模型为 469,803,737 字节。两包 inventory、ZIP CRC 与成员全集校验通过，原包及已保存试听证据保留。

## 复验

先在仓库根目录使用 PowerShell 7、uv 和当前 Codex CLI。下载与准备阶段允许联网；原始大文件、工具环境和运行包仅放在忽略的 `.tooling/issue-5/`，不访问学习库。

```powershell
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true
$manifest = Get-Content docs/evidence/issue-5/resources.json -Raw | ConvertFrom-Json
$materials = '.tooling/issue-5/downloads'
$modelSource = '.tooling/issue-5/model-source'
$sourceDownloads = '.tooling/issue-5/source-downloads'
New-Item -ItemType Directory -Force $materials, $modelSource, $sourceDownloads | Out-Null
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
foreach ($item in $manifest.source_companion_downloads) {
  $path = Join-Path $sourceDownloads $item.file
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
  docs/evidence/issue-5/sources.py $sourceDownloads $modelSource `
  .tooling/issue-5/windows-x64-sources-recheck
.tooling/issue-5/prepare-recheck/Scripts/python.exe -B -X utf8 `
  docs/evidence/issue-5/bundle.py $materials .tooling/issue-5/assets-recheck `
  .tooling/issue-5/windows-x64-recheck --sources .tooling/issue-5/windows-x64-sources-recheck.zip
```

脚本校验每份输入的字节数与 SHA-256，已有准备目录或输出不覆盖。Microsoft 来源固定到已签名安装器的下载地址；哈希不匹配时保留失败，不换用未核验版本。VC 安装器只随包保留，此处不执行。

[sources.py](sources.py)校验原始输入，输出来源目录、ZIP 与同名 `.measurements.json`；缺失 / 损坏材料在创建交付物前失败，已有目录、ZIP 或报告均保留。[measure.py](measure.py)校验运行目录全集、inventory、ZIP CRC 和压缩成员身份。四个位置参数依次为下载目录、被测包、该包的生成报告和新 ZIP；`--sources` 指向来源包测量，同时核对来源 ZIP 字节数与哈希，报告完整获取量与两 ZIP 总量。日期、OS / Python、时间与内存引用该包的生成报告；缺环境字段在创建 ZIP 前失败。CPU 型号、物理核数与 RAM 由主机 `Get-CimInstance` 另行记录。安装体积包括脚本、许可与证据，获取量不重复计仓库文件和原模型。

生成与播放使用 Codex 0.167.0-alpha.3.2 的 Windows AppContainer，配置仅作用于本次子进程；不改网卡、防火墙或其他进程。`--require-network-denied` 本身只检查外部限制，两个公网 IP 的超时或连接拒绝不算断网证据。

```powershell
codex sandbox -P issue5 `
  -c 'permissions.issue5.filesystem={":root"="read",":project_roots"="write"}' `
  -c 'permissions.issue5.network.enabled=false' -C $PWD -- `
  .tooling/issue-5/windows-x64-recheck/runtime/python.exe -B -X utf8 `
  .tooling/issue-5/windows-x64-recheck/scripts/sherpa_probe.py `
  .tooling/issue-5/windows-x64-recheck/assets `
  .tooling/issue-5/recordings-recheck --require-network-denied --play

```

输出保留 `report.json` 和八个 WAV；运行时失败保留部分报告与异常上下文，预检失败显示具体缺项或哈希错误。[CLI 回归](../../../scripts/test_sherpa_probe.py)覆盖缺资源、损坏资源、已有输出、改写统一文本及真实禁网生成与播放，并验证完整模型步幅和例句识别。两个音频回归均在修复前复现失败。ASR 只接受 `apprends` / `apprend` 的同音拼写差异，不据此证明短词音素正确。

完整九项检查包括五项音频 / 输入回归及四项来源交付检查。下例复用本机 `.tooling/issue-14/asr-check`；其他环境须先按 `asr-model.json` 备齐固定 ASR 文件，测试内校验身份：

```powershell
$env:LEXIQUE_SHERPA_ASSETS = (Resolve-Path .tooling/issue-5/windows-x64-recheck/assets).Path
$env:LEXIQUE_SHERPA_ASR_ASSETS = (Resolve-Path .tooling/issue-14/asr-check).Path
$env:LEXIQUE_SHERPA_SOURCE_DOWNLOADS = (Resolve-Path $sourceDownloads).Path
$env:LEXIQUE_SHERPA_MODEL_SOURCE = (Resolve-Path $modelSource).Path
New-Item -ItemType Directory -Force .tooling/issue-5/tests-temp | Out-Null
$env:TEMP = (Resolve-Path .tooling/issue-5/tests-temp).Path
$env:TMP = $env:TEMP
codex sandbox -P issue5 `
  -c 'permissions.issue5.filesystem={":root"="read",":project_roots"="write"}' `
  -c 'permissions.issue5.network.enabled=false' -C $PWD -- `
  .tooling/issue-5/windows-x64-recheck/runtime/python.exe -B -X utf8 `
  -m unittest discover -s .tooling/issue-5/windows-x64-recheck/scripts -p 'test_sherpa*.py' -v

.tooling/issue-5/windows-x64-recheck/runtime/python.exe -B -X utf8 `
  docs/evidence/issue-5/measure.py $materials .tooling/issue-5/windows-x64-recheck `
  .tooling/issue-5/recordings-recheck/report.json .tooling/issue-5/windows-x64-recheck.zip `
  --sources .tooling/issue-5/windows-x64-sources-recheck.measurements.json
```

测试子进程统一使用 `-B`，保留 60 秒硬限；完整检查后再由测量 CLI 核对运行目录与 inventory 相符，测试不向运行包写入字节码。所有生成、播放和测试进程均同步退出，没有本票常驻服务。

2026-11-07：九项禁网 CLI 检查全部通过（25.777 秒，无跳过）；完整原始材料、双配置、原生构建源与通知均进入来源 ZIP；本地源码获取补丁通过固定树的 `git apply --check`。v7 包内探针完成八条禁网生成与播放，完整测试后的运行目录 inventory 与两包 ZIP 校验通过。运行打包缺失来源 ZIP、测量时来源报告哈希不符的失败检查通过，均未创建输出。最终八条保存样本人工试听通过。完整合并检查结果见[验证记录](../../verification.md)。

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
| 固定引擎、模型、声线、前端、运行库、文件身份与分别许可 | 清单、原始许可、作者声明、完整软件来源包与逐文件 inventory 已保存；模型首选源形式见限制 |
| Sherpa 2 完整法语前端 | 外部 eSpeak → codepoints → Sherpa 音素映射路径实测通过 |
| 八条统一输入 | `fr-common-v1` 原样保留并校验哈希；不拼句，不补标点 |
| 真正禁网生成、WAV 与播放 | 同进程前后四次 WSA 10013，八条有效音频与同步播放调用通过 |
| Windows 硬件、时间、内存、完整体积 | 方法与两包体积已实测；总获取 255.44 MB、两 ZIP 合计 293.09 MB，已报告超过 200 MB |
| 可回放样本和人工质量 | 最终八条通过，含 école 词尾 /l/；逐文件通过记录与原否决记录均保留 |
| 四目标获取 / 编译路线 | 路线已列；其余三目标按用户要求未执行 |
| 可重复步骤、失败与进程 / 资源隔离 | 步骤和 CLI 检查已保存；无常驻进程，模型与工具环境独立 |

人工通过适用于保存的最终八条样本；v7 包运行音频未单独试听。软件来源和本次转换输入 / 控制脚本已交付，Tom 作者的 AGPL §1 首选修改形式仍待明确；这是当前模型公开再分发的证据缺口。动态导入与固定 CMake 不能证明全部静态 / 头文件编译成分或上游 wheel 的确切构建工具链；源码编译未执行。SIWIS 的 Lessac 基础权重授权链单独保留为未通过准入。VC 安装器未执行，干净机器安装与其他三目标按用户要求延期。#5 保持开放；产品默认声线准入与正式发行继续按各自门槛验收。
