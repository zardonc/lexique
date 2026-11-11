# 法英补充 MDX 与自选资源链接接入

2026-11-11，Windows x64 开发包提供一份 FreeDict 法英 0.4.1 MDX；设置提供八类、12 项自选来源／下载页链接。其余词典仅链接、不自动下载。完整默认词典／CEFR 按 D2 单独准入。

## 资源与许可

| 项目 | 已核实结果 |
| --- | --- |
| 原内容／转换日期 | 2018-08-11／2026-11-10 |
| 格式 | MDX 2.0、UTF-8、zlib、未加密；无 MDD |
| 文件／记录 | 273,758 字节；8,505 条物理记录、8,417 个键 |
| MDX SHA-256 | `b4fcca046acb225a9cc00998d99fa9f9390337f7c022be271167dd435b7e5a21` |
| 内容／转换器 | GPL-2.0-or-later／MIT；作者、完整源码与修改说明随包 |
| 分发角色 | 较旧且有缺项的法英补充词典，保留源译文与重复条目 |

[资源说明](../../../resources/freedict-fra-eng-mdx-0.4.1/README.md)、[manifest](../../../resources/freedict-fra-eng-mdx-0.4.1/manifest.json)和[完整原始源](../../../resources/freedict-fra-eng-0.4.1/README.md)给出来源及重建路径。原始源目录保持不变，便携包复用 `conjugation/freedict-fra-eng-0.4.1/` 中的完整原档案；MDX 文件、许可和 converter 位于 `dictionary/freedict-fra-eng-mdx/`。构建及打包校验实际 MDX 与完整对应源码／通知，失败发生于交付目录创建之前。

内置 ID 为 `builtin-fra-eng`，文件位于应用控制的资源目录并复用既有 MDX reader／HTML 隔离。来源摘要与编译时 manifest 一致后才赋予 FreeDict／GPL 来源信息；替换成另一份合法 MDX 仍返回损坏错误。两项内置词典可排序、不可移出；满额旧目录保留 15 份自定义 MDX 和原顺序，末尾补入新的内置词典，共 17 项。用户库 schema 保持 17。

## 检查与结果

所有命令在隔离 worktree 执行。Python 使用 uv 管理的 3.14.10，Node／pnpm 使用项目固定版本，Cargo 使用 `--locked`。完整日志位于忽略目录 `target/d2-research/mdx/`。

| 命令或检查 | 结果 |
| --- | --- |
| `uv run --no-project --offline --no-python-downloads --python 3.14.10 python -B -X utf8 scripts/freedict_mdx.py` | 通过；资源与原始源／通知一致 |
| 上述入口加 `--output target/d2-research/mdx/freedict-rebuild-2026-11-10 --writer target/d2-research/mdx/french-bundle-proof/writer_support/writemdict.py` | 通过；由完整 TEI 重建、全部记录及 MDX 字节一致 |
| `pnpm lint`、`pnpm typecheck`、`pnpm build` | 通过 |
| `pnpm rust:fmt` | 通过 |
| `pnpm rust:test:build` → `pnpm test:rust` | 预编译通过；执行阶段硬性 60 秒，152 项通过 |
| `cargo test --locked --release -p lexique-desktop --lib --no-run` → `pnpm test:desktop:lib` | 15 项通过；移出保护、满额旧目录、合法文件替换及真实 FreeDict 查词覆盖 |
| `pnpm --filter @lexique/desktop exec playwright test dictionary-resources.spec.ts dictionary-tabs.spec.ts` | 9 项通过；三语、640 px、八类／12 项、容量、正文隔离、迟到查询、ID-only 链接及可见失败 |
| `pnpm test:e2e --workers=4 --output <独立结果目录>` | 211 项通过；26 个测试文件，Playwright 自带无头 Chromium；查词、资源目录、设置、生词本、复习、变位、取词／OCR 及语音界面回归 |
| `python -B -X utf8 -m unittest discover -s scripts -p test_package_portable.py -v` | 12 项通过；唯一随包 MDX、原始源码／通知、坏资源拒绝、旧交付与用户数据保持、ARM64 打包夹具 |
| `pnpm test:build-args` | 两项通过；公开构建入口复制 MDX／词典／变位／词频，失败保留失败状态 |
| `pnpm desktop:build --no-bundle` | Windows x64 release 构建通过 |
| `python -B -X utf8 scripts/package_portable.py --output-dir target/d2-research/mdx/portable-freedict-20261110` | 33,998,493 字节、151 文件；逐文件及 ZIP 回读通过，一份法语 MDX、无用户数据 |
| `pnpm test:docs`、`git diff --check` | 通过；155 份文档、1,269 个本地链接、86 个忽略目录／外部链接跳过。文档检查使用独立临时 Git index 纳入本次新文件，原暂存状态保持 |

