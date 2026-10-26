# 架构与开发边界

```text
React / TypeScript
    └─ RuntimeAdapter（注入）
        ├─ 浏览器：明确报告预览，不冒充本地存储
        └─ 桌面：Tauri invoke → Rust → SQLite

Rust core → 同一 FSRS 调度入口 → 原生 / wasm32-unknown-unknown
Rust storage → 注入 SQLite connection → 事务迁移
```

## 当前模块

| 位置 | 职责 | 禁止混入 |
| --- | --- | --- |
| `apps/desktop/src` | 页面壳、主题、本地化、交互状态 | 原生文件操作、真实凭据 |
| `src/platform/runtime.ts` | Tauri IPC 与 Zod 响应校验；显式浏览器预览 | 假成功、吞掉 IPC 错误 |
| `apps/desktop/src-tauri` | 平台路径、应用启动、命令装配 | UI 状态与未授权的远程内容 |
| `crates/storage` | SQLite 迁移、生词记录、幂等保存与版本冲突、搜索排序 | 字典资源更新覆盖用户库 |
| `crates/dictionary` | 固定资源 SHA-256 校验、只读内存快照查询、TEI 解析为纯文本 | 用户库、网络、原始 HTML 执行 |
| `crates/core` | 可移植 Rust / WASM、固定 FSRS 与日历学习日调度 | 文件系统、Tauri、SQLite、令牌 |

首个迁移创建 `app_metadata`；第二个迁移增量创建生词记录与已提交保存操作。NFC 标准词头唯一约束与立即事务防止并发副本；操作编号绑定原请求，在响应丢失后返回当前记录且不重放写入；修订号检查和更新在同一事务中完成。收藏时摘要与来源为只读 JSON，独立笔记另列保存。只保存已提交操作，不持久化草稿。用户库与资源库独立；分类、卡片、进度、日志和同步尚未固化。遇到更高 schema 立即报错，禁止降级重建；迁移失败回滚且不提高版本。

M1 请求由 Rust 严格反序列化，前端请求 / 响应由 Zod 校验。标准词头最多 200 个 Unicode 标量且无控制字符，独立笔记最多 20,000 个标量；保留笔记原样，组合重音不作自动合并。新记录的来源和摘要在平台层重新查询已校验词典生成，IPC 不接受前端提供的来源或摘要。逻辑引用保留源查询拼写，避免 NFC 身份归一改变现有精确查询。搜索独立采用 NFD 去组合音标及小写后的字面子串；排序同值时使用稳定身份打破平局。

原生数据命令在阻塞任务中重新打开已初始化用户库（不在库丢失时静默新建），读取错误保持可见。窗口正常关闭先阻止默认退出，再通知 React；明确选择保存并得到成功确认、放弃，或当前没有未保存修改时才调用受限 `finish_close` 销毁窗口。macOS 菜单 Quit / Cmd+Q 替换为同一路径；macOS Dock Quit 的框架终止路径尚未覆盖，须在该平台验收前解决。main 窗口仅新增生词命令、事件监听和完成关闭权限，不开放文件系统、shell 或网络插件。

状态命令只读且无外部输入；查词命令只接受 1–512 UTF-8 字节的无控制字符查询，保留原始拼写，不接受路径。通过应用命令清单自动生成权限，仅授权本地 main 窗口。未开放文件系统、shell、HTTP 插件或远程窗口。IPC 响应用 Zod 校验；SQLite 初始化错误保持原始错误链，不转换成成功。只有框架边界创建平台资源，存储逻辑接收 connection。

原生构建用 Python 标准库从已准入源包生成 SQLite，并附带来源、原始源包和许可；构建前验证已有生成物，损坏则失败。运行时由平台层解析 `dictionary/dictionary.sqlite3`，每次查询校验构建时固定的哈希，再反序列化同一批字节为只读 SQLite 快照，避免资源被并发替换时读取未验证内容。`entries.key` 采用二进制完全匹配与参数化查询，同键记录按源序排列；TEI 仅解析为 React 文本，不执行标记。缺失、损坏、查询失败与无结果分别返回。用户库路径、schema 及迁移完全独立。

## 后续模块放置

#13 在共享核心提供 `scheduling::schedule` 及严格 JSON 入口 `schedule_json`，接收卡片、评分、Unix 秒时间与 IANA 时区，不读取设备时钟。输出记忆状态、阶段、完整步骤秒数或取整天数、绝对到期时间、经过学习日、模型参数和时区库版本；调用方须保存完整请求和结果作为历史，不以最新设置重建旧请求。固定 FSRS 6.6.2 的标量调度数学使用显式 vendored 兼容补丁，统一调用关闭默认 features 的 libm 0.2.16；公式、官方参数、步骤及取整不变。历史标识 `fsrs-rs/6.6.2+libm-0.2.16` 与未修改上游包区分，拒绝混用。Windows x64 native / 实际 WASM 对照通过，其他平台实际执行另验。此票不迁移用户库、不添加 IPC 或评分 UI。原生 CLI 与 WASM 的有界输入缓冲区调用同一 JSON 入口；缓冲区 ABI 为同步调用，下一次请求使旧指针失效。详见 [#13 验证](evidence/issue-13/README.md)。

