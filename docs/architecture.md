# 架构与开发边界

```text
React / TypeScript
    └─ RuntimeAdapter（注入）
        ├─ 浏览器：明确报告预览，不冒充本地存储
        └─ 桌面：Tauri invoke → Rust → SQLite

Rust core → wasm32-unknown-unknown（当前仅 schema_version ABI）
Rust storage → 注入 SQLite connection → 事务迁移
```

## 当前模块

| 位置 | 职责 | 禁止混入 |
| --- | --- | --- |
| `apps/desktop/src` | 页面壳、主题、本地化、交互状态 | 原生文件操作、真实凭据 |
| `src/platform/runtime.ts` | Tauri IPC 与 Zod 响应校验；显式浏览器预览 | 假成功、吞掉 IPC 错误 |
| `apps/desktop/src-tauri` | 平台路径、应用启动、命令装配 | UI 状态与未授权的远程内容 |
| `crates/storage` | SQLite 初始化与原子迁移 | 字典资源更新覆盖用户库 |
| `crates/dictionary` | 固定资源 SHA-256 校验、只读内存快照查询、TEI 解析为纯文本 | 用户库、网络、原始 HTML 执行 |
| `crates/core` | 可移植 Rust / WASM 入口 | 文件系统、Tauri、SQLite、令牌 |

首个迁移仅创建 `app_metadata` 并设置 schema version。笔记、分类、卡片、进度、日志与同步基线的物理结构尚未固化。未来迁移应增量增加；遇到更高 schema 立即报错，禁止降级重建。迁移在事务内完成，失败不提高版本。

状态命令只读且无外部输入；查词命令只接受 1–512 UTF-8 字节的无控制字符查询，保留原始拼写，不接受路径。通过应用命令清单自动生成权限，仅授权本地 main 窗口。未开放文件系统、shell、HTTP 插件或远程窗口。IPC 响应用 Zod 校验；SQLite 初始化错误保持原始错误链，不转换成成功。只有框架边界创建平台资源，存储逻辑接收 connection。

原生构建用 Python 标准库从已准入源包生成 SQLite，并附带来源、原始源包和许可；构建前验证已有生成物，损坏则失败。运行时由平台层解析 `dictionary/dictionary.sqlite3`，每次查询校验构建时固定的哈希，再反序列化同一批字节为只读 SQLite 快照，避免资源被并发替换时读取未验证内容。`entries.key` 采用二进制完全匹配与参数化查询，同键记录按源序排列；TEI 仅解析为 React 文本，不执行标记。缺失、损坏、查询失败与无结果分别返回。用户库路径、schema 及迁移完全独立。

## 后续模块放置

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

仓库中的配置仅包含构建、测试和应用标识，不包含部署配置、凭据、用户数据、服务端点或 OAuth 客户端配置。真实设置应在运行时注入并与秘密分开保存，绝不能放入 `VITE_*` 并打进前端。

安装器、签名、公证、更新和完整默认离线资源尚未就绪，`bundle.active` 因此关闭。开发图标是随骨架提供的原创临时 L 字形，不代表最终品牌。`pnpm desktop:build` 仅编译程序，不声称生成完整可分发安装包。CI 对四个系统 / 架构做编译检查，不能替代实际安装、首装断网和 TTS 验收。

依据：[Tauri Vite 配置](https://v2.tauri.app/start/frontend/vite/)、[命令调用](https://v2.tauri.app/develop/calling-rust/)、[权限能力](https://v2.tauri.app/security/capabilities/)。
