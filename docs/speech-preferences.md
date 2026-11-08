# 法语语音偏好

2026-11-08 用户确认后续开发继续使用已接入的 Supertonic 3，不再测试其他 TTS 模型，见 [ADR 0009](adr/0009-continue-with-supertonic-3.md)。2026-10-26 确认的“法语女声／男声”和“正常／慢速”设置继续适用，默认女声、正常。

- MLS medium 已因用户试听确认读音混乱、音质不佳撤回，不作为产品声线。失败组合为：女声源编号 `3698`（推理编号 `1`），男声源编号 `1840`（推理编号 `0`）；默认女声、正常语速。许可依据与边界见[替代准入记录](evidence/issue-14/replacement-admission.md)。
- 偏好独立于学习进度，保存到现有用户库的 `app_metadata`，不增加 schema 版本。声线使用稳定的 `female`／`male`；旧 `siwis`／`tom` 分别兼容为女声／男声，保存时使用新值，不丢失原语速。
- 设置分别保存声线与语速；保存失败保留待保存选择并显示错误，不显示保存成功。
- 浏览器预览不冒充桌面持久化，禁用保存并说明限制。
- Windows x64 已接入偏好持久化、CPU 离线合成、48 kbit/s Ogg Opus 编码、播放及音频版本／缓存维护，见 [#15](evidence/issue-15/README.md)、[#24](evidence/issue-24/README.md)及 [#38](evidence/issue-38/README.md)。
- 当前使用 Supertonic 3 int8：女声 F1（`sid=0`）、男声 M1（`sid=5`），八步去噪。用户确认初次英语和法语样本可用，且[试听对照](evidence/issue-14/supertonic-slower/listen.html)的 `0.6` 倍播放清晰自然。统一以 `speed=1.0` 合成，播放时常速使用 `1.0`、慢速使用 `0.6`，共享同一音频；偏好变化不自动替换生词选定音频，保留规则见 [ADR 0008](adr/0008-selected-word-audio-content-versions.md)。
- 用户确认单独字母 `y` 使用字母名 /iɡʁɛk/，试听候选由 `i grec` 合成。代词 `y` 的 /i/ 是不同用法，不能将字母名自动套用到代词；产品尚未实现语境选择。
- Kokoro [#6](evidence/issue-6/README.md) 已测试，用户判定音质不满足要求；Pocket TTS #7 未测试、取消计划。历史探针、录音和许可材料保留。
- Supertonic 3 的 OpenRAIL-M／通知义务、完整默认包体、干净机器和跨平台门槛继续按[资源准入清单](development-readiness.md)核查；继续使用不代表正式发行已通过。

历史四组合样本见[Supertonic 试听页](evidence/issue-14/supertonic-samples/listen.html)，许可条件及来源见[替代调查](evidence/issue-14/replacement-options-after-mls.md)。模型与格式比较见 [#14 对比报告](evidence/issue-14/comparison.md)；这些材料不构成后续替代模型测试计划。

Windows / macOS 的 x64 / ARM64 各提供完整版和 Lite 版，共八种发行物，均使用 Supertonic 3。完整版内嵌模型，不受 200 MB 包体上限限制；Lite 不内嵌模型，用户可在“额外资源”主动下载后离线使用。其他内置资源不变。Lite 下载和四架构正式发行仍待实现和验收；现有 Windows x64 语音开发包不等于 M5 完成。
