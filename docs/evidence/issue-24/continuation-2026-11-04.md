# #24 实施与验证

2026-11-05，Windows x64。schema17 的有界复习窗口、增量补位、选定音频、持久生成需求及统一生命周期已完成本机验证，用户确认干净系统离线测试通过。当前实现保留学习数据，并支持批量期间的查词、编辑、播放与评分。[验收覆盖](acceptance-matrix-2026-11-05.md)逐项记录50专项、原T01–T12／V01–V06及AC01–AC14；#24本期验收完成。

## 行为

- 队列元数据、需求候选、音频版本和卡片记录使用一致的 SQLite 快照；文件校验期间释放事务。热刷新复用未变化的元数据与音频，窗口最多4卡、候选最多50、每次最多8次文件检查、压缩缓存最多32MiB。
- 当前作答绑定其 progress revision；无关更新保留作答，过期 presentation 显示冲突。预读切卡后撤销目标更新，丢响应重试保留原 operation。
- 暂时文件错误以一秒退避允许显式重试。首次读取遵守持久发布意图；临时缓存命中更新 LRU，生成起算的绝对期限保持不变。
- 单词当前版本永久保护，上一版14天，最后归属解除及临时内容7天；200MiB软容量遵守保护优先。维护分批推进，未知文件保留。
- 首次建立或学习日重置时，队列按准入顺序一次分配稀疏位置；已保存的锚点、间隙重编和评分撤销继续保持原语义。
- 共享核心使用[IANA2026f数据](timezone-data-update-research.md)，Vancouver 学习日按全年 UTC−7计算。旧 due 和评分历史保持不变。

## 验证

最终二进制：`target/issue-24/accepted-binary.exe`，SHA-256 `61aae370b2e04fd0efec69fdafe142bf7ee99d918036857b4e195236493111e4`。资源来自已准入 CPU 语音开发包；所有业务 fixture 均在隔离目录中创建。命令日志位于 `target/issue-24-continuation/`，原始结果位于 `target/issue-24/`。

| 检查 | 命令／结果 |
| --- | --- |
| 前端 | `pnpm lint`、`pnpm typecheck`、`pnpm build`；100项浏览器测试通过；含受控跨两学习日恢复／时区重读、三语未知规格提示及逆序搜索响应 |
| 后端 | `pnpm rust:test:build` 后 `pnpm test:rust`；138项通过，执行硬限60秒；含首次批量位置零重编、间隙耗尽回滚／重开撤销、100／1k四种音频状态热快照SQL次数相同 |
| Rust静态／构建 | `cargo fmt --all --check`、`cargo check --locked --workspace`、`pnpm desktop:build --no-bundle`通过 |
| 调度 | `pnpm scheduling:build`、`pnpm test:scheduling`；74固定向量和4096校准输入，原生／实际WASM零差异 |
| 独立队列参考 | `native-oracle-24.mjs`，Vancouver识义序列22/22及音频序列7/7通过；含原始JSON／实体音频参考、外部连接变更、发布、评分／撤销及重启恢复 |
| 崩溃恢复 | `native-crash-24.mjs`，九个断点通过：intent、tmp写入／同步、rename、readback、DB完成及GC mark／delete／finish。真实Windows文件／SQLite子进程在边界被kill，`finish-binary.exe`两次重启验证恢复与幂等 |
| 发布／文件 | 缓存10项通过，包括真实Windows文件占用失败后保留mark、释放后删除／完成；crash fixture通过显式子进程入口执行。桌面库3项通过 |
| 原生产品 | 真实CPU生成、非静音解码、normal／0.6播放、批量期间交互、原规格修复、共享锁恢复、迁移／发布恢复、自动需求及容量维护 |
| 音频边界 | `native-audio-boundaries-24.mjs`四组通过：零额度日内双卡、header-only实际解码失败且不能评分、10MB／32MiB预算、16秒／90,494字节实际codec完整有限非静音解码与播放 |
| 超时与恢复 | `native-timeouts-24.mjs` startup／runtime通过：挂起真实owned worker，30秒启动超时／持久4秒剩余执行期限均停止原进程；执行attempt 3→4终止，新请求恢复，重启保留原预算 |
| 资源与增量 | `native-resources-24.mjs`，50k词／100k卡／100k历史，100次原生评分全部提交；每次重载1行metadata／解析1项progress、0重建；取样及退出均无worker |
| 审查 | 独立Standards／Spec对最终位置实现和相关回归均为0项可执行发现；文档逐项覆盖与链接检查通过 |
| 干净系统离线 | 用户于2026-11-05确认“Clean-system offline test passed”；执行方为用户，未提供测试机版本及逐项日志 |

