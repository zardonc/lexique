# #24 最终复核

2026-11-05，Windows x64。比较基线为 `c1acaeb`。schema 17 的有界队列、增量补位、选定音频、持久需求和统一生命周期通过最终检查。主管执行前按已保存声线核对完整合成规格；不支持的规格明确进入永久失败，保留当前选定内容、笔记及原任务预算。已有有效内容仍可复用。

## 构建与测试

使用 PowerShell 7、pnpm 11.25.0、Node.js 24.22.1、uv 管理的 Python 3.14.10 和 Rust 1.99.1；Cargo 命令保持 `--locked`。后端、Windows 缓存、桌面库及每项原生执行均保留硬 60 秒限制，预编译独立进行。

| 检查 | 命令 | 结果／日志 |
| --- | --- | --- |
| 前端规范 | `pnpm lint` | [通过](../../../target/issue-24/final-2026-11-05-00361120/final-lint.log) |
| 类型 | `pnpm typecheck` | [通过](../../../target/issue-24/final-2026-11-05-00361120/typecheck.log)；生产构建再次执行 `tsc --noEmit` |
| 前端构建 | `pnpm build` | [通过](../../../target/issue-24/final-2026-11-05-00361120/build.log) |
| 浏览器回归 | `pnpm test:e2e -- --workers=2` | [101 passed，43.2 秒](../../../target/issue-24/final-2026-11-05-00361120/test-e2e.log) |
| Rust 格式 | `pnpm rust:fmt` | [通过](../../../target/issue-24/final-2026-11-05-00361120/final-rustfmt.log) |
| Rust 编译 | `pnpm rust:check` | [通过](../../../target/issue-24/final-2026-11-05-00361120/final-rust-check.log) |
| 后端预编译 | `pnpm rust:test:build` | [通过](../../../target/issue-24/final-2026-11-05-00361120/final-rust-test-build.log) |
| 后端执行 | `pnpm test:rust` | [138 passed](../../../target/issue-24/final-2026-11-05-00361120/final-test-rust.log) |
| 原生测试预编译 | `cargo test --locked --release -p lexique-desktop --test speech_cache --lib --no-run` | [通过](../../../target/issue-24/final-2026-11-05-00361120/final-desktop-test-build.log) |
| Windows 缓存 | `pnpm test:speech:cache` | [10 passed，1 ignored](../../../target/issue-24/final-2026-11-05-00361120/final-speech-cache.log)；独立子进程入口 `publication_crash_fixture` 需显式隔离路径 |
| 桌面库 | `node target/issue-24/final-2026-11-05-00361120/test-desktop-lib.mjs` | [3 passed](../../../target/issue-24/final-2026-11-05-00361120/final-desktop-lib.log)；运行器保持硬 60 秒限制 |
| 原生／WASM 调度 | `pnpm scheduling:build`、`pnpm test:scheduling` | [74 固定向量＋4,096 校准输入，零差异](../../../target/issue-24/final-2026-11-05-00361120/test-scheduling.log) |
| WASM ABI | `node scripts/check-wasm.mjs` | [实际加载执行通过](../../../target/issue-24/final-2026-11-05-00361120/wasm-abi.log) |
| 桌面生产构建 | `pnpm desktop:build --no-bundle` | [通过](../../../target/issue-24/final-2026-11-05-00361120/final-desktop-build.log) |
| 文档差异／链接 | `git diff --check`、`python -X utf8 target/issue-24/final-2026-11-05-00361120/check-links.py` | 差异检查通过；[39 份文档的本地链接均有效](../../../target/issue-24/final-2026-11-05-00361120/markdown-links.json) |
| 设计文档 | `pwsh -NoProfile -File docs/evidence/issue-24/design-validation/validate-docs.ps1` | [通过](../../../target/issue-24/final-2026-11-05-00361120/design-validation.log)；50 专项、12 优先回归及 R／V／T／F 追踪完整 |

## 原生执行

本表使用同一固定生产二进制 [fixed-binary.exe](../../../target/issue-24/final-2026-11-05-00361120/fixed-binary.exe)，SHA-256 `03cc8a3d50ceba9d6484ffc442d4bd48e881cefc36682f00ff3b302ac035f460`。每个隔离目录的 `run-metadata.json` 登记实际执行身份；资源来自已准入 CPU 语音开发包，fixture 均新建，不读取用户学习数据。

