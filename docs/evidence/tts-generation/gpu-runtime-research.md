# Windows x64 GPU 运行库调查

2026-11-01。只调查当前准入的 Supertonic 3 int8、sherpa-onnx 2.0.1 / ONNX Runtime 1.28.2 的 GPU 路径；本调查未自行安装、下载二进制、修改模型或运行基准。实机依赖检查、归档解包和生成测量由[主实验记录](README.md)分别报告。以下可用版本和发布资产大小来自本日上游元数据；NVIDIA 包的提取体积单独引用主实验实测，均不是最终产品包体。

## 版本与可取得的构建

官方 `v2.0.1` 标签确实指向现有库自报的 `af2d8d81dbfa61c4cea4c568da78f3982edca005`。[标签 API][tag]。Windows CUDA 发布工作流采用此源码、开启 `SHERPA_ONNX_ENABLE_GPU` 和共享库，并归档安装后的 `bin`、`lib`、`include`；C/C++ API 两个目标均在安装清单中。[发布工作流][cuda-workflow]、[C/C++ API 安装][capi-cmake]。这证明预期包装内容；本调查没有读取二进制归档内容或独立复现其构建。

| 路径 | 本日可取得的官方 / 上游维护构建 | 与当前组合的关系 |
| --- | --- | --- |
| Windows x64 C/C++ API，CUDA 13 | sherpa-onnx 2.0.1 + ORT 1.28.2，CUDA 13.x / cuDNN 9.x；归档 480,541,871 字节 | 源码版本与 ORT 版本均匹配，是保留现有模型的直接试验候选。[发布 API][sherpa-release-api] |
| Windows x64 C/C++ API，CUDA 12 | sherpa-onnx 2.0.1 + ORT 1.28.2，CUDA 12.x / cuDNN 9.x；归档 597,231,017 字节 | 同版本候选；ORT 官方说明这一版本的 CUDA 12 构建实际要求 CUDA 12.8 或更新运行库。[发布 API][sherpa-release-api]、[CUDA 要求][cuda-doc] |
| Windows x64 Python，CUDA | 上游索引有 `2.0.1+cuda12.cudnn9` 与 `2.0.1+cuda`，CPython 3.7–3.15；cp312 前者 201,645,092 字节，后者 154,040,348 字节 | 固定提交的 wheel 工作流分别用 ORT 1.24.4、1.17.1；不是当前 ORT 1.28.2。`+cuda` 对应旧 CUDA 11.8 路径。[索引][cuda-index]、[文件元数据][wheel-api]、[wheel 工作流][wheel-workflow]、[安装说明][python-install] |
| Microsoft 原生 CUDA | ORT 1.28.2 发布有 `onnxruntime-win-x64-gpu_cuda12-1.28.2.zip` 和 `...gpu_cuda13-1.28.2.zip` | 证明 ORT 的同版本原生分发存在；它不单独提供 sherpa 的 Supertonic 调用层。[ORT 发布 API][ort-release-api] |
| Microsoft Python CUDA / DirectML | `onnxruntime-gpu` 本日最新 1.31.1，存在 1.28.0；`onnxruntime-directml` 最新 1.24.4，有 Windows x64 wheels | 两者的精确 `1.28.2` PyPI JSON 均返回 404。不要把安装另一个 Python ORT 包当成现有 sherpa 原生 DLL 自动获得 GPU。[GPU 元数据][pypi-gpu]、[GPU 1.28.0][pypi-gpu-128]、[DirectML 元数据][pypi-dml]、[sherpa 会话来源][session-source] |
| DirectML C/C++ | 官方 `Microsoft.ML.OnnxRuntime.DirectML` NuGet 最新 1.24.4，无 1.28.2；sherpa 2.0.1 发布没有 DirectML 命名资产 | 需要 DML-enabled sherpa 构建。固定源码的 DirectML CMake 默认下载 ORT **1.14.1** + DirectML 1.15.0；默认选项不能证明当前模型兼容，重建时须明确选择、验证版本。[NuGet 版本][nuget-dml]、[发布 API][sherpa-release-api]、[DirectML 构建来源][dml-cmake] |

