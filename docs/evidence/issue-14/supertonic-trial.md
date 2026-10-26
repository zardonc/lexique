# MLS 否决后的 Supertonic 3 小批试听

2026-10-26：用户否决全部 MLS 样本的法语读音和音质，已撤回该候选。许可证据与有效 WAV 不能覆盖人工否决；本轮换用不同引擎作小批筛查，没有修复或重新认可 MLS，也没有接入产品 TTS。

## 当前试验

- 资源：sherpa-onnx 维护者 Supertonic 3 int8 发布包；压缩 128,774,318 字节，七份必要文件 145,295,768 字节。独立核对发布包 SHA-256、七文件与固定 HF 镜像摘要，全部通过。来源及边界见[研究](replacement-options-after-mls.md)、[清单](supertonic3-model.json)和[许可归档](supertonic-licenses/ATTRIBUTION.md)。不是最终桌面完整下载量。
- 隔离运行库：`sherpa-onnx==2.0.1`、`sherpa-onnx-core==2.0.1`，不进入产品依赖或锁文件。Windows x64 CPU，两线程，八步去噪。
- 女声 F1=`sid 0`，男声 M1=`sid 5`；正常 `speed=1.0`，慢速 `0.8`。24 个法语样本加 2 个英语控制，共 26 个 44,100 Hz 单声道 PCM16 WAV。12 对法语样本逐条慢速更长，总时长倍率均约 `1.25`。
- 进程前后联网探测均返回 WinError 10013；26 次同步 Windows PlaySound 调用完成。加载约 1.10 秒，单次生成约 0.44–0.80 秒，峰值工作集 316,956,672 字节。这些是本机技术观测，不是声音正确的证明。
- [试听页](supertonic-samples/listen.html)清楚标明机器合成，提供原文、四组合及本地 ASR 对照。`y`／`i` 是普通文本对照，不是音素控制或已实现的多读音策略。

这是 **部分筛查**：只覆盖 `école`、`œil`、`nous sommes ici`、`J'apprends le français tous les jours.`、`y`、`i`；通过人工试听后仍需运行原 #14 全部 17 项、未知词与其他边界。短词、孤立 y 和自然度没有通过验收。

## 实际发现的语言参数问题

官方 Python 示例逐项修改 `GenerationConfig.extra["lang"]`。在本机 2.0.1 绑定中，此 getter 返回副本，修改后读取仍为 `{}`。先加“语言必须传入原生配置”的断言，真实 CLI 退出 1，复现该问题；改为完整赋值 `config.extra = {"lang": lang}` 后通过，断言保留在[探针](probe_supertonic.py)。该问题属于新引擎调用，不是旧 Piper MLS 失败的根因结论。

首轮丢失语言的样本保留在忽略目录 `run-01`，语言回归失败目录为 `run-02-language-repro`；当前公开样本仅来自 `run-03-fr`，未混用早期输出。生成过程没有随机选择模型或转换音频播放速度。

## 本地识别对照

[筛查脚本](screen_speech_asr.py)使用固定[Whisper tiny int8 资源](asr-model.json)，仅作临时诊断，不随产品分发。相同目标句 `J'apprends le français tous les jours.` 的结果为：

| 样本 | 识别结果 | 归一化词错误率 |
| --- | --- | ---: |
| 已试听 SIWIS | J'apprends le français tous les jours | 0 |
| 已试听 Tom | J'apprend le France et tous les jours. | 0.429 |
| 已否决 MLS 女声 | Je vais vous remerter tout, je vais vous remerter tout. | 1.429 |
| 已否决 MLS 男声 | J'apprends cette route, c'est tout, elle joue. | 1.000 |
| Supertonic 女声正常 | Je apprend le français tous les jours. | 0.286 |
| Supertonic 女声慢速／男声两档 | J'apprends le français tous les jours. | 0 |

词错误率包含插入项，可超过 1；只比较固定句子，不推广为模型总体分数。ASR 对多个短词和 `y`／`i` 不稳定，可能错拼或幻觉，不能据此确定其实际发音。完整 28 项对照见[机器报告](supertonic-samples/asr-screen.json)。归一化的重音与 œ 自检先失败后修正，通过后重跑全部对照；未手动修饰识别转写。

复验命令（使用未存在的输出路径，并在禁网受限进程内运行）：

```powershell
.tooling/issue-14/comparison/venv/Scripts/python.exe -X utf8 docs/evidence/issue-14/probe_supertonic.py .tooling/issue-14/supertonic-3/sherpa-onnx-supertonic-3-tts-int8-2026-05-11 .tooling/issue-14/supertonic-3/run-new
.tooling/issue-14/comparison/venv/Scripts/python.exe -X utf8 docs/evidence/issue-14/screen_speech_asr.py .tooling/issue-14/asr-check .tooling/issue-14/supertonic-3/run-new .tooling/issue-14/supertonic-3/asr-new.json
```

模型许可允许有条件再分发，实际发行仍须落实 OpenRAIL-M 用途条款、通知与修改说明，以及全部运行依赖；本次只准备材料。音质待人工确认，完整包体、干净机器、其他平台及产品合成／缓存均未验收，#14 保持未完成。