完整界面回归的[日志](../../../target/d2-research/mdx/continuation-e2e-20261111T092358320/frontend-e2e.log)与[HTML 报告](../../../target/d2-research/mdx/continuation-e2e-20261111T092358320/playwright-report/index.html)保存在独立目录；通过 `PLAYWRIGHT_HTML_OUTPUT_DIR` 和 `--output` 指定报告与截图位置。

新开发包 SHA-256 为 `3977224f51a8640a662585c351ebc333ef9f243ed88f17953744f68af2cff15b`。原生检查程序 SHA-256 为 `5d28f37c1f6f9b90df107c451b75a6dcd6be6ec71371d4eef36386ddc8bb7517`；原生检查使用新包的词典与源码，语音沿用已验收的 #49 资源。

## 原生 Windows 检查

`apps/desktop/e2e/native-dictionary-check.mjs` 通过已有 native harness 启动真实 WebView2，在独立新目录执行，单次硬性 60 秒；关闭后确认宿主退出码 0。四个模式在 `apps/desktop/` 下分别运行：

```powershell
node e2e/native-dictionary-check.mjs MODE ../../target/d2-research/mdx/native-resources ../../target/release/lexique-desktop.exe ../../target/release/conjugation ../../target/release/frequency NEW_DESTINATION
```

| 模式／结果 | 已覆盖行为 |
| --- | --- |
| [bundle](../../../target/issue-9/d2-mdx-native-bundle-20261110/result.json)；三项通过 | 首次真实 FreeDict HTML／GPL 来源、收藏、旧目录重启升级、用户 MDX／独立笔记／FSRS／选定音频保持、资源缺失恢复、内置保护、八类目录、原生拒绝任意 URL 输入 |
| [tabs](../../../target/issue-9/d2-mdx-native-tabs-20261111/result.json)；四项通过 | 原有导入、排序／重启、按需正文、物理重复、缺失／损坏／不支持／上限、HTML 隔离、移出后文件保留 |
| [lemmas](../../../target/issue-9/d2-mdx-native-lemmas-20261111/result.json)；三项通过 | 原形歧义、未保存笔记保护、同形名词／代词式／撇号、目标缺项、来源升级、重启及移出后的已提交操作重放保持学习数据 |
| [links](../../../target/issue-9/d2-mdx-native-links-20261111T094436886/result.json)；一项通过 | 使用目录 ID `larousse-synonyms` 调用真实系统浏览器启动器，学习数据保持；宿主退出码 0 |

四次均拦截非本地 WebView 请求，实际外部 WebView 请求为零。[中文原生目录](resources-native.png)与[法文窄屏目录](resources-narrow.png)已检查布局。

`links` 检查实际打开 Microsoft Edge 的 Larousse 同义词页面；用户确认地址为 `https://www.larousse.fr/dictionnaires/synonymes` 且页面正常加载。[浏览器观察记录](../../../target/issue-9/d2-mdx-native-links-20261111T094436886/browser-observation.json)与原生结果分开保留。

整机离线辅助入口为 [native-offline-dictionary-check.ps1](../../../apps/desktop/e2e/native-offline-dictionary-check.ps1)：等待物理网卡离线、确认 HTTPS 探测失败，再运行已有 `bundle` 检查并回读网络状态。脚本不修改网络设置；语法与既有证据目录拒绝／保持检查通过，实际断网执行按用户选择暂缓。

## 四平台库测试与远端构建