CUDA 13 的精确候选：

- [sherpa-onnx-v2.0.1-cuda-13.x-cudnn-9.x-onnxruntime1.28.2-win-x64-cuda.tar.bz2][cuda13-asset]
- GitHub 发布 API 的 SHA-256：`206f56c80cca702ca564fb21e48462f4210a13f941de32c336a4c6debb1e9efb`。[元数据][sherpa-release-api]。这是上游提供的摘要，下载后仍须本地核对。

## 额外依赖：开发验证与产品运行分别计算

CUDA 需要支持目标 CUDA 版本的 NVIDIA GPU / 驱动，以及可被进程找到的 CUDA、cuDNN、MSVC 运行 DLL。ORT 1.28.x 的 CUDA 13 构建要求 CUDA 13.0 或更新、cuDNN 9.x；CUDA 12 构建要求 CUDA 12.8 或更新、cuDNN 9.x。cuDNN 8 与 9 不可互换。[CUDA 要求][cuda-doc]。发布工作流只复制 ORT / sherpa 安装输出，没有另外安装或复制 CUDA / cuDNN 的步骤；是否有这些库随归档提供，必须以实际解包清单为准。[发布工作流][cuda-workflow]。

ORT 1.28.2 的 Windows 预加载清单列出 CUDA 13 的 `cublasLt64_13.dll`、`cublas64_13.dll`、`cufft64_12.dll`、`cudart64_13.dll`，以及 `cudnn64_9.dll` 和 cuDNN 分拆的 engines / heuristic / ops / adv / graph DLL；MSVC 包括 `vcruntime140.dll`、`msvcp140.dll`、`vcruntime140_1.dll`。这份清单不是本产品已验证的完整依赖清单。[固定 ORT 源码][preload-source]。官方允许用 CUDA / cuDNN Python extras 或已有兼容 PyTorch 提供运行 DLL，再预加载；因此完整 CUDA SDK 和 PyTorch 都不是推理产品必需的交付内容。[CUDA 预加载说明][cuda-doc]。

DirectML 需要 DirectX 12 设备及其图形驱动、相应 Windows / DirectML 运行库和 DML-enabled ORT / sherpa；它不需要 CUDA 或 cuDNN。DirectML 是 Windows 10 1903 起的系统组件，也有单独的 `DirectML.dll` 再分发包。Windows SDK / Visual Studio 是源码构建工具；调试层和 Graphics Tools 是可选调试依赖，不应记为普通用户必须安装的运行依赖。[ORT DirectML 要求][dml-doc]、[系统与再分发版本][dml-history]、[调试层][dml-debug]。DirectML 当前进入 sustained engineering；新的 Windows ML 路径不在本轮验证范围内。[ORT DirectML 要求][dml-doc]。

本轮独立实验若复制 / 安装 GPU DLL，不会自动增加产品发布依赖。产品若采用 CUDA，须另报最小 DLL 集合、压缩及安装占用、驱动门槛、再分发通知和缺依赖机器的离线获取方式；源码 SDK、头文件、导入库、调试器、实验 Python wheels 不应一起算进必需运行包。现有[开发准入门槛](../../development-readiness.md)仍适用。

### NVIDIA CUDA 13 Python 再分发包：实验固定版本

NVIDIA 的 Windows 安装指南明确说明 PyPI wheels 主要用于运行时，可用于独立于完整开发工具链的部署，但使用 pip 环境以外的宿主进程时需设置 DLL 搜索环境。[官方指南][nvidia-windows-pip]。主实验已隔离下载并核对以下 Windows x64 wheels 的 PyPI SHA-256；本调查复核了各版本官方元数据和已提取的 `METADATA`，依赖链是 `cuDNN → cuBLAS → NVRTC`、`cuFFT → nvJitLink`。[固定包与摘要][nvidia-pins-local]。这里的“未声明”只指 `Requires-Dist`，不表示无需 GPU 驱动或 Windows 运行库。

