# Issue #22：Windows x64 M2 集成验收完成

2026-11-01 收尾决定：用户确认 macOS 验收留待后续、不阻塞继续开发，且无其他问题时可关闭 #22。本轮独立审查与验证未发现其他阻塞，受审源码哈希复核未变，Windows x64 / 实际 WASM 范围验收完成。macOS 两目标编译/实机、其他架构实机和 M5 正式发行门槛仍按原边界保留；后续执行 macOS 验收时继续使用既有 [M1 阻塞清单](../../macos-m1-blockers.md)。

2026-11-01 完整验证结果见 [独立审查与执行报告](review-2026-11-01.md)。用户本轮补充确认全新安装的 Windows 在断网下可用且功能正常，已作为用户实测证据接受。两项独立代码审查未发现确定的新增缺陷；本轮可执行的 Windows x64 检查通过，包括当前正确 Tauri 构建的新包原生评分/播放/失败重试/移动/撤销，71 项浏览器与 70 项后端测试、实际 WASM、真实打包及 Windows ARM64 编译。macOS 两目标编译及其他架构实机仍有证据缺口，因此不把本报告等同全部平台或正式发行通过。下文保留历史候选和当时结论。

播放状态修复候选：`target/issue-22-playback-fixed/Lexique-0.1.0-windows-x64-portable-speech.zip`，166,811,289 字节，SHA-256 `77e3c5c6d9a619bbf757ab128521dbbdb2952df5b91d326e9a753b16e3ee9a81`，回读校验 1,072 个文件。听音卡在重播进行中不能评分，媒体错误后的迟到成功响应也不会解除评分限制；详情和队列复用相同播放处理。`pnpm lint`、`pnpm typecheck`、`pnpm build`、`pnpm desktop:build --no-bundle`、严格 Clippy 及 71 项浏览器 E2E 通过；浏览器使用模拟 IPC。新包的 `native-m2-check.mjs` 和 `native-speech-check.mjs` 使用隔离数据通过实际播放、慢速重播、失败与重试、评分、移动、撤销及重启恢复；文件符号链接检查因账户权限跳过。原生测试未限制网络，新包尚未在干净系统断网复测。

用户确认 `target/issue-22-anki-limits/Lexique-0.1.0-windows-x64-portable-speech.zip` 在干净 Windows 离线环境测试通过，确认的操作包括听音卡评分、失败后成功重试，以及移动后队列和额度恢复；此前用户还反馈离线启动、词典查询与播放、生词收藏和听音播放正常。这是用户实测报告，不推断未明确列出的系统版本、WebView2 / VC 安装状态或测试步骤。该包为 166,811,321 字节，SHA-256 `da10581a345fdefc802b41e6747fd0cb48f3dcbe924c933229a4c0684e8d6f92`；ZIP 内 1,072 个文件解压总计 261,124,579 字节。它包含应用、词典、离线语音及其内置 VC DLL；按用户决定依赖系统 WebView2，不捆绑其安装器，因此已具备系统 WebView2 的机器所需下载量即该 ZIP，缺失时还需用户另行取得微软安装器，安装器大小不包含在此数值中。

原生隔离测试另验证了评分和撤销写入失败的界面提示及进度不变、评分后实际自动收藏、批量部分失败后重试、移动后状态及撤销恢复；这些测试未限制网络。上述用户实测与自动化各自的边界不能互相替代。下文保留各次测试当时的历史结论。

本轮修复音频元素错误事件覆盖具体播放错误的问题后，重新构建 `target/issue-22-final-evidence/Lexique-0.1.0-windows-x64-portable-speech.zip`：166,811,343 字节，SHA-256 `e98e96e385031b51e51c2b40f35a9751dfa10828de375a28f8ecfa4ccb16bf28`，ZIP 1,072 文件解压总计 261,124,579 字节。与用户离线实测包的清单逐项比较，仅 `Lexique.exe` 哈希改变，其他文件一致。该新包的 `native-m2-check.mjs` 使用隔离数据库通过实际播放、评分 / 撤销失败注入及重试、批量部分失败及重试、设置、移动和撤销；没有限制网络。新包本身尚未在干净系统断网复测。

