# 浏览器 OCR 依赖通知

固定 Tesseract.js 7.0.0 的浏览器 worker 保留上游[打包通知](worker-bundled.txt)，Buffer／ieee754 的版权声明及完整许可如下。Buffer 原文来自官方 v6.0.3，ieee754 原文来自官方 v1.2.1；这两项为法律原文来源版本，worker 本身使用固定的 Tesseract.js npm 分发物。

API 的已安装依赖版本以 pnpm 锁文件为准。下表保留法律原文；idb-keyval 使用 Apache-2.0 通用原文。没有修改这些依赖或模型。

| 依赖 | 许可 | 法律原文与来源 |
| --- | --- | --- |
| Buffer | MIT | [原文](buffer.txt)、[官方版本](https://github.com/feross/buffer/tree/v6.0.3) |
| ieee754 | BSD-3-Clause | [原文](ieee754.txt)、[官方版本](https://github.com/feross/ieee754/tree/v1.2.1) |
| bmp-js 0.1.0 | MIT | [原文](bmp-js.txt) |
| idb-keyval 6.3.0 | Apache-2.0 | [原文](../tesseract-js.txt)、[来源](https://github.com/jakearchibald/idb-keyval) |
| is-url 1.2.4 | MIT | [原文](is-url.txt) |
| node-fetch 2.7.0 | MIT | [原文](node-fetch.txt) |
| regenerator-runtime 0.13.11 | MIT | [原文](regenerator-runtime.txt) |
| wasm-feature-detect 1.9.0 | Apache-2.0 | [原文](wasm-feature-detect.txt) |
| zlibjs 0.3.1 | MIT | [原文](zlibjs.txt) |

原生解码、压缩、数学及 Windows 绑定的通知见[原生依赖](../native/README.md)。
