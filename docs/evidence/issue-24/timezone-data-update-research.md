# 时区数据

2026-11-05。共享核心使用 chrono-tz 0.10.4 的上游运行时代码和 IANA 2026f 数据，原生与 WASM 通过同一 Cargo patch 选定。[来源、许可与再生成命令](../../../vendor/chrono-tz-0.10.4/LEXIQUE.md)随源码保存。

[B.C. 官方公告](https://news.gov.bc.ca/releases/2026AG0013-000209)规定2026年3月后全年 UTC−7。[IANA 2026f](https://data.iana.org/time-zones/tzdb-2026f/NEWS)同时包含 Vancouver 和 Cuba 的最新规则。chrono-tz 的[上游 0.10.4](https://github.com/chronotope/chrono-tz/releases/tag/v0.10.4)使用2025b；本地补丁更新数据及预生成表，保留运行时 API 和显式版本回放契约。

测试覆盖 Vancouver 新规则与变更前冬季、Havana 的2026f规则、学习日04:00边界、固定调度向量和原生／实际WASM一致性。已存绝对 due 和评分历史保持不变；指定其他数据库版本的历史回放使用对应核心构建。验收结果见[实施记录](continuation-2026-11-04.md)。
