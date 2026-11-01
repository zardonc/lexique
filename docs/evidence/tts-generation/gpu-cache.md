# 单词缓存空间与 GPU 生成实测

2026-11-01。验证用户追加的两个问题：仅长期保留单词音频时的空间，以及当前模型使用 GPU 的实际速度和额外依赖。范围为 Windows x64 独立实验；应用、缓存清理策略、卡片和用户学习数据未改。完整可复核摘要见 [gpu-cache-results.json](gpu-cache-results.json)，官方版本、算子及依赖调查见 [gpu-runtime-research.md](gpu-runtime-research.md)。

## 结论

- 每个生词保留一份当前格式的单词音频，**1 万词约为 88 MB 文件内容、本机约 111 MB 文件分配空间**。同时保留男女两份约为 222 MB 文件分配空间。现有两类卡片共用同一生词音频，慢速播放也复用该文件。
- 本机 RTX 4060 Laptop 上，**已加载模型的串行 24 词生成从 CPU 的 28.19 秒降至 CUDA 的 12.85 秒**，本轮中位数约快 2.19 倍；完整子进程为 30.85 → 16.20 秒。GPU 逐条生成有收益。
- 已优化的 12 路共享模型并发，CPU / CUDA 同为约 5.65 秒；完整子进程 CPU 9.98 秒、CUDA 11.02 秒。**GPU 没有带来稳定的额外批量吞吐收益**，且需要更多运行库和内存。
- 当前 GPU 候选保留 int8 模型、8 步、正常合成速度、2 推理线程、关闭空转，串行调用用于延迟优先。额外试验的 1D 卷积参数没有稳定优势，不纳入推荐。持续批量生成仍推荐此前的 CPU 常驻单模型 / 12 路方案。

这些是本机已测候选的取舍，不能证明其他硬件、所有文本或模型架构的全局最优。GPU 音频与 CPU 不逐位相同，尚未进行人工发音 / 音质验收；本轮不把 GPU 候选升级为产品准入。

## 单词音频的实际空间

使用已准入的公开 FreeDict 法中小样 `target/desktop-dictionary-sample/dictionary.sqlite3`，通过现有 `Dictionary` 的只读完整性、来源、条数及内容哈希校验。10,777 个 NFC 去重词头中，剔除含空白、多词条及不符合现有长度 / 控制字符限制的项，余 **8,047 个候选**；用种子 `20261101` 均匀抽取 128 个。包含专名与较少见词，不是按 A1–C1 使用频率加权的样本，不能代表所有生词或短语。

同一 128 词分别生成女声、男声，共 **256 份实际音频**；正常合成速度、8 步，沿用产品的 44.1 kHz 单声道、48 kbit/s Ogg Opus 编码。每 32 个请求单独运行，12 路共享模型，逐次硬限 60 秒。该轮用于量空间，批次时间不作为 GPU / CPU 排名。

| 指标 | 女声 | 男声 |
| --- | ---: | ---: |
| 平均文件内容 | 8,775 字节 | 8,812 字节 |
| 文件内容中位数 | 8,884.5 字节 | 8,951 字节 |
| 文件内容最小 / 最大 | 6,601 / 13,215 字节 | 6,839 / 13,209 字节 |
| 文件内容 P95 | 10,153 字节 | 10,360 字节 |
| 本机平均文件分配量 | 11,104 字节 | 11,104 字节 |
| 平均音频时长 | 1.432 秒 | 1.429 秒 |

文件分配量通过 Windows `GetFileInformationByHandleEx(FileStandardInfo).AllocationSize` 实际读取；没有把文件长度当成磁盘占用，也没有假设固定簇大小。当前平均分配量比文件内容约多 26%。以下按女声默认、每词一份或两声线各一份线性估算；男声单份估计只相差约 0.4%。**MB = 1,000,000 字节**。