本轮 `pnpm lint`、`pnpm typecheck`、`pnpm build`、`pnpm rust:fmt`、`pnpm rust:check`、先预编译后 `pnpm test:rust`、`pnpm test:tooling`（含单独重跑且未跳过真实语音包测试）、`pnpm scheduling:build`、`pnpm test:scheduling` 和 `cargo check --locked --workspace --target aarch64-pc-windows-msvc` 通过；Playwright 全部 70 项通过，浏览器使用模拟 IPC。实际 WASM 与原生 71 个固定向量、4,096 个校准向量的排程一致，最大数值误差为零。Windows ARM64 实际运行、两个 macOS 目标的编译及实际运行未在本机验证，M5 四目标门槛不变。

2026-10-31 统一队列候选：`target/issue-22-unified-final-1031/Lexique-0.1.0-windows-x64-portable-speech.zip`，166,812,558 字节，SHA-256 `de2c216882a8109af228190ab0db5777823bbed3af8857f29d8018b983f4b5b7`，回读 1,072 文件。用户确认新卡在首次成功评分时计入额度，创建不扣额。`pnpm lint`、`pnpm typecheck`、`pnpm build`、`pnpm rust:fmt`、`pnpm rust:check`、`pnpm rust:test:build`、`pnpm test:rust`、`pnpm desktop:build --no-bundle` 通过。固定时间后端检查覆盖十词二十卡分别计额及超额双卡次日出现；最终包原生隔离检查覆盖统一队列直接出卡、连续双卡评分、新卡额度计数、损坏音频反馈及禁止评分、移动与撤销。检查未限制网络；浏览器 E2E 本轮因本机缺少 Playwright 自带浏览器未运行，干净系统复测仍待进行。Anki 的新卡上限已对齐上述确认范围，其他排程行为并未宣称完全相同。

2026-10-31 用户决定：识义卡评分后，同词听音卡可在当天继续复习，仍遵守各自的到期时间、音频和额度限制。此决定取代下文此前的同词暂缓规则。新候选 `target/issue-22-same-day-1031/Lexique-0.1.0-windows-x64-portable-speech.zip` 为 166,812,243 字节，SHA-256 `4526ca59a5ec1e196751e9e6bdf6376feae5c757b910826b5298db9a0f380ac6`，回读校验 1,072 文件。`pnpm rust:fmt`、`pnpm rust:check`、`pnpm rust:test:build`、`pnpm test:rust` 和 `pnpm desktop:build --no-bundle` 通过；隔离 Windows 原生检查确认识义评分后同词听音卡出现并播放，并通过原有存储、失败重试、移动和撤销检查。同日双卡评分资格由后端测试验证；该原生检查未限制网络，尚待用户在干净系统复测。

2026-10-31 用户复测反馈：未选择释义时例句无法保存，选择框下的提示不够醒目；队列能显示听音卡数量，但识义卡评分后听音卡不再出现。后者符合当前同词新卡 / 长期卡评分后暂缓至下一学习日的规则；用户对是否保留该规则的选择待确认。保存提示现同时显示在保存按钮附近，选择释义的校验保持不变。隔离 Windows 原生检查复现了旧包保存按钮附近缺少提示，并在新构建验证提示、选择释义后保存、跳过识义卡进入听音卡及后续原生流程；该检查未限制网络。

提示修复的中间候选：`target/issue-22-feedback-1031-candidate/Lexique-0.1.0-windows-x64-portable-speech.zip`，166,811,118 字节，SHA-256 `6561f827ffe61f34fa8d03dd8125cda67dae1d2c042245e44e8a41ce9c658254`，1,072 文件回读通过。`pnpm lint`、`pnpm typecheck`、`pnpm build`、`pnpm desktop:build --no-bundle` 和此包的 `native-m2-check.mjs` 通过。Playwright 自带浏览器未安装，因此本轮浏览器回归未运行；原生检查使用实际 WebView2 和隔离数据库，不代表干净系统或禁网验收。

