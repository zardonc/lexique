# 系统取词进入翻译输入

2026-11-08，[#58](https://github.com/zardonc/lexique/issues/58)。系统设置提供可保存的“取词快捷键”，默认 `Ctrl+Alt+Shift+D`，冲突明确提示并可更换；已有自定义键保留。Windows 在激活 Lexique 前通过 UI Automation 取得选区，进入翻译模块的可编辑原文框。取词和输入均保留前 5,000 个 Unicode 字符，多余字符截断，末尾完整字符保留。查词与打开完整词典均需明确点击。

## 原生结果

最终运行位于 `target/issue-58/native-17/`，使用当前 release 程序、真实离线词典／变位／词频资源和既有非空学习夹具。结果见 [native-results.json](native-results.json)。

| 检查 | 结果 |
| --- | --- |
| 实际默认 `Ctrl+Alt+Shift+D` → 翻译 → 明确查词 | 通过 |
| 独立 WebView2 网页 → 自定义 `Ctrl+Alt+K` → 翻译 | 通过 |
| 超长网页选区 → 前 5,000 字，末尾 emoji 保留 | 通过 |
| Win32 RichEdit 编辑器 → 自定义键 → 翻译 | 通过 |
| 未保存笔记 → 继续编辑／明确放弃 | 通过 |
| 缺失词典保留原文，恢复后重试 | 通过 |
| 正常重启后快捷键配置与注册恢复 | 通过 |

SQLite 比较使用已有非空夹具：2 份生词、2 张卡、1 份进度、1 次评分、1 份选定音频和1 份保护关系全部保持。取词不保存原文、不收藏、不建卡、不修改学习进度或音频。

WebView2 内置 PDF 查看器的[截图](pdf-selection.png)确认文字已选中，但该查看器没有提供可读取的 TextPattern 选择；应用显示明确“不支持”状态，见 [pdf-results.json](pdf-results.json)。这是实际支持限制；可主动粘贴，屏幕 OCR 由 #59 接入。

## 自动检查

| 命令 | 结果 |
| --- | --- |
| `pnpm lint`、`pnpm typecheck`、`pnpm build` | 通过 |
| `pnpm rust:fmt` | 通过 |
| `pnpm rust:test:build` → `pnpm test:rust` | 148 项通过；执行硬限 60 秒 |
| `cargo test --locked --release -p lexique-desktop --lib --no-run` → `node scripts/test-rust.mjs --desktop-lib` | 5 项通过；执行硬限 60 秒 |
| `pnpm desktop:build --no-bundle` | Windows x64 release 通过 |
| `pnpm test:e2e` | 全套 188 项通过；取词专项 17 项 |

专项覆盖明确查词、原文保留、真实输入截断、旧／取消响应、快捷键冲突及保存、非法 IPC、读取失败、保存期间的取词、保存失败后的导航保护和应用内选择。原生检查命令从 `apps/desktop/` 执行：

```powershell
node e2e/native-selection-check.mjs ../../target/pr23-native-speech-20261101 ../../target/release/lexique-desktop.exe ../../target/desktop-conjugation-v6-stats ../../target/desktop-frequency-v1 ../../target/issue-57/native-2/Lexique-data ../../target/issue-58/native-17
```

Python 使用 uv 管理的解释器，构建／预编译进程将其目录放在当前进程 PATH 前，并设置 `PYTHONDONTWRITEBYTECODE=1`。完整命令日志保留在忽略的 `target/issue-58/`。

## Standards

两轴代码复核未发现未解决的文档标准违反或影响行为的代码异味。复用现有运行时、导航和保存保护，IPC 校验及 main 窗口权限保持受限。

## Spec

默认／自定义快捷键、翻译路由、5,000 字截断、明确查词、保存期间延后导航和 Windows／macOS 读取前后的来源焦点检查符合当前规格。没有未解决的规格问题。

## 限制

macOS 的 AX 选择、权限恢复与来源焦点实现已接入，实机执行和其他架构按用户要求暂缓。当前支持取决于来源应用提供 UI Automation／AX 选择；受保护输入不读取。浏览器预览和夹具不证明其他平台支持，正式发行继续按 M5 验收。
