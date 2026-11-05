# #24 开发前验证结果

2026-11-02；基准 `6850ffb095a69084b02ee40a702355f660c3bd97`。先冻结 [预算](preflight-plan.md)，再完成 V01–V05，之后按用户选择继续桌面实施。本文件记录候选验证；产品回归另见 [桌面实施](desktop-results.md)。全部数据库、文本及音频为隔离 fixture，未读取用户学习库。

| 验证 | 实际结果 | 可复核证据与边界 |
| --- | --- | --- |
| V01 | 改前 UI 重现首词合成阻塞后词建卡及导航；Windows 原生全局锁使生成期间 preview 等待 3548.1 ms | `preflight-24.spec.ts` 的改前失败记录、`target/issue-24/preflight-native-20261102-a/results.json`；原生旧二进制 SHA-256 `23c27dff9920e4c16b77a71f800e3b9cf97c2fb07287db1b894d6b179542c034`。模拟时序不是原生性能 |
| V02 | 36 轮、432 份音频完整解码；选择后台 2 + P0 保留 1、2 推理线程、关闭空转 | `measure_preflight_tts.py` + `native-preflight-24.mjs`；固定／随机路径、男女声、单模型并发与 CPU 关闭空转分别检查。后台 4 超过相对 P0 预算，拒绝采用 |
| V03 | 20／50／100 词候选真实 SQLite 短事务及 savepoint 通过；最长事务 8.27 ms，竞争笔记／评分等待最大 29.30 ms | `crates/storage/tests/preflight_24.rs`；唯一性、非法／未知 ID、失败回滚、commit 拒绝、幂等重试、暂停／进度保留。仅候选 API，不当产品实施证据 |
| V04 | UTC Unix 秒、完成起算七天、到期边界、NFC／声线／模型／语境键、临时转归属、多引用、旧文件保留、安全 GC 契约通过 | `scripts/test_preflight_audio.py`；来源时间未知的 legacy 不推测、不做 TTL 删除；文件发布及 Windows 安全边界在产品回归补验 |
| V05 | 受控 framing、状态／优先级／保留容量、取消、最多四次尝试、2／8／20 秒退避、总预算及恢复契约通过 | `scripts/test_preflight_audio.py`、`scripts/test_speech_protocol.py`；受控 worker 不证明真实模型、网络服务或原生磁盘 |
| V06 | deferred | M3 真实例句、M4 在线／认证／同步、M6 WASM 网页及浏览器缓存须沿 F01–F05 补准入和集成，不增加未配置的外发 |

V02 的候选热 P0 点击到合成完成（3 轮中位数／最大值，ms）：

| 种子／空转 | 无后台 | 后台 1 | 后台 2 | 后台 4 |
| --- | --- | --- | --- | --- |
| 固定／开启 | 582.3 / 861.4 | 719.5 / 728.7 | 851.7 / 901.4 | 1266.9 / 1352.1 |
| 固定／关闭 | 624.2 / 634.0 | 707.3 / 709.1 | 831.0 / 863.7 | 1152.8 / 1212.7 |
| 随机／关闭 | 619.7 / 642.6 | 718.1 / 729.6 | 851.5 / 876.3 | 1172.4 / 1183.8 |

后台 2／关闭空转的私有提交峰值 872.7 MiB、工作集峰值 367.2 MiB，低于冻结的 1.5 GiB／1 GiB；后台 4 的主要失败是相对 P0 延迟，不能用较快批次吞吐代替交互验收。候选的冷加载、预热、服务／等待、CPU、内存采样和完整轮次保留于 `target/issue-24/preflight-tts-20261102-a/results.json`。这些计量不是当前 supervisor 的端到端结果，也不替代人工法语音质。

复核命令（PowerShell 7，`python` 指向 LOCAL-DEV-ENV 的可用 Python；所有单次实验执行硬限 60 秒，首次编译分开）：

```powershell
pnpm rust:test:build
python -X utf8 -c "import sys,pathlib;sys.path.insert(0,'scripts');from measure_review import run;run(['target/debug/deps/preflight_24-bb4abbd67b588cdc.exe','--nocapture','--test-threads=1'],pathlib.Path('target/issue-24/preflight-v03-new.log'))"
python -X utf8 -m unittest discover -s scripts -p test_preflight_audio.py -v
python -X utf8 -m unittest discover -s scripts -p test_speech_protocol.py -v
python -X utf8 scripts/measure_preflight_tts.py target/issue-22-final-evidence/Lexique-0.1.0-windows-x64-portable-speech/speech target/issue-24/preflight-tts-new
```

实机环境沿用既有报告的 Windows 11 / Ryzen 9 8945HS / 32 GB / NVMe；未改变电源设置，未测试其他硬件、系统、干净系统断网或人工音质。后续产品测试保留独立文件和失败尝试；不覆盖此前测量，也不把历史候选收益当当前发行结论。
