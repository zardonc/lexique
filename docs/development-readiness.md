# 开发链路与外部资源准入清单

更新：2026-11-09。依据：`Requirements-Consolidated.md` v1.0、当前仓库及用户确认。需求材料中的建议与网页示例不自动成为已确认选型；外部资料不能代替本项目运行验收。

## 已确认的边界

1. 按下述 M0–M6 顺序推进。中间里程碑是开发验收，桌面正式发行仍包含已要求的默认词典、完整约定变位和法语离线语音。
2. 许可允许时，内置词典及变位可转换为只读结构化资源，优先评估 SQLite；具体格式尚未冻结。自定义 MDX/MDD 支持保留，资源与用户学习数据分别管理。
3. 默认资源需要明确允许目标转换与随包再分发，并且没有非商业用途限制；可接受署名、来源保留及数据的相同方式共享条件。代码、文字、媒体、模型和声线分别核查。
4. 项目自有源码采用 MIT，第三方内容不因此改为 MIT。
5. 每个平台 / 架构分别提供完整版与 Lite 版。完整版内嵌 TTS 模型，不受 200 MB 安装包上限限制，仍应尽量减少磁盘占用；Lite 版不内嵌 TTS 模型，由用户通过“额外资源”主动下载，保留既有包体目标。默认词典、变位及其他资源不变。分别报告压缩包、安装占用、模型下载及系统运行依赖；两版与下载功能尚未实现。
6. Windows 11 主包优先使用已有 WebView2；为缺少运行时的机器提供微软官方离线依赖包与检测提示。分别报告主包、依赖包和缺依赖机器的完整离线下载总量。
7. 首期 M3 纳入其他应用选词、屏幕区域 OCR、词频颜色和 WPM；词频采用 Lexique 3.83 字幕词元频率及产品 100／10 分档。WPM 为朗读速度，范围 80–240、步长 20、默认 180。四项已完成 Windows x64 检查，见[新增交付计划](lookup-notes-speech-plan.md)；其他平台测试按用户要求暂缓。

相关决策见 [资源与学习数据分离](adr/0001-separate-bundled-resources-from-learning-data.md)、[源码与资源许可](adr/0002-source-and-resource-licensing.md)。

已接入的 Supertonic 3 及 M5 完整版 / Lite 发行必须落实以下条件：随模型分发 OpenRAIL-M 全文，保留适用的署名、修改说明及依赖通知；在约束模型使用 / 分发的协议中传导 Attachment A 用途限制；发音和听音卡界面披露语音为机器合成。Windows x64 已有产品接入及 #38 新鲜包证据；具体正式交付方式、其他平台与干净机器仍须验收。[许可证据及边界](evidence/issue-14/supertonic-licenses/ATTRIBUTION.md)。

## 已选 TTS 与历史候选证据

2026-11-08 用户决定继续使用 Supertonic 3，停止其他 TTS 模型测试，见 [ADR 0009](adr/0009-continue-with-supertonic-3.md)。#5 保留已完成的独立对照；#6 为已测试但音质不满足要求；#7 为未测试、取消计划。后续产品开发不等待候选比较，原始探针、录音、报告与许可材料继续保留。

既有比较与 Supertonic 3 的发行记录分别保留下列三项结论；尚未执行或未决时列出缺少的证据，完成后引用对应报告。独立本地对照与应用随包的用途、合法获取／使用依据以及模型、声线、前端、运行库和源码交付义务分别核查。

| 结论 | 完成证据 |
| --- | --- |
| 离线可行性 | 固定完整组合与平台，在禁网进程生成有效音频并实际播放，保留准确输入、时间／内存和完整获取／安装体积 |
| 法语质量 | 对同版固定输入保存录音与用户试听结论，单独标注短词、词尾、变位和连读；波形／ASR 回归只证明其覆盖的行为 |
| 计划再分发 | 模型、声线、前端及运行库允许目标转换与交付，备齐适用的源码、署名、通知和下游条款；缺口归属具体组合及交付阶段 |