| 生词数量 | 一份音频的文件内容 MB | 一份音频的文件分配 MB | 两声线的文件分配 MB |
| --- | ---: | ---: | ---: |
| 1,000 | 8.8 | 11.1 | 22.2 |
| 5,000 | 43.9 | 55.5 | 111.0 |
| 10,000 | 87.8 | 111.0 | 222.1 |
| 30,000 | 263.3 | 333.1 | 666.2 |

此估计只计算最终 `.ogg`，不包含实验 `.f32`、模型、数据库、日志、备份、目录及文件系统元数据。更长词、短语、不同文件系统或保留多个模型版本会改变结果。容量规划可按每 1 万词、每声线约 150 MB 留出余量；这不是跨文件系统保证。

### 与当前应用的关系

- 当前应用只生成并缓存**单词**音频；例句文本已存在，例句朗读仍属 M3，尚无可供删除的产品例句音频缓存。此处计算的是未来区分清理策略时，保留单词的成本。
- `speech_generate` 按 `vocabulary_id` 读取 / 写入 `{id}.ogg`。同词识义卡、听音卡共享一份；现有实现只保留生成时选择的声线，切换偏好不自动重写旧音频。
- 正常 / 慢速以播放倍率 `1` / `0.6` 区分，不额外生成慢速文件。
- 现有清理依旧按单词音频最近访问时间，七天未访问后清理，启动后及每 24 小时扫描。本轮没有实现单词持久化、例句独立过期或删除数据。

长期保留时，后续应明确声线 / 模型版本变更后的重生成规则，避免无限保留版本或一直复用旧声线；损坏音频仍须可重试，不影响卡片进度。现有临时缓存的产品规则没有因容量估计而改变。

来源：[生成与复用入口](../../../apps/desktop/src-tauri/src/vocabulary.rs)、[缓存实现](../../../apps/desktop/src-tauri/src/speech_cache.rs)、[播放倍率](../../../apps/desktop/src/review-cards.tsx)、[产品基线](../../product.md)、[M3 范围](../../roadmap.md)。

## 真正使用 GPU 的对照

### 运行环境与兼容性

沿用首轮 CPU 实验的 Windows 11、Ryzen 9 8945HS、32 GB RAM，保持既有电源与系统设置；GPU 为 NVIDIA GeForce RTX 4060 Laptop、8,188 MiB 显存、计算能力 8.9，驱动 **621.01** 自报 CUDA UMD **13.5**。

应用现有 runtime 仍为 CPU-only。仅复制 `.tooling/issue-15-opus/speech-stage` 到新的 `.tooling/issue-24/gpu-cuda13-20261101/stage`，将副本中的六个原生 DLL 换成官方同版本 CUDA 13 归档中的文件。官方归档为 sherpa-onnx **2.0.1** / ORT **1.28.2**，发布摘要与实下载 SHA-256 均为：

```text
206f56c80cca702ca564fb21e48462f4210a13f941de32c336a4c6debb1e9efb
```

原有 Python 3.14 扩展、NumPy、模型、声线与编码器保留；新 DLL、六个 NVIDIA wheels 及提取文件逐项核对摘要。CPU-only 原始 stage 未覆盖。实验只给子进程增加 DLL 搜索目录 / PATH；没有安装全局 CUDA、PyTorch 或修改用户 PATH。成功初始化和生成证明这份副本的实际兼容性，不替代其构建来源审计或干净机器发行验收。

### GPU 执行的直接证据

独立诊断使用 `provider="cuda:<config>"`，开启 ORT profiling；计时轮关闭 profiling。四个子模型保存了**四个不同的 profile 文件**，没有同名覆盖。按固定 sherpa `LoadModels()` 的会话创建顺序对应：

| 子模型 | CUDA 节点执行事件 | CPU 节点执行事件 |
| --- | ---: | ---: |
| duration predictor | 500 | 274 |
| text encoder | 852 | 520 |
| vector estimator | 17,376 | 1,952 |
| vocoder | 698 | 82 |