| 实验包 / 固定版本 | Windows 运行文件 | PyPI 声明的依赖 |
| --- | --- | --- |
| [nvidia-cuda-runtime 13.4.92][nvidia-runtime-pypi] | `cudart64_13.dll` | 未声明 |
| [nvidia-cublas 13.8.0.4][nvidia-cublas-pypi] | `cublas64_13.dll`、`cublasLt64_13.dll`；另含 `nvblas64_13.dll` | `nvidia-cuda-nvrtc` |
| [nvidia-cufft 12.4.0.43][nvidia-cufft-pypi] | `cufft64_12.dll`；另含 `cufftw64_12.dll` | `nvidia-nvjitlink` |
| [nvidia-cudnn-cu13 9.27.0.42][nvidia-cudnn-pypi] | `cudnn64_9.dll` 及分拆 DLL | `nvidia-cublas` |
| [nvidia-cuda-nvrtc 13.4.92][nvidia-nvrtc-pypi] | `nvrtc64_130_0.dll`、`nvrtc-builtins64_134.dll` | 未声明 |
| [nvidia-nvjitlink 13.4.92][nvidia-nvjitlink-pypi] | `nvJitLink_130_0.dll` | 未声明 |

cuBLAS 最新 13.8.2.12 的 PyPI 文件列表只有 Linux wheels，因此实验选择有 Windows wheel 的 13.8.0.4，而不是升级到最新版本。[最新版本元数据][nvidia-cublas-latest]。NVRTC 是运行时编译库，其 builtins 必须与 NVRTC 来自同一个 CUDA Toolkit，不能独立替换任意版本。[NVIDIA NVRTC 13.4 文档][nvidia-nvrtc-doc]。实际 DLL 清单及逐文件哈希见[提取记录][nvidia-files-local]；表中“另含”的 DLL 尚不能据包内存在就认定为产品必需文件。

随包许可已逐份读取并核对本地哈希，六份 `METADATA` 都标为 `LicenseRef-NVIDIA-Proprietary`。CUDA runtime / cuBLAS / cuFFT / NVRTC / nvJitLink 的五份 `licenses/License.txt` 相同，SHA-256 为 `ad6f5853fba0ca0d159d0f58d49ae49830c2f8c93f7a92648b9ce90adb4c6ccd`，内容是 SDK EULA、CUDA Supplement 及第三方许可，仍带 2018 年日期。[随包 CUDA 许可][nvidia-cuda-license-local]。cuDNN 的独立许可哈希为 `49cf79bdb35734b52fe6203013b3bd759f81e998cd32aa2c65c51db9a88c61d2`，正文日期为 2020-01-28，附录仍列 `cudnn64_7.dll`。[随包 cuDNN 许可][nvidia-cudnn-license-local]。这些旧文本的文件列表不能单独证明现代全部 DLL 的再分发映射。

当前官方 CUDA 13.4 EULA Attachment A 列有 runtime、cuBLAS、cuFFT、NVRTC 和 JIT Linking Library，并允许文件名中的版本 / 架构变体；cuDNN 9.27 的官方附录列 runtime `.so` 和 `.dll` 可分发。两者均受 SDK 分发条件约束，包括应用具有额外功能、这些组件仅由该应用访问、分发协议与 NVIDIA 条款一致，以及保留适用的权利和第三方通知；并非 ORT 的 MIT 许可自动覆盖。[CUDA 13.4 EULA][nvidia-cuda-eula]、[cuDNN 9.27 EULA][nvidia-cudnn-eula]。固定二进制与适用条款的逐项匹配及最终通知仍属于产品准入工作。