2026-10-30 用户反馈：干净 Windows 11 断网启动正常；词典查询、播放和加入生词本正常；生词本显示、播放及创建识义卡和听音卡正常，听音卡播放正常。编辑例句时保留“未指定释义”，保存未成功，离开弹窗中的保存按钮也没有说明原因；闪卡页面只看到识义卡，当时尚未评分。此记录来自用户实测，包的确切版本、系统依赖状态及其他验收步骤未独立核实，不能视为完整 #22 离线验收。后续修复使离开弹窗说明必须选择释义，队列显示各类型数量，并允许跳过识义卡而不评分，以访问后续听音卡；相关浏览器回归使用模拟 IPC。

反馈修复候选：`target/issue-22-feedback-candidate/Lexique-0.1.0-windows-x64-portable-speech.zip`，166,811,115 字节，SHA-256 `586e2e26dfd3e2fd512aac7954b3f14be1e48f5c32593f991ad6a2f4dca188bd`，1,072 个文件回读校验。`pnpm lint`、`pnpm typecheck`、`pnpm build` 和 `pnpm desktop:build --no-bundle` 通过；`playwright test e2e/notebook.spec.ts e2e/review-queue.spec.ts --workers=1` 33 项通过，使用模拟 IPC。`native-m2-check.mjs` 对此候选及隔离数据库运行通过，报告评分、独立修改、移动及撤销后的状态保持；本次未限制网络，也未在原生脚本中单独验证新跳过入口。干净系统对该候选的复测仍待进行。

2026-10-30 干净系统候选：`target/issue-22-clean-system-candidate/Lexique-0.1.0-windows-x64-portable-speech.zip`，166,793,865 字节，SHA-256 `12a41040b95965f5801b3f4f1d95b3174b7b887781b756e9953bfff87addd95a`，1,072 个文件回读通过。ZIP 不含 WebView2；依用户决定使用系统 WebView2，缺失时由用户按包内说明自行获取微软 Evergreen Standalone Installer。已安装 WebView2 的机器无需额外下载；缺失时的完整离线下载量为本 ZIP 加用户所选微软安装器的实际字节数，该安装器未固定或测量。

对候选的所有 `.exe`、`.dll` 和 `.pyd` 用 `objdump -p` 检查直接 PE 导入：主程序没有直接导入 `VCRUNTIME` 或 `MSVCP`；语音 Python、Opus 和 NumPy 使用 `VCRUNTIME140.dll`，部分模块还使用 `VCRUNTIME140_1.dll`，两者位于 `speech/runtime/`。NumPy 私有 `msvcp140-*.dll` 和 OpenBLAS、Python、OpenSSL、sherpa-onnx 和 ONNX Runtime 的库均位于包内。其他直接导入包括 Windows 系统 DLL 和 Windows UCRT API 集；WebView2 由系统提供。打包器要求两个 VC DLL 存在，并通过固定清单核验所选运行库。PE 静态导入不能证明延迟加载或运行时加载的全部依赖，也不证明干净机器的实际搜索路径；[干净系统验收](clean-windows-test.md)仍是必要证据。

本轮 `pnpm lint`、`pnpm typecheck`、`pnpm build`、`pnpm rust:fmt`、`pnpm rust:check`、`pnpm rust:test:build` 后 `pnpm test:rust`、`pnpm test:tooling` 通过；真实语音资源打包测试未跳过。`pnpm test:speech:cache` 七项通过，其中 junction 检查因受限环境权限跳过；此前普通权限七项已实测。`pnpm test:scheduling` 实际原生 / WASM 的 71 + 4,096 向量通过。候选包的 `native-m2-check.mjs` 普通桌面权限全流程通过，结果位于 `target/issue-22-clean-candidate-moved/`；不作为禁网或干净机器证据。构建保留既有 Zod 注释和受限增量缓存硬链接回退警告。

2026-10-30 补充：使用当前 release 可执行文件重新生成的语音 portable 开发包，在普通桌面权限下运行更新后的 `native-m2-check.mjs` 全流程通过。隔离数据库验证了受损音频的播放失败提示及禁止评分、真实播放和听音评分、评分写入失败回滚及重试、评分后实际收藏自动加入、批量部分失败及成功重试、移动后队列和已用额度恢复、撤销写入失败提示及回滚、重试撤销后保留后续收藏和队列。结果及截图位于 `target/issue-22-current-moved-1030-normal6/`。测试进程已停止；该检查未限制网络，不能作为禁网播放证据。

