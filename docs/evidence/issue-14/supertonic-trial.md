# MLS 否决后的 Supertonic 3 小批试听

## Windows x64 独立验证包

2026-10-28：用户明确裁定保留 v9，接受 wheel 缺少签名构建证明与两张量化图校准过程未独立复现作为 #14 已知上游限制；保守归档已知及候选依赖通知，产品许可条款和机器合成披露留作 M2 / M5 验收。保留旧 v16，生成 `supertonic-evidence-v16-revised.zip`：159,725,647 字节，解压逻辑大小 241,177,527 字节，1,067 文件，SHA-256 `fae39f621200ab325b2d346063a21261641785d498dd01a58912ebdbd7750c1e`。六项打包检查、四项无音频探针检查、五项静态比较检查和 ZIP CRC 通过；1,016 个模型和运行库文件与 v9 哈希一致。真实音频生成按用户要求跳过。候选通知不证明 wheel 的全部依赖或正式发行的法律合规。

2026-10-28：v16 同步 Eigen 5.0.1 与 3.4.0 候选许可证和更新后的来源证据；ZIP 为 159,715,206 字节、解压逻辑大小 241,154,340 字节，SHA-256 `fb85144602aac88c27898a80941117728593e576deb86d9c0863ed97b0e44a34`，共 1,060 文件。CRC、六项打包检查及清单哈希通过；1,016 个模型及运行时文件与 v9 哈希一致。新增来源清单检查在 v15 失败、v16 通过；四项无音频探针检查及五项静态模型比较测试通过，真实生成按用户要求跳过。未重新生成或播放音频；包内许可材料不证明 wheel 实际依赖或全部再分发义务。

2026-10-27：v15 纳入更新后的归属说明、[静态模型比较记录](supertonic-static-comparison.md)、比较脚本和测试。ZIP 为 159,665,936 字节、解压逻辑大小 241,018,108 字节，SHA-256 `ebf26e30e24260a996014cc23886a408f69044d6caa54508b55a0fa64de6950e`，共 1,048 文件。ZIP CRC、六项打包检查及全部清单哈希通过；v9 与 v15 的 1,016 个模型及运行库文件哈希一致。四项无音频探针检查和五项静态比较测试通过，真实合成测试依用户要求跳过；本轮没有生成或播放音频。新增包内比较材料检查在 v14 失败、v15 通过；静态比较的非法量化 scale 回归先失败后修复。该比较只覆盖部分张量，不证明完整转换链；wheel 构建依赖和再分发义务仍未闭合。

2026-10-27：v14 记录了上游发布工作流、wheel 镜像及原始模型的部分文件对照。ZIP 为 159,657,392 字节、解压逻辑大小 241,004,574 字节，SHA-256 `1885c790c21bdc71e92ebd4262b0a01f04422478b97f82984efe0f9ce6201432`，共 1,045 文件；CRC、六项打包检查及四项无音频探针检查通过。与 v9 的模型和运行时文件哈希一致；没有生成音频。后来审查对归属说明中量化表述进行了修正，v14 包不含该后续修正。

2026-10-27：继续只读核查发现 v11 缺少 simple-sentencepiece 内嵌 Darts 的 BSD-2-Clause 通知，且其来源清单误写仓库；更正来源并保留 Darts 与 ThreadPool 的上游文件头后构建 v12。ZIP 为 159,656,130 字节、解压逻辑大小 241,000,944 字节，SHA-256 `82c5ab5895e53281b68011674481a2f823aa522b45548c4108ea8b25ebcabb04`，共 1,045 文件。CRC、六项打包检查与四项无音频探针检查通过；新必需通知检查在 v11 失败、v12 通过。v12 与 v9 的模型及运行时哈希仍完全相同，没有生成音频。v12 的来源说明尚不含随后取得的镜像时间和哈希对照；具体构建依赖仍未证实。

