# 应用内管理数据与跨云存储同步的格式评估

研究日期：2026-11-02 UTC。本文是选型建议，不是已确认规格或同步实现证据。

2026-11-08 补充：新增[选词／屏幕 OCR](translation-inputs.md)的原图和原文仅用于主动输入；[词频资源及派生颜色](word-frequency-colors.md)与用户内容分离，不纳入学习数据自动交换。[WPM 偏好](speech-preferences.md)是否跨设备共享仍在 M4 冻结范围，本格式建议不因本次需求更新而成为已实现协议。

## 推荐

以**有明确版本和字段约定的 UTF-8 JSON / JSONL**作为用户学习数据的同步交换格式，桌面本地继续使用 SQLite。云盘负责保存和传输文件；数据校验、编辑、合并、恢复与展示由应用负责。单设备备份可使用完整 JSON 快照；多设备同步推荐 JSON 提交清单及不可变 JSONL 快照 / 操作分段，须额外处理共同基线、并发修改和删除，不能把上传快照直接视为同步已经完成。综合机制与 M6 IndexedDB 宿主设计见[同步设计](sync-design.md)。

JSON 的对象、数组和 Unicode 字符串适合表达生词记录、分类、两种独立复习卡片、进度和历史之间的关系。UTF-8 是 RFC 8259 对跨系统交换 JSON 的编码要求；但业务字段、格式版本和冲突规则仍需 Lexique 自己定义。[JSON 标准](https://www.rfc-editor.org/rfc/rfc8259)

需要减小传输体积或打包附件时，可以使用包含 JSON 的 ZIP；ZIP 是包装层，不会增加同步或合并能力。JSONL 便于逐条处理和封存增量批次；复习历史增大后可另行分区，不反复复制全部历史。不建议让不同设备同时追加同一个远程日志文件，也不把本地 JSONL 写入作为已确认操作的持久性边界。

## 与当前产品决定的关系

较早[产品基线](product.md)与[开发路线](roadmap.md)采用可人工编辑 Google Sheets 的双向同步。2026-11-02 用户改为应用内查看 / 修改、跨云文件兼容和后台批量同步；此次规划已据此更新，正式跟踪在 [#1](https://github.com/zardonc/lexique/issues/1)。具体 SQLite + JSONL 协议、默认值和服务接入是待验证建议，没有冻结为 ADR 或开始同步实现。

建议继续遵守既有决定：

- [ADR 0001](adr/0001-separate-bundled-resources-from-learning-data.md)：内置资源与用户学习数据分开管理。
- [ADR 0004](adr/0004-one-vocabulary-record-with-reference-digest-and-notes.md)：同词一份生词记录，保留逻辑引用、微型兜底摘要和独立笔记。
- [ADR 0005](adr/0005-independent-progress-for-vocabulary-card-types.md)：识义卡与听音卡分别保留进度。
- [ADR 0006](adr/0006-learning-days-and-absolute-review-times.md)：保留实际到期时间、当次评分时间上下文与历史，不能按新设置重新解释旧评分。

这里的“单一数据源”指应用是唯一的数据写入入口与业务规则执行者。若多台设备都运行应用，它们仍各自持有本地副本；这个目标不会自动消除设备间冲突。

## 格式比较

以下适用性判断是针对 Lexique 的工程评估；云盘能够保存某格式，不代表能够理解其业务内容。

| 格式 | 主要优点 | 主要缺点 | 建议用途 |
| --- | --- | --- | --- |
| 版本化 JSON | 能保留关联和复杂状态；TypeScript、Rust、未来浏览器端都易解析；文本可检查，较容易迁移到其他工具 | 比二进制格式大；完整文件频繁上传成本高；应用需要定义 schema、升级与冲突处理 | 首选同步交换格式 |
| JSONL / NDJSON | 一行一条记录，适合大量历史和流式处理；可按批次组织增量 | 行序不能充当可靠的多设备顺序；追加、去重、引用关系及压缩仍需协议 | 推荐的多设备快照 / 操作分段格式 |
| ZIP 内含 JSON | 压缩重复字段；可打包清单和附件；整体下载、备份方便 | 打开需解包；修改一部分也常需重新上传整个包；不能靠压缩包实现记录合并 | 可选快照、备份和附件包装 |
| SQLite 快照 | 与当前本地数据库接近；可用官方备份接口生成一致副本 | 不便记录级合并；耦合内部数据库结构；云盘没有通用的数据库业务查看器 | 完整灾备或迁移备份 |
| CSV / XLSX | 便于表格阅读和其他工具处理；云盘预览能力较好 | 难以自然表达卡片关系、嵌套状态和完整历史；表格类型转换与外部编辑增加往返规则 | 由应用生成的查看、交换副本 |
| Markdown | 适合阅读笔记和例句；纯文本容易带走 | 完整卡片状态、引用和复习历史仍需额外元数据约定 | 笔记导出 |

SQLite 的问题是活跃文件的复制和跨设备覆盖方式，不是它不便携。官方明确说明事务期间的后台文件复制可能混合新旧内容；Online Backup API 或 `VACUUM INTO` 可以生成一致副本。因此不建议让云盘客户端直接同步正在运行的用户数据库及其日志文件。[SQLite 损坏情形与安全备份](https://www.sqlite.org/howtocorrupt.html)、[Online Backup API](https://www.sqlite.org/backup.html)

## 大量数据与 CSV / XLSX

JSON 没有固定行数上限，JSONL 也不以几万行作为格式容量限制；限制来自总字节、单条大小、内存、解析器与上传成本。以平均每条 1 KiB 为例，10 万条约 97.7 MiB、100 万条约 976.6 MiB，属于体积估算，不能推断实际同步耗时。完整 JSON 一次解析 / 重写会随总量增长；JSONL 允许流式校验和只传新分段，日常读写仍通过本地数据库索引完成。[JSON Lines 约定](https://jsonlines.org/)

直接用 CSV 存储平面数据可行，但完整学习库需要额外定义跨表引用、版本、删除、历史及类型；换成 CSV 不会解决同步冲突。XLSX 有工作簿重写、表格类型变换和平台约束：Excel 单表最多 1,048,576 行、单格 32,767 字符，数字精度为 15 位。[Excel 官方限制](https://support.microsoft.com/en-us/office/excel-specifications-and-limits-1672b34d-7043-467e-8e27-269d656771c3)。因此建议两者作查看 / 交换导出，并把 ID 写成文本；可导入不等于作为自动回写源。

分段大小、快照重放阈值、压缩与历史分区应按真实数据量测量；哈希分桶只改变寻址 / 目录分布，不减少数据量。[综合设计](sync-design.md)还区分传输日志清理与业务历史保留，避免通过“压缩”误删已确认学习。

## 对云存储服务的兼容性

| 服务 | 格式兼容性与接入方式 | 数据位置及预览差异 |
| --- | --- | --- |
| Google Drive | 官方允许保存任意文件类型；Drive API 可上传、更新普通文件。JSON、JSONL、ZIP、SQLite 均可作为文件传输，无需转换为 Google Sheets | 普通文件夹可见；`appDataFolder` 对用户和其他 Drive 应用隐藏，需 `drive.appdata`，不支持通过 Drive UI 查看其中内容 |
| OneDrive | Microsoft Graph 可创建、替换文件内容；同一份 JSON 或 ZIP 不需改成 Excel 格式 | `approot` 通常位于 `Apps/{应用名}`，用户仍可增删文件；官方指南说明支持家庭及工作/学校 OneDrive。权限与实际租户条件须在接入时验证 |
| Dropbox、WebDAV、S3 等 | 对提供普通文件或对象上传、下载能力的服务，同一数据格式原则上可复用 | 登录、路径或对象标识、变更检测、版本条件和限制由各服务适配器处理；本次未逐个验证这些服务 |

依据：[Drive 文件类型](https://support.google.com/drive/answer/37603?hl=en)、[Drive 上传接口](https://developers.google.com/workspace/drive/api/guides/manage-uploads)、[Drive 应用数据目录](https://developers.google.com/workspace/drive/api/guides/appdata)、[Graph 文件内容上传](https://learn.microsoft.com/en-us/graph/api/driveitem-put-content?view=graph-rest-1.0)、[OneDrive 应用目录](https://learn.microsoft.com/en-us/graph/onedrive-sharepoint-appfolder)。

建议初期优先使用清楚命名的可见专用文件夹，便于用户取回文件和更换云服务。应用只读取自己定义的同步文件并校验内容。若选择 Google 隐藏目录，则可读导出必须另存到用户可见的位置；Google 的隐藏目录不能作为跨云共同前提。OneDrive 应用目录也不能被视作防止用户删除文件的安全边界。

通用的是**文件内容**，不是供应商 API。增加 OneDrive 仍需要独立的授权、网络与文件操作适配器，但不需要再维护另一套表格字段映射。继续沿用[架构文档](architecture.md)中的网络、本地存储注入边界；本次不提前创建空模块。

## 云盘直接查看的局限与补偿

不能一概断言 JSON 或 ZIP 在云盘中完全无法打开：OneDrive 官方预览列表包含 JSON、Markdown、ZIP、CSV、XLSX 和 PDF；Google Drive 的常见预览列表包含文本、压缩包、XLSX 和 PDF，但没有对 JSON 提供同等明确的列举保证。文本或压缩包预览也不会呈现 Lexique 的卡片状态与复习操作。[OneDrive 预览列表](https://support.microsoft.com/en-gb/onedrive/file-types-supported-for-previewing-files-in-onedrive-sharepoint-and-teams?nochrome=true)、[Drive 预览列表](https://support.google.com/drive/answer/37603?hl=en)

如果用户需要离开应用阅读，建议由应用按需生成 PDF；需要表格筛选时再提供 CSV 或 XLSX。查看副本显示生成时间，所有修改仍在应用完成，副本不参与自动同步回写。“只读”是数据流程约定，不表示云盘文件不能被用户修改；导出副本也不保证包含可恢复的全部学习状态。

如果未来增加端到端加密，应在统一的数据封装层处理。加密同步文件不能依靠云盘普通预览查看；额外生成明文 PDF 会改变其内容的隐私边界，不能暗中自动导出。JSON 本身不是加密格式。

## 格式之外必须明确的同步规则

无论选择哪种文件格式，真正同步至少需要以下规则：

1. 明确数据集身份、格式版本、稳定的跨设备记录/卡片/操作身份和关系。当前本地整数 ID 不能直接被假定为全局唯一；可设计稳定同步身份与本地 ID 的映射，不为同步重建卡片。
2. 同步学习内容、两类卡片的独立进度、完整评分请求和结果、撤销信息及相关设置。同步范围仍需逐项确认；不把 OAuth 凭据、日志、词典包或模型混入学习数据。
3. 保存共同基线与修订关系。不同设备同时修改同一笔记，或同时评分同一卡片时，应保留分歧并在应用内裁决；不能只按文件修改时间覆盖整本数据。复习历史按稳定操作身份去重；保留事件不等于能自动合并其排程结果。
4. 保留删除标记及删除与编辑的冲突；清理标记前须有防止长期离线设备复活数据的规则。远程文件丢失或损坏不能直接解释为用户删除全部记录。
5. 在支持的服务上使用条件写入或修订检查，并验证其并发语义。不把“先读后写”当作原子操作。较通用的备选是上传独立、不可变的版本文件，保留并发现分叉；这仍需要应用对合并及完整发布制定规则。
6. 下载后先校验版本、大小、字段和引用，再在事务中应用；未完成上传、未知更高版本或校验失败保持可见且不覆盖本地数据。ZIP 还需限制解压体积与路径。

如果当前只需要单设备自动备份和恢复，版本化 JSON 快照加保留历史版本即可作为较小的首步。如果目标包含电脑和手机离线编辑后的自动会合，首版就需要上述同步语义；不应等实现完整文件覆盖之后再补。

## 已有项目参照与验证边界

[Joplin 同步规格](https://joplinapp.org/help/dev/spec/sync/)提供相关的成熟设计参照：本地优先保存，通用同步器处理项目上传、下载和删除，服务驱动实现文件创建、更新、删除和列举；它也展示了 S3 等不同服务如何位于适配层。这支持将数据格式与云存储操作解耦的设计方向，不证明 Lexique 已具备该能力。

该[GitHub 项目](https://github.com/laurent22/joplin)的[发行页](https://github.com/laurent22/joplin/releases)在本次查阅时有 2026-10-28 的 v3.7.22 发行，说明仍在维护。其[根目录许可](https://raw.githubusercontent.com/laurent22/joplin/dev/LICENSE)默认是 AGPL-3.0-or-later，子目录可能另有许可；这里参考架构和已公开陷阱，不复制其代码进入 MIT 项目。

初次格式评估核对现行仓库产品、领域词汇和相关 ADR，并通过 Context7 与一手网页核对 Google Drive、Microsoft Graph、预览格式、SQLite 备份和 Joplin 设计；当时仅新增本文件。随后按本次综合评估要求新增[同步设计](sync-design.md)，更新产品、架构、M4 / M6 规划与准入门槛，并修订相关工单的同步范围。这里的兼容性是文档层面的可行性，不能用来宣称同步或发行已经就绪。

初次格式评估的文档检查记录（后续综合更新另行检查）：

- PowerShell Markdown 链接检查：通过，文内相对文件链接均存在，外部 URL 语法有效；关键外部资料在研究时已实际读取。
- `git diff --no-index --check -- /dev/null docs/sync-format-evaluation.md`：无空白问题诊断；新文件与空文件存在差异，no-index 模式返回 1。另用 PowerShell 检查行尾空白和文件末尾空行。
- `git status --short` 与既有文件差异检查：仅新增本评估，既有产品及 #24 文档修改保留。
- 应用编译、账号授权、云端写入和多设备冲突测试：未运行；本次仅评估文档，没有实现变更。

综合设计更新的验证记录（2026-11-02）：

- `gh issue list --state all --limit 200 --json number,title,body,labels,state,url`：找到已有总规格 #1 的 M4 / M6 同步范围，更新该票，不另建重复票。
- `git diff --check -- README.md docs/product.md docs/roadmap.md docs/development-readiness.md docs/architecture.md docs/deferred-lookup-requirements.md`：通过。两份新同步文档各运行 `git diff --no-index --check -- /dev/null <文档>`，无空白问题诊断；返回 1 表示新文件差异。
- PowerShell 文档检查：8 份文档的 UTF-8 / 行尾、75 个相对链接、2 个本地标题锚点及 71 个外部 URL 语法通过。关键外部资料已在研究阶段实际读取；此检查不等于对所有外部链接进行运行时可用性测试。
- `gh issue edit 1 --body-file <UTF-8 临时正文>` 后使用 `gh issue view 1 --json number,title,body,labels,updatedAt,url` 回读：正文与准备稿一致，原正文完整保留，标题及 `ready-for-agent` 标签未变。发布更新时间为 `2026-11-02T18:43:36Z`。
- scoped diff 与工作区核对：已有产品 CPU / 音频政策及 #24 研究修改保留。
- 应用 / Rust / 浏览器运行、真实云账号授权、传输、冲突、持久性和 GC 测试：未运行；本轮只交付设计、规划和工单更新，未修改运行验收声明或学习数据。

文档验证（2026-11-05）：

- `git diff --cached --check`：通过。
- `pnpm test:docs`：97 份项目 Markdown、595 个本地链接通过；67 个本机证据链接按检查器规则跳过，无问题。
- 同步设计与总规格 #1、现有领域规则及架构边界一致；具体协议仍待 M4-S0 冻结。
- 应用、Rust、浏览器及真实云服务测试未运行；本次仅提交规划文档，不扩大运行或发行验收结论。