新 ZIP 位于 `target/issue-22-delivery-current-1030b/`，大小 166,793,681 字节，SHA-256 `b77b2cafad0192b529a03bb167c69e19a3e06e2c697a315458b3cfc7737b5be6`，回读校验 1,072 个文件；它仍不包含 WebView2 / VC 系统运行库。`cargo clippy --locked -p lexique-core -p lexique-storage --all-targets -- -D warnings` 通过。禁网最终包全流程、完整离线依赖体积和干净机器验证仍未完成。

后续更新（2026-10-30）：以下访问被拒的调查已推进为修复。真实原生语音命令确认硬链接发布失败；改为不覆盖移动后，禁网 AppContainer 中生成及缓存回读通过，学习数据保留。完整过程和验证边界见 [缓存发布修复](cache-fix.md)。下文保留此前检查当时的结果，禁网播放及完整 M2 验收仍未完成。

2026-10-30，在 Windows x64 上使用隔离的测试数据库与便携包副本执行以下检查。用户学习数据未参与测试；原生调试通过本地 WebView2 回环连接运行，并非断网测试。

| 检查 | 结果 |
| --- | --- |
| `pnpm lint`、`pnpm typecheck`、`pnpm build` | 通过；Vite 报告依赖 PURE 注释警告 |
| `pnpm rust:fmt`、`pnpm rust:check`、`pnpm rust:test:build`、`pnpm test:rust` | 通过；先独立编译，执行阶段仍使用硬 60 秒超时 |
| `pnpm test:e2e` | 本机 Edge 66 项通过；浏览器使用模拟 IPC |
| `pnpm test:resources`、`pnpm test:tooling` | 通过，包括真实语音资源打包；默认临时目录权限失败后改用项目 `target/tmp` |
| `pnpm wasm:build`、`node scripts/check-wasm.mjs` | 实际加载 WASM 和 ABI 检查通过 |
| `pnpm scheduling:build`、`pnpm test:scheduling` | 实际原生 / WASM：71 固定及 4,096 校准向量，数值最大误差为零，排程字段无差异 |
| `cargo check --locked --workspace --target aarch64-pc-windows-msvc` | 通过；先调整 Python PATH，实际 ARM64 运行未测 |
| `native-speech-check.mjs` | 真实生成、播放、慢速重播、失败替换和重启恢复通过；文件符号链接检查因权限跳过 |
| `native-queue-check.mjs` | 队列、设置、评分、重启、撤销和继续评分通过；此脚本使用合成音频头 |
| `native-review-sort-check.mjs` | 使用 #21 完整样本复验同时间排序、双卡最早到期和重启恢复通过；较早的不同样本缺少搜索标记，首次运行超时 |
| `native-m2-check.mjs` | 真实语音生成后评分，编辑例句、调整设置、暂停并启用自动加入；错误撤销被拒且目标保留，移动目录并重启，成功撤销后独立修改保留；调度内容恢复，修订号保持单调 |

打包命令使用 #14 已接受的 `supertonic-evidence-v16-revised` 和 #15 Opus 资源：

```powershell
python -X utf8 scripts/package_portable.py --speech-bundle .tooling/issue-14/supertonic-evidence-v16-revised --speech-encoder .tooling/issue-15-opus/speech-stage/opusenc.dll --speech-sources .tooling/issue-15-opus --output-dir target/issue-22-delivery-20261030
node apps/desktop/e2e/native-m2-check.mjs target/issue-22-delivery-20261030/Lexique-0.1.0-windows-x64-portable-speech .tooling/issue-15-native-cache-check/Lexique-data/data/lexique.sqlite3 target/NEW-CHECK target/NEW-MOVED-CHECK
```