2026-10-27：复审发现 hclust 上游源文件还有额外版权声明，补入许可说明后构建 v11。ZIP 为 159,653,525 字节、解压逻辑大小 240,995,571 字节，SHA-256 `02a6db9914100a3bc1685b7a68d4c46094cad02bcf4589f46dbeb9b2efdb2444`，共 1,043 文件；CRC 及六项打包检查通过。与 v9 的逐文件比较仍仅有新增 hclust 许可、变更探针和两份许可说明，模型和运行时不变；未生成音频。新增的必需许可检查在 v9 失败、v11 通过；wheel 构建和转换来源仍未完全证明。

2026-10-27：按用户要求不再生成音频，构建 v10 以纳入 hclust 许可与探针显式校验。ZIP 为 159,653,402 字节、解压逻辑大小 240,995,208 字节，SHA-256 `1d02cf625e10ad2d8987c184454be2073d909290a130f967309db8ac7140df8a`，共 1,043 文件；CRC 检查通过。逐文件清单与 v9 比较：仅新增 hclust 许可，变更探针、来源清单及许可说明；模型和运行时文件哈希均相同。v10 构建时尚未补入后续发现的 fastcluster 2020 文件头版权声明，该声明在 v11 说明中补齐。v10 的打包测试六项及不生成音频的探针测试四项通过，真实生成测试依用户要求跳过；没有对 v10 声称独立实播。此包仍是验证材料，wheel 构建和模型转换来源未完全证明，不能作为正式产品分发准入。

2026-10-27：使用 v9 准备目录的运行时、探针和模型，在受限进程新生成 38 条 WAV 并调用同步播放；前后两个公网地址的探测均返回 WinError 10013，削波样本为零，加载 1.331 秒，峰值工作集 320,520,192 字节。音频和报告位于 `.tooling/issue-14/supertonic-v9-run/`，试听包 `.tooling/issue-14/supertonic-v9-audio.zip` 包含 39 文件、2,357,489 字节，SHA-256 `207e36f1369d7c1556cd204b4f91592988263c756a4c985761c286d711b69a43`，ZIP CRC 检查通过。这不是从验证 ZIP 解压运行。用户随后确认 v9 声音正确，并要求后续不再生成音频确认；反馈未逐项列出试听文件、设备或播放速度。

随后审查发现探针的 `assert` 校验在 Python 优化模式下会失效；空输入列表在 `-O` 下错误返回成功的 CLI 回归先失败，改为显式异常后通过。此修正仅在工作区源码中，已有 v9 ZIP 和准备目录仍包含旧探针，不应将它们视为修正后的包。

本轮验证：`test_prepare_supertonic.py` 六项通过；`test_probe_supertonic.py` 五项含真实禁网生成通过，之后将输入校验提前并再次运行四项无资源检查通过（真实生成项跳过）；`pnpm lint`、`pnpm typecheck`、`pnpm build` 和 `git diff --check` 通过。首次测试使用沙箱默认临时目录遇到权限错误，改用项目内 TEMP 后通过；pnpm 首次因受限 DNS 无法下载，使用既有项目 Corepack 缓存后通过。

2026-10-27：核对 CPython 嵌入包的 `libtommath.dll`，签名有效且签署者为 Python Software Foundation；去除 PE 证书表并还原签名修改的字段后，其 Git blob 与 CPython Tcl/Tk 二进制依赖完全一致。固定 Tcl 源码的 LibTomMath 许可已加入 v9；新增检查在 v8 失败、v9 六项通过。v9 ZIP 159,650,967 字节，解压逻辑大小 240,989,324 字节，SHA-256 `4d68fd5a14985ab6bbc66d9db3ef19ba0bea4bb49f1426ddd86bc4a1a5e73235`，共 1,042 文件。

同日比对上游 Windows x86-64 公开发布库：ONNX Runtime DLL 与 wheel 相同，但 sherpa-onnx C API DLL 哈希不同，不能把该公开构建记录当作 wheel 构建证明；实际构建依赖和 OpenSSL 归属仍需核验。