| 能力 | 建议实现位置 | 先决条件 |
| --- | --- | --- |
| 词典 / 变位资源 | 独立 Rust crate + 桌面资源适配 | 许可、数据格式、覆盖集与 HTML 隔离 |
| FSRS | 可编译 WASM 的纯 Rust 核心 | 算法实现 / 参数确认与跨端固定样例 |
| TTS / 音频 | 桌面平台适配，按需独立 crate | 模型许可、法语质量、四架构验证 |
| 凭据 / OAuth | 桌面系统凭据库；Web 经 BFF | 权限、回调、访问控制与刷新策略 |
| 同步 | 独立业务模块，网络 / 本地存储注入 | 身份、基线、冲突、删除及恢复语义 |
| Web 复习 / BFF | 第二阶段新增应用，复用已验证核心 | WASM 真正调度、认证、限流和 CSRF |

暂不创建空 BFF 或假接口。当前没有 HTTP 服务，无需为了骨架引入账户、端点或密钥。未来新增端点必须同时带输入 schema、鉴权、限流与适用的 CSRF 防护。外部词典 HTML 不可直接渲染或获得原生权限。

## 配置与发行

语音偏好通过受限 `speech_preferences` 命令读取／保存，沿用 `app_metadata` 中单条 JSON；不提高 schema 版本。Rust 枚举及前端 Zod 严格限制声线 `siwis`／`tom` 和语速 `normal`／`slow`。未知字段、非法响应和损坏配置保持错误，不覆盖为默认值；只有配置不存在时使用女声／正常默认值。当前没有合成或播放命令，偏好不改变学习进度或既有音频。

Windows 的 `tauri.windows.conf.json` 通过 Tauri `appDirectoriesOverride` 将五类应用目录定位到可执行文件旁的 `Lexique-data/`；WebView 未设置独立 `dataDirectory`，由框架继承被覆盖的 local-data 路径。入口在启动框架线程前将子进程继承的 `TEMP` / `TMP` 定位到同一目录下的 `temp/`，并固定 `WEBVIEW2_USER_DATA_FOLDER` 到 `webview/EBWebView/`，避免继承的外部路径覆盖框架设置。用户库、WebView profile、配置、日志和临时文件因此随目录移动，不依赖启动工作目录。不可写时保留失败，不回退系统目录；原 AppData 内容保持原样。操作系统自身的事件日志、预取和安全软件记录不由应用控制。

`pnpm desktop:portable` 构建 Windows x64 release 后，仅打包可执行文件、已准入词典资源、许可与使用说明，生成独立 ZIP 和 SHA-256 清单，并逐项回读校验压缩包。新交付必须使用空路径；不打包运行时 `Lexique-data/`，也不覆盖旧交付。当前仍依赖系统已有 WebView2，不启动安装器或首次联网下载；这不等于缺运行依赖机器、签名或完整资源发行已验收。

公开原生构建入口由 `scripts/build-desktop.mjs` 用参数数组调用已安装 Tauri CLI，在最终进程边界追加 Cargo 的 `-- --locked`，避免 pnpm 消耗转发分隔符。portable 使用同一入口，可通过 `--output-dir` 指定新交付父目录；隔离工具测试已接入现有 CI。macOS 专属退出及实际运行统一由 [blocked 清单](macos-m1-blockers.md) 管理。

仓库中的配置仅包含构建、测试和应用标识，不包含部署配置、凭据、用户数据、服务端点或 OAuth 客户端配置。真实设置应在运行时注入并与秘密分开保存，绝不能放入 `VITE_*` 并打进前端。

安装器、签名、公证、更新和完整默认离线资源尚未就绪，`bundle.active` 因此关闭。开发图标是随骨架提供的原创临时 L 字形，不代表最终品牌。`pnpm desktop:build` 仅编译程序，不声称生成完整可分发安装包。CI 对四个系统 / 架构做编译检查，不能替代实际安装、首装断网和 TTS 验收。

依据：[Tauri Vite 配置](https://v2.tauri.app/start/frontend/vite/)、[命令调用](https://v2.tauri.app/develop/calling-rust/)、[权限能力](https://v2.tauri.app/security/capabilities/)。