原生脚本从 `apps/desktop` 执行，入口为 `node e2e/<script>.mjs <resource-package> <frozen-binary> <fresh-destination>`；交付资源为 `../../target/issue-24/delivery-2026-11-05/Lexique`。window入口另传词数／状态／样本数；全坏压力分两组各50次，合并100次报告。timeout入口另传startup或runtime；oracle音频入口另传UTC和audio；crash入口另传预编译的 speech_cache 测试可执行文件和断点。每个原生执行维持硬60秒。新fixture只复制分发资源，并单独记录当前执行身份。

## 性能

Ryzen9 8945HS／32GB RAM／Windows11／NTFS；未清空操作系统缓存。每组100次尝试，最近秩分位数，编译与性能测量分开。

| 场景 | p50 ms | p95 ms | max ms | 失败 |
| --- | ---: | ---: | ---: | ---: |
| 最终二进制，10k词／20k卡健康热窗口 | 47.8 | 92.7 | 99.3 | 0/100 |
| 最终二进制，10k词metadata-ready全坏，两组各50次 | 477.3 | 885.0 | 2101.4 | 0/100 |
| 评分点击→持久提交→下一卡DOM | 10.2 | 11.6 | 24.7 | 0/100 |
| 最终二进制，50k词／100k卡／100k历史：评分IPC | 192.5 | 220.3 | 232.4 | 0/100 |
| 同一50k混合历史：评分后队列IPC | 196.1 | 228.9 | 245.3 | 0/100 |
| 100词健康热窗口 | 4.2 | 4.7 | 5.4 | 0/100 |
| 1k词健康热窗口 | 6.6 | 7.6 | 8.3 | 0/100 |
| 50k词／100k卡元数据压力 | 210.8 | 221.6 | 232.7 | 0/100 |
| 10k词50%缺音频 | 55.8 | 115.8 | 464.1 | 0/100 |
| 10k词100%缺音频 | 63.8 | 247.5 | 508.3 | 0/100 |

最终10k健康窗口达到p95≤100ms目标；热请求零文件探测、零音频payload、零元数据行重载和零progress解析。全坏场景保持结构预算并返回窗口，延迟包含修复争用。DOM评分测量包含IPC，提供post-return展示耗时的上界。50k混合历史队列p99为239.6ms、评分p99为229.2ms，独立报告大历史下的实际成本。

57.25秒资源取样中，宿主工作集33.3→63.5MB、私有提交10.4→41.5MB，重启分别回到32.9／10.2MB；句柄366→372→371、重启366，监听器185→183、重启185。owned日志逐份检查无fixture词头、笔记或来源URL。50k资源fixture共享一份真实选定音频；独立文件／codec／字节压力由音频边界组验证。

各原生结果的`run-metadata.json`记录实际二进制。早期协议验证使用`complete-binary.exe`（`33a7e17521fc89457826e42659f839681c452026e8f1203090192dfc947463d1`）；CRASH及部分规模／错误分片使用`finish-binary.exe`（`564c4a907ee3bc44987ae43b9b7ed7a282ae89d265527e50e7345308bff7a712`）。元数据50k分片仅前100词有实体音频，其结果与混合历史组分别列出。

## 用户离线测试交付

交付为[便携ZIP](../../../target/issue-24/delivery-2026-11-05/Lexique-windows-x64-issue-24.zip)及`target/issue-24/delivery-2026-11-05/Lexique`目录，含最终`Lexique.exe`、词典、CPU语音资源与原许可；不含`Lexique-data`或用户学习数据。解压或复制整个目录到测试机可写位置，保留相邻资源目录。

交付测试范围为：测试机断网启动、查`école`并保存笔记与加入双卡、真实听音及normal／0.6播放、批量期间查词／编辑／评分、关闭重开后的数据保留，以及关闭后移动整个目录再重开。用户于2026-11-05确认干净系统离线测试通过，并要求完成#24；本记录登记用户报告的整体通过，不推定未提供的Windows／WebView2版本或逐项观测。

## 限制

原生产品附加检查记录一次WebView2进程访问异常（ProcessFailed kind=0，exit=-1073741819）；崩溃模块为RTSSHooks64.dll＋0x1490af，按用户既定决定不属于#24兼容性验收；模块记录保留在`target/issue-24/accept-product/crash-module.json`，ProcessFailed诊断位于该fixture的`Lexique-data/logs/speech-cache.log`。资源结果是本机有界分片，工作集／私有提交在预热后增长，尚无小时级趋势、物理休眠、物理冷盘及其它硬件结果；跨学习日／时区恢复使用受控时间与可见性。未诱发操作系统磁盘满或物理掉电；I/O失败保留与发布／清理进程kill恢复已有执行证据。未来M3／M4／M6及其它平台沿原计划，当前结果不扩大正式发行就绪结论。
