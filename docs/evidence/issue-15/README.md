# 单个生词创建双卡、生成语音并重启恢复

2026-10-29。规格为 [#15](https://github.com/zardonc/lexique/issues/15)，前置 #13 / #14 当前均已关闭。实现沿用已同意的公开边界：真实 SQLite 存储 API、RuntimeAdapter 界面、Windows x64 原生语音 / 实播，以及后来同意的 portable 打包命令。没有修改用户学习数据或推进入 #16 的评分。

## 已实现与验证

- schema 3 增量迁移保留 M1 生词、只读摘要、逻辑引用、独立笔记及保存操作；真实 schema 2 记录升级前后完整公开序列化内容相等。迁移冲突回滚，更高版本拒绝，升级不批量建卡。
- 生词详情的“加入复习”经原生接口和立即事务建立识义卡 / 听音卡；同词同类型唯一，重复加入、提交失败重试和数据库重开保持身份及生词内容。两卡不复制记录，也不通过重建重试语音。
- 语音开发包使用 [#14 已接受的 Supertonic 3 候选](../issue-14/README.md)及[固定源码构建的 Opus DLL](opus-encoder.md)。本地合成输出目标 48 kbit/s 的 Ogg Opus；缺失 / 损坏缓存可重试，生成失败保留两卡和旧音频。机器合成标识显示在听音卡上。
- 识义正面展示词头；听音正面隐藏词头、进入播放一次、翻面不播放，背面显示摘要和“待补例句”，笔记 / 来源按需展开。重播读取正常 / 0.6 倍速偏好；失败提示重播和跳过。关闭后重开会取消上一轮延迟播放；成功重播清除失败标志，再次加入复用音频。当前没有评分或排程计数可被跳过改写。
- 临时缓存位于可执行文件同级 `Lexique-data/temp/speech/`，按最近使用保留七天；启动即检查，之后每次清理完成休眠 24 小时。能力目录句柄、nofollow、规范文件名和目录锁保护路径；过期应用临时文件可清理，占用 / 无关文件保留。清理错误持久保存到 `Lexique-data/logs/speech-cache.log`，不依赖控制台。

## 检查结果

| 命令或公开边界 | 结果 |
| --- | --- |
| `pnpm lint`、`pnpm typecheck`、`pnpm build` | 通过 |
| `pnpm test:e2e`，`PLAYWRIGHT_EXECUTABLE_PATH` 指向现有 Edge | 28 项通过，其中 [review.spec.ts](../../../apps/desktop/e2e/review.spec.ts) 10 项 |
| `pnpm rust:test:build`，随后 `pnpm test:rust` | 29 项通过，包括建卡 4 项和迁移 / 初始化 5 项；执行阶段保留硬 60 秒超时 |
| `cargo fmt --all --check`、`cargo clippy --locked --offline --workspace --all-targets -- -D warnings`、workspace 编译 | 通过 |
| `pnpm desktop:build --no-bundle` | 最新 Windows x64 release 构建通过 |
| `pnpm wasm:build`，随后 `node scripts/check-wasm.mjs` | 编译后的 WASM 实际加载和 schema 3 ABI 调用通过；没有改变 FSRS 算法 |
| `pnpm test:resources` | 7 项通过 |
| `pnpm test:tooling`，配置下述真实资源 | 2 项构建参数、7 项基础打包和 3 项语音打包通过；真实集成项未跳过 |
| `pnpm test:speech:cleanup`，配置 `LEXIQUE_TEST_EXECUTABLE` 为最新 release | 4 项原生测试通过；含目录联接、跨进程锁协调、废弃临时文件和无 stderr 的日志 / 重启恢复 |
| [原生检查脚本](../../../apps/desktop/e2e/native-speech-check.mjs) | 损坏缓存、同进程并发生成、失败替换保留旧音频、成功替换及真实播放器 / 卡面 / 重启检查通过 |
| 规范 / 规格两项 code-review，基线 `7430729` | 无剩余缺陷；成功重播标志、持久诊断及废弃临时音频问题修复后通过回归 |

原生测试从独立合成生词库复制 `école`，观察真实 `HTMLAudioElement` 的播放事件、时间推进和结束状态，没有替换 `play()`。打开听音卡仅播放一次，翻面不增加播放次数；重播为 0.6 倍速。关闭 / 重启后，公开 IPC 返回的生词记录、两卡身份和缓存音频与之前相等。截图为 [正面](listening-front.png)和[背面](listening-back.png)。缓存文件符号链接检查因当前权限不足明确跳过，目录联接保护另由原生清理测试验证。

保留的警告：Vite 对依赖 PURE 注释的解释警告；Windows 沙箱增量缓存硬链接失败后复制的警告。最初沙箱打包因系统临时目录权限失败，获准在普通进程重试后通过。测试只关闭自己的应用 / 服务进程。

## 交付和试听

开发交付位于 `target/issue-15-delivery-20261029/Lexique-0.1.0-windows-x64-portable-speech/`，ZIP 和 `.zip.sha256` 在同一父目录。解压整个 ZIP 后启动 `Lexique.exe`；新目录不包含个人 `Lexique-data/`，运行数据之后在程序旁创建。旧交付保持原样。

| 对象 | 逻辑字节数 | SHA-256 |
| --- | ---: | --- |
| 开发 ZIP | 165,205,335 | `81ae90b7a1cc9e1b646289e4cf3a9d91084efa709484f07485fbf539bbd83fa8` |
| `Lexique.exe` | 11,589,120 | `94635df8223a60ad352f0e9e631dc72645b221fa21fe86893804d4d575f87221` |
| 新生成的 `école` Opus | 8,275 | `911fa59adb80bd47671e84ccb63ae2a81dad3354d9c98f83287e2b7793072cab` |
| 同一 Opus 独立解码的 WAV | 119,568 | `495238b8e3266f4f46d9539cccb367ff6990b35bc03ebff8a758487b5bcb3db1` |

ZIP 回读校验 1,072 个文件；解压文件逻辑总量为 258,873,423 字节，其中语音资源 241,571,209 字节。该计量不包括系统既有 WebView2 / VC 运行库、NTFS 分配空间和后来生成的运行数据。原生检查使用的较早新交付与最终交付具有相同的可执行文件、模型和运行库；最终更新了使用说明。

试听：[实际新生成的女声 Opus](audio/native-ecole-female.opus)和[独立解码 WAV](audio/native-ecole-female.wav)。输入为准确的 `école`，默认女声、正常速度生成；慢速在播放时使用 0.6 倍速。opusinfo 识别 libopus 1.6.1 / libopusenc 0.3、单声道、原始 44,100 Hz；opusdec 得到 59,762 个有效 PCM16 帧，峰值 11,799。48 kbit/s 是编码目标，短文件实际码率可变化。技术解码及播放通过不代替用户的法语听感判断；此新文件尚未获得人工验收。

## 复验输入与命令

资源不随源码提交，来自本机已保留的固定证据包。目录和旧输出均保持原样；重新运行须使用新的输出目录。

```powershell
$env:COREPACK_HOME=(Resolve-Path '.tooling/corepack').Path
$env:PLAYWRIGHT_EXECUTABLE_PATH='C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe'
$env:LEXIQUE_SPEECH_BUNDLE=(Resolve-Path '.tooling/issue-14/supertonic-evidence-v16-revised').Path
$env:LEXIQUE_OPUS_ENCODER=(Resolve-Path '.tooling/issue-15-opus/speech-stage/opusenc.dll').Path
$env:LEXIQUE_OPUS_SOURCES=(Resolve-Path '.tooling/issue-15-opus').Path
$env:LEXIQUE_TEST_EXECUTABLE=(Resolve-Path 'target/release/lexique-desktop.exe').Path
pnpm test:tooling
pnpm test:speech:cleanup
python -X utf8 scripts/package_portable.py --speech-bundle $env:LEXIQUE_SPEECH_BUNDLE --speech-encoder $env:LEXIQUE_OPUS_ENCODER --speech-sources $env:LEXIQUE_OPUS_SOURCES --output-dir target/issue-15-new-delivery
node apps/desktop/e2e/native-speech-check.mjs target/issue-15-new-delivery/Lexique-0.1.0-windows-x64-portable-speech .tooling/issue-15-native-cache-check/Lexique-data/data/lexique.sqlite3 target/issue-15-new-native-check
```

本轮通过的原生检查输出在 `target/issue-15-native-final-20261029/`。其源包是 `target/issue-15-final-20261029/Lexique-0.1.0-windows-x64-portable-speech/`；最终交付的代码 / 资源哈希相同，使用说明更新后另存。文件哈希和可复验的检查汇总见 [report.json](report.json)。

## 保留的边界

- 新音频主观听感由用户确认；不把 API 播放成功描述为扬声器输出测量。
- 本轮原生窗口实播没有施加外部网络限制，未做完整桌面禁网验收；#14 的独立受限网络证据不自动扩展为此结果。
- 连续运行 24 小时后的触发、跨进程同时生成、长时间播放期间清理及不支持硬链接的文件系统未测。启动清理和锁协调已实测。
- Windows x64 当前机器通过；其他目标、干净机器、签名 / WebView2 缺失、完整版 / Lite 下载及完整 M5 许可 / 包体仍待后续。现有 #14 上游 provenance / 量化证据边界保持用户已接受的范围。
- #16 接评分 / 排程持久化，后续工单接队列、撤销、批量管理、自动加入、例句编辑和复习日期排序；不宣称整个 M2 完成。
