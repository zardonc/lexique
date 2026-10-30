# 法语模型、读音与音频格式对比

2026-10-26，接续 #14。用户要求比较其他模型、核查压缩格式，并澄清音频保存行为。
本次是隔离实验，没有修改产品 TTS、数据库、缓存、IPC 或清理策略。

## y 的读音范围

`y` 作代词 / 副词时为 `/i/`，字母名称为 `/iɡrɛk/`。
依据 [Larousse 代词 / 副词词条](https://www.larousse.fr/dictionnaires/college/y/42887)和
[法兰西学术院字母词条](https://www.dictionnaire-academie.fr/article/A9Y0001)。
此前把读成字母名笼统列为“发音问题”范围过宽；现有探针的准确小写 `y → i` 修正只表示
词汇代词控制，不是最终产品的全部多读音规则。没有删除原字母名样本，也没有在产品中应用硬编码。

两套 Piper 在原样输入 `y` 时也输出字母名音素。本次分别保留原样 `y` 和显式 `/i/` 控制，
不把后者冒充自动识义结果。词典页可以按用法展示两种读音；未区分用法就将两种声音连播作为
听音卡题面，会影响题义。义项选择、读音显示和是否连播仍待产品确认，本轮没有替用户冻结规则。

## 模型试听与复验

[打开完整试听页](comparison/listen.html)。页面包含 17 个同文本输入的三模型对照、
单独的 `/i/` 控制，以及同一个 école 的五种编码。所有播放器均有标签；没有自动播放。

| 组合 | 模型权重 | 输出采样率 | 本次 CPU 加载 | 单项生成范围 |
| --- | ---: | ---: | ---: | ---: |
| Kokoro v1.0 int8 / ff_siwis | 92,361,271 字节 | 24,000 Hz | 1.27 秒 | 0.112–0.394 秒 |
| Piper 1.8.0 / SIWIS medium | 63,201,294 字节 | 22,050 Hz | 3.02 秒 | 0.018–0.136 秒 |
| Piper 1.8.0 / Tom medium | 63,511,038 字节 | 44,100 Hz | 2.62 秒 | 0.025–0.134 秒 |

Kokoro 沿用前轮记录；Piper 同一 Windows x64 / Ryzen 9 8945HS / 16 逻辑 CPU 运行。
采样率、模型、默认响度归一化和合成机制不同，没有人为匹配响度、统一噪声或设通过阈值；
上述不是受控性能排名，也没有由耗时推断哪种音质更好。模型文件体积不是完整运行包体积。

两套 Piper 各完成 17 个准确输入及 1 个显式音素控制，均生成非静音有效 PCM WAV，
同步 Windows PlaySound 调用完成。生成进程前后对两公网 IP:443 的探针均为 Windows 10013。
未使用缓存 / 预生成音频代替推理，未修改系统网络。

来源、哈希与结果：

- [固定模型 / 配置清单](piper-models.json)，上游快照 `5041a7a49ae5b3a0f30e6837769eb03fdbb231bd`。
- [SIWIS 逐项报告](comparison/siwis/report.json)、[Tom 逐项报告](comparison/tom/report.json)。
- [比较脚本](compare_piper.py)：使用 [Piper 官方公开 API](https://github.com/OHF-Voice/piper1-gpl/blob/main/docs/API_PYTHON.md)，未复制上游私有实现。
- 运行时依赖版本在各 report 中；原始模型只保存在忽略的 `.tooling/issue-14/comparison/models/`。

Piper 代码采用 GPL-3.0；[SIWIS 模型卡](https://huggingface.co/rhasspy/piper-voices/blob/5041a7a49ae5b3a0f30e6837769eb03fdbb231bd/fr/fr_FR/siwis/medium/MODEL_CARD)
列出 CC BY 4.0 数据及 Lessac 微调来源，既有的最终权重授权链疑问仍保留。
[Tom 模型卡](https://huggingface.co/rhasspy/piper-voices/blob/5041a7a49ae5b3a0f30e6837769eb03fdbb231bd/fr/fr_FR/tom/medium/MODEL_CARD)
指向 AGPLv3 数据来源。本次是本地比较，不把模型或配置纳入默认分发包，不将模型卡的数据许可
自动当成最终权重授权。完整来源与再分发义务仍须逐项闭合；当前没有新增已准入默认模型。

复验（现有 Python 3.12，联网只用于准备；生成命令必须在外部禁网环境运行）：

```powershell
$env:UV_CACHE_DIR = "$PWD/.tooling/uv-cache"
uv venv --python 3.12 .tooling/issue-14/comparison/venv
uv pip install --python .tooling/issue-14/comparison/venv/Scripts/python.exe piper-tts==1.8.0 imageio-ffmpeg==0.6.0
New-Item -ItemType Directory .tooling/issue-14/comparison/models
Get-Content docs/evidence/issue-14/piper-models.json -Raw | ConvertFrom-Json | ForEach-Object {
  Invoke-WebRequest $_.url -OutFile (Join-Path '.tooling/issue-14/comparison/models' $_.file)
}
$env:TEMP = "$PWD/.tooling/issue-14/comparison"
$env:TMP = $env:TEMP
.tooling/issue-14/comparison/venv/Scripts/python.exe -X utf8 docs/evidence/issue-14/compare_piper.py `
  .tooling/issue-14/comparison/models/fr_FR-siwis-medium.onnx .tooling/issue-14/comparison/siwis-new
.tooling/issue-14/comparison/venv/Scripts/python.exe -X utf8 docs/evidence/issue-14/compare_piper.py `
  .tooling/issue-14/comparison/models/fr_FR-tom-medium.onnx .tooling/issue-14/comparison/tom-new
```

## 音频文件大小

对既有 17 个 Kokoro WAV 编码，不重新推理、不改变源采样率 / 声道；总原时长 11.568 秒。
单位为文件逻辑字节，不是磁盘簇分配量。短音频的容器开销不可忽略。

| 格式 / 设置 | 17 文件合计字节 | 相比 PCM WAV 节省 | 验证 |
| --- | ---: | ---: | --- |
| WAV，单声道 24 kHz / PCM 16-bit | 556,012 | — | 原始基准 |
| FLAC，无额外 metadata padding | 336,085 | 39.6% | 解码 PCM 逐字节相同 |
| Ogg Opus，24 kbit/s，voip | 37,187 | 93.3% | 全部解码成功，解码样本数一致 |
| M4A / AAC-LC，32 kbit/s | 65,669 | 88.2% | 全部解码成功；部分文件解码尾部填充最多 872 样本 |
| MP3，48 kbit/s | 79,612 | 85.7% | 全部解码成功，解码样本数一致 |

只有 FLAC 是无损；解码成功和时长相符不证明有损格式的语音细节可接受。
Opus 的方向适合继续评估，[官方说明](https://opus-codec.org/)涵盖低码率语音与存储用途。
建议首先试听 Opus 24 kbit/s，必要时提高码率；AAC / MP3 作为兼容性比较项，不在本轮冻结格式。

[格式结果](comparison/formats/report.json)含每文件大小、哈希、解码样本数差异和编码参数。
[格式脚本](compare_formats.py)使用 imageio-ffmpeg 0.6.0 附带 FFmpeg 7.1；二进制 SHA-256 在 report。
这套工具仅供实验，未加入产品依赖或默认包，后续仍须按实际编码库审查许可与分发义务。

```powershell
.tooling/issue-14/comparison/venv/Scripts/python.exe -X utf8 docs/evidence/issue-14/compare_formats.py `
  .tooling/issue-14/comparison/formats-new
```

59 个 HTML audio 控件已在本机 Chrome 146 的独立上下文触发 `canplay`，零失败；五种格式的
école 均可解码，Opus 容器报告时长略含编码前跳量，而解码回原采样率的样本数一致。
这是 Chrome 的文件解码检查，不是 Tauri WebView2、macOS、Windows ARM64 或听感验收。
既有 Windows PlaySound 只用于 WAV；产品接入压缩文件还需要对应的播放器 / 解码边界。

## 当前行为与拟议保存规则

当前应用还没有 TTS 音频生成 / 播放入口。#14 探针及本次比较会明确写出音频文件，
没有播放后删除或退出清理；`.tooling/` 被 Git 忽略不等于自动清理，已保存证据也继续保留。

M2 的“再次加入只补缺失 / 失败音频”表示期望复用已有音频；但实际落盘结构、缓存容量和
清理政策未实现，不能声称应用已经采用某种持久化策略。

建议，尚未实施或确认：

- 生词本 / 听音卡音频作为可重建的持久缓存，重启后复用，播放完不删除；与学习进度分开。
- 临时查词音频可采用有容量上限的缓存；清理时不删除卡片、评分或笔记。
- 成功写入并验证最终压缩文件后再清理本次生成的中间 WAV，不在转换成功前删除唯一可用音频。
- 缓存身份包括文字、读音变体、声线及模型版本，避免两个 y 读音共用一个键，或升级后误用旧缓存。
- 所有运行目录遵守 Windows portable 的 `Lexique-data/` 边界；容量和自动清理触发条件留待产品确认。

## 检查边界

两套 Piper 的 36 项生成 / WAV / 播放断言通过；68 项编码均成功解码，17 项 FLAC PCM 相等；
59 个浏览器音频控件可解码。本次只新增实验脚本和证据，未改应用依赖、生产代码、学习数据或旧交付。
新脚本通过实际运行自检；文档链接、JSON、样本哈希及 diff 检查通过。
未重跑无变化的前端 / Rust 全套，未建立压缩缓存或多读音产品功能，未作最终主观音质、完整包体或四平台准入。