2026-10-27：审查 v7 发现 CPython 嵌入包附带的 `LICENSE.txt` 未包含官方文档中的 Expat 与 Unicode 通知；固定 CPython 3.14.10 提交的完整许可文档已归档并加入 v8。新增检查在 v7 因缺少该文件失败，v8 六项检查通过。v8 ZIP 为 159,649,043 字节，解压逻辑大小 240,985,671 字节，SHA-256 `04ec348a197568ab9c2f47ee5d5dacc1155904644f6788c561a046fbfe153b7c`，共 1,041 文件。v8 未重新生成或试听音频；不能将 v7 实播直接算作 v8 验证。该文档自身明确其第三方许可列表并不完整，其他运行库义务及实际二进制构建依赖仍未确认。

同日补测 v8 准备目录：受限 Windows x86-64 进程重新生成 38 条 WAV 并调用同步播放，生成前后对两个公网地址的探测均为 WinError 10013；模型加载 1.019 秒，峰值工作集 318,836,736 字节。该次不是从 ZIP 解压运行，也没有人工试听 v8 样本。

2026-10-27 用户反馈：已试听 v8 生成的音频，男女声均正常。反馈未逐项说明文件、设备和速度，不据此声称 38 条逐一验收或干净机器已验证。

2026-10-27 安全更新：历史 v1-v6 验证包包含 CPython 3.12.10，缺少后续安全修复，仅作为历史证据保留，不用于发行或处理不可信输入。[Python 3.12.16 发布公告](https://www.python.org/downloads/release/python-31216/)列出后续安全修复，且 3.12.10 是该分支最后的官方 Windows 二进制包。本次将准备脚本切换到 [Python 3.14.10](https://www.python.org/downloads/release/python-31410/) 官方 Windows x86-64 嵌入包，并固定对应 NumPy 和 sherpa-onnx CPython 3.14 wheel 的上游 SHA-256。CI Python 版本也固定到 3.14.10；不修改系统 Python。

新版 v7 ZIP 159,636,570 字节、解压逻辑大小 240,922,610 字节，SHA-256 `cae34b132a19c77412368e9daae265ba33bf142bd919a111fd3ad16f7a56db60`；六项打包检查通过。准备目录中以新版运行时在受限进程生成 38 条 WAV 并调用同步播放，前后两个网络探测均返回 WinError 10013，模型加载 1.008 秒、峰值工作集 320,389,120 字节。三项失败路径测试通过，集成测试单独跳过（已另行执行真实探针）；新运行时的解压副本、用户试听和远端 CI 尚未验证。该包仍不是产品发行包，模型及运行库许可准入仍未完成。

同日从 v7 ZIP 解压副本重新运行：受限 Windows x86-64 进程新生成 38 条 WAV 并调用同步播放，逐文件 SHA-256 匹配、削波计数为零；前后对两个公网地址的联网探测均返回 WinError 10013。模型加载 1.020 秒，峰值工作集 320,688,128 字节。播放调用不代表人工试听；新生成样本未经过用户试听，亦未在干净机器复验。

2026-10-27 用户反馈：试听 v7 生成的音频，男女声均正常。反馈未逐项说明试听的文件、设备或播放速度，不据此推断全部 38 条均已逐条验收，也不代表干净机器离线合成验证。

2026-10-26 复验：准备脚本从固定 wheel 中排除重复的 Scripts 原生库及其导入库，保留 Python 模块实际加载的库。新版 ZIP 为 **158,381,008 字节**，解压文件逻辑大小 **238,534,083 字节**，SHA-256 `8aa5773ddd38b2ce00a6f1c4c2fec78b0a95f6f2aaa1d2c528e5d7aad9315537`；较下述 v1 分别减少 8,310,199 和 22,644,950 字节。新包的 1,026 项清单哈希及 ZIP CRC 均通过，连同清单共 1,027 个文件；真实 ZIP 解压副本在受限进程中生成并调用播放 38 条，前后两个联网探测均返回 WinError 10013，模型加载 1.208 秒，峰值工作集 316,084,224 字节。五项打包检查通过，包括重复运行库和逐文件清单检查。该测量仍不包含桌面应用、词典、WebView2 或系统运行库安装器，不代表完整发行体积。

上游固定版本的 [Windows x64 构建配置](https://github.com/k2-fsa/sherpa-onnx/blob/af2d8d81dbfa61c4cea4c568da78f3982edca005/cmake/onnxruntime-win-x64.cmake)引用独立构建的 ONNX Runtime 1.28.2。实际 DLL 的 SHA-256 `7f66f939a881baf4f46a2216496798edf4a1429878b646d12674aa62f27d8a25` 与其 MT Release 归档中的 DLL 一致，不同于 MD Release。MT 归档哈希与上游配置一致；其 MIT 许可和第三方通知已加入[来源清单](supertonic-licenses/sources.json)。这不证明 sherpa-onnx 自身及其他依赖的许可与通知完整，许可准入仍未通过。

在 v2 基础上加入上述许可材料后，v3 ZIP 为 **158,414,965 字节**，解压文件逻辑大小 **238,868,060 字节**，SHA-256 `75b0043d2bab62b3b44adb3ffe32e591f3ad0b0d8aad0496aa124938dfe92615`；1,028 项清单哈希、ZIP CRC 和五项打包检查通过。v3 新增的是许可文件和清单，未重复进行解压副本的完整播放验证；前述播放结果对应 v2。

进一步核对固定 sherpa-onnx 构建配置后，v4 加入 utf8proc、pybind11、nlohmann/json 和 simple-sentencepiece 的上游许可及来源哈希。v4 ZIP 为 **158,424,212 字节**，解压文件逻辑大小 **238,891,051 字节**，SHA-256 `884908257233b2840cec8c910c45ab1fc08e09b80aa6656965fa4a5ff0a7f469`；1,032 项清单哈希、ZIP CRC 和五项打包检查通过。新增检查在 v3 上先因缺少许可文件失败，在 v4 上通过。v4 未重复播放，v2 的实播结果不自动证明 v4 的实播。构建声明不能代替二进制依赖清单，KissFFT、kaldifst、OpenFST、Eigen 等传递依赖和实际构建选项仍需核验。

2026-10-27：v5 补充固定 kaldi-native-fbank 源码声明的 KissFFT 版权声明和 BSD-3-Clause 许可；文件与固定上游 Git blob、SHA-256 均一致。针对新增许可文件的检查在 v4 上先失败，在 v5 上通过。v5 ZIP 为 **158,426,421 字节**，解压文件逻辑大小 **238,895,433 字节**，SHA-256 `9d2394dbbaa93c06f5a04c17e40cce012c91d2fd1384d4a8e6e0d16b7d1412da`；1,034 项清单哈希、ZIP CRC 和五项打包检查通过。v5 没有重复解压播放；v4 的播放结果不能自动证明 v5 实播。源码声明不证明 wheel 实际链接 KissFFT，其他传递依赖、发行义务和产品集成仍需核验。

2026-10-27：v6 补充固定源码声明的 kaldifst 许可及 OpenFST 版权和作者文件；原始文件与固定上游 Git blob 和来源清单中的 SHA-256 一致。新增检查在 v5 上因缺少文件失败，在 v6 上五项检查通过，包括归档 CRC 和清单校验。v6 ZIP 为 **158,432,887 字节**，解压文件逻辑大小 **238,911,906 字节**，SHA-256 `9967b8df3c363df58b1a5ef79f4c9d17fe980e800b76787c7ca46d99d7f7f553`，清单及内容共 1,038 个文件。v6 未重新解压实播；这些源码声明不能证明 wheel 的实际链接，完整传递许可及产品发行仍未验收。

同日复测：从 v6 ZIP 解压副本在受限 Windows x86-64 进程中重新生成 38 条 WAV，并完成同步播放 API 调用；逐文件 SHA-256 核对通过，削波计数为零，前后对两个公网地址的联网探测均返回 WinError 10013。模型加载 1.024 秒，峰值工作集 315,064,320 字节。API 返回不证明音频已被人听到或质量通过；此前用户接受的试听结果对应 v4 样本，不是 v6 新生成样本的人工验收。

同日通过原生版本 API 核查 v6：Python 扩展和 C API DLL 均报告版本 `2.0.1`、Git 提交前缀 `af2d8d81`，与所记录的 sherpa-onnx 源码提交前缀相同；该自报前缀不是构建证明，不能确定完整依赖版本。新增检查从包内 CPython 独立加载两个组件并比较其版本信息。

设置 `LEXIQUE_SUPERTONIC_BUNDLE` 指向 v6 目录后，以其 `runtime/python.exe -X utf8 -m unittest discover -s docs/evidence/issue-14 -p test_prepare_supertonic.py -v` 执行，六项通过，包括该版本检查及包清单、损坏输入和既有输出保护。

2026-10-27 用户确认：本票以 Windows x86-64 为实际运行验收范围；Windows ARM64、macOS x86-64 和 macOS ARM64 记录为待后续验证，不作为 #14 关闭的阻塞项。它们仍是最终桌面发行的要求，不能据此宣称已构建或运行通过。

对 Windows x64 文件的导入表检查显示，CPython 依赖 `VCRUNTIME140.dll`、Windows UCRT 和系统 DLL；前两个 VC runtime DLL 文件存在于嵌入式 Python 包内，但这不证明干净系统可运行，也不证明全部再分发义务已满足。

2026-10-27：从 v4 ZIP 解压副本在受限 Windows x86-64 进程中重新生成 38 条 WAV，并完成同步播放 API 调用；生成前后对两个公网地址的探测均返回 WinError 10013。模型加载 1.086 秒，峰值工作集 315,506,688 字节。生成报告及可离线打开的试听页打包供用户在新设备验证；试听结果待用户反馈，播放预生成文件不等于在新设备验证离线合成。Windows ARM64、macOS x86-64 和 macOS ARM64 保持待后续验证。

试听 ZIP 保存在忽略目录 `.tooling/issue-14/supertonic-v4-listening.zip`，包含 38 条 WAV、`listen.html` 和生成报告，共 2,386,534 字节，SHA-256 `6c11a3a4e99f85d5c3b3f848b78ce198a08bb51ead82a78735fa58c81c3769a6`；ZIP CRC 与 38 条音频的报告哈希均通过。新设备解压后打开 `listen.html`，可选择女声 / 男声及 1.0 / 0.6 倍播放。该 ZIP 不含模型和运行库，不是安装包。

同日使用 `compare_supertonic_opus.py <输出目录> <WAV目录>` 对 v4 的 38 条 WAV 编码 Ogg Opus，分别采用 libopus `voip` 24 和 48 kbit/s。WAV 总计 5,467,662 字节，24k 为 147,017 字节，48k 为 288,090 字节；76 条 Opus 均已解码且非静音。独立对照包 `.tooling/issue-14/supertonic-v4-wav-opus-listening.zip` 包含三种音频、试听页和报告，大小 2,795,237 字节，SHA-256 `c1e3edd7fe410aa572017bfdb1bfd4454774c4e546e8eda7d88d9ba7a964865a`；ZIP CRC 及全部 114 条音频的报告哈希通过。本机浏览器对默认样本的三种格式成功加载元数据；新设备的格式支持和听感等待用户验证，不据此改变产品音频格式决策。

2026-10-27 用户反馈：WAV、Opus 24 kbit/s 和 48 kbit/s 内容正常、音质可接受；更换设备后常速和 0.6 倍慢速播放也正常。本次 `sommes` 的 24 kbit/s 样本听到类似底噪的轻微噪音，但不影响听清；前次对应试听未发现此问题。用户未指明声线或噪音是否存在于同次 WAV / 48k，暂不推定原因。核对发现两次男女声 `sommes` 的原始 WAV 哈希均不同，不能把听感差异直接归因于 Opus；需针对同一声线、同次 WAV 和两档 Opus 做对照。试听预生成文件仍不证明新设备的离线合成。

同日用户再次确认：上述现象更准确是 24 kbit/s 在同一句中声音时轻时响，而非已证实的底噪；48 kbit/s 未出现该问题。用户决定应用新生成语音默认采用 Ogg Opus 48 kbit/s。现象原因未查明，测试听感不保证所有输入或设备均无波动；产品编码与播放尚未接入。

以下保留 v1 原始结果作对照。

[准备脚本](prepare_supertonic.py)对固定 CPython、NumPy、sherpa-onnx wheels、七个模型资源和模型许可文件逐一校验后，构造独立验证目录与 ZIP，保留逐文件 SHA-256 清单；缺失或校验失败不创建输出，已有输出不覆盖。Windows x64 本机实测 ZIP 为 **166,691,207 字节**，解压文件逻辑大小 **261,179,033 字节**，SHA-256 `ea444b2fcea9123428ecd8f7baffef53bb9abb40ac5af610824f0b5053eea242`，ZIP CRC 校验通过。

这只是运行探针，不是 Lexique 完整版或 Lite 发行包；不含桌面程序、词典、WebView2 或 VC++ 运行库安装器，测试机使用已有系统运行库。不得将该数字当作最终安装包或干净机器的安装占用。模型七文件占 145,295,768 字节；词典和其他资源的现有集成方案不因 Lite 变化。

```powershell
.tooling/issue-14/comparison/venv/Scripts/python.exe -X utf8 docs/evidence/issue-14/prepare_supertonic.py .tooling/issue-14/materials .tooling/issue-14/supertonic-wheelhouse .tooling/issue-14/supertonic-3/sherpa-onnx-supertonic-3-tts-int8-2026-05-11 .tooling/issue-14/supertonic-evidence-new
```

上述来源为本地已获取材料，输出路径必须不存在。系统运行库和全部传递依赖的分发许可、其他平台原生构建与干净机器验收仍未闭合；此验证包不构成已获准的产品分发。

验证包内 CPython 在同一 Windows x64 受限进程中重新生成并调用播放 38 条样本，退出码 0，前后对两个公网地址的连接均返回 WinError 10013；模型加载 1.186 秒，峰值工作集 316,956,672 字节。ZIP 中 1,031 个清单文件的 SHA-256 全部匹配，没有额外条目；另有清单本身，共 1,032 个文件。人工试听结论沿用用户对原样本的确认，不把播放 API 返回等同听感。此次执行从准备目录运行，不是另一台干净机器或从解压 ZIP 的安装验证。

```powershell
.tooling/issue-14/supertonic-evidence-v1/runtime/python.exe -X utf8 .tooling/issue-14/supertonic-evidence-v1/docs/evidence/issue-14/probe_supertonic.py .tooling/issue-14/supertonic-evidence-v1/assets .tooling/issue-14/supertonic-evidence-v1-run
.tooling/issue-14/comparison/venv/Scripts/python.exe -X utf8 -m unittest discover -s docs/evidence/issue-14 -p test_prepare_supertonic.py -v
```

失败路径测试 4 项通过：缺失输入、损坏运行时、已有输出目录或归档。测试没有覆盖 Lite 下载、安装、下载中断或重试，因为这些产品能力尚未实现。运行前需外部限制该进程的网络；探针自身只检查拒绝，不修改防火墙。

其他目标的候选获取路线：Windows ARM64 可参考上游 [ARM64 构建](https://k2-fsa.github.io/sherpa/onnx/install/windows/generated/build_cpu/windows_arm64_cpu_build.html) 或 [预编译库](https://k2-fsa.github.io/sherpa/onnx/install/windows/generated/download/windows_arm64.html)；macOS x86-64 和 ARM64 可参考上游 [Python 平台说明](https://github.com/k2-fsa/sherpa/blob/master/docs/source/onnx/python/install.rst)及 [2.0.1 发布资产](https://github.com/k2-fsa/sherpa-onnx/releases/tag/v2.0.1)。这只是获取线索，尚未固定这三种组合的二进制和哈希、构建产品适配层、运行或测试许可；Windows x64 wheel 不可直接复用于它们。

## 追加的 CLI 回归与下载量核对

用户已确认完整样本及 Opus 24 / 48 kbit/s 音质；Opus 仍是实验格式，未进入产品。新增 [CLI 回归](test_probe_supertonic.py)：缺失、损坏资源和已有输出均失败且不覆盖证据；固定模型在受限 Windows x64 进程重新生成 38 个音频，逐个回读非静音 WAV，并核对两次网络探测和原始输入。未知拼写能产生音频不意味着拼写存在或发音正确。

PyPI 的 Windows x64 CPython 3.12 `sherpa_onnx` 2.0.1 wheel 为 2,178,304 字节，SHA-256 `9f25448a1af23a013064264b9e2241e0c5d0e1941aeb82dafa969520c694efb9`；`sherpa_onnx_core` 2.0.1 wheel 为 16,663,244 字节，SHA-256 `08890e9ea23ed2f59f81d5840c63b6b5cc8674d0f6e48d9069126a848aa0e022`。已下载并核对 PyPI 摘要。加上模型原压缩包 128,774,318、既有 CPython 3.12.10 embed 包 11,133,606、NumPy wheel 12,573,091 和 VC++ 离线安装器 25,635,768 字节，六个独立下载文件合计 **196,958,331 字节**。这不是最终压缩包：尚未纳入应用、词典、WebView2 离线依赖、完整许可与通知，也未检验重新封装后的大小或安装占用，不能据此宣布低于 200 MB。

```powershell
$env:TEMP = "$PWD/.tooling/issue-14"
$env:TMP = $env:TEMP
$env:LEXIQUE_SUPERTONIC_ASSETS = "$PWD/.tooling/issue-14/supertonic-3/sherpa-onnx-supertonic-3-tts-int8-2026-05-11"
.tooling/issue-14/comparison/venv/Scripts/python.exe -X utf8 -m unittest discover -s docs/evidence/issue-14 -p test_probe_supertonic.py -v
```

这组 CLI 回归使用现有虚拟环境；上面的独立验证包另行记录。两者均不是干净机器安装验证，完整分发许可义务仍未闭合。

## 后续试听反馈和慢速候选

用户确认对常速音频以 `0.6` 倍播放的效果清晰自然，并确认 `i grec` 作为单独字母 `y` 的读音。用户随后也确认完整样本及 Opus 试听；这些确认不自动覆盖产品链路。

原定 17 项现已分别以女声和男声正常生成，另有英语和字母名对照，共 38 个文件，可在[完整样本试听页](supertonic-full/listen.html)选择 1 倍或 0.6 倍播放。每个文件的哈希均与[报告](supertonic-full/report.json)一致，生成前后网络探测均遭拒绝。原始输入 `y` 保留作为对照，字母名使用另一个 `i grec` 样本。

用户后续明确要求所有语音以正常速度生成，慢速只在播放时使用 0.6 倍；[试听对照](supertonic-slower/listen.html)已改为对同一常速文件应用播放倍率。之前生成的慢速样本作为历史证据保留，不作为当前方案。

历史记录：用户确认此前英语和法语音频可用，要求慢速再慢一些；当时生成的慢速候选保留作为对照，当前方案使用播放变速。

法语代词 `y` 的 /i/ 属于不同语境，不能将已确认的字母名自动当作所有用法的发音。尚未实现音素指定、组合双读音、产品合成或缓存。

以下内容保留初次筛查时的记录，其中“待试听”描述的是当时状态。

2026-10-26：用户否决全部 MLS 样本的法语读音和音质，已撤回该候选。许可证据与有效 WAV 不能覆盖人工否决；本轮换用不同引擎作小批筛查，没有修复或重新认可 MLS，也没有接入产品 TTS。

## 当前试验

- 资源：sherpa-onnx 维护者 Supertonic 3 int8 发布包；压缩 128,774,318 字节，七份必要文件 145,295,768 字节。独立核对发布包 SHA-256、七文件与固定 HF 镜像摘要，全部通过。来源及边界见[研究](replacement-options-after-mls.md)、[清单](supertonic3-model.json)和[许可归档](supertonic-licenses/ATTRIBUTION.md)。不是最终桌面完整下载量。
- 隔离运行库：`sherpa-onnx==2.0.1`、`sherpa-onnx-core==2.0.1`，不进入产品依赖或锁文件。Windows x64 CPU，两线程，八步去噪。
- 女声 F1=`sid 0`，男声 M1=`sid 5`；正常 `speed=1.0`，慢速 `0.8`。24 个法语样本加 2 个英语控制，共 26 个 44,100 Hz 单声道 PCM16 WAV。12 对法语样本逐条慢速更长，总时长倍率均约 `1.25`。
- 进程前后联网探测均返回 WinError 10013；26 次同步 Windows PlaySound 调用完成。加载约 1.10 秒，单次生成约 0.44–0.80 秒，峰值工作集 316,956,672 字节。这些是本机技术观测，不是声音正确的证明。
- [试听页](supertonic-samples/listen.html)清楚标明机器合成，提供原文、四组合及本地 ASR 对照。`y`／`i` 是普通文本对照，不是音素控制或已实现的多读音策略。

这是 **部分筛查**：只覆盖 `école`、`œil`、`nous sommes ici`、`J'apprends le français tous les jours.`、`y`、`i`；通过人工试听后仍需运行原 #14 全部 17 项、未知词与其他边界。短词、孤立 y 和自然度没有通过验收。

## 实际发现的语言参数问题

官方 Python 示例逐项修改 `GenerationConfig.extra["lang"]`。在本机 2.0.1 绑定中，此 getter 返回副本，修改后读取仍为 `{}`。先加“语言必须传入原生配置”的断言，真实 CLI 退出 1，复现该问题；改为完整赋值 `config.extra = {"lang": lang}` 后通过，断言保留在[探针](probe_supertonic.py)。该问题属于新引擎调用，不是旧 Piper MLS 失败的根因结论。

首轮丢失语言的样本保留在忽略目录 `run-01`，语言回归失败目录为 `run-02-language-repro`；当前公开样本仅来自 `run-03-fr`，未混用早期输出。生成过程没有随机选择模型或转换音频播放速度。

## 本地识别对照

[筛查脚本](screen_speech_asr.py)使用固定[Whisper tiny int8 资源](asr-model.json)，仅作临时诊断，不随产品分发。相同目标句 `J'apprends le français tous les jours.` 的结果为：

| 样本 | 识别结果 | 归一化词错误率 |
| --- | --- | ---: |
| 已试听 SIWIS | J'apprends le français tous les jours | 0 |
| 已试听 Tom | J'apprend le France et tous les jours. | 0.429 |
| 已否决 MLS 女声 | Je vais vous remerter tout, je vais vous remerter tout. | 1.429 |
| 已否决 MLS 男声 | J'apprends cette route, c'est tout, elle joue. | 1.000 |
| Supertonic 女声正常 | Je apprend le français tous les jours. | 0.286 |
| Supertonic 女声慢速／男声两档 | J'apprends le français tous les jours. | 0 |

词错误率包含插入项，可超过 1；只比较固定句子，不推广为模型总体分数。ASR 对多个短词和 `y`／`i` 不稳定，可能错拼或幻觉，不能据此确定其实际发音。完整 28 项对照见[机器报告](supertonic-samples/asr-screen.json)。归一化的重音与 œ 自检先失败后修正，通过后重跑全部对照；未手动修饰识别转写。

当前探针只生成常速音频并包括 `i grec`，下面命令不会重现历史慢速样本；使用未存在的输出路径，并在禁网受限进程内运行：

```powershell
.tooling/issue-14/comparison/venv/Scripts/python.exe -X utf8 docs/evidence/issue-14/probe_supertonic.py .tooling/issue-14/supertonic-3/sherpa-onnx-supertonic-3-tts-int8-2026-05-11 .tooling/issue-14/supertonic-3/run-new
.tooling/issue-14/comparison/venv/Scripts/python.exe -X utf8 docs/evidence/issue-14/screen_speech_asr.py .tooling/issue-14/asr-check .tooling/issue-14/supertonic-3/run-new .tooling/issue-14/supertonic-3/asr-new.json
```

模型许可允许有条件再分发，实际发行仍须落实 OpenRAIL-M 用途条款、通知与修改说明，以及全部运行依赖；上述历史筛查当时 #14 尚未完成。完整产品包体、干净机器、其他平台及产品合成／缓存均未验收；当前准入决定见本页开头。