含一次 `bonjour` 预热和一次计量输出。四个模型都执行了 CUDA 节点，因此不是请求 GPU 后全部回退 CPU。vector estimator 中仍有 CPU `MatMulInteger`，以及 CUDA `MemcpyFromHost` / `MemcpyToHost`；当前 int8 图为混合执行。节点事件数量和 host-side profile 时间不能换算成 GPU 计算比例、GPU 利用率或速度收益。

CPU-only runtime 另做负向检查：请求 CUDA 后仍能生成合法 Ogg，但明确打印 `Fallback to cpu`，实验 runner 将这一结果记为失败并以 **1** 退出。这条预期拒绝通过，防止把合法音频或配置成功当成 GPU 已使用。

### 三轮交叉计时

同一 12 个法语单词重复两次，共 24 请求；女声、8 步、正常合成速度、固定请求种子 `42 + id`。每个配置一次加载一个模型，线程池贯穿预热 / 计时；预热数量与调用并发数相同。配置顺序用固定种子随机化，CPU 和 GPU 配置串行交叉执行，无两组推理同时运行。**所有本次计时配置关闭线程空转**。

21 次确认试验分别受完整子进程 60 秒硬限约束。计时包含推理、Opus 编码及原始证据文件写入；“热批量”不含准备 / 预热，“完整子进程”包含启动、资源验证、模型加载、预热、批量生成与退出，均不包含事后的独立证据核验。它们不包含应用 IPC、缓存发布或播放。

| 配置 | 24 词热批量中位数，秒 | 完整子进程中位数，秒 | 私有提交峰值 MiB | 工作集峰值 MiB |
| --- | ---: | ---: | ---: | ---: |
| 原 CPU runtime，4 线程串行 | 28.19 | 30.85 | 775.3 | 293.1 |
| CUDA，2 线程串行，默认 GPU 选项 | 12.85 | 16.20 | 1,994.8 | 920.5 |
| CUDA，2 线程串行，1D 卷积 padding 选项 | 12.45 | 15.79 | 1,988.0 | 918.6 |
| 原 CPU runtime，2 线程 / 12 路共享 | 5.65 | 9.98 | 1,222.0 | 644.0 |
| CUDA，2 线程 / 12 路共享，默认选项 | 5.65 | 11.02 | 4,262.2 | 1,032.7 |
| CUDA，2 线程 / 12 路共享，1D 卷积选项 | 5.60 | 10.82 | 3,739.6 | 1,019.8 |
| GPU-enabled runtime 请求 CPU，2 线程 / 12 路共享 | 5.56 | 10.12 | 1,219.3 | 617.4 |

GPU-enabled runtime 的 CPU 对照输出与原 CPU runtime 逐位相同；不把替换原生库的其他变化误当成 GPU 收益。50 毫秒同时采样的私有提交与工作集分别报告，提交量不是驻留 RAM，也不是显存；本轮未测显存峰值或耗电。

