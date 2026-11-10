# 选词、OCR、词频与 WPM 的后续交付计划

日期：2026-11-09。来源：[总规格 #1](https://github.com/zardonc/lexique/issues/1)与新增需求。S1–S4 的 Windows x64 检查已完成，见 [#56](evidence/issue-56/README.md)、[#57](evidence/issue-57/README.md)、[#58](evidence/issue-58/README.md)及 [#59](evidence/issue-59/README.md)。现有 M1、M2、#24 与 M3 变位验收保留，其他平台测试按用户要求暂缓。

## 固定边界

- 首期 M3 包含应用内及其他应用选词、屏幕区域 OCR、生词词频颜色和共用 WPM 设置。Windows 11／macOS × x64／ARM64 各有实际验收；系统能力不能以图片导入或浏览器 mock 替代。
- 继续使用 Supertonic 3、CPU、女声／男声及 48 kbit/s Ogg Opus；停止其他 TTS 模型测试。[目标朗读速度](speech-preferences.md)为 80–240 WPM，步长 20，默认 180，播放时近似换算；音频内容版本与保护规则保持。
- 同一标准词头仍是一份生词记录。词频采用 Lexique 3.83 `freqlemfilms2` 的词性去重合计，100／10 为产品分档边界；词频不替代 CEFR，也不覆盖用户内容。
- 已准入词频资源、OCR 运行库与法语数据随完整版及 Lite 提供，首次断网可用；Lite 仅将 TTS 模型改为用户主动下载，新增资源的体积与通知单独验收。
- 平台能力经既有运行时边界接入；词频资源保持只读、与用户库分离。截图只用于主动识别；查词不自动保存或建卡。
- 选词及本地 OCR 不依赖在线翻译或云同步。M4 的服务、凭据及 JSONL／outbox 协议仍分别进入门槛，不因为本次规划而变为已实现。

## 实施顺序与纵向切片

| 顺序 | 完整交付链路 | 前置及退出条件 |
| --- | --- | --- |
| S1：共用 WPM [#56](https://github.com/zardonc/lexique/issues/56) | 设置数值／试听 → 原生保存 → 单词、听音卡、变位表达共用播放 → 重启恢复 | 复用现有 Supertonic 3；完成两声线基准、80／180／240 及粗粒度交互验收；已保存 normal／slow 分别读取为 180／100，学习及音频身份不变 |
| S2：词频颜色 [#57](https://github.com/zardonc/lexique/issues/57) | 固定源数据／许可 → 去重转换 → 只读查询 → 生词列表及详情标签 → 断网重启 | 原始样本独立核对，跨词性合计、0／10／100 和未知均可验证；正式衍生数据通知、覆盖及体积完成 |
| S3：其他应用选词 [#58](https://github.com/zardonc/lexique/issues/58) | 系统触发 → 焦点切换前读取选择 → 翻译模块核对 → 真实离线词典结果 | 逐平台确定默认快捷键、支持应用、获取方式与权限；测试无选择、冲突、拒绝、取消及笔记保护；首期包含系统选词 |
| S4：屏幕 OCR [#59](https://github.com/zardonc/lexique/issues/59) | 快捷键／按钮框选 → 截图预览及自动本地 OCR → 可编辑原文／选词 → 同一查词结果 | Windows x64 已验收；默认 Ctrl+Alt+Shift+O，结果保留前 5,000 字；完整运行库、语言数据和通知随包，平台专属执行与 M5 另验 |
| S5：其余离线核心 | #9 D1–D3 的有界切片、完整默认词典／分级、语法笔记、分类、导入导出及例句朗读 | #9 保持 needs-triage，逐片冻结行为、来源和预算；D4 已交付，不重复建设；例句接入时使用 S1 WPM |
| S6：M4 | 普通翻译／OpenAI 兼容服务；另按 S0–S5 冻结并实现文件同步 | 服务与同步独立验收；权限、成本、原子持久化、冲突／删除／恢复和云端实际结果各有证据 |
| S7：M5 | 四目标完整版／Lite 共八种成品 | S1–S5 的首期能力、完整资源、干净机器首装断网、Lite 模型下载后断网、许可、依赖、签名／公证及实机验收 |
| S8：M6 | 实际 WASM／IndexedDB 浏览器复习及同一同步协议 | 持久化、强关、配额、多标签页和跨端会合；桌面系统选词／截屏的验收不能直接转称浏览器支持 |

S1–S4 的 Windows x64 验收完成，四票均为 #1 的原生子票；#59 复用 #58 的翻译输入与请求生命周期。当前下一步为 #9 的[有界切片草案](lookup-enhancement-slices.md)，D4 已交付，C1–C4 保持各自未决门槛。

## 文档与验证入口

需求细节见[选词与 OCR](translation-inputs.md)、[词频颜色](word-frequency-colors.md)、[语音偏好](speech-preferences.md)，全局安排见[路线](roadmap.md)、[资源准入](development-readiness.md)和[架构](architecture.md)。按变更边界运行 README 中的前端、Rust 与原生检查；后端测试先编译并保持执行阶段 60 秒硬超时。各切片的命令、passed／failed／skipped 与限制记录在对应证据页。
