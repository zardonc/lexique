# 开发链路与外部资源准入清单

更新：2026-10-24。依据：`Requirements-Consolidated.md` v1.0、当前仓库及本轮用户确认。需求材料中的建议与网页示例不自动成为已确认选型；外部资料只证明候选存在，不能代替本项目运行验收。

## 已确认的边界

1. 按下述 M0–M6 顺序推进。中间里程碑是开发验收，桌面正式发行仍包含已要求的默认词典、完整约定变位和法语离线语音。
2. 许可允许时，内置词典及变位可转换为只读结构化资源，优先评估 SQLite；具体格式尚未冻结。自定义 MDX/MDD 支持保留，资源与用户学习数据分别管理。
3. 默认资源需要明确允许目标转换与随包再分发，并且没有非商业用途限制；可接受署名、来源保留及数据的相同方式共享条件。代码、文字、媒体、模型和声线分别核查。
4. 项目自有源码采用 MIT，第三方内容不因此改为 MIT。
5. 每个平台 / 架构按完成首装离线功能所需的压缩下载总量计量，目标小于 100 MB，预期上限 200 MB，功能正确优先。安装占用与系统运行依赖另列；超限先报告明细供用户决定，不通过首次启动下载移走默认核心资源。
6. Windows 11 主包优先使用已有 WebView2；为缺少运行时的机器提供微软官方离线依赖包与检测提示。分别报告主包、依赖包和缺依赖机器的完整离线下载总量。

相关决策见 [资源与学习数据分离](adr/0001-separate-bundled-resources-from-learning-data.md)、[源码与资源许可](adr/0002-source-and-resource-licensing.md)。

## 开发链路与验收

| 里程碑 | 可验收结果 | 前置条件 |
| --- | --- | --- |
| M0 资源与原生基线 | 资源准入小样、可重复原生构建与启动证据；变位 / TTS 有独立可行性验证计划 | 合法真实数据样本、依赖获取环境、可用 Rust / MSVC / SDK；关键资源有可核验的获取与许可路径 |
| M1 第一条本地链路 | 离线查询 → 创建笔记 → SQLite 保存 → 关闭应用并重启后恢复同一内容 | 至少一个已准入词典小样；明确来源引用、内容快照与笔记身份；原生读写链路可运行 |
| M2 复习核心 | 评分、调度及必要复习记录可恢复；相同固定输入在原生与实际 WASM 中一致 | 锁定 FSRS 核心、参数和时间语义；卡片与笔记的身份关系明确 |
| M3 完整离线核心 | 约定变位可反查并高亮全部匹配；单词、变位和例句能真实离线发音 | 变位覆盖矩阵、完整 TTS 组合；词典 / 分级模块分别满足约定覆盖 |
| M4 外部能力 | 翻译 / AI 可用；Google 授权、双向同步及失败恢复可验收 | 服务条件与凭据方案、测试项目 / Sheet、冲突和删行语义；两条能力可分别开发验收 |
| M5 桌面发行 | Windows / macOS × x64 / ARM64 四包，首装断网、完整资源体积与许可声明可复验 | 四架构实际运行渠道、最低 macOS、安装器 / 签名 / 公证策略、完整资源准入与离线依赖 |
| M6 浏览器版 | 移动浏览器实际调用 WASM 完成闪卡复习，并延续学习进度 | 浏览器基线、本地持久化、同步、OAuth 回调、BFF 访问控制与部署方式 |

M0 进入首条链路的硬门槛是一个合法真实词典小样和可运行原生基线。完整变位、TTS 和分级资源调查可以并行；未通过对应门槛时不能承诺该模块或正式发行已具备能力。全量转换、试听和兼容测试留给后续验证工作。

## 外部资源清单

