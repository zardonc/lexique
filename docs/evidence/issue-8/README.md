# Windows 小样离线查词验证

日期：2026-10-25。规格：[issue #8](https://github.com/zardonc/lexique/issues/8)。下方先记录 debug 查词与故障验证，随后补充实际交付的 release portable ZIP；这不代表完整默认资源或四目标发行通过。

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

## Windows portable 交付补充

按用户新增要求，Windows 改为免安装 ZIP；运行文件统一放在 `.exe` 同级的 `Lexique-data/`。不自动导入或删除旧 AppData，不打包个人运行数据。

| 项目 | 实测值 |
| --- | --- |
| 构建命令 | `pnpm desktop:portable`；release、`--no-bundle -- --locked` |
| 交付 | `Lexique-0.1.0-windows-x64-portable.zip`，4,972,902 字节 |
| ZIP SHA-256 | `0a48d529fc98c619a76e353faa661716b61c9894b99e68c728a65554b219c116` |
| Lexique.exe | 10,581,504 字节；SHA-256 `b70531a4f49a8744667f7b42cad84efaf20e7875d7d0f05d9dae1faff04ff383` |
| 解压载荷 | 16,143,343 逻辑字节，10 个文件；不含运行数据 |
| 运行依赖 | 本机已有 WebView2 155.0.4283.62；未运行安装器或下载依赖 |

打包入口验证 PE 为 Windows x64、词典符合准备阶段哈希，再只复制程序、固定词典、来源和许可、使用说明及 SHA-256 清单。ZIP 中每项回读校验；输出目录、ZIP 或单独残留的 `.zip.sha256` 已存在时均拒绝创建 / 覆盖，旧文件哈希保持不变。原生导入表用 `dumpbin /DEPENDENTS` 检查，仅列出 Windows 系统 DLL / UCRT，未列出额外 VC++ redistributable DLL；这不能代替干净机器运行验收。

从最终 ZIP 解压至带中文和空格的目录，工作目录仍为仓库根目录。启动器另设置一个仓库内、但位于程序目录外的 `WEBVIEW2_USER_DATA_FOLDER`，确认入口覆盖该值，外部目录始终未创建。实际应用进程 10292 与其六个 WebView 后代均保持上文的禁网 AppContainer 令牌；`école` 返回学校、法语释义和相同来源，见 [最终包首次查词](portable-ecole.png)。

正常关闭退出 0 后，将整个解压目录从 `portable 最终 B` 移至 `portable 移动后 C`，数据库和 WebView profile 一起携带。重启进程 45868 与六个 WebView 后代仍禁网，实际命令行的 `--user-data-dir` 已定位到新目录下的 `Lexique-data/webview/` 子树；旧程序路径与外部环境变量指定路径没有重新生成。再次查询 `école` 返回相同内容与来源，见 [移动后重启查词](portable-moved.png)，随后正常关闭退出 0。

首次运行、移动前后及重启后的 portable 用户库 SHA-256 均为 `877f6eea4d3d5e6d8ca4f8b7b0dcae86df2ea7ce88441752dfb883c8c3ce1ad2`；只有既有 `app_metadata` 表，未创建业务表。旧 Roaming 用户库保持同一哈希，真实 Local AppData 中原有 WebView 的 173 个文件大小和修改时间前后完全一致。新运行的 `data/`、`temp/`、`webview/` 均位于程序旁；配置、缓存、日志的框架路径也已统一覆盖，但当前没有功能向它们写入。

目录故障使用本次生成的独立程序副本，把预期 `Lexique-data` 路径预先占为普通文件；最终 release 程序退出 101，阻挡文件和旧用户库保持原样，不回退 AppData。这覆盖无法创建便携目录的失败；没有改变目录 ACL 或系统安全设置。旧构建在加入 portable 配置前已经观察到“exe 旁无数据库”的回归失败。独立审查发现的继承 WebView 路径和遗留校验文件覆盖问题均已修正并验证。

这次交付仍是未签名的 Windows 11 x64 开发小样，需要已有 WebView2。未验收缺 WebView2 的干净机器、系统事件日志 / 预取等应用无法控制的系统记录、其他架构、macOS、完整资源或笔记恢复；ZIP 的约 5 MB 不能作为完整产品包体估算。CI 产物与最终远端检查结果在 issue #8 的交付记录中列明。
