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
  src-tauri/              Tauri 入口、状态 / 查词 / 生词本命令与关闭协调
crates/core/              可编译为 WASM 的共享 Rust 核心入口
crates/storage/           SQLite 迁移、生词记录、幂等保存与集成测试
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
```

已有 portable 交付会被保留；再次交付可运行 `pnpm desktop:portable --output-dir <新目录>`。构建入口会在最终进程边界追加 Cargo 的 `--locked`，无需经 pnpm 手动传递分隔符。

便携交付默认位于 `target/portable/Lexique-0.1.0-windows-x64-portable.zip`，附带 `.zip.sha256`。解压整个 ZIP 到可写目录，保持 `dictionary/` 与 `Lexique.exe` 同级；首次启动生成 `Lexique-data/`，退出后可将整个目录一起移动或备份。更新时保留 `Lexique-data/`。打包命令拒绝覆盖同名交付目录或 ZIP；重复交付使用新的 `--output-dir`，保留使用中的旧目录。CI 产物 `windows-x64-portable` 提供相同布局。

当前便携包是未签名的 Windows 11 x64 开发小样，需要系统已有 WebView2；不启动依赖安装器、不联网补下载。无 WebView2 的干净机器和 ARM64 / macOS 发行尚未验收，完整离线依赖交付仍属于后续发行门槛。

资源独立入口：`pnpm resources:build target/dictionary-sample`、`pnpm resources:query target/dictionary-sample/dictionary.sqlite3 "œil"`、`pnpm test:resources`。源包和许可已随仓库保存，生成及查询无需联网；`build` 输出目录必须是新目录。原生构建自动准备 `target/desktop-dictionary-sample`，复用前验证内容；损坏时直接失败，人工检查后移走该生成目录再重建。可先运行 `pnpm resources:prepare`。来源、覆盖缺项、只读查询契约和体积见 [FreeDict 小样记录](resources/freedict-fra-zho-2025.11.23/README.md)。

E2E 结束后在 `apps/desktop/test-results/` 产生桌面 / 手机截图，在 `apps/desktop/playwright-report/` 产生报告。若使用已安装浏览器，可设置 `PLAYWRIGHT_EXECUTABLE_PATH` 为其可执行文件路径。测试自己的服务由 Playwright 启动与关闭，不复用其他开发服务。

## 当前交付边界

已具备三语导航、纯白 / 暖色 / 深色、可收起的较深灰色侧栏、移动布局、原生状态命令、SQLite 原子初始化和真实 WASM ABI 入口。查词页内不重复放置复习栏，服务设置不包含 Baidu。

桌面词典页已接入固定的 FreeDict 法中小样，完全匹配原始拼写并分开显示同键记录、释义和来源；可试 `école`、`œil`、`aujourd’hui`、`or`、`vacances`。缺项与资源错误分别显示；浏览器预览不能读取内置词典。

M1 生词本支持同词一份的逻辑引用、收藏时只读摘要与独立纯文本笔记。点击“加入生词本”后需明确保存；笔记可为空、可清空，最多 20,000 个 Unicode 标量（组合重音分别计数）。查询结果、摘要与来源不能由笔记编辑覆盖。支持搜索词头 / 摘要 / 笔记，以及词头、创建时间、修改时间的双向排序和偏好恢复；保存失败保留输入，旧版本冲突拒绝覆盖。Windows 正常关闭经过保存 / 放弃 / 继续编辑选择；已保存记录可离线重启恢复。验证与平台限制见 [M1 证据](docs/evidence/issue-10/README.md)；必须在 macOS 执行的项目统一记录为 [blocked](docs/macos-m1-blockers.md)。

共享核心使用 `fsrs-rs 6.6.2` 官方默认参数及显式的 `libm 0.2.16` 数学兼容补丁，修订标识为 `fsrs-rs/6.6.2+libm-0.2.16`。Windows x64 原生 / 实际 WASM 的 71+4,096 向量严格排程一致，包含原先相差一天的两个反例。复验、上游差异、许可与限制见 [#13 证据](docs/evidence/issue-13/README.md)。尚未接入用户库或评分界面，也不是浏览器复习产品。

完整词典 / MDX / MDD、完整变位、离线 TTS、语法笔记与分类、持久化复习、Google OAuth / Sheets、翻译与 AI、练习、浏览器复习与 BFF 尚未实现。原型的演示数据、密钥表单和伪同步没有被搬进生产代码。

仓库包含已准入的 FreeDict 法中开发小样源包及其 CC BY-SA 3.0 许可；尚无正式发行的完整默认资源、模型、个人学习数据、服务凭据或 OAuth 配置。项目自有源码采用 [MIT](LICENSE)；第三方组件、数据、模型和声线分别遵守其上游许可，其他默认资源尚需逐项准入。已确认的开发链路、资源候选及验证门槛见 [开发准备清单](docs/development-readiness.md)。

产品范围见 [产品基线](docs/product.md)，模块边界见 [架构](docs/architecture.md)，实施顺序见 [开发路线](docs/roadmap.md)，实测范围见 [验证记录](docs/verification.md)。