输出目录必须不存在。ZIP 为 165,610,559 字节，SHA-256 `f825397f4441608821f732261b45425ed7f4fa9bb9a1e366f2bde60ab3f3a12d`；回读校验 1,072 文件。解压文件总逻辑大小 261,056,585 字节，语音资源 241,571,209 字节；主程序 13,772,288 字节，SHA-256 `72c3a6d900b7f2e1f31f251c4769782a6f4e67b3da45e8c2994afab175ea82c4`。不包括系统 WebView2 / VC 运行库和运行数据，不能当成完整离线依赖下载量或干净机器占用。许可证及上游固定资源说明沿用 [#14](../issue-14/README.md) 和 [#15](../issue-15/README.md)；#15 的 `école` 音频已获用户听感确认，不能推断本轮新文件已逐一人工验收。

受限 AppContainer 探针确认测试进程及应用进程没有网络能力，`1.1.1.1:443` 返回 WinError 10013，DNS 查询失败。此前窗口自动化审批超时，后续已取得访问并在隔离数据库上操作原生窗口：离线查找并收藏 `bonjour`，自动加入创建卡片；识义卡翻面、评分、撤销及队列恢复通过。重启后 `bonjour` 和 `école`、例句及暂停状态仍显示；已有听音卡可打开，但未独立确认播放。批量加入时 `école` 成功，`bonjour` 卡片已加入但生成语音返回 `拒绝访问。(os error 5)`，重试仍失败，另一项成功结果保留。相同数据在普通权限下重试后界面显示语音已生成，缓存出现 `2.ogg`（9,555 字节）。受限环境下访问被拒的具体步骤尚未定位，不能将其归因于特定缓存操作，也不能将普通权限下成功生成视为离线生成通过。

补充隔离检查：受限进程连接 `1.1.1.1:443` 返回 10013 后，直接运行同一便携副本的 Python 运行时、实际语音工作进程、模型和 Opus 编码器，输入 `bonjour` / `female`，退出码 0，输出 9,555 字节且以 `OggS` 开头，stderr 为空。该检查不经过桌面缓存与 IPC，也未验证解码或播放；它证明受限环境下工作进程可生成输出，不证明完整应用离线生成通过。

缓存隔离探针直接编译桌面端 `speech_cache.rs`，在 `target/issue-22-cache-probe-data` 的独立目录调用实际读取和写入：受限进程读取不存在的音频返回 `Ok(None)`，写入返回 `os error 5`；单独逐步执行时临时文件创建、写入和同步成功，硬链接返回 `os error 5`。同一探针在普通权限下写入与硬链接均成功。这是此前应用错误的一个可复现候选原因，但应用进程中的具体失败调用尚未跟踪，不能据此证明普通离线 Windows 环境存在同样限制。探针源码及输出保留在 `target/issue-22-cache-probe.rs` 和 `target/issue-22-cache-probe-data`，不涉及用户学习数据。

进一步对照实际缓存代码的两条发布分支：同一受限进程内，使用新 ID 的无替换写入仍返回 `os error 5`，使用另一新 ID 的替换分支通过重命名成功，随后读取返回写入的 `OggSprobe` 字节。该探针使用合成数据，不证明音频有效或应用界面已恢复。替换分支可能覆盖已有目标，不能直接替代新文件发布的并发语义；尚需在真实应用调用处确认错误并验证不覆盖已有目标的替代方案。

同一受限进程中的独立 `MoveFileW` 路径探针：目标不存在时移动成功；目标已存在时返回 Windows 错误 183（AlreadyExists），目标字节保持原样。此实验没有使用受限目录句柄，不能证明满足缓存防重解析点的安全约束，也未验证并发或桌面端行为，不作为产品修复。

仍需：受限网络下成功生成和播放、双卡独立评分、部分失败后的成功重试及移动后的完整交互；评分或撤销失败时界面反馈及进度不变；评分后的实际自动加入操作；完整离线系统依赖压缩体积和干净机器验证；macOS 两目标编译，以及 Windows ARM64 和 macOS 两目标实际运行。上述拒绝测试仅验证 IPC 响应和撤销目标，不能证明界面反馈。macOS M1 阻塞和 M5 四目标发行门槛不变。完整 M2 验收尚未通过。