发布者答复是一种证据来源，也可依据许可原文和公开源码独立判断条件；确需澄清时先准备本地问题。每张票按自身验收范围判定关闭，未解的再分发条件保留为该组合的发行门槛。停止比较不豁免 Supertonic 3 的许可、干净机器、其他平台或 M5 验收。

## 开发链路与验收

| 里程碑 | 可验收结果 | 前置条件 |
| --- | --- | --- |
| M0 资源与原生基线 | 资源准入小样、可重复原生构建与启动证据；变位 / TTS 有独立可行性验证计划 | 合法真实数据样本、依赖获取环境、可用 Rust / MSVC / SDK；关键资源有可核验的获取与许可路径 |
| M1 第一条本地链路 | 离线查询 → 创建笔记 → SQLite 保存 → 关闭应用并重启后恢复同一内容 | 至少一个已准入词典小样；明确来源引用、内容快照与笔记身份；原生读写链路可运行 |
| M2 复习核心 | 支持用户选择或开启收藏自动加入后建卡；识义卡与听音卡分别评分、调度及恢复，支持撤销；听音卡有真实单词离线发音；相同固定输入在原生与实际 WASM 中一致 | 按已逐项确认的规则先验证 `fsrs-rs v6.6.2` 原生 / 实际 WASM 及语音可行性，再接完整持久化与 UI；首条运行验收为 Windows x64 及实际 WASM；语音许可、法语质量、禁网播放与包体均须有证据 |
| M3 完整离线核心 | 约定变位反查／全部匹配与离线发音；共用 WPM、词频颜色、其他应用选词及屏幕区域 OCR 查词 | 新增能力按各自资源／平台准入及原生验收；词典／分级分别达到约定覆盖，例句复用 WPM |
| M4 外部能力 | 翻译 / AI 可用；应用内管理数据，跨云文件后台批量同步及失败恢复可验收 | 服务与凭据方案、测试云目录、事务 outbox、身份 / 因果 / 删除 / 恢复、来源认证和安全 GC；Drive / OneDrive 适配分别验收，两条业务能力可独立开发 |
| M5 桌面发行 | Windows / macOS × x64 / ARM64 × 完整版 / Lite 共八种发行物；完整版首装断网语音、Lite 主动下载后断网语音、资源体积与许可声明可复验 | 四架构实际运行渠道、最低 macOS、安装器 / 签名 / 公证策略、完整资源准入与离线依赖 |
| M6 浏览器版 | 移动浏览器实际调用 WASM 完成闪卡复习，确认后强关恢复，再以同一文件协议延续进度 | IndexedDB 业务 / outbox 原子事务、浏览器持久性与配额限制、多标签页 / 升级、OAuth 回调、BFF 访问控制及部署 origin 恢复 |

M0 进入首条链路的硬门槛是一个合法真实词典小样和可运行原生基线，当前已通过。后续默认词典／分级覆盖、剩余离线模块与已选 Supertonic 3 的发行准备按各自门槛推进；未通过时不能承诺该模块或正式发行已具备能力。

2026-10-25 M2 访谈 Q1–Q42 已逐项确认，详见[M2 确认稿](m2-review-core.md)；后续已发布[规格和工单](m2-ticket-plan.md)。卡片身份、创建与重试、精简卡面、部分例句保存、暂停 / 恢复、播放失败及跨重启一级撤销的规则已记录。缺例句允许复习；缺音频的听音卡保留进度并标“待生成语音”，可用前不进入可评分队列。再次加入不重置进度或解除主动暂停。

FSRS 使用固定版本默认参数，不提供优化与额外间隔随机扰动；两类卡共用 70%–99%、默认 90% 的保持率，修改只影响后续评分。本地凌晨 4 点换学习日；长期间隔取整为 1–36,500 天并在目标学习日凌晨 4 点到期，分钟步骤保持完整间隔。每日新卡上限默认 20、复习上限默认 50，两项可为 0；新卡默认同时受复习上限约束，可开启忽略开关。新卡首次成功评分计新额度，跨日学习／长期复习占复习额度，日内学习重复不限额；当天改设置保留已用次数。同词两卡不互相暂缓，独立排程。验证覆盖取整、换日、DST／换区、时钟倒退、额度／撤销、幂等重试、编辑保护、播放失败及队列恢复，沿 [ADR 0005](adr/0005-independent-progress-for-vocabulary-card-types.md)、[ADR 0006](adr/0006-learning-days-and-absolute-review-times.md)和 [M2 规则](m2-review-core.md)。

