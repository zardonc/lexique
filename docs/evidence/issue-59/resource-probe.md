# 法语 OCR 与屏幕采集准入

2026-11-08，[#59](https://github.com/zardonc/lexique/issues/59)，需求为“首期支持屏幕区域 OCR 并核对文字后离线查词”。固定 Tesseract.js／core 7.0.0 + tessdata_fast `fra`，XCap 0.9.9 提供原生屏幕快照。许可、固定来源和法律原文见 [资源目录](../../../resources/tesseract-fra/README.md)。

## 本地 OCR

隔离安装使用 `pnpm --dir target/ocr-probe install --ignore-workspace --ignore-scripts`，未修改应用依赖。`node target/ocr-probe/probe.mjs` 在本地 HTTP 服务提供固定 worker、六份 core、法语数据和真实截图；浏览器拒绝所有非本机请求。CSP 仅额外允许本地 worker 和 `wasm-unsafe-eval`。

| 项目 | 结果 |
| --- | --- |
| 三行真实法语截图 | `L'école d'aujourd'hui.`／`Où est le café ?`／`Ça coûte 10 euros.` |
| 重音、撇号和换行 | 通过 |
| 冷启动与识别 | 321.2 ms；该样本参考 confidence 87 |
| 完整本地资源 | 27,082,459 字节，含独立 API 浏览器脚本 |
| 实际核心 | 自动选择 Relaxed SIMD LSTM |
| 外部请求 | 0；未使用 CDN 或持久语言缓存 |

结果、请求清单与截图保留在 `target/ocr-probe/`。这证明固定组合的本地可行性，应用内 CSP、故障、取消、包体与原生路径另外验收。

## 原生快照

隔离 Rust 探针执行 `cargo run --release --manifest-path target/ocr-capture-probe/Cargo.toml`，仅依赖固定 XCap 0.9.9。每幅快照即时检查位图尺寸和 RGBA 长度，随后释放，不保存桌面内容。

| 显示器原点 | 实际位图 | 缩放 | 采集 |
| --- | --- | --- | --- |
| `(0, 0)` | 1,920×1,080 | 125% | 25 ms |
| `(-2520, 0)` | 2,520×1,680 | 175% | 47 ms |

均通过，含负坐标、多显示器与不同缩放。框选按截图像素尺寸与实际 CSS 视口比例转换，不能把 CSS 坐标直接当物理像素。完整日志保留在 `target/finish-issues/ocr-capture-probe.log`。

## 接入边界

每次用户请求取得冻结的屏幕快照，在各显示器提供受限框选窗口；选定区域后返回翻译模块，保留预览并自动启动本地 OCR，文字可编辑。图片导入及粘贴先经过原生 PNG／JPEG 解码与尺寸／内存上限，再由用户点击识别。请求编号与取词共用；旧结果、取消和离开页面失效。截图、文字不写入学习数据或日志。

截图快捷键默认 `Ctrl+Alt+Shift+O`，可保存并更换，注册冲突可见。macOS 只在主动截屏时检查／请求屏幕录制权限，拒绝、撤销和恢复保持真实状态。

## 限制

本页为资源准入，尚不代表屏幕框选产品流已验收。macOS 与其他架构执行按用户要求暂缓；正式发行继续验收完整依赖、通知、首次禁网和资源体积。
