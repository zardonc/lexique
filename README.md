# Lexique

法语离线词典与复习应用的开发骨架。Lexique 是暂定工作名，当前版本为 `0.1.0`，不是可用的学习产品。

基于 2026-10-24 的需求基线与最新原型调整：Tauri 2、React、TypeScript、Rust、SQLite。第一阶段面向 Windows 11 / macOS 的 x64 与 ARM64，第二阶段为包含实际 WASM 的自部署复习应用。

## 开始开发

安装 Node.js 22.12+、pnpm 11.25.0、Python 3.12+（仅标准库，用于内置资源转换）与 Rust。原生开发还需要 [Tauri 平台依赖](https://v2.tauri.app/start/prerequisites/)：Windows 的 MSVC C++ Build Tools / Windows SDK / WebView2，macOS 的 Xcode command line tools。Rust 工具链文件会安装所需组件和 WASM 目标。

```sh
pnpm install --frozen-lockfile
pnpm dev                 # 浏览器开发预览
pnpm desktop             # Tauri 原生窗口，初始化本地 SQLite
```

浏览器预览不会模拟 SQLite 成功，也不保存学习数据。Windows 使用便携目录：数据库位于 `.exe` 同级的 `Lexique-data/data/lexique.sqlite3`，WebView 缓存与临时文件分别位于 `Lexique-data/webview/` 和 `Lexique-data/temp/`，配置、缓存和日志也统一位于 `Lexique-data/`。路径基于可执行文件，与启动工作目录无关；目录不可写则启动失败，不回退到 AppData。既有 AppData 数据不会自动迁移或删除。macOS 暂沿用系统应用数据目录 `io.github.zardonc.lexique`。

## 仓库结构

```text
apps/desktop/             React 界面、运行时适配器与浏览器 E2E
  src-tauri/              Tauri 入口、状态 / 查词 / 生词本 / 建卡及语音命令
crates/core/              可编译为 WASM 的共享 Rust 核心入口
crates/storage/           SQLite 迁移、生词记录、幂等保存及双卡身份
crates/dictionary/        校验内置资源、只读 SQLite 快照查询、TEI 文本解析
docs/                     产品基线、模块职责、开发顺序与验证记录
scripts/                  60 秒后端测试限制、WASM 实际执行检查
.github/workflows/        前端、存储、WASM 及四目标原生编译检查
```

## 检查与构建

```sh
pnpm lint
pnpm typecheck
pnpm build
pnpm test:docs           # 跟踪文档的格式与仓库链接；本机证据链接单独计数
pnpm test:docs:validator # 隔离 Git fixture 验证文档检查的成功及失败路径
pnpm --filter @lexique/desktop exec playwright install chromium
pnpm test:e2e
pnpm rust:fmt
pnpm rust:check
pnpm rust:test:build      # 先编译测试，避免将首次编译计入运行预算
pnpm test:rust            # 真正执行后端测试，硬性 60 秒超时并终止进程树
pnpm wasm:build
node scripts/check-wasm.mjs
pnpm scheduling:build     # 编译同一调度入口的原生 CLI 与 WASM，编译不计入执行预算
pnpm test:scheduling      # 固定向量 + 4,096 个校准向量，实际执行并严格比较日期 / 阶段
pnpm desktop:build       # 默认 Cargo --locked；仅构建原生可执行文件，暂未开启安装包与签名
pnpm desktop:portable    # Windows x64 release ZIP，免安装，解压后运行 Lexique.exe
pnpm test:tooling        # 构建参数传递与隔离打包回归，不编译原生程序
pnpm test:speech:cleanup # Windows 缓存清理；需设置 LEXIQUE_TEST_EXECUTABLE
cargo test --locked --release -p lexique-desktop --test speech_cache --lib --no-run
pnpm test:speech:cache   # Windows 缓存发布 / 并发 / 重解析点；执行硬限 60 秒
pnpm test:desktop:lib    # 原生桌面库测试；执行硬限 60 秒
```

已有 portable 交付会被保留；再次交付可运行 `pnpm desktop:portable --output-dir <新目录>`。构建入口会在最终进程边界追加 Cargo 的 `--locked`，无需经 pnpm 手动传递分隔符。

文档检查与 #24 设计验证共用格式及链接规则，检查 Git 跟踪的 Markdown。`vendor/` 与 `.gitattributes` 标注 `-text` 的原样许可保持上游格式；`target/`、`.tooling/` 的本机证据链接及 `AGENTS.md` 的可选环境指针单独计数。CI 独立运行文档检查及其回归；Windows portable job 预编译缓存和桌面库测试后分别执行，两者各有 60 秒硬限。

Windows ARM64 开发样例可在已配置该目标工具链时先运行 `node scripts/build-desktop.mjs --no-bundle --target aarch64-pc-windows-msvc`，再运行 `python -X utf8 scripts/package_portable.py --architecture arm64`。打包器检查可执行文件的 PE 架构并保留已有交付；它不构成 ARM64 实机或正式发行验收。

便携交付默认位于 `target/portable/Lexique-0.1.0-windows-x64-portable.zip`，附带 `.zip.sha256`。解压整个 ZIP 到可写目录，保持 `dictionary/`、`conjugation/` 与 `Lexique.exe` 同级；首次启动生成 `Lexique-data/`，退出后可将整个目录一起移动或备份。更新时保留 `Lexique-data/`。打包命令拒绝覆盖同名交付目录或 ZIP；重复交付使用新的 `--output-dir`，保留使用中的旧目录。CI 产物 `windows-x64-portable` 提供相同布局。

当前便携包是未签名的 Windows 11 x64 开发小样，需要系统已有 WebView2；不启动依赖安装器、不联网补下载。无 WebView2 的干净机器和 ARM64 / macOS 发行尚未验收，完整离线依赖交付仍属于后续发行门槛。

Windows x64 语音开发小样可在构建后运行 `python -X utf8 scripts/package_portable.py --speech-bundle <已校验的 v16-revised 目录> --speech-encoder <固定 opusenc.dll> --speech-sources <Opus 源包目录> --output-dir <新目录>`。三个语音参数必须一起提供；打包器检查固定资源哈希、保留分别许可，并生成名称带 `-speech` 的新交付。默认 portable 不含模型，加入复习时会保留两卡并明确显示语音生成失败。这些开发小样不等于 M5 完整版 / Lite 发行；复验输入、命令和试听音频见 [#15 证据](docs/evidence/issue-15/README.md)。

资源独立入口：`pnpm resources:build target/dictionary-sample`、`pnpm resources:query target/dictionary-sample/dictionary.sqlite3 "œil"`、`pnpm test:resources`。源包和许可已随仓库保存，生成及查询无需联网；`build` 输出目录必须是新目录。原生构建自动准备 `target/desktop-dictionary-sample`，复用前验证内容；损坏时直接失败，人工检查后移走该生成目录再重建。可先运行 `pnpm resources:prepare`。来源、覆盖缺项、只读查询契约和体积见 [FreeDict 小样记录](resources/freedict-fra-zho-2025.11.23/README.md)。

变位资源入口：`pnpm conjugation:build target/conjugation-sample`、`pnpm conjugation:query target/conjugation-sample/conjugation.sqlite3 sommes`、`pnpm conjugation:table target/conjugation-sample/conjugation.sqlite3 falloir`、`pnpm test:conjugation`。固定 Lefff 3.4 词形及句法（LGPL-LR）、Grammalecte 2.3.0 元数据（GPL-3.0-only）和 RosaeNLG 省音资料（CC BY-SA 3.0）随仓库保存。原生构建自动准备格式 4 的 `target/desktop-conjugation-v4`，或先运行 `pnpm conjugation:prepare`；已有生成物须通过完整性及转换器版本检查，失败则保留原物，使用新目录重建。构建入口把两种资源放到可执行文件旁，portable 保留原始源、许可、纠正资料和五份转换源码；用户学习数据不参与资源准备。

桌面「动词变位」现可精确查询简单词形、选择所有候选、查看 23 个独立区块并辨认全部匹配格。可试 `sommes`、`suis`、`paie`、`falloir`。七个重点时态排在最前，未确认的结构和缺少词形明确标注；浏览器预览不模拟本机资源。普通复合／非限定表达覆盖全部已准入普通动词，23 区块报告无缺失或未确认状态；缺陷、代词专用及罕见／历史用法单独标明。可试 `avons aimé`、`je suis allée`、`en ayant aimé`、`j’habite`、`j’occirai`；来源策略、Windows 验证及首进程 CLI 延迟超预算见 [#41 证据](docs/evidence/issue-41/README.md)。英文释义已以独立字段接入词头和候选项：固定 FreeDict fra-eng 0.4.1（GPL-2.0-or-later），覆盖 1,377 个身份，其余显示暂缺；来源用法与代词式义项分别标明，见 [#42 证据](docs/evidence/issue-42/README.md)。代词式生成、着色及变位朗读仍属后续工单。来源与覆盖见 [#39 验证](docs/evidence/issue-39/README.md)，桌面与 Windows 原生检查见 [#40 验证](docs/evidence/issue-40/README.md)。

2026-11-05，#39 在同一 CLI 加入固定 Grammalecte 2.3.0 的传统组别、普通主动式助动词与用法元数据，保留原始标记、纠正依据及未知／不适用状态；构建报告列出全资源覆盖。数据许可分别为 Lefff 的 LGPL-LR 与元数据的 GPL-3.0-only，许可、源数据和转换材料随资源保留。新生成资源格式为 2；旧小样保持原样，使用新输出目录重新构建。查询须保留数据库旁的 `grammalecte-2.3.0/` 目录。见 [#39 证据与限制](docs/evidence/issue-39/README.md)。

E2E 结束后在 `apps/desktop/test-results/` 产生桌面 / 手机截图，在 `apps/desktop/playwright-report/` 产生报告。若使用已安装浏览器，可设置 `PLAYWRIGHT_EXECUTABLE_PATH` 为其可执行文件路径。测试自己的服务由 Playwright 启动与关闭，不复用其他开发服务。

## 当前交付边界

已具备三语导航、纯白 / 暖色 / 深色、可收起的较深灰色侧栏、移动布局、原生状态命令、SQLite 原子初始化和真实 WASM ABI 入口。查词页内不重复放置复习栏，服务设置不包含 Baidu。

桌面词典页已接入固定的 FreeDict 法中小样，完全匹配原始拼写并分开显示同键记录、释义和来源；可试 `école`、`œil`、`aujourd’hui`、`or`、`vacances`。缺项与资源错误分别显示；浏览器预览不能读取内置词典。

M1 生词本支持同词一份的逻辑引用、收藏时只读摘要与独立纯文本笔记。点击“加入生词本”后需明确保存；笔记可为空、可清空，最多 20,000 个 Unicode 标量（组合重音分别计数）。查询结果、摘要与来源不能由笔记编辑覆盖。支持搜索词头 / 摘要 / 笔记，以及词头、创建时间、修改时间的双向排序和偏好恢复；保存失败保留输入，旧版本冲突拒绝覆盖。Windows 正常关闭经过保存 / 放弃 / 继续编辑选择；已保存记录可离线重启恢复。验证与平台限制见 [M1 证据](docs/evidence/issue-10/README.md)；必须在 macOS 执行的项目统一记录为 [blocked](docs/macos-m1-blockers.md)。

共享核心使用 `fsrs-rs 6.6.2` 官方默认参数及显式的 `libm 0.2.16` 数学兼容补丁，修订标识为 `fsrs-rs/6.6.2+libm-0.2.16`。Windows x64 原生／实际 WASM 的 71+4,096 向量严格排程一致，包含原先相差一天的两个反例，见 [#13 证据](docs/evidence/issue-13/README.md)。M2 已接入用户库、评分、队列及一级撤销，并在 [#22](docs/evidence/issue-22/review-2026-11-01.md) 收尾；浏览器持久复习仍未实现。

#15 已接入生词详情的单个“加入复习”、识义卡与听音卡、真实错误及重启恢复。schema 3 增量保留 M1 内容；再次加入保持两卡身份，仅对缺失或失败音频重试。语音开发包使用 Supertonic 3 本地资源生成目标 48 kbit/s 的 Ogg Opus；听音卡进入播放一次、翻面不自动播放，重播读取正常 / 0.6 倍速偏好，正面不显示词头。音频位于 `Lexique-data/temp/speech/`，错误保存在 `Lexique-data/logs/speech-cache.log`。#24 已用绝对期限和选定内容保护取代早期“七天未使用”规则，具体保留及维护边界见[架构](docs/architecture.md)。Windows x64 的 #15 实播和重启检查属于历史证据；新构建及长期维护分别验收。

完整词典 / MDX / MDD、完整变位及例句朗读、语法笔记与分类、Drive / OneDrive 文件同步、翻译与 AI、练习、浏览器复习与 BFF 尚未实现。原型的演示数据、密钥表单和伪同步没有被搬进生产代码。

2026-11-02 同步规划已改为应用内管理数据与跨云后台批量文件同步；桌面 SQLite + 事务 outbox、不可变 JSONL、浏览器 IndexedDB 宿主及安全清理仍是待验证设计。见[综合评估](docs/sync-design.md)和[M4 / M6 规划](docs/roadmap.md)，不表示已接入云服务或浏览器持久化。

仓库包含已准入的 FreeDict 法中开发小样源包及其 CC BY-SA 3.0 许可；尚无正式发行的完整默认资源、模型、个人学习数据、服务凭据或 OAuth 配置。项目自有源码采用 [MIT](LICENSE)；第三方组件、数据、模型和声线分别遵守其上游许可，其他默认资源尚需逐项准入。已确认的开发链路、资源候选及验证门槛见 [开发准备清单](docs/development-readiness.md)。

产品范围见 [产品基线](docs/product.md)，模块边界见 [架构](docs/architecture.md)，实施顺序见 [开发路线](docs/roadmap.md)，实测范围见 [验证记录](docs/verification.md)。

2026-11-05，#24 及 #25–#34 的 Windows x64 本期验收完成。schema 17 已接入有界四卡窗口、真实修订／delta、按实际近期和本日需求修复、共享选定音频及统一 GC／容量／分块对账。三语运行状态分别报告登记逻辑字节、Windows 分配空间、永久／保护／可清理内容和保留的未知文件。完整性能、生命周期及平台限制见[实施记录](docs/evidence/issue-24/continuation-2026-11-04.md)与[验收清单](docs/evidence/issue-24/acceptance-matrix-2026-11-05.md)，提交前检查见[最终复核](docs/evidence/issue-24/final-review-2026-11-05.md)。
