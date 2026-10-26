# M2 单词离线语音验证

工作记录，2026-10-26。候选为 Kokoro v1.0 int8 + ff_siwis +
kokoro-onnx 0.6.1 + eSpeak NG 1.52.0。技术生成不等于音质通过；
本票保持打开，等待修正后完整样本的最终试听验收。

## 复验

`resources.json` 固定每个下载文件的来源、版本身份、SHA-256、字节数和分别许可。
原始大文件与运行环境在忽略目录 `.tooling/issue-14/`；不加入 Git，也不写入用户库。
使用 Python 3.12 运行以下准备步骤；准备阶段允许联网，生成阶段禁止网络。

```powershell
$materials = '.tooling/issue-14/materials'
New-Item -ItemType Directory $materials
Get-Content docs/evidence/issue-14/resources.json -Raw | ConvertFrom-Json | ForEach-Object {
  Invoke-WebRequest -Uri $_.url -OutFile (Join-Path $materials $_.file)
}
python -X utf8 docs/evidence/issue-14/prepare.py $materials .tooling/issue-14/bundle
```

准备程序拒绝已有输出，校验所有文件后解包，生成逐文件 `inventory.json` 与完整 ZIP，
最后检查 ZIP CRC、记录压缩 / 解压字节数与 SHA-256。可变下载地址未来变化时校验失败，
不能悄悄接受新文件；VC++ 安装器尤其可能更新。

在**外部已限制网络的 Windows 进程环境**内运行：

```powershell
$env:TEMP = "$PWD/.tooling/issue-14"
$env:TMP = $env:TEMP
.tooling/issue-14/bundle/runtime/python.exe -X utf8 scripts/speech_probe.py `
  docs/evidence/issue-14/inputs.json .tooling/issue-14/bundle/assets `
  .tooling/issue-14/recheck --require-network-denied --play
```

输出目录必须不存在。工具自身不设置防火墙；`--require-network-denied` 只检查同一进程
前后对两个公网 IP:443 的连接均返回 Windows 10013。超时、DNS 失败、拒绝连接不算禁网。
本次使用 Codex Windows 受限 AppContainer，未修改系统防火墙、网卡或其他进程。
设置 TEMP / TMP 是因为 phonemizer 会临时复制 eSpeak DLL；限定在本票目录，退出后无常驻进程。
离开该环境复验需要等价的外部进程网络限制，不能去掉该参数后宣称离线通过。

`--play` 调用同步 Windows `PlaySound`，禁用找不到文件时的默认提示音。
API 返回记录不证明扬声器实际可听或法语质量；必须另行试听。
工具不读取缓存、预生成音频或学习库，只从固定本地模型重新推理并创建新 WAV。

## 样本与失败边界

`inputs.json` 前 8 项逐字保留 #5–#7 的统一样本 v1；后 9 项为 #14 增补：
孤立短词 a、à、y、eu、eau、un、chat；人为未知词 qzxyplouf；长词
anticonstitutionnellement。输入不补句、不补标点；短句仅保留既有对照，不扩展例句朗读产品。

输入允许 1–200 字符的拉丁字母、组合符、空格和有限法语标点，拒绝控制字符、数字、
表情与其他文字系统。未知拼写仍会由 G2P 推断音素，不保证词典收录或发音正确。
工具保留未过滤音素；显式移除 eSpeak 的法语连读 / 词间 `-` 分隔符，
其他未知音素及空音素报错，不沿用上游静默丢弃行为。上游对长音素串按最大 510 音素分块；
本工具限制的是输入文本 200 字符，不把它误称为 510 音素硬限。

每项单独记录输入、前端音素、模型音素、有效 PCM WAV 回读、哈希、耗时和播放结果；
单项失败保留错误并使进程非零退出。资源缺失 / 校验失败不生成输出，不回退到在线服务。
未知音素检查不是完整语言识别，前端自行略去字符仍需音质 / 覆盖验收。

