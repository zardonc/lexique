# Issue #22：Windows x64 M2 集成证据（未完成）

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
