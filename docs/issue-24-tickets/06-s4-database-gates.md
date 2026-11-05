# B06／S4：按实测瓶颈完善索引、投影及数据库并发门槛

## Parent

[#24 复习队列、选定单词音频与安全清理规格](https://github.com/zardonc/lexique/issues/24)。

## What to build

对已可用窗口链路的数据库打开、JSON解析、候选／摘要及锁等待作同期测量，交付是否需要进一步结构优化的明确结论。只在证据支持时将进度／额度投影、索引或小读池贯穿真实存储／IPC接入；无必要时记录选择及目标状态，不强制引入WAL／全局写actor。

## Acceptance criteria

- [ ] 在S3固定链路上按S0预算测健康／混合／积压与100／1k／10k规模，摘要扫描与窗口分别计量；每稳态case保存至少100次尝试／失败、p50／p95／max，不用少量样本放宽目标。
- [ ] 明确记录需优化／无需增加结构及证据；不采用可选投影／WAL时将对应条件项标为不适用并说明，保留未达性能目标而不写passed。
- [ ] 若投影，完整state／revision仍权威，评分／撤销／回填及所有现有写点同事务维护；原始JSON独立oracle验证后才启用，空progress／NULL due新卡不漏，未来同步写点记录门槛。
- [ ] 索引避免已有PK／UNIQUE重复，按实际查询计划／返回行／访问量和行为验证，NULL／学习日／排序／额度规则不变；无需把查询打印字符串当唯一断言。
- [ ] 若WAL／多连接checkpoint，先升级至SQLite修复版本或验证明确回补并记录实际version／source_id；统一配置／回读foreign_keys、busy_timeout、journal_mode、synchronous，学习数据维持FULL。
- [ ] 2／4小读池及验证并发按实测选择，文件／合成不持长读写事务；写锁／checkpoint／迁移失败可恢复，portable WAL／SHM留本目录，热库文件不作普通云文件复制。
- [ ] T24-C03、C06条件验证及P01（热10k DB＋窗口p95≤100ms待测）有真实结果／不适用原因；选定版本、额度、顺序与UI结果不变，相关README检查及60秒限制保持。