初次发现孤立 `y` 的前端输出 `iɡʁˈɛk`，即字母名，而非代词 / 副词读音。
用户确认声音可继续评估、先修复 y 再验收。按词汇语境为准确输入小写 `y` 显式指定 `/i/`，
依据 [Larousse 的代词词条](https://www.larousse.fr/dictionnaires/college/y/42887)；
报告保留前端原输出及 `pronunciation_override`，不改输入、不拼句、不使用预生成缓存。
这是单词词汇用法修正，不是字母名称发音功能；大写 Y 或句内 y 不套用此孤立词修正。
修正前音频保留为 `samples/11-before.wav`，对应事实见 `y-before.json`，修正后为 `samples/11.wav`。
用户尚未确认修正后的完整样本，不能解除 #14 的集成阻塞。

## 许可与资源身份

- 模型：Kokoro-82M v1.0，官方快照 `f3ff3571791e39611d31c381e3a41a3af07b4987`。
  [官方模型卡](https://huggingface.co/hexgrad/Kokoro-82M/blob/f3ff3571791e39611d31c381e3a41a3af07b4987/README.md)
  明确权重 Apache-2.0，并说明许可训练来源；不是仅凭外层代码许可证推断。
  量化 / ONNX 转换来自 kokoro-onnx `model-files-v1.0`；保留导出工具源码和模型卡，
  转换资源继续按模型 Apache-2.0，不能按转换器 MIT 重新授权。
- 声线：同一模型仓库的 `voices/ff_siwis.pt`。与 `voices-v1.0.bin` 内 ff_siwis 的
  float32 `(510, 1, 256)` 张量逐字节比较一致。原始 PT 随证据保留；运行时不依赖 PyTorch。
  同包其他声线不使用。声线 / 权重按官方模型授权，SIWIS 训练数据另保留 CC BY 4.0
  来源署名，不将其混同 Piper SIWIS 从其他权重微调的授权链。
- [官方声线说明](https://huggingface.co/hexgrad/Kokoro-82M/blob/f3ff3571791e39611d31c381e3a41a3af07b4987/VOICES.md)
  提醒短文本弱点；它不能替代本次单词音质验收。
- 转换与推理封装：kokoro-onnx 0.6.1，MIT；源码快照
  `3596b26764286a7de9d90c363e988d50578918e5`，锁定 PyPI wheel；保留原许可。
- 前端：phonemizer 3.4.0，GPL-3.0-or-later；附 PyPI 对应源码及许可证。
  espeakng-loader 0.2.4 包装代码 MIT；其 DLL 和数据分别适用 eSpeak NG GPL 及数据 notices。
  DLL 自报 1.52.0，loader 构建工作流固定上游 1.52.0，附该版本完整源码及 loader 构建脚本。
  loader 源码快照 `0ddc87adf77e5850d7eeb542ac8a87d421b64daa`。
  探针拒绝 `PHONEMIZER_ESPEAK_LIBRARY` 覆盖，构造前验证固定 DLL / 数据树哈希并预加载
  指定 DLL；报告记录实际路径、哈希与自报版本，加载失败不进入上游系统回退。
- 运行库：CPython 3.12.10 官方 Windows x64 embeddable（PSF 与内含 notices）、
  ONNX Runtime 1.31.0（MIT 与 ThirdPartyNotices）、NumPy 2.5.4（BSD 及捆绑组件许可）；
  其他依赖逐项列于 manifest，wheel 内 dist-info licenses 原样保留。
- 发行义务：保留模型 Apache、训练数据署名及各依赖 notices；前端 GPL 对应源码、
  构建步骤及可替换性须随交付保留。探针组合含 GPL 组件，不能将整个组合标成 MIT。
  本项目自有脚本仍按项目 MIT；未将 GPL DLL 链入 Tauri，后续产品集成需要明确分发边界。
- `prerequisite/vc_redist.x64.exe` 为微软官方独立离线安装器，受微软条款约束，
  不自动安装；本机已有系统运行库。干净机器的安装、许可展示及重启行为未实测。

原始许可 / 来源文档放在包内 `notice/`，对应源码在 `source/`，逐文件哈希在
`inventory.json`。这是一套本地验证交付，不是默认产品资源准入或正式发行声明。

## 其他目标路线

| 目标 | 获取 / 构建路线 | 本票空缺 |
| --- | --- | --- |
| Windows x64 | 本清单 CPU ONNX Runtime、eSpeak DLL、CPython embed | 有本机运行；无干净机器验收 |
| Windows ARM64 | 上游 loader 工作流有 `-A ARM64`；ORT 官方 ARM64 原生库；可用 Rust/C API 驱动相同 ONNX 与声线 | 当前 cp312 x64 wheels 不可复用；整套绑定、构建及实机未验 |
| macOS x64 | ORT CPU wheel / 原生库、loader 的 x86_64 dylib；CPython 框架或独立原生驱动 | 未获取固定清单、未编译链接、未生成或播放 |
| macOS ARM64 | ORT CPU wheel / 原生库、loader 的 arm64 dylib；相同模型与声线 | 未获取固定清单、未编译链接、未生成或播放 |

以上路线来自选定上游源码与分发说明，只是获取线索；不等于三目标可交付。
未规定性能通过阈值，未实现完整变位或例句朗读、桌面 IPC、评分 UI、缓存或 M5 发行。

## 本机结果

初次严格检查保留在 `first-attempt.json`：15 项生成，两项对照短句因前端 `-` 被拒绝。
显式处理分隔符后 17 项生成 / WAV 回读 / 同步播放调用通过；嵌入式 CPython 完整包重新
禁网生成与播放结果见 `report.json`，所有音质判定仍待用户试听。

原始 PT 的 tensor storage 与转换声线字节一致的检查已执行；首次外连探针和包内运行前后
外连均得到 10013；不是单凭没有网络日志判断离线。y 修正后完整 17 项再次生成并播放；
其余 16 项 WAV 哈希不变，y 的变化有单独回归覆盖。

下载材料总量 215,446,330 字节，包含解包后重复的 wheels 和 Python ZIP。
最终去重压缩交付量、安装文件总量、主包及 VC++ 明细见 `measurements.json`；
完整 ZIP 含模型、声线、前端、运行库、许可、GPL 源码及 VC++ 离线安装器。
安装量按文件逻辑字节求和，不是 NTFS 分配空间，也不包括系统现有 VC++ 安装占用。
本票不含 WebView2：独立探针不使用 WebView；与现有桌面包合并时另计，缺 WebView2 的
完整首装总量未测，不能以本票 ZIP 小于 200 MB 宣称 M5 包体通过。

本机为 Windows 11 x64 build 26300、16 逻辑 CPU，CPU 标识及峰值工作集记录于报告；
硬件营销型号和物理内存另记录于 measurements。计时是本进程测量：加载从 Kokoro 构造器
开始（不含 Python import），首段为完整首个 WAV，没有流式首块；生成计时含 G2P、推理、
PCM 编码及写入回读，不含播放。工作集为本进程系统峰值，不等于模型权重或整机内存。

工具检查：先运行 CLI 回归看到红灯，再实现得到绿灯；增加真实 CLI 集成检查：

```powershell
$env:LEXIQUE_SPEECH_BUNDLE = "$PWD/.tooling/issue-14/bundle"
python -X utf8 -m unittest discover -s scripts -p test_speech_probe.py -v
```

集成项需要真实资源及外部网络限制，缺少时明确跳过，其余输入 / 缺资源 / 校验失败 /
拒绝覆盖检查无需模型依赖。检查过程与最终结果见 `checks.md`。