实施先验证核心 / 实际 WASM 及离线语音可行性，再接完整存储与 UI。#13 与 #14 已按各自范围验收，Supertonic 3 的已接受缺口及实际运行证据见[准入决定](evidence/issue-14/README.md)。Windows M2 产品接入及 #24 优化已完成约定验收；正式分发义务、完整包体和其他目标实机仍待验证，不能据此宣称 M5 完成。

2026-11-07，#5 的 Sherpa 2.0.1 / 外部 eSpeak / Piper Tom 固定组合完成 Windows x64 独立可行性验收：禁网生成、同步播放与八条人工试听通过。独立软件来源包、原始模型 / 双配置、本次转换与构建控制脚本已准备，完整获取量 255.44 MB；[证据与门槛](evidence/issue-5/README.md)。应用随包语音沿用现有 Supertonic 3，其商业再分发许可与现有打包通知路径见[调查](evidence/issue-5/redistributable-french-models.md)；OpenRAIL-M 下游条款、正式完整版／Lite 交付、其他平台和干净机器首装继续在 M5 验收。Tom 的首选修改形式、源码重建及确切上游 wheel 构建来源缺口按该对照组合记录。

2026-10-29 更新：#15 的单个生词建双卡、真实语音开发包、卡面与 Windows x64 重启恢复已接入并验证，见 [#15 证据](evidence/issue-15/README.md)。新生成音频由用户试听判定；本轮完整桌面实播没有施加网络限制，既有 #14 的禁网证据单独保留。完整评分 / 队列 / 撤销、连续 24 小时清理实测、其他目标及正式分发义务仍待后续，不宣称完整 M2 或 M5 通过。

## 外部资源清单

