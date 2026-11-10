# 共用朗读 WPM：校准与验证

2026-11-08，[#56](https://github.com/zardonc/lexique/issues/56)。设置为 80–240 WPM、步长 20，默认女声／180；单词、听音卡与逐表达变位使用同一保存偏好与换算规则。保存不更换选定音频，不改变学习或保留关系。

## 实测基准

Windows x64，AMD Ryzen 9 8945HS，8 核／16 线程。沿用 Supertonic 3 int8 F1／M1、CPU、正常合成 `speed=1.0`、八步去噪和 48 kbit/s Ogg Opus；模型创建与编码复用应用现有实现。

三条句子分别有 15、17、21 个书写词，每声线共 53 词。含字母／数字的空白分隔项计一词，撇号／连字符内部不拆分，纯标点不计；完整音频时长保留句内停顿。基准为总词数 × 60 ÷ 总时长，播放速率为目标 WPM ÷ 声线基准。

| 声线 | 基准 WPM | 80 WPM 速率 | 180 WPM 速率 | 240 WPM 速率 |
| --- | ---: | ---: | ---: | ---: |
| 女声 F1 | 149.25668599453368 | 0.535989389 | 1.205976126 | 1.607968168 |
| 男声 M1 | 149.14890109597928 | 0.536376731 | 1.206847645 | 1.609130193 |

运行时基准保留六位小数。完整输入、帧数、时长和生成耗时见 [calibration.json](calibration.json)，测量入口见 [calibrate.py](calibrate.py)。[试听页](listen.html)在同一份正常合成音频上播放六个端点并开启音高保持；六个 Ogg 文件均通过编码时长检查。

2026-11-08，用户试听女声／男声的 80／180／240 WPM 后确认：“Clear and natural at all speeds”。这六个端点的清晰度与自然音高通过人工听感确认。

2026-11-08，用户报告干净系统测试通过，其他平台测试按用户要求暂缓。

复测从仓库根目录执行，输出目录必须不存在：

```powershell
.\target\pr23-native-speech-20261101\speech\runtime\python.exe -B -X utf8 docs/evidence/issue-56/calibrate.py target/pr23-native-speech-20261101/speech target/issue-56/calibration-new
```

测量不重建用户选定音频。模型许可、使用条件和来源沿用 [Supertonic 通知](../issue-14/supertonic-licenses/ATTRIBUTION.md)。

## 自动检查

固定 pnpm 11.25.0、Rust 1.99.1。PowerShell 7 中为调用 Rust／Python 的 pnpm 命令使用固定工具链目录：

```powershell
$wpmCargo = rustup which --toolchain 1.99.1 cargo
$wpmPython = uv python find --offline
$env:PATH = (Split-Path -Parent $wpmPython) + ';' + (Split-Path -Parent $wpmCargo) + ';' + $env:PATH
```

| 命令 | 结果 |
| --- | --- |
| `pnpm lint`、`pnpm typecheck` | passed |
| `pnpm desktop:build --no-bundle`（包含 `pnpm build`） | passed；生产前端与 Windows x64 release 可执行文件 |
| `pnpm --filter @lexique/desktop exec playwright test e2e/speech-playback.spec.ts` | passed，2 项，包含六个端点及六份编码时长 |
| `pnpm --filter @lexique/desktop exec playwright test e2e/automatic-queue.spec.ts` | passed，12 项 |
| `pnpm test:e2e` | passed，154 项，50.0 秒 |
| `pnpm rust:test:build` → `pnpm test:rust` | passed，先预编译，再执行 145 项 |
| `pnpm rust:check` | passed，锁定 workspace 编译 |
| `cargo test --locked --release -p lexique-desktop --lib --test speech_cache --no-run` | passed，独立预编译 |
| `pnpm test:desktop:lib` | passed，3 项 |
| `pnpm test:speech:cache` | passed，10 项；1 项既有 crash fixture ignored，需要独立路径与原生重启脚本 |
| 对修改的 17 个原生 `.mjs` 入口执行 `node --check` | passed |
| `pnpm rust:fmt`、`pnpm test:docs`、`git diff --check` | passed；文档与本地链接无问题 |

前端检查覆盖全部合法档位、键盘端点／预设、草稿试听、保存失败与重试、当前播放保持原速、两声线及未知旧声线提示。原生检查验证真实 SQLite 的旧格式读取、新格式保存、非法值、损坏偏好、失败写入、重启恢复、共用声线换算与缺失资源错误；生词、摘要、笔记、卡片与进度、当前答案、选定版本和保护关系保持。

原生命令从 `apps/desktop/` 执行：

```powershell
node e2e/native-wpm-check.mjs ../../target/pr23-native-speech-20261101 ../../target/release/lexique-desktop.exe ../../target/desktop-conjugation-v6-stats ../../target/issue-56/native-wpm-2
```

结果：passed，10 项播放；实际 WebView2 的音频时间推进、端点速率及 `preservesPitch` 检查通过。Rust 与原生 runner 的执行硬限均保持 60 秒，编译独立进行。完整日志、原生 `wpm-results.json` 与检查截图保留在忽略的 `target/issue-56/`；浏览器桌面／390 px 三语布局截图已检查。开发服务与原生进程在检查结束后关闭。

## 限制

- WPM 是完整句子基准的近似换算，单个短词不保证精确达到目标词数。
- 本次未重新执行 #24 的独立强关 fixture；其他平台测试按用户要求暂缓，Lite 下载及八种正式发行物仍按 M5 验收。例句朗读入口仍未接入。
