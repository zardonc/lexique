# M0：Kokoro 法语声线与离线运行时验证

2026-11-07（America/Vancouver），GitHub [#6](https://github.com/zardonc/lexique/issues/6)。Windows x64 上，Kokoro v1.0 int8 + `ff_siwis` + kokoro-onnx 0.6.1 + eSpeak NG 1.52.0 完成八条独立法语输入的禁网生成、有效 WAV 回读与同步播放调用。

离线运行通过；法语音质待用户逐项试听；正式再分发仍需核对转换和二进制构建来源及许可义务。应用随包语音继续采用现有 Supertonic 3，见 [#5 的候选比较](../issue-5/redistributable-french-models.md)。

[八条试听样本](samples/listen.html) · [逐项原始报告](samples/report.json) · [完整测量](measurements.json) · [硬件与环境](environment.json) · [26 项获取材料及分别许可](../issue-14/resources.json) · [探针源码](../../../scripts/speech_probe.py)

## 固定组合与来源

| 部分 | 本次固定配置及核验 |
| --- | --- |
| 模型 | Kokoro-82M v1.0；官方快照 `f3ff3571791e39611d31c381e3a41a3af07b4987`，运行文件 `kokoro-v1.0.int8.onnx` |
| 声线 | 法语女声 `ff_siwis`；`voices-v1.0.bin` 内 float32 `(510, 1, 256)` 张量与官方 `ff_siwis.pt` 存储字节完全相同 |
| 推理封装 | kokoro-onnx 0.6.1；固定源码 `3596b26764286a7de9d90c363e988d50578918e5` 的 `Kokoro.from_session` |
| 文本前端 | phonemizer 3.4.0，espeakng-loader 0.2.4；DLL 自报 eSpeak NG 1.52.0，固定 loader 源码 `0ddc87adf77e5850d7eeb542ac8a87d421b64daa` |
| 推理运行时 | ONNX Runtime 1.31.0，显式 `CPUExecutionProvider`，intra-op 2 线程、inter-op 1 线程；NumPy 2.5.4 |
| Python | 官方 CPython 3.12.10 Windows x64 嵌入包；`python312._pth` 仅含包内标准库、解释器目录与 `Lib/site-packages` |
| 法语及音频配置 | 前端 `fr-fr`，速度 1.0，`trim=True`；24,000 Hz、单声道、16-bit PCM |

资源清单逐项固定 URL、字节数、SHA-256 和许可；包括全部 13 个 Python wheel、四个软件源码归档、两个模型文件、官方声线、Python、VC 安装器和四份 notices。实际包内文件另以 `inventory.json` 逐项记录，测量时核对 ZIP CRC、成员全集和内容哈希。

探针从原始文本调用固定的法语 eSpeak 前端，记录未过滤音素与模型音素，显式去除词间 `-` 分隔符；空音素或模型词表外字符会使该项失败。模型接收这组音素，而不是凭多语言模型名称推断法语可用。固定 DLL / 数据树哈希、版本和实际路径进入报告；拒绝环境变量替换前端，指定 DLL 预加载失败时明确报错。现有孤立 `y` 词汇读音规则由 CLI 回归保留，本次八条输入不包含 `y`。

## 分别许可

- 模型权重：[官方模型卡](https://huggingface.co/hexgrad/Kokoro-82M/blob/f3ff3571791e39611d31c381e3a41a3af07b4987/README.md)声明 Apache-2.0。量化 ONNX 与转换声线来自 [model-files-v1.0](https://github.com/thewh1teagle/kokoro-onnx/releases/tag/model-files-v1.0)，按模型授权记录，转换器的 MIT 不替代权重许可。
- 声线：同一官方仓库的 `voices/ff_siwis.pt` 及[声线说明](https://huggingface.co/hexgrad/Kokoro-82M/blob/f3ff3571791e39611d31c381e3a41a3af07b4987/VOICES.md)，张量身份已核对。未发现声线专用的独立许可文本；按仓库声明及训练来源分别记录。
- SIWIS 训练来源：Junichi Yamagishi、Pierre-Edouard Honnet、Philip Garner、Alexandros Lazaridis，[数据集 DOI 10.7488/ds/1705](https://doi.org/10.7488/ds/1705)。数据集的[实际许可文本](https://datashare.ed.ac.uk/bitstreams/d7204860-2dc9-48e6-bb9a-604a3a49d9ce/download)确认为 CC BY 4.0；此数据许可与模型权重许可分别适用。运行包不需要训练数据集或 PyTorch。
- 封装及运行库：kokoro-onnx 与 ONNX Runtime 为 MIT；NumPy 及其余 wheel 的分别许可见清单与各自 `.dist-info`。官方 Python 保留 PSF 和随包第三方 notices。
- 文本前端：phonemizer 3.4.0 为 GPL-3.0-or-later；espeakng-loader 包装代码为 MIT，内含 eSpeak NG DLL / 数据适用 GPL 及数据 notices。包内保留 phonemizer 对应源码、eSpeak NG 1.52.0 完整源码和固定 loader 构建脚本。
- VC 前提：固定的 Microsoft x64 Redistributable 安装器有独立条款，随包保存但未执行。[本项目 LICENSE](../../../LICENSE)只用于项目代码。

模型卡、声线说明、Apache 文本及 loader 许可位于包内 `notice/`；原始官方声线在 `provenance/`；软件源码在 `source/`；依赖 notices 保留于 `runtime/Lib/site-packages`。[随包说明](package-readme.txt)记录这些路径。

## Windows x64 测量

与 #5 使用同一 AMD Ryzen 9 8945HS、8 核 / 16 线程、33,555,279,872 字节 RAM，Windows 11 Home 10.0.26300。Kokoro 使用 CPython 3.12.10，#5 使用 3.14.10。进程起点为 `2026-11-08T06:33:48.466455+00:00`，对应温哥华 11 月 7 日。

| 指标 | 实测 |
| --- | ---: |
| 模型加载 | 1.4001283 秒 |
| 完整加载（运行库 import、前端校验、模型） | 4.3235254 秒 |
| 首条输入到可用音频数组 | 0.8693316 秒 |
| 八条纯合成累计 | 3.1157273 秒 |
| 八条音频累计 | 6.7200000 秒 |
| 进程峰值工作集 | 266,145,792 字节 |
| 完整获取材料 | 215,446,330 字节（215.45 MB） |
| 获取材料逐项展开 | 338,767,206 字节 |
| 完整运行与来源 ZIP | 184,562,539 字节（184.56 MB） |
| 安装逻辑体积 | 300,798,267 字节（2,220 个文件） |

完整获取量**超过 200 MB，已报告用户**。它包括模型、声线、全部依赖、来源和离线 VC 安装器；资源在运行前准备齐全。MB 使用十进制。展开量是原始归档内普通文件字节和非归档材料的总和；安装量为被测目录普通文件逻辑字节总和，含清单、许可和来源，不代表 NTFS 分配空间或系统已有 VC 占用。

ZIP 保存在本机忽略目录 `.tooling/issue-6/windows-x64-v2.zip`，SHA-256 为 `2a04d3ba504a0b43338eca9d826796ac378aa97b359d6c4611ea1d6d24e9ca2b`。[measure.py](measure.py)核对全部获取材料、官方声线张量、运行依赖版本、固定输入、WAV 身份、播放记录及四次 WinError 10013，再生成清单与完整 ZIP。

模型加载从 ONNX session 构建开始，含声线加载；完整加载另计运行库 import 和前端校验。逐项记录 G2P、纯合成、音频可用、PCM 写入 / 回读和同步播放耗时。Kokoro 同步 API 返回的是完整该条音频数组；#5 的首个 PCM 指 Sherpa 回调，两个首音字段不是相同 API 边界。首次 G2P 的初始化计入首条时间。进程重新启动，OS 文件缓存未清除；这些数字不是冷启动或流式播放测量。

## 样本与试听重点

统一输入 [`fr-common-v1`](../issue-5/inputs.json) 原字节 SHA-256 为 `9ecb2760c2fc8de956d8c080752ff18b5c196adba626fe178c6ce34004001372`。每项独立生成，不补句、不补标点、不复用已有音频。八个文件均有效、非空、非静音；前后两组网络连接得到 10013，八次 Windows `PlaySound` 同步调用完成。法语音质均为**待人工试听**。

| 输入 / WAV | 秒 | 试听重点 |
| --- | ---: | --- |
| [être](samples/01.wav) | 0.491 | 孤立短词、/ɛ/ 与词尾 /tʁ/ |
| [sommes](samples/02.wav) | 0.555 | 变位的 /sɔm/、词尾清晰度 |
| [œil](samples/03.wav) | 0.448 | /œj/、孤立单音节 |
| [école](samples/04.wav) | 0.597 | /e/、重音节奏、词尾 /l/ |
| [aujourd'hui](samples/05.wav) | 0.811 | /dɥi/ 衔接和整体节奏 |
| [nous sommes ici](samples/06.wav) | 0.981 | sommes 的变位、词间衔接与多余辅音 |
| [les amis arrivent](samples/07.wav) | 1.088 | les amis 的 /z/ 连读与整句衔接 |
| [J'apprends le français tous les jours.](samples/08.wav) | 1.749 | 鼻化元音、重音与句末语调 |

[试听页](samples/listen.html)显示通用合成语音说明及待测状态；Obscura 验证八个音频链接均返回 HTTP 200，WAV 头均为单声道 24 kHz / 16-bit PCM。实际播放证据来自原生探针。官方声线说明提示短文本可能较弱，长句生成成功不能代替前五个孤立词的质量验收。

## 复验

仓库根目录、PowerShell 7、uv、当前 Codex CLI。获取与准备阶段可联网；新目录均在忽略的 `.tooling/issue-6/`。脚本核对固定材料，已有包、ZIP、清单或测量报告保留。

```powershell
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true
$manifest = Get-Content docs/evidence/issue-14/resources.json -Raw | ConvertFrom-Json
$materials = '.tooling/issue-6/materials-recheck'
$baseline = '.tooling/issue-6/baseline-recheck'
$package = '.tooling/issue-6/windows-x64-recheck'
if (Test-Path -LiteralPath $materials) { throw 'Choose a new materials directory' }
if (Test-Path -LiteralPath $package) { throw 'Choose a new package directory' }
New-Item -ItemType Directory -Path $materials | Out-Null
foreach ($item in $manifest) {
  $path = Join-Path $materials $item.file
  Invoke-WebRequest $item.url -OutFile $path
  if ((Get-Item -LiteralPath $path).Length -ne $item.bytes -or
      (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLower() -ne $item.sha256) {
    throw "resource checksum mismatch: $($item.file)"
  }
}
uv run --python 3.12.10 --no-project python -B -X utf8 `
  docs/evidence/issue-14/prepare.py $materials $baseline
New-Item -ItemType Directory -Path $package | Out-Null
foreach ($directory in @('assets', 'notice', 'prerequisite', 'provenance', 'runtime', 'source')) {
  Copy-Item -LiteralPath (Join-Path $baseline $directory) -Destination $package -Recurse
}
foreach ($file in @('resources.json', 'LICENSE-project.txt')) {
  Copy-Item -LiteralPath (Join-Path $baseline $file) -Destination $package
}
Copy-Item -LiteralPath docs/evidence/issue-5/inputs.json -Destination (Join-Path $package 'inputs.json')
Copy-Item -LiteralPath scripts/speech_probe.py -Destination $package
Copy-Item -LiteralPath docs/evidence/issue-6/package-readme.txt -Destination (Join-Path $package 'README.txt')
```

生成与播放使用 Codex 0.167.0-alpha.3.2 的 Windows AppContainer。配置仅作用于本次子进程，不修改网卡、防火墙或其他系统网络设置。`--require-network-denied` 检查外部网络限制；超时或普通连接拒绝不算成功的离线证据。

```powershell
codex sandbox -P issue6 `
  -c 'permissions.issue6.filesystem={":root"="read",":project_roots"="write"}' `
  -c 'permissions.issue6.network.enabled=false' -C $PWD -- `
  .tooling/issue-6/windows-x64-recheck/runtime/python.exe -B -X utf8 `
  .tooling/issue-6/windows-x64-recheck/speech_probe.py `
  .tooling/issue-6/windows-x64-recheck/inputs.json `
  .tooling/issue-6/windows-x64-recheck/assets `
  .tooling/issue-6/recordings-recheck --require-network-denied --play

.tooling/issue-6/windows-x64-recheck/runtime/python.exe -B -X utf8 `
  docs/evidence/issue-6/measure.py .tooling/issue-6/materials-recheck `
  .tooling/issue-6/windows-x64-recheck .tooling/issue-6/recordings-recheck/report.json `
  .tooling/issue-6/windows-x64-recheck.zip
```

输出八个 WAV 和 `report.json`；逐项失败保留错误并返回非零。输出已存在、输入无效、资源缺失 / 损坏或前端覆盖在创建音频目录前报错。[CLI 测试](../../../scripts/test_speech_probe.py)六项在禁网 Windows 进程中通过，覆盖上述失败、输出保留，以及真实 WAV 和报告字段。真实合成测试子进程保持 60 秒限时。

```powershell
$env:LEXIQUE_SPEECH_BUNDLE = (Resolve-Path .tooling/issue-6/windows-x64-recheck).Path
codex sandbox -P issue6 `
  -c 'permissions.issue6.filesystem={":root"="read",":project_roots"="write"}' `
  -c 'permissions.issue6.network.enabled=false' -C $PWD -- `
  .tooling/issue-6/windows-x64-recheck/runtime/python.exe -B -X utf8 `
  -m unittest discover -s scripts -p test_speech_probe.py -v
Remove-Item Env:LEXIQUE_SPEECH_BUNDLE
```

## 检查记录

2026-11-08；完整日志保存在本机忽略目录 `target/issue-6/`。通用 Python 检查使用 uv 的标准 CPython 3.12.10，模型集成检查使用包内 Python 与外部禁网限制。Cargo 检查将 `rustup which cargo --toolchain 1.99.1` 所在目录放在 PATH 首位，采用仓库固定工具链。

| 命令 / 检查 | 结果 |
| --- | --- |
| `pnpm lint`、`pnpm typecheck`、`pnpm build` | 通过 |
| 标准 Python `unittest discover -s scripts -p 'test_*.py' -v` | 103 通过、10 跳过；缺对应资源的 Sherpa / ASR / 来源、Windows release / Opus 以及独立禁网集成前提 |
| 禁网 `test_speech_probe.py` | 六项通过，含真实模型生成与 WAV / 报告检查 |
| `measure.py` 自检及五项失败检查 | 完整材料、声线、WAV、清单与 ZIP 通过；已有 ZIP / 报告 / 清单、缺失材料及同长度损坏材料均明确失败并保留输出 |
| `pnpm test:build-args` | 两项通过 |
| `pnpm rust:test:build` 后 `pnpm test:rust` | 143 项通过；运行器保持 60 秒硬限 |
| `pnpm test:docs`、`pnpm test:docs:validator` | 文档 / 仓库链接与 13 项验证器检查通过 |
| `pnpm test:e2e` | 两次完整运行均为 145 通过、1 失败；`automatic-queue.spec.ts:37` 的 `clock.pauseAt` 报 `Cannot fast-forward to the past`，在队列断言前失败 |
| 同一 T01 单独运行（`--grep 'a continuous event stream' --workers=1`） | 通过；完整并行套件的时钟设置失败仍保留为验证限制 |

## 四目标获取与编译路线

模型与声线文件跨平台复用；完整 Python、NumPy、ORT 和前端依赖需与目标 ABI 匹配。[固定 loader 构建工作流](https://github.com/thewh1teagle/espeakng-loader/tree/0ddc87adf77e5850d7eeb542ac8a87d421b64daa/.github/workflows)给出三个其余目标的 eSpeak 构建配置；[ORT 源码构建入口](https://onnxruntime.ai/docs/build/inferencing.html)及[固定 CPython 源码](https://github.com/python/cpython/tree/v3.12.10)用于没有匹配运行包的目标。

| 目标 | 获取 / 编译路线 | 实机状态 |
| --- | --- | --- |
| Windows x64 | 本次固定官方 CPython amd64 嵌入包、PyPI win_amd64 wheels 与 loader 0.2.4；也可按固定源码使用 VS C++ / Windows SDK 构建 | 八条禁网生成、WAV 回读与同步播放调用通过；音质待试听 |
| Windows ARM64 | 获取相同版本的原生 ARM64 Python / NumPy / ORT 运行依赖；缺 wheel 时按对应源码构建。loader 工作流以 CMake `-A ARM64` 构建 eSpeak 和数据，保留 ARM64 VC 前提 | 路线有上游配置；完整包获取、编译、链接、禁网生成与播放未实机验证 |
| macOS x64 | 匹配 CPython 3.12、NumPy / ORT wheels 或源码；固定 loader 工作流设置 `CMAKE_OSX_ARCHITECTURES=x86_64` 构建 eSpeak；系统播放入口可用 `afplay` | 未实机验证 |
| macOS ARM64 | 匹配原生 ARM64 Python / NumPy / ORT；固定 loader 工作流设置 `CMAKE_OSX_ARCHITECTURES=arm64`，原生构建前端；播放入口可用 `afplay` | 未实机验证 |

## 限制与剩余验收

人工质量尚未通过，尤其是孤立短词、变位、重音和连读。`PlaySound` 返回成功证明原生播放调用完成，不证明扬声器听感；浏览器音频解码未在完整媒体运行时验证。其余三个目标需要各自的平台播放、内存测量与禁网环境；当前 CLI 的这些 API 仅支持 Windows。

官方声线字节身份已证实，但 int8 ONNX 的量化 / 校准过程未独立复现；固定 wheel 与上游源码的精确构建对应、完整静态编译成分及最终 GPL / 数据 / 模型再分发义务仍待发行核验。GPL 前端使整个组合不能笼统标为 MIT。VC 安装器未执行，干净机器首次运行未验证。独立包不含桌面应用和 WebView2，不能用其 ZIP 体积证明产品 M5 下载量；#6 保留待试听状态，其他 TTS 候选及 M1 可独立推进。
