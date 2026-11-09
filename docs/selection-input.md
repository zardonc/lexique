# 选词输入规格

对应 [#58](https://github.com/zardonc/lexique/issues/58)。首期平台为 Windows 11／macOS × x64／ARM64；用户于 2026-11-08 暂缓其他平台执行，Windows x64 继续原生验证。

## 获取与输入

用户按全局快捷键时先记录来源，读取完成或真实失败后才显示并激活 Lexique。Windows 使用 UI Automation 的 focused element／TextPattern／GetSelection，在独立 MTA 线程读取，核对来源前台窗口且拒绝密码字段，不操作剪贴板；WebView 的可访问元素可属于独立 renderer 进程。macOS 使用 ApplicationServices 的 focused application／focused UI element／selected text；仅主动操作时提示辅助功能授权，拒绝／撤销时返回权限状态，每次操作重新检查，授权后可直接重试。

读取同时最多一项，等待上限两秒；系统 provider 超时不能中断时保留这一占用，后续操作明确显示忙碌，不堆积线程。没有选择、没有可读选择 API、来源焦点改变、超时、权限和读取错误分别保留。用户可主动粘贴或使用截图查词。

默认取词快捷键为 `Ctrl+Alt+Shift+D`。设置页“取词快捷键”接受带 Control／Alt／Super 修饰键的系统键位，冲突可见并允许更换。新键注册及旧键释放成功后才保存到用户库 `app_metadata` 的 `selection-shortcut`；保存失败恢复原键，重启从持久配置重新注册。未配置时才使用默认键，已保存的自定义键保留；损坏设置显示错误。此偏好不提高 schema 17。

应用内也有“查询应用内选中文字”入口，先读取当前 DOM／文本框的选择，再走同一原生事件边界和导航保护。所有取词结果进入翻译模块的可编辑原文框，不自动查词或打开词典模块。翻译输入和取词结果按 Unicode 字符最多保留前 5,000 字，多余部分截断，完整保留末尾字符；用户可编辑原文，或主动选择单词／短语后查询离线词典。只在查词入口去除外围空白，保留编辑器中的原始大小写、重音、撇号、连字和空白。查询必须非空、无控制字符且不超过 512 UTF-8 字节。

`selection-input` 仅发送到 main 窗口，严格校验 pending／captured／failed、单调请求编号及有界文字。首次事件经过现有未保存笔记保护；笔记正在保存时保留最新取词，保存完成后再检查导航保护，失败仍保留未保存输入与离开提示。同请求后续事件更新当前输入。取消、编辑、新请求和离开页面使旧结果失效。查词沿用 `RuntimeAdapter.lookupDictionary`、`DictionaryResults` 和完整词典导航；收藏仍需明确操作。原文不写入用户库、日志或同步。

## 依赖与设计依据

- `tauri-plugin-global-shortcut 2.4.1`，MIT／Apache-2.0，[官方用法](https://v2.tauri.app/plugin/global-shortcut/)。
- `windows 0.62.2`，MIT／Apache-2.0，Windows UI Automation／COM 系统 API；没有新增随包运行时。
- macOS `accessibility-sys 0.2.0` 与 `core-foundation 0.10.1`，MIT／Apache-2.0，链接系统 ApplicationServices／CoreFoundation。
- [STranslate](https://github.com/STranslate/STranslate)（MIT）的 MTA、TextPattern 和单次读取设计；[SelectedTextKit](https://github.com/tisfeng/SelectedTextKit)（MIT）的来源应用与 AX 选择获取。接口按本项目实现，保留原始空白与错误上下文。[Easydict](https://github.com/tisfeng/Easydict)（GPL-3.0）仅作交互参考。

## 原生应用矩阵与退出条件

Windows 验证使用隔离的浏览器网页、文本编辑器和可选文字 PDF；记录实际应用／版本、来源焦点、原始文字、全局键位及离线结果。macOS 对 Safari／文本编辑器／Preview 分别记录相同链路，并覆盖辅助功能拒绝、撤销与恢复。应用不暴露 TextPattern／AXSelectedText 时明确不支持，可转主动粘贴／OCR；受保护输入不读取。

执行记录须覆盖快捷键更改／冲突／重启、原始重音与撇号、空与长选区、失效响应、资源缺失／恢复、未保存笔记和学习数据保持。浏览器夹具只证明 UI／IPC 生命周期；其他平台暂缓执行仍须保留为发行验收，不据编译或 mock 宣称系统支持。
