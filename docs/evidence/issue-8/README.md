# Windows 小样离线查词验证

日期：2026-10-25。规格：[issue #8](https://github.com/zardonc/lexique/issues/8)。这只验证本机 Windows x64 开发构建，不代表安装器、完整默认资源或四目标发行通过。

## 构建与固定资源

| 项目 | 实测值 |
| --- | --- |
| 系统 / 架构 | Windows 11，NT 10.0.26300.0，x64 |
| 应用 | Lexique 0.1.0；`pnpm desktop:build --debug --no-bundle` |
| 可执行文件 SHA-256 | `df731c8b4ba0106f90d783c8136fa3d90be4ae3dbcbb7deebdaaf2e1c4920e22` |
| 内置资源 | FreeDict / WikDict fra-zho 2025.11.23，转换 v1 |
| SQLite SHA-256 | `11cdba7ed8fbdcd6f6a388586c11a181def517d840aae0baa6ef4a4ab5c55c51` |
| 记录 / 原始键 | 10,947 / 10,777 |

程序读取可执行文件旁的 `dictionary/dictionary.sqlite3`；该目录还包含原始源包、发布校验值、来源与原始 `COPYING`。构建准备不联网，资源错误不会触发下载或生成假结果。

## 离线方法

使用 Codex 的 Windows 受限执行环境启动最终可执行文件，保持启动器存活至应用退出。启动器用 Python 标准库调用 `OpenProcessToken` / `GetTokenInformation`，验证自己与实际应用的 `TokenIsAppContainer` 为真，`TokenCapabilities` 完全相同且不含 `S-1-15-3-1`、`S-1-15-3-2`、`S-1-15-3-3` 网络能力。启动前实测 `1.1.1.1:443` 被系统以 WinError 10013 拒绝，`example.com:443` 的 DNS 查询报 11001。

另以只读进程快照和令牌检查确认首次窗口及重启窗口各自的六个 `msedgewebview2.exe` 后代进程均处于相同 AppContainer、没有上述网络能力。普通受限进程无法枚举这些后代，CIM 访问也被拒绝，因此仅进程检查使用正常权限；被测应用与启动器始终保留受限令牌。未改变系统防火墙、网络、代理或安全设置。能力与隔离依据见 Microsoft 的 [AppContainer 说明](https://learn.microsoft.com/en-us/windows/win32/secauthz/implementing-an-appcontainer)、[令牌查询 API](https://learn.microsoft.com/windows/win32/api/securitybaseapi/nf-securitybaseapi-gettokeninformation) 和 [能力 SID 表](https://devblogs.microsoft.com/oldnewthing/20220503-00/?p=106557)。

实际窗口由 Computer Use 操作和截取；页面来自内嵌生产构建 `http://tauri.localhost/`，没有 Vite 服务或浏览器注入数据。原始令牌 / 探针 JSON、启动脚本和窗口文字保存在本地忽略目录 `.tooling/dictionary-admission/`，下面的截图进入版本控制。

## 实际观察

| 输入 / 操作 | 真实窗口结果 |
| --- | --- |
| `école` | 1 条；学校；`e.kɔl`；法语释义与来源 |
| `œil` | 1 条；眼睛 |
| `aujourd’hui` | 1 条；今天、今日；弯撇号保留 |
| `or` | 3 条独立记录；连词与名词记录没有合并 |
| `vacances` | 1 条；假期、休假；确实存在的复数形式 |
| `aujourd'hui` / `sommes` | 明确无完全匹配；不造词形、不归一化撇号 |
| 临时移走构建资源副本 | 资源不存在；旧释义清除 |
| 将副本换为损坏字节 | 资源已损坏；旧释义清除 |
| 在该副本路径创建空目录 | 查询失败；可展开原始读取错误 |
| 恢复原副本 | SHA-256 恢复原值，`école` 再次成功 |
| 正常关闭并重启同一构建 | 关闭进程退出 0；重启仍禁网；`école` 返回相同内容与来源 |

首个查词窗口进程为 37216。为明确记录正常关闭，后续进程 45212 经窗口关闭退出 0，再启动 46396 完成相同查询并退出 0。启动与重启的网络探针结果一致。

既有 `%APPDATA%/io.github.zardonc.lexique/lexique.sqlite3` 在首次启动前、查词 / 失败 / 替换后以及重启后的 SHA-256 均为 `877f6eea4d3d5e6d8ca4f8b7b0dcae86df2ea7ce88441752dfb883c8c3ce1ad2`。没有写入学习数据或新增业务表；这不证明笔记保存与恢复。

## 截图

- [首次 école 结果](ecole.png)
- [完整释义与来源](source.png)
- [or 三条记录计数](or.png)
- [精确匹配无结果](no-match.png)
- [资源不存在](missing.png)
- [资源损坏](corrupt.png)
- [查询失败](query-failed.png)
- [重启后相同结果](restart.png)

截图中的词典内容来自随仓库附带的 FreeDict / WikDict 小样，归属 Karl Bartel、FreeDict、WikDict、Wiktionary contributors、DBnary，按 [CC BY-SA 3.0](https://creativecommons.org/licenses/by-sa/3.0/) 提供；页面 UI 为项目源码生成。完整数据来源与转换说明见 [资源记录](../../../resources/freedict-fra-zho-2025.11.23/README.md)。
