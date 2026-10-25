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

浏览器预览不会模拟 SQLite 成功，也不保存学习数据。桌面数据库保存在 Tauri 的系统应用数据目录 `io.github.zardonc.lexique` 中，独立于项目与资源目录；初始化失败会阻止启动并保留原始错误。

## 仓库结构

```text
apps/desktop/             React 界面、运行时适配器与浏览器 E2E
  src-tauri/              Tauri 入口、只读状态 / 查词命令与最小权限
crates/core/              可编译为 WASM 的共享 Rust 核心入口
crates/storage/           SQLite 初始化、事务迁移与集成测试
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
pnpm desktop:build       # 仅构建原生可执行文件；暂未开启安装包与签名
```

资源独立入口：`pnpm resources:build target/dictionary-sample`、`pnpm resources:query target/dictionary-sample/dictionary.sqlite3 "œil"`、`pnpm test:resources`。源包和许可已随仓库保存，生成及查询无需联网；`build` 输出目录必须是新目录。原生构建自动准备 `target/desktop-dictionary-sample`，复用前验证内容；损坏时直接失败，人工检查后移走该生成目录再重建。可先运行 `pnpm resources:prepare`。来源、覆盖缺项、只读查询契约和体积见 [FreeDict 小样记录](resources/freedict-fra-zho-2025.11.23/README.md)。

E2E 结束后在 `apps/desktop/test-results/` 产生桌面 / 手机截图，在 `apps/desktop/playwright-report/` 产生报告。若使用已安装浏览器，可设置 `PLAYWRIGHT_EXECUTABLE_PATH` 为其可执行文件路径。测试自己的服务由 Playwright 启动与关闭，不复用其他开发服务。

## 当前交付边界

已具备三语导航、纯白 / 暖色 / 深色、可收起的较深灰色侧栏、移动布局、原生状态命令、SQLite 原子初始化和真实 WASM ABI 入口。查词页内不重复放置复习栏，服务设置不包含 Baidu。

桌面词典页已接入固定的 FreeDict 法中小样，完全匹配原始拼写并分开显示同键记录、释义和来源；可试 `école`、`œil`、`aujourd’hui`、`or`、`vacances`。缺项与资源错误分别显示；浏览器预览不能读取内置词典。

完整词典 / MDX / MDD、完整变位、离线 TTS、笔记与分类、FSRS、Google OAuth / Sheets、翻译与 AI、练习、浏览器复习与 BFF 尚未实现。WASM 当前只导出版本查询，不能算作 FSRS 或浏览器复习验收。原型的演示数据、密钥表单和伪同步没有被搬进生产代码。

仓库包含已准入的 FreeDict 法中开发小样源包及其 CC BY-SA 3.0 许可；尚无正式发行的完整默认资源、模型、个人学习数据、服务凭据或 OAuth 配置。项目自有源码采用 [MIT](LICENSE)；第三方组件、数据、模型和声线分别遵守其上游许可，其他默认资源尚需逐项准入。已确认的开发链路、资源候选及验证门槛见 [开发准备清单](docs/development-readiness.md)。

产品范围见 [产品基线](docs/product.md)，模块边界见 [架构](docs/architecture.md)，实施顺序见 [开发路线](docs/roadmap.md)，实测范围见 [验证记录](docs/verification.md)。