主实验基线是驱动 621.01、自报 CUDA UMD 13.5；其六包实测下载合计 **1,109,494,717 字节**、提取 **1,629,838,001 字节**、其中 DLL **1,618,957,904 字节**。[主实验提取记录][nvidia-files-local]。这些是实验固定版本及完整 wheel 提取结果，包含部分头文件 / 导入库；既不代表应用已准入这些依赖，也不代表产品必须随包提供全部文件或完整 CUDA SDK。本调查没有重复 GPU 性能或安装验证。

## int8 图为何可能在配置 GPU 后变慢

ORT `quantize_dynamic` 明确使用 `QUInt8` 激活，源码注释为 “dynamic activation only supports uint8”；sherpa 的 Supertonic 转换脚本对部分图使用动态 MatMul / Gemm 量化。这解释的是上游转换路径，当前四个准入文件仍须按实际图和哈希确认。[ORT 动态量化][quantize-source]、[Supertonic 转换][supertonic-convert]。

| ORT 算子 / 类型 | CUDA EP，1.28.2 | DirectML EP，1.28.2 来源表及可取得的 1.24.4 表 |
| --- | --- | --- |
| `DynamicQuantizeLinear` | 没有列出 CUDA kernel | 列出支持 |
| `MatMulInteger` | 仅 `int8 × int8`；不覆盖常见 `uint8 × int8` | 列出 int8 / uint8 组合 |
| `DynamicQuantizeMatMul`、`MatMulIntegerToFloat` | 没有列出 CUDA kernel | 列出支持 |

来源：[ORT 1.28.2 算子矩阵][kernels-128]、[ORT 1.24.4 算子矩阵][kernels-124]。动态量化的主要计算可能因此留在 CPU，其他节点在 GPU，形成图分区、同步和内存复制。DirectML 的矩阵较适合这些量化算子，但表格不证明实际图形状、全部算子或当前 sherpa 组合能运行。

另外，固定 sherpa Supertonic 实现用 CPU `MemoryInfo` 建输入，在每一步 vector estimator 调用后读取输出并更新主机 `xt_flat`；四个子模型都用普通 `Session::Run`。即使计算节点进入 CUDA，这条调用路径也存在重复主机 / 设备传输的成本。[Supertonic 实现][supertonic-impl]、[子模型调用][supertonic-model]。ORT 官方建议通过 I/O Binding 减少复制；这不是仅调整线程就能消除的成本。[CUDA 性能说明][cuda-doc]。更换为 fp16 / 重量化或改调用层均需另行验证模型和产品边界，不属于本调查。

## 无代码改动的实际执行诊断

固定 sherpa 支持 `provider="cuda:<config-file-path>"`，配置为每行 `key=value`。可分别给计时和诊断使用不同配置；诊断示例：

```ini
LogSeverityLevel=0
ProfilingFilePrefix=C:/absolute/experiment/path/ort-profile
```

`ProfilingFilePrefix`、`LogSeverityLevel` 是 sherpa 直接处理的顶层键；`SessionConfig.*` 则去掉前缀后传给 ORT `AddConfigEntry`。不要给前两个键加 `SessionConfig.`。[会话配置来源][session-source]。

ORT 1.28.2 的非 minimal 构建在 verbose 级别打印 `Node placements` 与执行器节点数量；profile 节点事件含 `args.provider`、`args.op_name`。汇总 CPU / CUDA 节点、重要量化算子和 Memcpy 事件，再结合真实生成时间，才能区分完整 CPU 回退、混合执行和有收益的 GPU 执行。少量 CPU 形状算子本身不能说明 GPU 未使用。[节点日志][placement-source]、[profile 事件][executor-source]。ORT 在会话销毁时结束并写出 profiling；Supertonic 的四个会话共用配置，应检查保存的文件能覆盖哪些子模型。[会话析构][ort-session-source]、[子模型会话][supertonic-model]。