默认 CUDA 串行三轮热批量为 **12.85 / 6.21 / 13.69 秒**，有明显波动，不按最快一轮宣称普遍提速四倍。额外参数 `cudnn_conv1d_pad_to_nc1d=1` 串行为 **12.45 / 6.91 / 13.58 秒**；默认 / 修改 padding 的批量中位数只差约 3%，12 路只差约 1%，没有稳定优势。保留默认 GPU 选项。该候选来自 [ORT 官方 Conv1D 性能选项](https://onnxruntime.ai/docs/execution-providers/CUDA-ExecutionProvider.html#convolution-input-padding-in-the-cuda-ep)，未修改模型或降步数。

前置筛选另比较 CUDA 的 1 / 2 / 4 / 8 / 12 路调用；12 词中默认 12 路约 2.82 秒。padding 试验的 24 词、16 路约 5.86 秒，比该轮 12 路的 5.64 秒更慢，内存也更高，没有把更多并发纳入推荐。

未停止其他原有进程，记录了每轮背景 CPU 占用；GPU 时间波动没有完成因果归因。首轮 CPU 实验的串行 23.35 秒是不同等待设置 / 测量轮次，应继续保留其原证据，不能与本轮结果拼成严格同期对照。首条冷启动 GPU 延迟、GPU 长句 / 男声 / 全词库、其他 GPU、AMD / DirectML、macOS / ARM64 均未测。

### 音频与数值边界

本轮 **893 份计量音频和 306 份预热音频**的 Ogg / PCM 大小、元数据、有限数值与实际文件回读通过；计量音频同时核对请求身份、PCM 哈希、样本数、时长、峰值和 RMS，缓存轮重新核对文件分配量。504 份确认输出在同配置三次重复中与参考哈希相同；CUDA 两种选项各 24 份共享输出与其串行参考相同。

CUDA 与 CPU 的音频哈希不相同：本次采样数差为 **-4 至 +1 个**（44.1 kHz 下最多约 0.091 毫秒），RMS 差约 **-2.0% 至 +1.7%**。这些检查支持数值稳定与共享调用的样本一致性，不证明发音正确或音质等价。GPU 新输出仍需人工试听，未发布为新默认。

空间样本的 256 份中有一份女声原始 PCM 峰值约 1.11，既有编码器会限幅；它不影响文件空间测量，但不能把整个公开词典样本视为完整音质通过。GPU 确认的 504 份没有原始满幅越界。

## GPU 的额外依赖与体积

需要 GPU-enabled sherpa / ORT，以及兼容 NVIDIA 驱动、CUDA runtime、cuBLAS、cuFFT、cuDNN 和它们需要的 NVRTC / nvJitLink。现有 MSVC runtime 也仍需要。**推理不要求完整 CUDA 开发工具包或 PyTorch**；CUDA 只适用于兼容 NVIDIA GPU，不能直接覆盖 AMD / macOS。

实验固定六个 Windows wheel 版本：`nvidia-cuda-runtime==13.4.92`、`nvidia-cublas==13.8.0.4`、`nvidia-cufft==12.4.0.43`、`nvidia-cudnn-cu13==9.27.0.42`、`nvidia-cuda-nvrtc==13.4.92`、`nvidia-nvjitlink==13.4.92`。版本与 CUDA / cuDNN 主版本须匹配，NVRTC 与其 builtins 也须配套；不是安装一个 Python ORT 包就能让现有 sherpa DLL 获得 GPU。

| 本轮完整实验材料 | 逻辑字节 |
| --- | ---: |
| 官方 sherpa CUDA 归档下载 | 480,541,871 |
| 归档解包，含 bin / lib 重复文件 | 625,636,878 |
| 六个 NVIDIA wheels 下载 | 1,109,494,717 |
| 六个 wheels 完整提取 | 1,629,838,001 |
| 其中 NVIDIA DLL | 1,618,957,904 |
| CUDA 路径原生 DLL，排除未使用的 TensorRT provider | 288,497,104 |
| 原 CPU 路径原生 DLL | 22,358,016 |
| 以上 DLL 相对 CPU 的净增 | **1,885,096,992** |

即本轮完整实验 DLL 集合相对现有 CPU 库约净增 **1.89 GB / 1.76 GiB**，下载材料总计约 1.59 GB。头文件、导入库、重复解包文件、旧 CPU 副本和实验 PCM 不属于产品必需音频缓存。此集合尚未裁剪到真正的最小运行依赖，不应声称最终产品必须增加整整 1.89 GB，亦未实测最终压缩包体。

NVIDIA 组件适用其有条件再分发条款，不能由 ORT MIT 或 sherpa Apache-2.0 标签覆盖。固定 wheel 许可文本与当前官方 EULA 的差异、依赖链及通知边界见 [调查记录](gpu-runtime-research.md)。GPU 若进入产品，仍须按 [开发准入门槛](../../development-readiness.md)完成目标平台、最小运行集合、匹配版本与发行通知核验；本轮仅完成已授权的隔离验证。

## 复现和检查

入口为 [measure_tts_acceleration.py](../../../scripts/measure_tts_acceleration.py) 和 [verify_tts_acceleration.py](../../../scripts/verify_tts_acceleration.py)，复用现有 worker 的资源哈希 / 请求 / PCM / 编码检查和已修正的内存采样。CPU 原 runner 只增加独立实验用 provider 参数；应用 worker 未改。

当前隔离 stage 和依赖已准备。若重建，先取得调查记录中的官方 CUDA 13 归档并核对上述发布摘要，再复制现有 CPU stage 到一个不存在的新目录，在副本 `runtime/Lib/site-packages/sherpa_onnx/lib` 中替换摘要记录的六个 DLL；六包可用 `uv pip install --target <新的隔离目录>` 按上述精确版本安装，不安装到原 runtime 或全局环境。所选路径、原生替换和 NVIDIA 包 / 文件哈希均保存在 `.tooling/issue-24/gpu-cuda13-20261101/{download,native-replacements,nvidia-pinned,nvidia-runtime}.json`，摘要同时收录于已追踪的 JSON。

```powershell
$ttsPython = (uv python find).Trim()
& $ttsPython -B -X utf8 scripts/measure_tts_acceleration.py --self-check
& $ttsPython -B -X utf8 scripts/measure_tts_generation.py --self-check
& $ttsPython -B -X utf8 -m unittest discover -s scripts -p test_tts_generation_monitor.py -v

& $ttsPython -B -X utf8 scripts/measure_tts_acceleration.py cache .tooling/issue-15-opus/speech-stage target/issue-24/tts-cache-new --words 128

& $ttsPython -B -X utf8 scripts/measure_tts_acceleration.py bench .tooling/issue-24/gpu-cuda13-20261101/stage target/issue-24/tts-cuda-new --provider cuda --configs 'serial,1,2,8,off' 'shared,12,2,8,off' --repeats 3 --cycles 2 --cpu-control .tooling/issue-15-opus/speech-stage --compare-default --cuda-options cudnn_conv1d_pad_to_nc1d=1 --dll-dirs .tooling/issue-24/gpu-cuda13-20261101/nvidia-runtime/nvidia/cu13/bin/x86_64 .tooling/issue-24/gpu-cuda13-20261101/nvidia-runtime/nvidia/cudnn/bin

& $ttsPython -B -X utf8 scripts/verify_tts_acceleration.py
git diff --check
```

所有生成目录必须不存在，且位于 `target/issue-24/` 的新子目录。provider 选项只允许本次关注的 CUDA 配置，拒绝未知键、值和换行注入；DLL 搜索目录限定为 `.tooling/issue-24/`。每个完整试验最多运行 60 秒，超时终止其拥有的进程树并保留 stderr / stdout / 状态。

固定证据目录为 `tts-cache-space-20261101`、`tts-cuda-diagnostic-20261101`、`tts-cuda-screen-20261101`、`tts-cuda-pad-pilot-20261101`、`tts-cuda-confirm-20261101`，另有 `tts-cuda-unavailable-20261101` 预期拒绝。独立 verifier 核验这些本次目录，不会自动把任意新跑目录认作原证据。`--export <新路径>` 只新建摘要文件；阶段 source 字节 / 哈希按原轮次归档，当前脚本不能冒充先前阶段字节。

实机通过：38 个成功试验、上述输出和空间 / 汇总回算、4 个真实 CUDA profile、504 份重复 / 共享一致性、发布归档 / wheels / 提取文件摘要、CPU-only 请求 CUDA 的失败识别、两项 runner 自检、6 项内存采样回归、3 个 Python 模块编译，以及文档 diff / 本地链接检查。调查记录的 42 个远端来源由研究员检查为 HTTP 200。应用未改，本轮未重跑 TS、Rust、产品 UI、正式 portable 或离线发行检查，也未修改既有准入结论。任务启动的试验进程均已退出，未停止原有服务。