| 源码 | 通过的断言 | 原始结果 |
| --- | --- | --- |
| [native-saved-spec-24.mjs](../../../apps/desktop/e2e/native-saved-spec-24.mjs) | queued／retry_wait／中断执行的十种不支持规格均永久失败且不发布；合法规格完成；当前音频、笔记、终态及预算跨重启保留 | [结果](../../../target/issue-24/final-2026-11-05-00361120/saved-spec/results.json) |
| [native-product-24.mjs](../../../apps/desktop/e2e/native-product-24.mjs) | 六词真实 CPU 合成，单声道、有限且非静音的完整解码及实际播放；批量期间查词、编辑和评分提交；重启保留进度，缓存命中不加载模型 | [结果](../../../target/issue-24/final-2026-11-05-00361120/product/results.json) |
| [native-selected-audio-27.mjs](../../../apps/desktop/e2e/native-selected-audio-27.mjs) | 全局偏好不更换选定音频；normal／0.6 实播；损坏后按原女声修复，保留十四天上一版、笔记和卡片进度 | [结果](../../../target/issue-24/final-2026-11-05-00361120/selected-audio/results.json) |
| [native-window-24.mjs](../../../apps/desktop/e2e/native-window-24.mjs) | 10,000 词／20,000 卡：窗口四卡、总准入 20、初次最多八次 probe；100 次健康热请求无失败，零 probe／payload／元数据重载／progress 解析 | [结果](../../../target/issue-24/final-2026-11-05-00361120/window-10000/results.json) |

健康热窗口最近秩 p50／p95／max 为 53.9／83.5／86.1 ms，100 次尝试均通过；操作系统缓存未驱逐。原生脚本顺序执行，各有硬 60 秒上限。执行结束未发现本次宿主、语音 worker、WebView2 或测试服务残留，预览端口 1421 未监听。

从 `apps/desktop` 运行以下命令；每次目的目录必须是新路径。将 uv Python 所在目录加入本次进程的 PATH。

```powershell
$taskPythonPath = uv python find 3.14.10
if ($LASTEXITCODE -ne 0) { throw 'Required Python unavailable' }
$env:PATH = (Split-Path -Parent $taskPythonPath) + [IO.Path]::PathSeparator + $env:PATH
$taskPackage = '../../target/issue-24/delivery-2026-11-05/Lexique'
$taskBinary = '../../target/issue-24/final-2026-11-05-00361120/fixed-binary.exe'
node e2e/native-saved-spec-24.mjs $taskPackage $taskBinary ../../target/issue-24/recheck-saved-spec
node e2e/native-product-24.mjs $taskPackage $taskBinary ../../target/issue-24/recheck-product
node e2e/native-selected-audio-27.mjs $taskPackage $taskBinary ../../target/issue-24/recheck-selected-audio
node e2e/native-window-24.mjs $taskPackage $taskBinary ../../target/issue-24/recheck-window 10000 healthy 100
```

## 规范与规格

| 轴 | 结果 | 剩余 findings |
| --- | --- | ---: |
| Standards | 遵守仓库规范；无确定违反或需要处理的代码气味 | 0 |
| Spec | 已保存完整规格在执行前核对，错误可见；选定内容、恢复预算及有效缓存复用符合规格 18／20 | 0 |

原生回归与独立两轴复核均通过。实施范围与完整验收对应关系见[实施记录](continuation-2026-11-04.md)和[验收覆盖](acceptance-matrix-2026-11-05.md)。

## 限制

干净系统离线通过由用户报告，对应此前交付的 `61aae370…` 二进制；本表的新二进制未重做该检查。九个发布／GC 进程 kill 断点、50k 混合历史资源及其它专项仍按[验收覆盖](acceptance-matrix-2026-11-05.md)登记的执行身份解释，本次不重新执行。原生 CDP 使用 disable-gpu／允许自动播放参数；浏览器 fixture 不证明原生存储。其它平台／硬件、物理冷盘、小时级趋势、物理休眠、掉电／磁盘满及正式发行仍沿既定门槛验收。