| 资源 | 待验证主候选与证据 | 必须确认的内容 | 截止时点 |
| --- | --- | --- | --- |
| 法法、法英、法中词典 | Kaikki / Wiktionary 三种释义语言均有可评估来源 | 固定快照、许可及署名；分别核验释义语言、常用词覆盖、例句、词形 / 词义去重与转换后体积 | M1 前至少准入一个真实小样；完整三语内容在词典模块及 M5 验收 |
| 完整变位与反查 | Lefff 3.4、RosaeNLG 的 Lefff 衍生数据与离线生成方案 | 数据 LGPL-LR 条款、语式 / 时态矩阵、助动词、代词式、缺陷动词、多匹配及生成样例 | M3 开发前确认覆盖与生成路径；M5 全量验收 |
| A1–B2 分级词表 | 尚无符合当前政策且已准入的完整来源；FLELex 为 CC BY-NC-SA 4.0 | 允许目标发行的来源或额外授权；词元 / 多词表达、分级依据、数量与质量 | 分级模块开发前；不阻塞 M1，仍是默认词库发行门槛 |
| 生词词频颜色 | [Lexique 3.83 原作者版本](https://chrplr.github.io/openlexicon/datasets-info/Lexique383/README-Lexique.html)，数据 CC BY-SA 4.0；选择 `freqlemfilms2` | 完整源、词元／词性去重合计、100／10 及未知、通知、覆盖、体积和原生恢复见 [#57](evidence/issue-57/README.md) | Windows x64 转换／界面／开发包已验收；不代替分级词表，其他平台与 M5 另验 |
| 屏幕／图片 OCR | 固定 Tesseract.js／core 7.0.0、tessdata_fast 法语 `fra`、XCap 0.9.9；来源与法律原文见[资源目录](../resources/tesseract-fra/README.md) | 完整本地运行库、输入上限、重音／撇号、125%／175% 负坐标双屏和开发包通知通过，见 [#59](evidence/issue-59/README.md) | Windows x64 准入及产品流已验收；其他平台实际执行、M5 完整发行与首装禁网另验 |
| 自定义 MDX/MDD | `mdictlib` 0.2.8，MIT；声明支持 v1/v2、MDX/MDD，LZO 可选，v3 未支持 | 合法样本、版本 / 编码 / 压缩矩阵、MDD 媒体、HTML 隔离、四架构链接及依赖许可 | 自定义词典模块开发前；结构化内置小样可先完成 M1 |
| 离线 TTS | 继续使用已接入的 Supertonic 3，不再测试替代模型；范围及上游证据缺口见[准入决定](evidence/issue-14/README.md) | 现有单词／变位证据保留；例句接入、正式发行许可、四架构实机、干净机器、体积和启动成本 | Windows M2 单词与 M3 变位已验收；例句按剩余 M3 范围、成品按 M5 验收 |
| FSRS | 官方 `fsrs-rs v6.6.2` 已选为首选验证候选，BSD-3-Clause；有同一 Rust 核心的浏览器 WASM 方案 | 固定核心版本、模式、参数、时间规则与固定向量；算法升级边界；优化器依赖的实际体积 | M2 开发前完成候选验证并锁定调度规则；现有版本 ABI 不代表学习逻辑验收 |
| Google OAuth / Drive 文件同步 | 官方桌面 OAuth + PKCE；优先验证 `drive.file` 是否足够 | 项目归属、测试 / 生产配置、Desktop 客户端、测试账户 / 文件目录、API、回调与刷新；发行品牌 / 域名 / 隐私页；对象 ID / 重试 / changes 及恢复 | M4 对应适配器接入前；不阻塞 M1–M3 |
| OneDrive / Microsoft Graph 文件同步 | 同一 JSON / JSONL 协议，供应商适配独立 | 应用注册、账户 / 租户与实际权限、回调 / 凭据、item ID / ETag / delta、上传会话、冲突策略与恢复 | M4 第二适配器接入前；桌面镜像目录模式须另验，不能代替 API 证据 |
| 普通翻译 / AI | 普通翻译独立适配；AI 使用用户的 OpenAI 兼容服务 | 服务条件、网络、三语方向、长度 / 配额 / 成本、错误、数据去向及凭据注入 | M4 对应适配器开发前；无需提前取得全部服务密钥 |
| 工具链与验证平台 | 仓库有四目标编译 CI 配置，本轮未验证远端运行 | pnpm、Rust、MSVC / Windows SDK、WebView2；macOS 构建与四架构真实运行 / 安装渠道 | M1 前先有一个可运行原生环境；M5 前完成全部目标 |
| 发行与自部署设施 | Windows 官方 WebView2 离线依赖包；后续签名 / 公证身份、Web 域名与 BFF | 依赖版本与分发条款、最低 OS、离线安装说明、更新策略；Web HTTPS、回调与访问控制 | 对应安装器 / Web 实现前；不阻塞最早本地业务链路 |

### 词典、变位与分级数据

- Kaikki 提供[法英](https://kaikki.org/dictionary/French/index.html)、[法法](https://kaikki.org/frwiktionary/Fran%C3%A7ais/index.html)及中文版[法語](https://kaikki.org/zhwiktionary/%E6%B3%95%E8%AA%9E/index.html) / [法语](https://kaikki.org/zhwiktionary/%E6%B3%95%E8%AF%AD/index.html)数据。中文版两组不能直接相加，需按语言代码和词形 / 词义去重。
- 页面词形统计和原始 / 后处理体积不能证明学习者词头覆盖或成品包体。后处理下载正在弃用，导入方案应评估[原始 JSONL](https://kaikki.org/dictionary/rawdata.html)并固定可重现快照。
- Wiktionary 文字需核对具体来源和 CC BY-SA 等适用条件；导入文本例外及音频 / 图片分别处理。提取器代码许可不能覆盖内容许可。[Wikimedia 条款](https://foundation.wikimedia.org/wiki/Policy:Terms_of_Use/en)。
- [Lefff 原作者](https://pauillac.inria.fr/~sagot/index.html#lefff)列出 3.4 与 LGPL-LR；[衍生数据说明](https://raw.githubusercontent.com/RosaeNLG/rosaenlg/master/packages/french-verbs-lefff/README.md)保留该数据许可，不能按外层包的 Apache 标签处理。形态反查能力不代表全部复合时态已覆盖，本轮未成功取得并校验原作者完整归档。
- [FLELex 官方下载](https://cental.uclouvain.be/cefrlex/flelex/download/)明确 CC BY-NC-SA 4.0，不符合默认资源政策；分级语料频率不能直接当成唯一教学必背清单。本轮未找到可直接准入的替代完整 A1–B2 词表。
- [mdictlib](https://github.com/lonelam/mdictlib)是解析器候选，不赋予第三方词典内容再分发权；声明支持的格式仍需本项目实测。
- 2026-11-07，[#3 证据](evidence/issue-3/README.md)完成固定 `mdictlib 0.2.8 + lzo` 的 Windows x64 独立读取验证：六组原创合法配对、12 个真实 MDX 与两个同名 MDD 通过，校验／截断／大小／不支持头部失败可复现。自定义词典正式适配、外部 HTML 显示及四目标执行仍按对应门槛验收；第三方文件未准入默认包或公开夹具。

### 离线语音与系统依赖

- sherpa-onnx 引擎采用 Apache-2.0。v2.0.0 移除了内置 espeak-ng / piper-phonemize；Piper 推理仍存在，法语文本到音素需外部方案或 lexicon。旧 SIWIS 示例的 `data_dir` 不能证明新版组合可运行。[版本说明](https://github.com/k2-fsa/sherpa-onnx/releases/tag/v2.0.0)、[变更 PR](https://github.com/k2-fsa/sherpa-onnx/pull/4055)。
  2026-11-07 [#5 独立证据](evidence/issue-5/README.md)：2.0.1 + Piper Tom + 外部 eSpeak 前端在 Windows x64 禁网进程完成八条统一输入的 44,100 Hz WAV 生成、回读与播放。`silence_scale=1.0` 保留安静词尾；用户确认八条样本正确，包含 école 的 /l/。运行获取含准备工具为 191.78 MB，计入独立软件来源材料并去重后的总获取为 255.44 MB，运行 ZIP 为 170.71 MB；音频回归和包校验通过。Tom 按作者 AGPL-3.0 与声线条件记录，公开再分发该对照模型的对应源码形式仍待核验。SIWIS 授权链未闭合；应用分发采用现有 Supertonic 3 路线，模型条款交付和其他三目标／干净机器实测仍属 M5。
- SIWIS 单个 ONNX 权重约 63.2 MB，另需配置、tokens、文本前端及运行库。总仓库标签与模型卡中的训练数据许可不等价，需固定权重并核对授权链和署名。[权重](https://huggingface.co/rhasspy/piper-voices/tree/main/fr/fr_FR/siwis/medium)、[模型卡](https://huggingface.co/rhasspy/piper-voices/blob/main/fr/fr_FR/siwis/medium/MODEL_CARD)。
- [官方 Rust 构建脚本](https://raw.githubusercontent.com/k2-fsa/sherpa-onnx/master/sherpa-onnx/rust/sherpa-onnx-sys/build.rs)有四目标预编译归档映射，但不能代替本项目链接、启动、断网播放证据；自动下载须限定在受控构建阶段。
- [Kokoro #6](evidence/issue-6/README.md) 已完成本地对照，用户判定音质比现用 Supertonic 3 差，结论为“Tested and does not meet requirements”。[Pocket TTS 法语权重](https://huggingface.co/kyutai/pocket-tts/tree/main/languages/french)的历史调查保留，#7 未测试、取消计划；不再安排获取、合成或试听。历史主权重约 219 MB，不等于压缩安装包大小。
- Tauri 文档估算内嵌 WebView2 离线安装器约增加 127 MB；最终按具体版本测量。已确认采用已有运行时与独立官方离线依赖包，并报告完整离线总量。[安装器文档](https://v2.tauri.app/distribute/windows-installer/)。

### 调度与云端接口

- [fsrs-rs](https://github.com/open-spaced-repetition/fsrs-rs)和[fsrs-browser](https://github.com/open-spaced-repetition/fsrs-browser)提供候选实现。排程和参数优化是不同调用，但不能承诺通过一个 feature 完全移除优化器；精简体积和原生 / WASM 一致性需实际构建验证。
- Google 桌面应用是 public client，应使用 PKCE；客户端秘密不能作为保密边界。[桌面授权文档](https://developers.google.com/identity/protocols/oauth2/native-app)。
- `drive.file` 限制的是应用相关或用户明确授权的文件，不是文件夹沙箱；是否满足实际接口需验证。[Drive scope 文档](https://developers.google.com/workspace/drive/api/guides/api-specific-auth)。
- Google 生产授权涉及独立测试 / 生产项目、品牌、所持域名及公开主页 / 隐私说明，属于同步发行准备事项。[生产要求](https://developers.google.com/identity/protocols/oauth2/production-readiness/policy-compliance)。

## 新增输入与 WPM 准入（2026-11-08）

词频只读资源及 OCR 运行库／法语数据准入后均随完整版和 Lite 提供；首次断网验收覆盖它们的完整文件与依赖。Lite 仍仅省去 TTS 模型，各新增资源分别计量包体与安装占用、保留通知。

[系统选词与屏幕 OCR](translation-inputs.md)按 Windows／macOS 独立验证：焦点切换前的选择获取、快捷键注册与冲突、支持应用、macOS 辅助功能／屏幕录制权限、取消／撤销／恢复、多屏与缩放、资源失败及未保存笔记保护。截图及 OCR 原文仅供主动操作，不自动写日志或云同步。浏览器预览不能替代原生证据。

[词频颜色](word-frequency-colors.md)已固定完整 Lexique 3.83 源、法律原文和转换来源；原始样本、只读资源、100／10 与未知、三语／三主题及原生资源故障恢复检查通过。46,947 个词元的 SQLite 为 10,424,320 字节；普通及含语音开发包均验证完整通知，见 [#57](evidence/issue-57/README.md)。其他平台与 M5 正式发行按各自门槛验收。

[WPM](speech-preferences.md)复用已选 Supertonic 3，不再比较其他 TTS 模型。分别记录两声线的代表性法语基准，实际试听最低 80、默认 180、最高 240；保存及旧偏好读取、所有现有播放器和重启恢复使用同一规则。目标是粗粒度、易用、清晰的朗读设置，短词不承诺逐项精确 WPM，速度变化不重写选定音频或学习进度。

## 资源准入记录

每项候选成为正式依赖或随包内容之前，记录以下证据：

- 来源 URL、上游版本 / 提交或快照日期、文件名、校验和及转换版本。
- 内容语言、覆盖与缺项；词典、例句、媒体和声线分别标注来源。
- 适用许可原文、署名、转换 / 再分发条件，以及随包声明或源码义务。
- 默认随包或用户自备的角色、平台 / 架构、全部配套文件及运行依赖。
- 原始、转换后、压缩下载及安装体积，连同测量方法和版本。
- 样本、环境、通过 / 失败 / 未测结果，以及依赖它的里程碑。

仅有网页、包标签、下载链接或模型参数量时，状态仍为“候选”。首个词典样本需覆盖重音、撇号、`œ`、同形多义及屈折形式；完整三语覆盖与默认全量包另行验收。

2026-11-09，固定 Tesseract.js／core 7.0.0、完整 `fra` 和 XCap 0.9.9 已完成 Windows x64 本地 OCR 准入与双屏框选验收。语言数据为 1,130,365 字节，六份内嵌 WASM 核心由上游能力探测选择；法律原文覆盖 core 编解码依赖、42 份固定原生依赖及浏览器 worker／Buffer／ieee754 等材料。完整来源与[本地性能样本](evidence/issue-59/resource-probe.md)、原生结果及含语音开发包体积见 [#59 证据](evidence/issue-59/README.md)。其他平台、系统权限恢复和 M5 完整发行按各自门槛验收。

## M1 实现前的工程证据与计划（历史）

本节保留 2026-10-24 的资源调查与工程基线快照；其中“没有真实查询、笔记写入”和“M1 spec 前”的描述仅适用于当时，当前进展见文末状态更新。

2026-10-24 追加资源探查：已从用户指定站点下载一个 MDX 小样并实际解码 18,085 条记录；另已下载并完整解析 FreeDict 法英 / 法中两个小包。结果、哈希、包内许可和尚未通过的准入条件见[资源小样调查](resource-feasibility.md)。这是资源候选的局部验证，不代表已经选定默认词典、完成 M0 或新增了应用查词能力。TTS 候选仍未生成 / 试听，Pocket 法语权重获取条件另有门槛。

代码仍是界面壳、只读运行状态、SQLite 元数据初始化和 WASM 版本 ABI；没有真实查询、笔记写入或 FSRS。启动时创建过数据库不能代替应用级重启恢复验收。

2026-10-24 环境复核：已通过现有 nvm 将 Node.js 从 v24.15.0 升级并启用 v24.22.1 LTS 安全版，二进制 SHA-256 与官方发布值一致，OpenJS 签名有效；npm 11.19.1、pnpm 11.25.0 可用。锁定依赖安装、lint、类型检查、前端构建及使用 Edge 的两项浏览器 E2E 均通过。详见[验证记录](verification.md)。

本机 Windows x64 原生开发环境已补齐：rustup 1.29.2、Rust / Cargo 1.99.1、rustfmt、Clippy、WASM 与 Windows ARM64 目标、Visual Studio 2022 Build Tools 17.14.42、MSVC 14.44.35207、Windows SDK 10.0.26100.0；既有 WebView2 155.0.4283.62 可用。格式检查、核心 / 存储 Clippy、3 项 SQLite 集成测试、WASM 编译与实际 ABI 调用、全仓库检查、Windows ARM64 编译检查、x64 调试构建及真实桌面启动均通过。原生窗口显示“本地存储已就绪”，测试后已关闭。Rust 用户 PATH 已持久化，旧终端需重新打开。正常权限下 npm、Rust 和 crates.io 官方端点可访问；受限工具环境中的 DNS 失败不能作为机器网络故障的结论。

M0 的本机原生基线已验证，下一步是固定一个合法真实词典小样及来源记录，验证读取和持久化所需接口；并行验证变位、TTS 组合关键风险。M1 spec 前仍须确认笔记字段、来源引用 / 内容快照与稳定身份，本轮没有冻结这些业务结构。

最低 macOS、完整变位矩阵、确切资源 / 模型版本、FSRS 参数与时间规则、同步冲突、服务凭据、签名 / 公证及 Web 部署细节均是对应里程碑的显式门槛，没有默认认定已完成。

本次环境复核已完成锁定依赖、前端与上述原生基线验证；尚未下载全量词典或模型、进行语音试听、FSRS 原生 / WASM 向量、macOS 验证、签名发行包或四架构安装测试。候选调查不构成生产准入或运行通过。

## 当前 M1 状态

2026-10-25：M1 [保存规则](m1-note-saving.md)已确认并实现。Windows x64 已验证 FreeDict 小样查询、三层记录、显式保存、幂等与冲突处理、搜索排序、禁网重启恢复及 release portable，见[验证记录](verification.md#m1-windows-复盘修复与-macos-阻塞)。macOS 专属编译、退出保护与原生恢复仍为 **blocked — 等待 macOS 执行环境**，按[阻塞清单](macos-m1-blockers.md)集中执行；Windows ARM64 只有编译证据。完整资源、其他平台及同步／正式发行仍须满足上述门槛。

2026-11-01 Windows M2 及实际 WASM 已在 [#22](evidence/issue-22/review-2026-11-01.md) 按用户授权收尾；#24 不重新开放已确认的排程、额度或卡片身份规则。2026-11-05，schema 17 的选定版本／共享保护、队列修订／窗口／delta、自动修复和独立 GC／容量／对账通过 Windows x64 本期验收。原生、差分、性能和用户干净系统离线结果见[实施记录](evidence/issue-24/continuation-2026-11-04.md)与[验收清单](evidence/issue-24/acceptance-matrix-2026-11-05.md)；其他平台和 M5 正式发行继续按各自门槛执行。

## 同步与浏览器存储准入补充（2026-11-02）

应用作为学习数据入口、跨云兼容、后台批量与确认后本地恢复是本次明确要求。较早 Sheets 项目 / 删行门槛改为文件协议门槛；[格式评估](sync-format-evaluation.md)与[同步设计](sync-design.md)是候选方案，正式跟踪在 [#1](https://github.com/zardonc/lexique/issues/1)。当前浏览器仍是预览，现有 M2 实际 WASM 算法证据不能替代 IndexedDB / 浏览器同步验收。

接入前必须冻结同步共享范围、稳定操作与同词身份、因果分支 / 撤销 / 删除、不同设备额度、未知高版本、来源认证及密钥恢复、设备成员 / epoch、快照一致 frontier 和 GC 依赖。验证本地业务与 outbox 原子提交；云端结果不明重试、缺口、外部文件修改与删除不得清空本地。清理须有相关设备持久 ack / rebase、上一检查点及增量恢复链，并区分业务历史与可丢弃传输日志。

M6 还须验证 IndexedDB 事务完成、严格持久性提示支持、存储配额 / 驱逐 / 私密模式、多标签页与升级阻塞、后台挂起和 origin / profile 恢复；不以 `beforeunload` 或 WASM 内存队列作为保存边界。Google / OneDrive 的权限、API / 目录镜像差异和实际上传完成语义各自核验；未有运行证据前仅声明格式可适配，不声明已支持服务或零丢失保证。

## 变位英语资源准入（2026-11-06，issue #42）

FreeDict French-English 0.4.1 的官方源归档、发布 SHA-512、完整 AUTHORS／COPYING 与转换来源已固定；内容保留 GPL-2.0-or-later，应用／转换器仍为 MIT。仅映射明确动词词性、英语翻译及精确词头；代词用法单独标注，不借用同形名词或法语例句。7,826 个变位身份中 1,377 个有英语释义，共 1,953 义项，其余明确不可用，覆盖独立统计。完整来源、再分发材料、Windows 实机结果、体积及性能见 [#42 资源证据](evidence/issue-42/README.md)。这是部分英语资源准入；新鲜离线包、其他 OS／架构及正式发行仍按各自门槛验收。

## 代词式资源准入（2026-11-06，issue #43）

固定 Grammalecte 标记与 Lefff 正面代词框架准入 2,955 个独立身份，覆盖 2,954 个原形；其余原形逐项保留排除原因，无词频上限。独立配合期待、例外、人称限制、en 用法及原始许可见[来源政策](../resources/lefff-syntax/pronominal-rules.md)。普通身份和旧生成物保留，23 区块报告无缺失／未确认项。英语仍为部分覆盖：全部 10,781 个身份中 1,493 个有释义，共 2,089 义项。数据库、内存快照、延迟及实际执行见 [#43 证据](evidence/issue-43/README.md)；新鲜离线包、其他 OS／架构和正式发行门槛继续单独验收。

## Windows x64 变位验收（2026-11-07，#38）

#39 的六项元数据准入条件已核对并关闭；#39–#49 的约定变位范围已完成 Windows x64 验收。#49 新鲜包包含匹配的格式 6 reader、完整已准入 TTS 运行时／模型／声线／编码器，以及独立来源、许可和八份转换源码；用户报告断网测试通过。原包保持不变，学习数据与资源分别保存。全部 23 区块的普通／代词式覆盖、反查、逐表达实播、着色／排序恢复、词典导航和数据保持见 [#38 收尾](evidence/issue-38/README.md)。

当前资源为 11,005,952 字节、1,451,539 格和 10,781 个身份；英语覆盖单独为 1,493 个可用、9,288 个暂缺身份。原始形式和未知维度保留，第三组不生成词尾颜色。CLI 保留完整校验，最终六进程全部通过原有 1,700 ms 预算；数据与已验收包逐字节相同，格式 6 须配套对应 reader。此结果满足此前 #42／#43 的 Windows x64 新鲜离线包门槛。用户机器／运行时版本、禁网方式及逐项日志未提供，按用户报告归档。

其他平台实机、缺少 WebView2 的首装禁网环境、完整默认词典／分级模块、例句朗读及 M5 四目标完整版／Lite 发行继续分别验收；M4 同步和 M6 浏览器持久化门槛保持不变。