可另做严格覆盖诊断：`SessionConfig.session.disable_cpu_ep_fallback=1`。ORT 文档说明此键使未获 GPU 等执行器完整覆盖的图在初始化时失败；它不是性能设置，并可能因正常的 CPU 形状节点失败。[配置键定义][fallback-key]。

检查记录：`git diff --no-index --check -- NUL docs/evidence/tts-generation/gpu-runtime-research.md` 返回新文档差异状态 1，无空白诊断；正文 / 范围审阅通过。PowerShell 引用解析确认 46 个引用定义全部有对应关系、6 个本地链接存在；最初 31 个及补充 11 个远端来源的 `Invoke-WebRequest -Method Head` 全部 HTTP 200。六份 NVIDIA 随包许可经 `Get-FileHash -Algorithm SHA256` 核对，均与提取记录相符。固定版本源码、发布 API、PyPI / NuGet 元数据和官方说明已阅读。本调查未自行执行二进制兼容、DLL 加载、GPU 合成、音质、离线或最终产品发行验收；主实验已完成的实测以其记录为准。

[tag]: https://api.github.com/repos/k2-fsa/sherpa-onnx/git/ref/tags/v2.0.1
[sherpa-release-api]: https://api.github.com/repos/k2-fsa/sherpa-onnx/releases/tags/v2.0.1
[cuda13-asset]: https://github.com/k2-fsa/sherpa-onnx/releases/download/v2.0.1/sherpa-onnx-v2.0.1-cuda-13.x-cudnn-9.x-onnxruntime1.28.2-win-x64-cuda.tar.bz2
[cuda-workflow]: https://github.com/k2-fsa/sherpa-onnx/blob/af2d8d81dbfa61c4cea4c568da78f3982edca005/.github/workflows/windows-x64-cuda.yaml
[capi-cmake]: https://github.com/k2-fsa/sherpa-onnx/blob/af2d8d81dbfa61c4cea4c568da78f3982edca005/sherpa-onnx/c-api/CMakeLists.txt
[cuda-index]: https://k2-fsa.github.io/sherpa/onnx/cuda.html
[wheel-api]: https://huggingface.co/api/models/csukuangfj2/sherpa-onnx-wheels/tree/main/cuda/2.0.1?limit=1000
[wheel-workflow]: https://github.com/k2-fsa/sherpa-onnx/blob/af2d8d81dbfa61c4cea4c568da78f3982edca005/.github/workflows/build-wheels-win-x64-cuda.yaml
[python-install]: https://k2-fsa.github.io/sherpa/onnx/python/install.html
[ort-release-api]: https://api.github.com/repos/microsoft/onnxruntime/releases/tags/v1.28.2
[pypi-gpu]: https://pypi.org/pypi/onnxruntime-gpu/json
[pypi-gpu-128]: https://pypi.org/pypi/onnxruntime-gpu/1.28.0/json
[pypi-dml]: https://pypi.org/pypi/onnxruntime-directml/json
[nuget-dml]: https://api.nuget.org/v3-flatcontainer/microsoft.ml.onnxruntime.directml/index.json
[dml-cmake]: https://github.com/k2-fsa/sherpa-onnx/blob/af2d8d81dbfa61c4cea4c568da78f3982edca005/cmake/onnxruntime-win-x64-directml.cmake
[cuda-doc]: https://onnxruntime.ai/docs/execution-providers/CUDA-ExecutionProvider.html
[preload-source]: https://github.com/microsoft/onnxruntime/blob/v1.28.2/onnxruntime/__init__.py
[dml-doc]: https://onnxruntime.ai/docs/execution-providers/DirectML-ExecutionProvider.html
[dml-history]: https://learn.microsoft.com/en-us/windows/ai/directml/dml-version-history
[dml-debug]: https://learn.microsoft.com/en-us/windows/ai/directml/dml-debug-layer
[quantize-source]: https://github.com/microsoft/onnxruntime/blob/v1.28.2/onnxruntime/python/tools/quantization/quantize.py
[supertonic-convert]: https://github.com/k2-fsa/sherpa-onnx/blob/af2d8d81dbfa61c4cea4c568da78f3982edca005/scripts/supertonic/convert.py
[kernels-128]: https://github.com/microsoft/onnxruntime/blob/v1.28.2/docs/OperatorKernels.md
[kernels-124]: https://github.com/microsoft/onnxruntime/blob/v1.24.4/docs/OperatorKernels.md
[session-source]: https://github.com/k2-fsa/sherpa-onnx/blob/af2d8d81dbfa61c4cea4c568da78f3982edca005/sherpa-onnx/csrc/session.cc
[supertonic-impl]: https://github.com/k2-fsa/sherpa-onnx/blob/af2d8d81dbfa61c4cea4c568da78f3982edca005/sherpa-onnx/csrc/offline-tts-supertonic-impl.cc
[supertonic-model]: https://github.com/k2-fsa/sherpa-onnx/blob/af2d8d81dbfa61c4cea4c568da78f3982edca005/sherpa-onnx/csrc/offline-tts-supertonic-model.cc
[placement-source]: https://github.com/microsoft/onnxruntime/blob/v1.28.2/onnxruntime/core/framework/session_state.cc
[executor-source]: https://github.com/microsoft/onnxruntime/blob/v1.28.2/onnxruntime/core/framework/sequential_executor.cc
[ort-session-source]: https://github.com/microsoft/onnxruntime/blob/v1.28.2/onnxruntime/core/session/inference_session.cc
[fallback-key]: https://github.com/microsoft/onnxruntime/blob/v1.28.2/include/onnxruntime/core/session/onnxruntime_session_options_config_keys.h
[nvidia-windows-pip]: https://docs.nvidia.com/cuda/cuda-installation-guide-microsoft-windows/index.html#pip-wheels
[nvidia-runtime-pypi]: https://pypi.org/pypi/nvidia-cuda-runtime/13.4.92/json
[nvidia-cublas-pypi]: https://pypi.org/pypi/nvidia-cublas/13.8.0.4/json
[nvidia-cufft-pypi]: https://pypi.org/pypi/nvidia-cufft/12.4.0.43/json
[nvidia-cudnn-pypi]: https://pypi.org/pypi/nvidia-cudnn-cu13/9.27.0.42/json
[nvidia-nvrtc-pypi]: https://pypi.org/pypi/nvidia-cuda-nvrtc/13.4.92/json
[nvidia-nvjitlink-pypi]: https://pypi.org/pypi/nvidia-nvjitlink/13.4.92/json
[nvidia-cublas-latest]: https://pypi.org/pypi/nvidia-cublas/13.8.2.12/json
[nvidia-nvrtc-doc]: https://docs.nvidia.com/cuda/nvrtc/index.html
[nvidia-cuda-eula]: https://docs.nvidia.com/cuda/eula/index.html
[nvidia-cudnn-eula]: https://docs.nvidia.com/deeplearning/cudnn/backend/v9.27.0/reference/eula.html
[nvidia-pins-local]: ../../../.tooling/issue-24/gpu-cuda13-20261101/nvidia-pinned.json
[nvidia-files-local]: ../../../.tooling/issue-24/gpu-cuda13-20261101/nvidia-runtime.json
[nvidia-cuda-license-local]: ../../../.tooling/issue-24/gpu-cuda13-20261101/nvidia-runtime/nvidia_cuda_runtime-13.4.92.dist-info/licenses/License.txt
[nvidia-cudnn-license-local]: ../../../.tooling/issue-24/gpu-cuda13-20261101/nvidia-runtime/nvidia_cudnn_cu13-9.27.0.42.dist-info/licenses/License.txt