2026-11-11，[验证提交 e97a77f](https://github.com/zardonc/lexique/commit/e97a77f588c7d7b562251fe7d9d07b9b6fe8dd8b) 的 [CI](https://github.com/zardonc/lexique/actions/runs/43085339848) 11 项任务全部通过。四目标均先 `cargo check --locked --workspace`，预编译测试后执行 `pnpm test:rust`，另预编译 release 桌面库后执行 `pnpm test:desktop:lib`。

| 平台／原生目标 | 核心／词典／存储 | 桌面库 | 后端执行步骤 |
| --- | --- | --- | --- |
| Windows x64／`x86_64-pc-windows-msvc` | 152 通过 | 15 通过 | 17 秒 |
| Windows ARM64／`aarch64-pc-windows-msvc` | 152 通过 | 15 通过 | 40 秒 |
| macOS 15 Intel／`x86_64-apple-darwin` | 152 通过 | 13 通过 | 28 秒 |
| macOS 26 Apple Silicon／`aarch64-apple-darwin` | 152 通过 | 13 通过 | 28 秒 |

native job 使用 `CARGO_PROFILE_TEST_OPT_LEVEL=1`，保留调试断言；`RUST_TEST_THREADS=1` 隔离无关夹具写入，各用例内部的并发仍照常执行。存储、词典注册与语音缓存夹具的目录名包含进程内序号，固定重复时钟值的回归检查通过。原 `scripts/test-rust.mjs` 的硬性 60 秒执行上限保持不变，首次编译单独进行；表中时间来自 CI 步骤起止时间。

同一提交的前端 lint／typecheck／build、211 项 Playwright 回归、Linux 后端／MDict／WASM 排程校准，以及三个系统的词典 7 项、变位资源 36 项、便携打包 12 项检查均通过。三系统资源任务及 Windows portable 的 `test:tooling` 步骤，语音打包检查各有两项因缺少编译产物／模型夹具而跳过；Windows portable 任务的缓存检查 11 项通过，崩溃子进程专用夹具一项按设计忽略，桌面库 15 项通过。

CI 便携 ZIP 为 34,006,635 字节、151 个文件，仅含一份 FreeDict 法英 MDX；归档完整性、MDX、对应源码与许可及已发布校验值全部回读通过。结果见[四平台摘要与日志](../../../target/d2-research/mdx/verification-43085339848/ci-summary.json)和[归档回读](../../../target/d2-research/mdx/verification-43085339848/final-artifact-result.json)。

## 合并前复核

2026-11-11，相对 `origin/main`（`ea2c95e`）完成 Standards 与 Spec 双轴审查，两轴均为零项发现。当前范围、来源页 IPC、内置保护、旧目录容量、资源源码／通知和学习数据保持符合仓库规范及随包规格。

本地重新执行 `pnpm lint`、`pnpm typecheck`、`pnpm build`、`pnpm test:build-args`、`pnpm test:docs`、`pnpm rust:fmt`、`pnpm rust:check`，全部通过。按上表入口重新校验 FreeDict 与便携打包，12 项打包测试通过；预编译后 152 项核心／词典／存储和 15 项桌面库测试通过，60 秒执行上限保持；211 项 Playwright 用例全部通过。完整日志保存在 `target/pr-review-20261111/`。

已从 GitHub 当前代码提交的 check-runs 回读确认，上述四平台 CI 的 11 项任务全部成功。此次本地检查沿用前述原生与发行验收边界。

## 尚未验收的范围

Windows 系统浏览器启动已验证；macOS 的 `open` 仍需实际操作验证。整机断网、缺 WebView2 的干净机器首装、macOS／Windows ARM64 的 GUI 实机与 M5 完整版／Lite 正式发行未执行。

WikDict 部分原作表达的目标再分发依据仍未闭合，只提供原格式来源链接；其他候选文件未下载到分发包，大小或版次未知均在目录注明。MDD 媒体、独立脚本／样式、TSV／StarDict 直接导入和完整 D2／CEFR 未接入，详见[研究报告](../../d2-resource-admission-research.md#法语随包范围与用户自选链接)。