| 资源 | 待验证主候选与证据 | 必须确认的内容 | 截止时点 |
| --- | --- | --- | --- |
| 法法、法英、法中词典 | Kaikki / Wiktionary 三种释义语言均有可评估来源 | 固定快照、许可及署名；分别核验释义语言、常用词覆盖、例句、词形 / 词义去重与转换后体积 | M1 前至少准入一个真实小样；完整三语内容在词典模块及 M5 验收 |
| 完整变位与反查 | Lefff 3.4、RosaeNLG 的 Lefff 衍生数据与离线生成方案 | 数据 LGPL-LR 条款、语式 / 时态矩阵、助动词、代词式、缺陷动词、多匹配及生成样例 | M3 开发前确认覆盖与生成路径；M5 全量验收 |
| A1–B2 分级词表 | 尚无符合当前政策且已准入的完整来源；FLELex 为 CC BY-NC-SA 4.0 | 允许目标发行的来源或额外授权；词元 / 多词表达、分级依据、数量与质量 | 分级模块开发前；不阻塞 M1，仍是默认词库发行门槛 |
| 自定义 MDX/MDD | `mdictlib` 0.2.8，MIT；声明支持 v1/v2、MDX/MDD，LZO 可选，v3 未支持 | 合法样本、版本 / 编码 / 压缩矩阵、MDD 媒体、HTML 隔离、四架构链接及依赖许可 | 自定义词典模块开发前；结构化内置小样可先完成 M1 |
| 离线 TTS | sherpa-onnx + Piper SIWIS 是小包优先评估方向，尚未选定 | 引擎、权重、声线、音素前端和配套数据的版本及许可；短词 / 连读 / 变位音质、四架构、体积和启动成本 | M0 及早验证关键风险；M3 前锁定组合；M5 成品验收 |
| FSRS | 官方 `fsrs-rs`，BSD-3-Clause；有同一 Rust 核心的浏览器 WASM 方案 | 同一核心版本、模式、参数、时间规则与固定向量；算法升级边界；优化器依赖的实际体积 | M2 开发前选定并验证；现有版本 ABI 不代表学习逻辑验收 |
| Google OAuth / Sheets / Drive | 官方桌面 OAuth + PKCE；优先验证 `drive.file` 是否足够 | 项目归属、测试 / 生产配置、Desktop 客户端、测试账户 / Sheet、API、回调与刷新；发行品牌 / 域名 / 隐私页；同一 Sheet 的恢复方法 | M4 接入前；不阻塞 M1–M3 |
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

### 离线语音与系统依赖

