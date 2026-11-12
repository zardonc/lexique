# 例句朗读

[#65](https://github.com/zardonc/lexique/issues/65) 接入生词详情／编辑草稿、识义卡和听音卡背面的主例句及更多例句手动播放。详情沿用现有编辑界面，播放当前输入，不提交保存。三语按钮披露机器合成语音，复用 `SpeechButton`、`RuntimeAdapter.previewSpeech` 和既有 Supertonic 3 离线链路。

仅朗读副本进行 NFC、换行／制表符和连续空白整理；法语缺项不发请求，其他控制字符或超过 200 个 Unicode 标量显示明确提示。保存上限仍为每字段 20,000 个字符，原文不截断。新的播放读取已保存声线／WPM，沿用音高保持、七天临时缓存及真实失败重试。选择另一例句、编辑正在播放的文本、折叠更多例句、关闭卡片或离开页面释放播放器并忽略迟到响应。例句之间共用取消范围，包含详情到卡背的切换。

## 检查

- `pnpm install --frozen-lockfile`：passed，pnpm 11.25.0。
- `pnpm lint`、`pnpm typecheck`、`pnpm build`：passed。
- `pnpm test:e2e example-speech.spec.ts review.spec.ts speech-button.spec.ts --workers=2`：34 passed，覆盖新增例句及既有复习／单词发音。
- `pnpm test:e2e example-speech.spec.ts --workers=1`：6 passed；覆盖草稿规范化及原文保持、200／201、缺项／控制字符、超长保存、两类卡背、更多例句折叠、取消／迟到响应、错误重试、WPM／音高、三语和键盘播放。
- `git diff --check`：passed。

## 原生复验入口与限制

从 `apps/desktop` 执行 `node e2e/native-example-speech-check.mjs <语音资源包目录> <新构建的 Lexique.exe> <仓库 target/issue-65/新目录>`。复用既有原生夹具和 60 秒硬限；以全新用户库生成单词和双卡后，比对例句首次合成／播放前后的学习及选定单词音频，重启并隐藏模型后验证缓存播放，新句无缓存时验证真实错误。WebView 置为 offline，原生语音使用本地资源。

集成分支 Windows x64 检查：`pnpm desktop:build --no-bundle` passed；上述原生入口使用既有 `target/pr23-native-speech-20261101` 语音包、本次 `target/release/lexique-desktop.exe` 和全新 `target/issue-65/native-2`，结果 passed。首次无缓存合成与真实播放、正常退出重启、隐藏模型后的缓存播放及新句缺模型错误均通过；真实 SQLite 学习内容、双卡、进度、评分和选定音频快照保持。检查使用新可执行文件旁的配套词典；完整日志与结果保留于忽略的 `target/`。

浏览器测试注入 RuntimeAdapter 的 IPC 响应，不证明原生合成、缓存或实播。本票未更改 Rust、IPC、schema、持久化或语音模型；其他平台实机、整机禁网、人工音质及正式发行分别验收。
