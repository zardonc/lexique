# 法语离线 OCR 资源

默认组合为 Tesseract.js 7.0.0／Tesseract.js-core 7.0.0 和 `tessdata_fast` 的 `fra.traineddata`。运行库使用固定 pnpm 依赖；法语数据和法律原文在本目录保存，完整版与 Lite 均提供。CPU／WASM 识别不调用 CDN，不依赖翻译或 AI 服务。

法语数据来自 [tessdata_fast](https://github.com/tesseract-ocr/tessdata_fast/tree/87416418657359cb625c412a48b6e1d6d41c29bd)，固定提交 `87416418657359cb625c412a48b6e1d6d41c29bd`；原文件 1,130,365 字节，SHA-256 `ced037562e8c80c13122dece28dd477d399af80911a28791a66a63ac1e3445ca`。未修改模型。

Tesseract.js 7.0.0 提交 `42eae669e4b3a66429d8516f078912cc747a89df`；core 7.0.0 提交 `acffef2b66eb44a31df297e11d905f4b39001068`。运行准备保留普通、LSTM、SIMD 和 Relaxed SIMD 共六份内嵌 WASM 的 `.wasm.js` 核心，不强制单一硬件变体；worker 按上游能力探测选择。worker、core 和语言数据均用显式本地路径，语言数据不压缩、缓存关闭。

绑定、core、Tesseract 和模型为 Apache-2.0。core 的 Leptonica、GIF／JPEG／PNG／TIFF／WebP、zlib 和 OpenLibm 通知保留在 [licenses](licenses)；浏览器 worker、Buffer／ieee754 和 API 依赖的完整原文见[浏览器通知](licenses/browser/README.md)。WebP 的专利授予原文一并保留。OpenLibm 运行库为 MIT／BSD／ISC／Sun 许可；其 LGPL 条目仅适用于测试，固定上游 `build-openlibm.sh` 执行 `ARCH=wasm32 install-static`，不构建或分发这些测试。

屏幕采集使用 XCap 0.9.9（Apache-2.0）；图像解码采用 image 0.25.10，仅启用 PNG／JPEG。Windows x64 原生依赖的来源及法律原文见[原生通知](licenses/native/README.md)，包括 image、base64、PNG／JPEG、moxcms／pxfm、Windows 绑定和适用的 Unicode 许可。图像输入最多 16 MiB、4,096×4,096；屏幕快照的各显示器最多 8,192×8,192、合计最多 64 百万像素，框选结果沿用图像上限。文字进入翻译时遵守前 5,000 个 Unicode 字符的截断。

来源、版本、通知与 Windows 原生准入见 [资源探针](../../docs/evidence/issue-59/resource-probe.md)。源码和适用许可链接保持原样，应用接入代码采用仓库 MIT。
