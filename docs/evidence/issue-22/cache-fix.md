# Windows 受限环境中的新语音缓存发布修复

2026-10-30。范围是 Windows x64 AppContainer 中的 `os error 5`，不推断普通离线 Windows 具有相同限制。

## 环境与根因

依照根目录 `LOCAL-DEV-ENV.md` 使用 PowerShell 7、nvm 管理的 Node.js 24.22.1、项目缓存中的 pnpm 11.25.0 和 uv 管理的 Python。受限进程无法使用默认 Corepack / uv 缓存，构建命令分别使用项目 `.tooling/corepack` / `target/uv-cache`；`python` 的 WindowsApps 占位程序不能运行，故用 `uv python find --offline` 定位真实解释器，并仅为当前命令调整 PATH。期间 uv 安装位置更新，最终解释器为 Python 3.14.10；没有改变网络、ACL 或系统设置。

`target/issue-22-cache-regression.exe` 在受限环境连续三次于实际 `speech_cache::write(..., false)` 返回 `拒绝访问 (os error 5)`，每次不足一秒。最小场景为读取不存在的缓存成功，随后发布一个新文件失败。正式回归 `publishes_new_audio_without_overwriting_an_existing_file` 在修复前同样失败。

诊断构建在原生应用启动后自动调用实际 `speech_generate`，不依赖窗口可见性。隔离副本使用既有测试数据库及已验证语音资源；进程令牌为 AppContainer、无互联网能力，`1.1.1.1:443` 返回 10013。两次命令都返回明确标记的硬链接发布错误：`publish hard link: 拒绝访问 (os error 5)`。该标记位于工作进程退出成功及 Ogg 输出校验之后、实际缓存硬链接调用的错误分支，因而确认了本次原生语音命令的失败调用，而不再只是邻近缓存探针。

## 实现与安全约束

Windows 的新文件发布改为 `MoveFileW`，其目标存在时失败而不覆盖。路径来自仍打开的缓存目录句柄 `GetFinalPathNameByHandleW`，不是重新使用前端或外部传入的路径；源临时名和目标正整数 ID 均在应用内部生成。继续使用原有 nofollow 目录打开、目录锁、临时文件创建 / 写入 / 同步及失败清理；已有音频的明确替换仍走原分支。未增加依赖、迁移或改动卡片进度。

该路径方案遵循已安装的 [cap-std](https://github.com/bytecodealliance/cap-std) 4.0.3 Windows 实现：目录句柄不设置 `FILE_SHARE_DELETE`，防止目录及其祖先在路径操作期间被改名或移走。包元数据许可为 `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT`；参考其 `dir_utils.rs` 和 `rename_unchecked.rs`，没有复制上游代码。接口依据为 [MoveFileW](https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefilew) 和 [GetFinalPathNameByHandleW](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfinalpathnamebyhandlew)。

直接带 RootDirectory 的重命名方案没有采用：本次受限环境中 Win32 版本返回参数错误，Nt 版本返回无效句柄，后者在普通权限下通过。不能据此确定其受限环境内部机制；最终实现不依赖该方案。

## 回归和实测

独立编译后运行，执行阶段保留现有硬 60 秒限制：

```powershell
$env:COREPACK_HOME = (Resolve-Path '.tooling/corepack').Path
$env:UV_CACHE_DIR = (Resolve-Path 'target/uv-cache').Path
$taskPython = uv python find --offline
$env:PATH = (Split-Path -Parent $taskPython) + ';' + $env:PATH
cargo test --locked --release -p lexique-desktop --test speech_cache --no-run
pnpm test:speech:cache
pnpm rust:test:build
pnpm test:rust
```

- 受限环境：新文件与明确替换、不覆盖已有文件、八个协调写入者、32 轮不参与锁的目标创建竞争、占用目标时替换失败保留旧音频、目录目标拒绝、目录及祖先不能被移走共六项通过；释放目录句柄后可以移动并恢复。junction 创建被环境拒绝，重解析点项明确跳过。
- 普通权限：七项缓存回归全部通过，包括真实 junction 下读取、新建和替换都拒绝，并保留外部目标文件。测试仅创建并清理 `target/speech-cache-tests` 中的自有隔离目录；已接入 Windows portable CI，远端 CI 本轮未运行。
- 修复后同一禁网原生诊断路径：首次生成及再次调用均返回 9,555 字节、以 `OggS` 开头的音频；测试数据库全部应用表内容前后相同。报告位于 `target/debug-22/after/Lexique-data/debug-22.json`，修复前报告位于 `target/debug-22/before/Lexique-data/debug-22.json`。这证明实际模型、编码、桌面缓存发布和回读，不证明禁网播放。
- `pnpm rust:fmt`、`pnpm rust:check`、预编译后的 `pnpm test:rust`（65 项）、`pnpm lint`、`pnpm typecheck` 和 `node --check scripts/test-rust.mjs` 通过。保留编译器对既有增量缓存硬链接失败的复制回退警告；前端构建保留既有 Zod PURE 注释警告。
- 删除诊断标记后的 `pnpm desktop:build --no-bundle` 通过；原最小缓存探针按当前源码重新编译到 `target/debug-22/cache-regression-fixed.exe` 并通过，旧失败二进制保留作对照。
- 最终无诊断入口的程序在普通权限下运行 `node apps/desktop/e2e/native-speech-check.mjs target/issue-22-delivery-20261030/Lexique-0.1.0-windows-x64-portable-speech .tooling/issue-15-native-cache-check/Lexique-data/data/lexique.sqlite3 target/issue-22-cache-fix-native`，其中 `LEXIQUE_TEST_EXECUTABLE` 指向 `target/release/lexique-desktop.exe`。真实播放、慢速重播、翻面、跳过、腐坏缓存恢复、并发生成、生成失败保留旧音频、再次生成及重启恢复通过。文件 symlink 检查因创建权限跳过；目录 junction 由上述普通权限缓存回归实际覆盖。截图保留在 `target/issue-22-cache-fix-native/`，脚本启动的应用和连接已停止。此项不施加网络限制，不用它补充禁网播放结论。

临时诊断标记及启动入口已从产品源码删除；调查程序及隔离资源副本保留在明确标识的 `target/debug-22/`，不参与产品构建。用户学习数据、既有交付和此前证据未被改写。

禁网播放、干净机器、其他目标的实际运行以及完整 M2 / M5 验收仍需独立验证。