- sherpa-onnx 引擎采用 Apache-2.0。v2.0.0 移除了内置 espeak-ng / piper-phonemize；Piper 推理仍存在，法语文本到音素需外部方案或 lexicon。旧 SIWIS 示例的 `data_dir` 不能证明新版组合可运行。[版本说明](https://github.com/k2-fsa/sherpa-onnx/releases/tag/v2.0.0)、[变更 PR](https://github.com/k2-fsa/sherpa-onnx/pull/4055)。
- SIWIS 单个 ONNX 权重约 63.2 MB，另需配置、tokens、文本前端及运行库。总仓库标签与模型卡中的训练数据许可不等价，需固定权重并核对授权链和署名。[权重](https://huggingface.co/rhasspy/piper-voices/tree/main/fr/fr_FR/siwis/medium)、[模型卡](https://huggingface.co/rhasspy/piper-voices/blob/main/fr/fr_FR/siwis/medium/MODEL_CARD)。
- [官方 Rust 构建脚本](https://raw.githubusercontent.com/k2-fsa/sherpa-onnx/master/sherpa-onnx/rust/sherpa-onnx-sys/build.rs)有四目标预编译归档映射，但不能代替本项目链接、启动、断网播放证据；自动下载须限定在受控构建阶段。
- [Kokoro](https://huggingface.co/hexgrad/Kokoro-82M)有法语声音候选，仍需核验短词和完整前端；[Pocket TTS 法语权重](https://huggingface.co/kyutai/pocket-tts/tree/main/languages/french)主文件约 219 MB、尚未含其他资源，暂不作为小包默认首选。权重大小不等于压缩安装包大小。
- Tauri 文档估算内嵌 WebView2 离线安装器约增加 127 MB；最终按具体版本测量。已确认采用已有运行时与独立官方离线依赖包，并报告完整离线总量。[安装器文档](https://v2.tauri.app/distribute/windows-installer/)。

### 调度与云端接口

- [fsrs-rs](https://github.com/open-spaced-repetition/fsrs-rs)和[fsrs-browser](https://github.com/open-spaced-repetition/fsrs-browser)提供候选实现。排程和参数优化是不同调用，但不能承诺通过一个 feature 完全移除优化器；精简体积和原生 / WASM 一致性需实际构建验证。
- Google 桌面应用是 public client，应使用 PKCE；客户端秘密不能作为保密边界。[桌面授权文档](https://developers.google.com/identity/protocols/oauth2/native-app)。
- `drive.file` 限制的是应用相关或用户明确授权的文件，不是文件夹沙箱；是否满足实际接口需验证。[Drive scope 文档](https://developers.google.com/workspace/drive/api/guides/api-specific-auth)。
- Google 生产授权涉及独立测试 / 生产项目、品牌、所持域名及公开主页 / 隐私说明，属于同步发行准备事项。[生产要求](https://developers.google.com/identity/protocols/oauth2/production-readiness/policy-compliance)。

## 资源准入记录

每项候选成为正式依赖或随包内容之前，记录以下证据：

- 来源 URL、上游版本 / 提交或快照日期、文件名、校验和及转换版本。
- 内容语言、覆盖与缺项；词典、例句、媒体和声线分别标注来源。
- 适用许可原文、署名、转换 / 再分发条件，以及随包声明或源码义务。
- 默认随包或用户自备的角色、平台 / 架构、全部配套文件及运行依赖。
- 原始、转换后、压缩下载及安装体积，连同测量方法和版本。
- 样本、环境、通过 / 失败 / 未测结果，以及依赖它的里程碑。

仅有网页、包标签、下载链接或模型参数量时，状态仍为“候选”。首个词典样本需覆盖重音、撇号、`œ`、同形多义及屈折形式；完整三语覆盖与默认全量包另行验收。

## 当前工程证据与下一步

代码仍是界面壳、只读运行状态、SQLite 元数据初始化和 WASM 版本 ABI；没有真实查询、笔记写入或 FSRS。启动时创建过数据库不能代替应用级重启恢复验收。

2026-10-24 环境复核：已通过现有 nvm 将 Node.js 从 v24.15.0 升级并启用 v24.22.1 LTS 安全版，二进制 SHA-256 与官方发布值一致，OpenJS 签名有效；npm 11.19.1、pnpm 11.25.0 可用。锁定依赖安装、lint、类型检查、前端构建及使用 Edge 的两项浏览器 E2E 均通过。详见[验证记录](verification.md)。

原生开发仍未就绪：PATH 中没有 `cargo` / `rustc` / `rustup`；在用户 `.rustup` 目录找到的残留工具链为 Rust / Cargo 1.91.1，低于项目锁定的 1.99.1，且未安装 WASM 目标。Tauri 诊断未检测到包含 MSVC 与 Windows SDK 的 Visual Studio / Build Tools；WebView2 155.0.4283.62 已安装。正常权限下 npm、Rust 和 crates.io 官方端点均返回 HTTP 200；受限工具环境中的 DNS 失败不能作为机器网络故障的结论。历史记录及 CI 配置不能作为当前原生构建已通过的证明。

下一步为 M0：恢复可重复的依赖获取与原生开发环境，固定一个合法真实词典小样及来源记录，验证读取和持久化所需接口；并行验证变位、TTS 组合关键风险。M1 spec 前仍须确认笔记字段、来源引用 / 内容快照与稳定身份，本轮没有冻结这些业务结构。

最低 macOS、完整变位矩阵、确切资源 / 模型版本、FSRS 参数与时间规则、同步冲突、服务凭据、签名 / 公证及 Web 部署细节均是对应里程碑的显式门槛，没有默认认定已完成。

本次环境复核已完成锁定依赖与前端验证；尚未下载全量词典或模型、运行原生应用构建、语音试听、FSRS 原生 / WASM 向量或四架构安装测试。候选调查不构成生产准入或运行通过。
