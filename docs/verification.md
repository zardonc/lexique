# 骨架验证记录

2026-11-07 #38 收尾：#39 的六项准入条件已逐项核对并关闭，#39–#49 的 Windows x64 变位范围完成验收。56 条用户故事与来源、完整生成、紧凑存储、显示偏好、语音、导航和学习数据保持分别对应既有公开 CLI／实际原生证据，见 [#38 收尾报告](evidence/issue-38/README.md)。#49 用户断网通过及其已验收原包保持不变；英语为部分覆盖，其他 M3 模块、其他平台实机和 M5 正式发行继续单独验收。

本次重新运行既有元数据公开 CLI 个案，八个独立期待词元通过（46.340 秒）；core／storage／dictionary 的 locked 全目标 strict Clippy 通过。后端 CI 准备路径与当前 `target/desktop-conjugation-v6-stats/`、原生构建、打包及测试夹具一致。构建器使用事务内回滚页计量，支持没有可选 SQLite `dbstat` 表的环境；最终资源格式／数据和原生查询逻辑不变，原生成物与用户已验收 ZIP 保留。完整执行与审查结果见 #38 收尾。

2026-11-07 #49 验收与 CLI 延迟：用户已报告所准备的 Windows x64 完整语音包断网验收通过，已验收 ZIP 和原交付目录保持不变；机器／WebView2 版本、禁网方法和逐项日志未提供，按用户报告归档，见 [验收结果](evidence/issue-49/offline-acceptance.md)。此前原生综合证据继续覆盖同一包的完整表达生成／播放、着色／排序恢复、双向导航及学习数据保持。

CLI 校验在既有单次加载缓存内复用已排序形态槽，每条释义解析一次，保留全部来源／数据库／释义验证。句法排序由 19,782 次减至 125 次；新准备目录 `target/desktop-conjugation-v6-cli/` 与已验收包的 11,005,952 字节格式 6 数据库逐字节相同，旧目录保留。原基线首进程 3,871.2 ms、暖中位 1,206.2 ms；最终六进程首项 1,100.1 ms、暖中位 936.2 ms、暖最大 1,096.7 ms，全部通过原有 1,700 ms 预算。预算工具在任一项超限时返回失败并保存全部样本；未清除 OS 缓存。

36 项公开 CLI、143 项单独预编译的后端、Rust 格式／locked workspace 编译、生产前端及匹配资源的 Windows x64 reader 构建、2 项构建参数和 8 项便携包检查通过。正面 TTS 打包及失败输入共 3 项无跳过，使用短任务 TEMP；较深临时路径触发 Windows 路径长度限制的结果单独保留。原浏览器／原生、lint／typecheck 和独立词典证据保留，本次未重跑这些未修改的行为。命令、样本及边界见 [CLI 延迟验证](evidence/issue-49/cli-latency.md)；其他平台、缺 WebView2 首装机器及 M5 正式发行仍未验收。

2026-11-07 #49：已准备匹配格式 6 的 Windows x64 重建 reader 和完整已准入 TTS 资源，ZIP 为 177,115,811 字节、1,119 个文件逐项校验，不含学习数据；交付原目录和 ZIP 未运行，原生检查使用独立解压副本。包内八份 MIT 转换器、LGPL-LR 词尾规则及原始许可／来源可在独立工作区重建逐字节相同的 11,005,952 字节数据库。170 个模型、24,307 个词干引用、1,451,539 格、10,781 个身份及全资源 23 区块覆盖已核对；英文覆盖独立为 1,493 个可用身份、9,288 个暂缺身份和 2,089 个义项。#42 和 #43 已先核对原实施证据、更新并关闭。

lint／typecheck／生产前端和 Windows reader 构建、Rust 格式／locked workspace 编译、独立预编译后的 143 项后端及 3 项桌面库、36 项变位 CLI、7 项词典资源、146 项浏览器及 2＋8＋3 项工具检查通过，正面 TTS 打包无跳过。独立解压资源另通过七项既有公开 CLI 断言；真实 Windows 新数据首次启动、八项完整表达备选生成实播、男声慢速／正常复用、导航／未保存修改／资源恢复、三模式／重排／重置重启及选定音频的识义／听音预览通过。六份既有合成记录、十二张卡片与进度及七份音频保留；导航的一份额外明确保存记录只修改预期笔记和修订。新数据浏览和播放不收藏或建卡，后端及原生执行保持 60 秒硬限。

首次启动到原生 IPC 可用为 1,068.3 ms；原生查询首项 52.4 ms、暖中位 53.2 ms。独立六次 CLI 进程首项 1,790.1 ms、暖中位 1,377.7 ms、暖最大 2,811.9 ms，首项和最大项超过 1,700 ms 证据预算；未清除 OS 缓存。Windows 11 Home 10.0.26300／WebView2 157.0.4341.42 的两个原生 WebView2 访问异常经转储定位至 RTSSHooks64.dll＋0x1490af，既有 RTSS 兼容性限制未解决。完整包、命令、结果及限制见 [#49 准备证据](evidence/issue-49/README.md)；[新包断网验收](evidence/issue-49/offline-acceptance.md)仍待用户执行，#49 保持开放，其他平台、缺运行依赖机器及 M5 正式发行未验收。

2026-11-06 #48：每个显示的完整变位表达和备选均可独立播放，主语、反身代词及动词部分按原文送入现有离线语音入口，不截断或发送斜杠并列。沿用 CPU 主管、女／男声和正常／0.6 倍慢速、临时内容绝对期限及真实错误；机器语音与模型标识可见，法语表达和界面控制分别使用正确语言。选择另一表达、切换词元、重新查询或离开页面释放音频并忽略迟到响应，保留已提交共享合成及并发词典播放的既有生命周期；着色和排序不变。

13 项逐表达浏览器检查和既有单词播放回归通过；完整 146 项浏览器套件、lint／typecheck／生产前端及匹配格式 6 的 Windows x64 reader 按 [#48 证据](evidence/issue-48/README.md)记录。实际 Windows 生成并解码／播放拼写、性数及代词式备选，选择上项之后播放另一表达会停止旧音频，男声同一音频在慢速／正常播放复用；语音资源缺失时可见失败，恢复并正常重启后同项成功。检查 1,458 个显示表达均在 200 字符输入边界内，11 个临时版本自生成完成后保留七天。六份既有合成记录、来源与笔记、十二张卡片及进度、选定当前／上一版音频、原语音偏好、着色／排序及七份既有文件不变。执行保留 60 秒硬限，没有新增后端、资源、IPC 或 schema；不重跑未修改的后端／资源／CLI／语音包套件。组合新包断网验收仍为 #49，其他平台、额外人工音质准入及正式发行未验证。

2026-11-06 #47：明确动词词条与专用变位入口共用同一表，保留原查询、所选普通／代词式身份及其他候选；返回要求对应词典动词，缺项／歧义不代选。既有精确词典查询、标准词头及独立笔记规则不变，直／弯撇号候选保留各自词条和学习记录身份。所有入口沿用未保存修改保护，键盘焦点到达对应标题，三语和主题可用，#45 着色与 #46 排序继续独立恢复。

15 项导航浏览器检查、完整 133 项浏览器、lint、typecheck、生产前端及匹配格式 6 的 Windows x64 release 构建通过。实际 Windows 导航／缺项／歧义、资源缺失和损坏后的恢复、保存失败／放弃／保存后离开、原生重复及冲突结果、正常关闭和重启通过，执行保留 60 秒硬限。六份既有合成记录、十二张卡片及进度、来源、语音偏好、选定音频和七份音频文件不变；额外明确保存的一份非复习动词记录只在明确编辑保存时改变笔记及修订。完整命令和结果见 [#47 证据](evidence/issue-47/README.md)。本轮未修改后端、资源／语言规则或打包器，不重跑这些单元／CLI／语音包套件。组合新包断网验收仍为 #49，变位逐形式播放仍为 #48，其他平台及正式发行未验证。

2026-11-06 #46：全部 23 区块可通过选择框及键盘可用的前移／后移按钮重排，跨动词保留，缺陷动词的不适用区块仍展示。重置恢复约定的七个重点时态及其余区块的准确默认顺序；重点身份、全部匹配和 #45 着色保持独立。顺序使用现有 `app_metadata` 的独立 JSON 键，严格拒绝缺项、重复、未知及错误类型，schema 17 不变，不迁移学习数据。读写、损坏及不可用设置明确报错并可重读恢复。

118 项浏览器、143 项预编译后端、lint／typecheck／生产前端及 Windows release 构建／Rust 格式／locked 编译／严格 storage Clippy 全部通过。正面 TTS 打包与既有工具检查 2＋8＋3 项无跳过，使用匹配格式 6 的最终重建 reader。实际 Windows 验证自定义顺序在三个正常重启中随三种独立着色设置保留，重置后的默认顺序经第四次正常重启恢复；六份合成记录、十二张卡片和进度、语音偏好及七份音频不变。Obscura 预览确认排序可见且明确不保存。本轮资源／语言规则／CLI 未修改，不重跑完整 CLI 套件；完整命令、原生结果及限制见 [#46 证据](evidence/issue-46/README.md)。组合新包断网验收仍为 #49，其他平台及正式发行未重测。

2026-11-06 #45：三种直接着色模式默认选词尾，`nous avions lavé` 分别标出 `avions lavé`／`lavé`／`é`，`lavais` 标出 `ais`。Python／Rust 使用独立语言规则，不把存储 cut／suffix 当词尾；全资源未解决的规则项为零，95,658 个第三组单元格的词尾全部为空。原始源、性数／拼写备选和七个重点／匹配标记保留，斜体提供独立非颜色线索。模式存入现有 `app_metadata`，schema 17 不变；读取、写入、损坏及不可用偏好存储的错误明确显示并可恢复。

36 项公开 CLI、143 项预编译后端、114 项浏览器、3 项桌面库及 lint／typecheck／构建／Rust 格式／locked 编译／严格 dictionary 和 storage Clippy 全部通过。匹配最终重建 reader 的正面 TTS 打包及既有工具检查无跳过。实际 Windows 验证全部三种模式经正常重启恢复，六份合成记录、十二张卡片和进度、语音偏好及七份音频字节不变。格式 6 新准备目录为 `target/desktop-conjugation-v6-color/`，11,005,952 字节（10.50 MiB），旧目录保留。六次新 CLI 进程首次 1,436.5 ms、后五次中位 1,381.9 ms，均低于 1,700 ms 证据预算；原生 IPC 首次 26.5 ms、暖中位 28.4 ms，未清除 OS 缓存。#41 已核对证据、更新并关闭；规则、完整命令、原生结果与限制见 [#45 证据](evidence/issue-45/README.md)。本轮未施加禁网、未重测其他平台或正式发行；组合新包断网验收仍由 #49 承担。

2026-11-06 #44：公开 CLI、原生查询与前端匹配均接受 NFC 等价编码、大小写、直／弯撇号及 Unicode 空白差异，保留重音、完整表达、源拼写、全部身份和稳定解释顺序。格式 6 新准备目录为 `target/desktop-conjugation-v6-query/`，数据库与已验证存储基线逐字节相同；旧资源保留。34 项资源 CLI、143 项预编译后端、107 项浏览器及 lint／typecheck／Rust 格式／locked 编译／严格 dictionary Clippy／Windows release 构建通过。工具检查 2＋8＋3 项全部通过，包含匹配重建 reader 的正面 TTS 打包检查。Windows 原生查询、逐表达高亮、输入保留、无结果／非法输入／资源错误及正常重启通过，六份合成记录、十二张卡片和七份音频保持不变。原生 IPC 首次 23.1 ms、后五次暖中位 21.3 ms，无 OS 缓存清除。#40 已核对全部验收证据并关闭；完整命令与实际结果见 [#44 证据](evidence/issue-44/README.md)。本轮未施加禁网、未重测其他平台或正式发行；#49 仍负责组合新包离线验收。

2026-11-06 变位存储：格式 6 使用 170 个共享简单变位模型及词干反查，运行时构造复合／代词式表达。新构建的全部 1,451,539 格、10,781 个身份及元数据与保留的格式 5 原物在 Python 和原生运行时等价；数据库由 239.04 MiB 缩为 10.46 MiB。31 项公开 CLI、142 项后端及 Windows 原生查询／重启／数据保留检查通过；公开预编译入口及 backend CI 均先准备两种资源，后端执行仍保持 60 秒硬限。

新的 Windows x64 TTS 开发包由对应格式 6 的重建 reader 生成，压缩包 177,043,746 字节、1,117 文件逐项通过校验，不含学习数据。三项语音打包检查包含正面资源验收，真实原生六词生成／解码／播放与正常退出／重启恢复通过；变位浏览保持六份合成记录、十二张卡片及七份音频完全不变。原生 IPC 首次 33.8 ms、暖中位 33.3 ms；CLI 首进程 1730.9 ms 超过既有 1,700 ms 预算，后五次暖中位 1457.5 ms。一次原生语音执行出现 WebView2 访问异常，另一个新目录执行通过；本轮原生脚本未施加禁网，用户已报告该新包离线测试通过，未提供测试机器／版本及逐项观察。浏览器整套、本轮其他 OS／架构及正式发行未重跑。完整命令、reader／资源身份和限制见[存储验证](evidence/conjugation-storage/README.md)。

2026-11-05 #40：桌面变位页已通过现有运行时适配器读取真实 Lefff / Grammalecte 资源，支持简单词形精确反查、全部候选选择、23 个独立区块、七个重点时态排序及全部匹配格的非颜色标识。104 项浏览器测试、资源 CLI、Rust 与隔离打包检查通过；真实 Windows x64 检查验证缺失／损坏／操作失败、正常重启和已有记录／两张卡／选定音频保留。原始异常词形、缺少形式与未验证构造仍明确标注未确认。完整 M3 生成、变位朗读、全套禁网包及其他平台仍待对应工单；详见 [#40 证据](evidence/issue-40/README.md)。

2026-11-05 #39：独立变位 CLI 已接入固定 Grammalecte 2.3.0 的传统组别、普通助动词与用法元数据，保留原始标记、纠正依据及未知／不适用状态。7,826 个 Lefff 词元和 397,413 格不变；传统组别 6,573 项获支持、1,253 项未确认，普通助动词 1,310 项获支持、6,361 项未确认、155 项不适用。格式 2 的许可、源材料、覆盖、真实输出、查询计量及失败／重建检查见 [#39 证据](evidence/issue-39/README.md)。桌面接入与完整 M3 生成准入仍待后续，不以本次元数据验证宣称完成。

2026-11-05 #4：固定 Lefff 3.4 原始形态数据、双 SHA-256 与 LGPL-LR，独立离线 CLI 已验证 `sommes` → sommer / être、全部匹配格标记、代表性源表及损坏／缺失失败路径。342,771 源动词记录转换为 397,413 格，SQLite 33,095,680 bytes，反查索引 7,942,144 bytes。语式／时态矩阵、未确认的复合／代词式和原始异常项见 [#4 证据](evidence/issue-4/README.md)。此结果不代表桌面变位已接入或 M3 完整范围获批。

2026-11-05 M2 合并前检查：`cargo clippy --locked -p lexique-core -p lexique-storage -p lexique-dictionary --all-targets -- -D warnings`、`pnpm rust:fmt` 和 `pnpm rust:check` 通过。音频发布边界将 hash、字节数及生成时间归入 `AudioPublicationContent`，全部调用点保持原值；既有事务、选择、验证及错误规则不变。`pnpm rust:test:build` 后 `pnpm test:rust` 138 项通过，执行保持硬 60 秒限制。`git merge-tree --write-tree origin/main HEAD` 无冲突；本轮未重新执行干净系统离线或其他平台实机验收。

2026-11-05 CI 检查补齐：Windows portable job 使用 `cargo test --locked --release -p lexique-desktop --test speech_cache --lib --no-run` 预编译后分别运行缓存和桌面库测试，各保持 60 秒硬限。本机 `pnpm test:desktop:lib` 3 项、`pnpm test:speech:cache` 10 项通过；崩溃子进程 fixture 仍为显式 ignored。`pnpm rust:test:build` 后 `pnpm test:rust` 138 项通过。独立文档 job 运行 `pnpm test:docs:validator` 和 `pnpm test:docs`：13 项隔离 Git 回归、95 份项目 Markdown 通过，覆盖格式、跟踪文件链接及本机证据分类；#24 设计验证的 50 用例／12 优先回归／R、V、T、F 追踪仍通过。`node --check scripts/test-rust.mjs` 与 `git diff --check` 通过。GitHub Actions 运行结果待推送后确认；产品验收沿用 #24 已登记的执行身份。

2026-11-05 #24 最终复核：以 `c1acaeb` 比较，Standards／Spec 均通过且剩余 findings 为零。lint／typecheck／build、Rust 格式／locked 编译、138 项后端、101 项浏览器、10 项 Windows 缓存及 3 项桌面库通过；74 固定＋4,096 原生／实际 WASM 调度输入零差异。固定生产二进制的十种不支持持久规格拒绝、合法合成、选定内容／预算重启保留、六词真实生成实播及原规格修复均通过；10k 健康热窗口 100 次无失败，p95 83.5ms，结构及增量预算保持。命令、二进制身份及原始结果见[最终复核](evidence/issue-24/final-review-2026-11-05.md)。新二进制未重做干净系统离线、九断点及长时／其它平台检查；此前用户离线报告与专项按各自执行身份保留。

2026-11-05 #24：Windows x64本期实现与验收完成，用户确认干净系统离线测试通过。schema17有界窗口／delta、选定音频／后继执行、共享保护／维护、一致SQLite快照及批量初始位置分配均有实际验证；共享核心使用IANA2026f。138项后端、100项浏览器、10项Windows缓存、74固定调度向量与4096原生／实际WASM校准通过；独立识义／音频参考、九个进程kill断点、原生启动／执行超时恢复通过。最终健康10k热窗口p95为92.7ms；50k词／100k卡／100k历史的100次评分逐次仅加载1行metadata／解析1项progress、0重建。资源增长、性能分片、二进制身份和便携测试交付见[实施记录](evidence/issue-24/continuation-2026-11-04.md)与[82项覆盖](evidence/issue-24/acceptance-matrix-2026-11-05.md)。干净系统结果依据用户报告；小时级／物理故障、其它硬件／平台及正式发行边界集中记录。

2026-11-04 #24 队列快照与位置边界：真实双连接回归先复现音频可用性检查期间阻止写者提交，现短快照先释放、必要位置事务单独提交，并按 `data_version` 最多重读三次；近 i64 上限的 UNIQUE 失败先复现，现安全重编号且提交失败完整回滚。桌面音频候选改为批读，文件 worker 不再逐词／资产查询。101 项后端、2 项原生 desktop lib、87 项浏览器测试及 lint／typecheck／build／rustfmt／locked check 通过；重建固定二进制的双额度零 21 次与万词 21 次 native IPC 检查通过。仍是 #26 部分交付，不代表 #24 完成；版本／窗口／保留和完整验收仍待后续票。命令、原始路径、hash 和限制见[本轮诊断记录](evidence/issue-24/queue-diagnosis-2026-11-04.md)。

2026-11-04 #24 桌面检查进度批读：音频检查 worker 改用每批至多100个卡片ID的 LEFT JOIN 读取进度，移除逐卡 `review_progress` 调用。真实 SQLite SQL-count 回归先在相同逐卡实现上得到2次单卡查询失败，再由批读通过；空progress、输入顺序、跨批重复ID、未知卡与损坏JSON检查通过。预编译后97项Rust测试、2项Windows desktop lib测试、locked workspace check、rustfmt及diff check通过。desktop检查覆盖真实文件、缺失／腐坏、未来到期、跳过和另一连接持写锁；fixture不是实播音频。仍有逐词key／asset查询、全候选文件扫描及跨阶段快照缺口，不宣称#26或#24完成；未运行新原生应用验收／性能矩阵，RTSS兼容性排除。

2026-11-04 #24 legacy 登记无变化快路径：真实 SQLite trace 回归证明相同 hash 批次改前仍执行 1 次 BEGIN，改后为 0；先按至多 100 个身份批量读登记，仅有变化时进入原立即事务并重新检查。替换提交失败保留旧 hash、成功更新及非法 hash 拒绝均通过。预编译后 96 项 Rust 测试、locked workspace check、rustfmt 及 diff check 通过。此改动减少热刷新写者争用，但未将 legacy 迁移移出队列路径，未实现选定版本、有界文件窗口或新的保留协议，也未作新原生验收；#24 仍未完成，RTSS兼容性排除。

2026-11-04 #24 存储队列锁竞争修复：新增真实文件 SQLite 回归，另一连接持有 `BEGIN IMMEDIATE` 时，已有当天位置的稳定队列刷新改前报 `database is locked`，改后成功且顺序一致。队列事务由 Immediate 改为 Deferred，稳定读不预占写者；首次位置／日切仍在同一事务写入，竞争升级失败保持可见。预编译后 96 项 Rust 测试、locked workspace check、rustfmt 和 diff check 通过；未运行新原生验收。桌面 legacy 登记仍写库，完整短快照／条件修订、有界窗口及版本保留协议仍待实现，不将本次修复计为 #24 完成。RTSS 兼容性继续排除。

2026-11-04 #24 队列查询部分优化：存储队列以一次 LEFT JOIN 读取卡片及 progress，并复用已读取的 position map；新卡缺 progress 仍按 New 处理。真实 SQLite 热队列 trace 回归在两卡输入上改前分别观察到 2 次逐卡 progress 查询及 2 次逐卡 position 查询，改后均为 0；`cargo fmt --all --check`、`cargo check --locked --workspace`、`pnpm rust:test:build` 和 `pnpm test:rust`（95 项）通过。桌面音频检查仍逐卡／词读取并扫描文件，热刷新仍取写事务；窗口、选定版本及保留协议未完成，未作原生性能或完整 #24 验收。用户决定 RTSS 兼容性不纳入 #24 验收，不再检查，历史异常不因此成为已修复。

2026-11-04 #24 双额度零修复：原生隔离 fixture 的 `native-s0-baseline.mjs ... gap-red` 改前 21 次均仅返回识义卡 `[1]`（exit 1），重建后 21 次均返回 `[1,2]`（exit 0）。原因是桌面层在两项额度为零时跳过听音校验，现交由存储层按日内学习资格裁决；校验仍使用既有全量路径。`pnpm rust:fmt`、`pnpm rust:check`、`pnpm desktop:build --no-bundle`、`pnpm rust:test:build` 和 `pnpm test:rust`（94 项）通过。此 fixture 使用仅含 Ogg 头的合成文件，不证明解码或播放；有界窗口、批量快照、选定音频版本及保留协议仍未完成，#24 不能据此关闭。

2026-11-04 #25 审查修复及最终复核：补齐固定基点的源码补丁与 16 文件校验清单，隔离 checkout 的 `--build-only` 通过，九个观察源码 hash 与历史一致；资源 CLI／实际合成入口的新目录保护和 60 秒子进程边界通过。最终独立 Standards 通过（0 项规范违反、1 项 P3 维护建议），Spec 通过（0 项可执行发现），S0 可闭票。干净修复快照重新通过 20 项 S0 回归和语法／格式检查；执行及冻结输入 hash 一致后，核对并复用此前冻结源码的 94 项后端测试、三组规模各 100 次热请求、100 次固定操作、六任务发布阶段及旧／新布局各 145 份完整解码证据。模型 2／4 后台相对延迟仍超过冻结预算，完整历史矩阵、50k 执行、其他平台及正式发行未重验；命令、原始失败及限制见 [审查修复记录](evidence/issue-25/fix-review.md)和[结果](evidence/issue-25/fix-results.json)。

2026-11-03 #25 S0：新增隔离观察程序、独立 raw-state oracle、竞争控制及冻结预算，产品行为未改。当前双额度零漏听音卡和热刷新写事务已复现；用户关闭 RTSS／切性能模式后重测万词原生 100 次热请求，p95 895 ms，未来 100 ms 目标未达。模型 2 后台＋1 交互的三次对照通过冻结预算，实际笔记／评分／缓存相对退化仍未通过。当前 Actor／resident／安全发布分段已补测；完整边界、命令、审查及后续负责票见 [S0 证据](evidence/issue-25/README.md)。这不代表 #24 新协议、其他平台、禁网新包或正式发行已验收。

2026-11-03 #24 万词取卡与 WebView2 跟进：最终固定 release 在 RTSS 关闭时，一万词全到期三次为 1,684.9／768.3／691.7 ms，通过本轮 2,000 ms 预算；同条件起始 HEAD 旧版为 15,821.2／7,478.0／4,561.1 ms，超标。采用有界并行完整音频检查及缓冲预分配，并修复满语音积压回滚建卡和腐坏缓存终止 supervisor。多个隔离转储把 WebView2 访问异常定位到 RTSSHooks64.dll + 0x1490af；环境变量及 bootstrap 绕过失败且已移除，RTSS 开启兼容性未解决。最终 94 后端、8 安全缓存、2 desktop 单元、87 浏览器及4原生清理检查通过；固定候选的15次启动、三轮真实合成／解码／播放和生命周期／迁移／定向重试均通过，测试后恢复 RTSS 且无任务进程残留。独立最终 Standards／Spec 审查受模型速率限制未完成，不能继承历史审查通过；其他硬件／平台、64 KiB／历史混合、长时及新包断网仍未测。详见[跟进报告](evidence/issue-24/followup-results.md)及[精简结果](evidence/issue-24/followup-results.json)。下方记录保留为历史证据。

2026-11-02 #24 桌面实施：V01–V05 开发前验证后，接入自动队列、分页批次、持久有界 CPU 任务及语义音频缓存／归属／GC 恢复。审查后补上当前卡播放失败重新生成、就绪音频重新核验、单词定向重试和单项建卡错误传播。最终 87 项浏览器回归、93 项后端测试及 Windows 安全缓存检查通过；新 release 的真实合成／完整解码／播放、取消／杀 worker／退出／重启和旧音频迁移证据分别记录。新二进制没有继承旧包断网／干净系统结论；长时休眠、其他平台和未来 F01–F05 保留未测／deferred。具体命令、失败修正和逐项边界见 [本轮报告](evidence/issue-24/desktop-results.md)及 [开发前结果](evidence/issue-24/preflight-results.md)。

2026-11-01 #24 性能测量与方案：使用新建合成SQLite及Windows缓存目录，对100/1,000/10,000生词、缓存有效/缺失/损坏、到期/额度/暂停/跳过作队列对照，并补测批量建卡和真实TTS模型复用。候选仅在隔离证据程序中执行，产品实现和用户数据未改。lint/typecheck/build、Rust格式/编译、预编译后70项后端测试及72项浏览器回归通过；逐词10,000词建卡触发60秒执行硬限并终止。其他硬件/平台性能未测，资格筛选部分探索时间受并发TTS影响，保留完整读取量及独立目录复用对照。具体推荐、实施步骤、逐次结果与边界见 [#24性能方案](evidence/issue-24/README.md)。

2026-11-01 PR #23 后续验证：用户指定基点 `210f8a3`，独立规范审查发现并发生成检查退化为有效缓存复用，已在并发调用前重新制造坏缓存并补测通过；规格审查无问题，复审后未解决发现为 0。此次前端 lint / typecheck / build、22 项生词本回归、预编译后 70 项后端测试及真实 Windows x64 缓存修复 / 并发生成 / 实播 / 重启恢复通过。同一 `39fa8a5` 的 PR / push CI 共 20 项通过；文件 symlink 因权限跳过，未重跑断网验收及完整平台实机检查。详见 [后续独立验证](evidence/pr-23/README.md#2026-11-01-后续独立验证)。

2026-11-01T18:06:02Z 按用户明确选择关闭 PR #23，未合并；远端分支保留，main 未变。本次测试补强与验证记录尚未提交或推送。

2026-11-01 PR #23 复核：修复批量加入复习被坏语音缓存阻断的问题，复用原生生成命令的校验、有效缓存复用及损坏替换。新增回归先失败后通过；22 项生词本测试、72 项完整浏览器回归、70 项后端测试、重建后 71+4,096 原生 / 实际 WASM 对照与 Windows x64 原生批量修复 / 实播 / 重启检查通过。原生快照确认进度与暂停状态保持；普通权限下执行，未施加网络限制。原 HEAD 的 PR / push CI 各 10 项通过，含四目标编译；实机与正式发行门槛保持。详见 [PR 复核](evidence/pr-23/README.md)。

2026-11-01 #22 收尾：用户确认 macOS 验收留待后续执行，暂不阻塞继续开发；复核受审源码未变，没有发现其他阻塞问题，Windows x64 / 实际 WASM 的 M2 验收完成，按用户授权关闭 #22。macOS 两目标编译及实机仍未测，既有 macOS M1 blocked 清单和 M5 四目标正式发行门槛继续保留。完整验证和用户实测范围见 [收尾报告](evidence/issue-22/review-2026-11-01.md)。

2026-11-01 完整验证 #22：用户本轮确认应用在断网、全新安装的 Windows 中可用且功能正常，已纳入用户实测证据。两个独立代码审查未发现确定的新增缺陷；本轮前后端、71 项浏览器 E2E、70 项后端测试、真实资源打包、实际原生/WASM 的 71+4,096 向量、Windows ARM64 编译，以及重新完成 Tauri 构建后的原生综合/语音/排序均通过。缓存 junction 和四项原生清理检查通过；新包工作进程在禁网下生成音频。新包自动化未限制网络，macOS 两目标编译与其他架构实机仍未测，macOS M1 与 M5 门槛保持；失败尝试、修正、包哈希和完整边界见 [本轮完整报告](evidence/issue-22/review-2026-11-01.md)。下方较早记录保留为历史证据。

用户确认此前 `issue-22-anki-limits` 包在干净 Windows 离线环境通过听音卡评分、失败后重试和移动后队列及额度恢复。原包哈希与体积、用户报告范围和剩余平台限制见 [#22 证据](evidence/issue-22/README.md)。下方保留历史记录，不代表用户实测尚未发生。

后续播放状态修复通过 lint、typecheck、build、严格 Clippy、71 项浏览器 E2E 和新 Windows x64 包的原生综合及语音检查；浏览器使用模拟 IPC，原生检查未限制网络，文件符号链接检查因权限跳过。新包本身尚未在干净系统断网复测，包信息见 [#22 证据](evidence/issue-22/README.md)。

后续错误提示修复后，完整浏览器回归 70 项通过；真实语音资源打包测试、预编译后的 60 秒后端测试、实际 WASM 的 71 + 4,096 向量、Windows ARM64 编译及新包原生隔离检查通过。新包与用户实测包只有主程序不同，新包尚未断网或在干净系统复测；其具体哈希、体积和平台边界见 [#22 证据](evidence/issue-22/README.md)。

闪卡每日限额更新：`pnpm lint`、`pnpm typecheck`、`pnpm build`、`pnpm rust:fmt`、`pnpm rust:check`、`pnpm rust:test:build` 和 `pnpm test:rust` 通过，后端仍使用 60 秒硬超时。存储测试覆盖新卡和复习上限、跨日学习优先、日内重复、开关、失败回滚、重启和撤销。设置页的两项浏览器测试通过；完整浏览器回归 69 项通过、1 项失败：音频失败测试期待模拟错误 `speaker unavailable`，实际界面显示通用“播放失败”，单独重跑仍失败。

Windows x64 完整语音 portable 包通过隔离原生评分、播放、失败重试、设置开关、重启、移动和撤销检查，使用合成数据库副本；本轮未限制网络或在干净系统测试。包为 `target/issue-22-anki-limits/Lexique-0.1.0-windows-x64-portable-speech.zip`，SHA-256 `da10581a345fdefc802b41e6747fd0cb48f3dcbe924c933229a4c0684e8d6f92`。#22 保持开放。

2026-10-31：闪卡队列改为统一直接显示和连续复习；新卡按首次成功评分逐卡计入额度，超额卡片在后续学习日保留。后端完整测试及 Windows x64 最终候选的隔离原生检查通过，包含双卡连续评分、额度和音频失败；未限制网络，也未在干净系统复测。包及验证细节见 [#22 证据](evidence/issue-22/README.md)；该范围不代表已实现 Anki 全部队列行为。

2026-10-31：按用户复测决定取消同词评分后的暂缓；两卡继续独立排程并分别计入额度。后端完整测试通过，包括同日双卡评分及暂停、重启和撤销；Windows x64 新候选原生隔离检查确认识义评分后听音卡出现并播放。该检查未限制网络，干净系统复测仍待完成；候选包及范围见 [#22 证据](evidence/issue-22/README.md)。下文保留此前同词暂缓规则的历史验证。

2026-10-31：词典及生词详情新增单词测试播放按钮。`pnpm lint`、`pnpm typecheck`、`pnpm build`、`pnpm rust:fmt`、`pnpm rust:check`、预编译后的 `pnpm test:rust` 和 `speech-button.spec.ts` 通过；真实 Windows x64 便携包在普通权限下验证了两个页面的合成与播放结束、非法输入拒绝、词典播放不创建生词，以及既有复习回归。第一次原生重跑 WebView 会话在合成中意外关闭，原因未确定；第二次完整通过。新包回读 1,072 文件，ZIP SHA-256 `701016cac4d6a5fef4a3469ed2bf48c6f4f4cf027b1c8439068d45095bd907da`。本轮未验证禁网或干净系统，不改变 #22 状态。

## M2 集成验收（issue #22，未完成）

2026-10-30：Windows x64 原生组合、真实语音及实际 WASM 等检查和包体计量见 [#22 证据](evidence/issue-22/README.md)。后续诊断在禁网 AppContainer 的实际 `speech_generate` 中确认错误发生于缓存硬链接发布。新文件现通过从锁定目录句柄取得的路径执行不覆盖移动；禁网原生诊断构建成功生成并回读 9,555 字节 Ogg，再次调用成功，隔离数据库表内容不变。普通权限七项缓存回归通过，受限环境六项通过、junction 创建项因权限跳过；重解析点、防目录重定位、并发不覆盖和替换失败保留旧音频见 [修复记录](evidence/issue-22/cache-fix.md)。禁网播放全流程、完整离线系统依赖体积、macOS 编译和其他目标运行仍缺证据，完整 M2 尚未通过。

同轮移除诊断标记并重新构建后，普通权限下的原生播放、慢速重播、生成失败保留旧音频、重新生成及重启恢复通过。没有将普通权限实播等同于禁网实播。

## M2 下次复习时间排序（issue #21）

2026-10-30：生词本按已学习卡最早原始到期时间排序，已加入但未学习的记录随后，未加入的记录最后，同组按词头稳定排序。评分和加入复习后刷新列表；排序偏好保存并恢复。来源为 [#9 的 D4](https://github.com/zardonc/lexique/issues/9)。

- `pnpm lint`、`pnpm typecheck`、`pnpm build`、`pnpm rust:fmt`、`pnpm rust:check`、`pnpm rust:test:build` 后 `pnpm test:rust`：通过，后端执行保留 60 秒限制。SQLite 测试覆盖双卡不同到期、同时间、未学习、暂停、搜索及数据库重开。
- `pnpm test:e2e`：Edge 66 项通过，包括评分后列表刷新和新排序选项。首次并行测试期间修改源文件触发热重载，4 项失败；代码稳定后完整重跑通过。
- `pnpm desktop:build --no-bundle` 和 `node apps/desktop/e2e/native-review-sort-check.mjs target/issue-20-sense-20261030 target/release/lexique-desktop.exe target/issue-21-native-20261030-b`：Windows x64 隔离新数据库中两个单词真实评分后按不同到期时间重排，重启恢复通过；测试进程已停止。

2026-10-30 补验：SQLite 测试通过公开评分接口生成隔离数据库，两个单词的识义卡同一时间到期；其中一个单词的听音卡较晚到期，另一个已学习单词的到期时间位于两者之间。暂停双卡并将两项额度设为 0 后，原生窗口仍按 `avion`、`zèbre`、`bonjour` 排序，未学习和未加入记录随后；搜索筛出两个同时间记录后和重启后顺序一致。先以 `LEXIQUE_NATIVE_REVIEW_SORT_FIXTURE` 指向不存在的新文件运行 `cargo test --locked -p lexique-storage --test vocabulary review_due_sort_uses_earliest_card_and_survives_reopen_and_pause -- --exact`，再运行 `node apps/desktop/e2e/native-review-sort-check.mjs target/issue-20-sense-20261030 target/release/lexique-desktop.exe target/issue-21-native-fixture-20261030-c target/issue-21-review-sort-fixture-20261030-c.sqlite3`。脚本读取两卡真实进度并断言到期关系，测试进程已停止，未使用用户数据。

其他平台留待后续，不阻塞本票；撤销、暂停及批量选择的最终组合由 #22 补验。浏览器测试使用原生接口 fixture，不证明原生持久化。

## M2 批量管理与暂停（issue #19）

2026-10-29 独立验收：审查 `18feb64...058c18e`，未发现确定的规格缺陷或规范违规；双卡创建在自动加入和手动加入处重复，属于维护性风险。补充公开存储测试验证暂停后恢复仍遵守新卡额度、听音不可用过滤及同词暂缓。复跑 `pnpm lint`、`pnpm typecheck`、`pnpm build`、`pnpm rust:fmt`、`pnpm rust:check`、`pnpm rust:test:build` 后 `pnpm test:rust`（57 项，执行硬 60 秒限制）及 Edge `pnpm test:e2e`（57 项）均通过。Windows x64 release 重新构建后，以先前创建的合成数据库副本运行 `native-management-check.mjs`，设置、暂停、队列过滤、重启、恢复和新收藏自动建卡通过。

2026-10-29：schema 7 增量保存单词级暂停；自动加入偏好默认关闭，仅在新建生词事务中建卡。批量操作逐词报告结果；卡片已创建但语音失败与建卡失败分开显示。暂停和恢复不修改进度，评分入口与队列均检查暂停，撤销评分不改变后续暂停状态。

- `pnpm lint`、`pnpm typecheck`、`cargo fmt --all --check`、`cargo check --locked --workspace`、`pnpm desktop:build --no-bundle`：通过。构建使用仓库 pnpm 缓存及本机 Python 3.12；保留既有 Rollup 注释和 Rust 硬链接回退警告。
- `pnpm rust:test:build` 后 `pnpm test:rust`：通过，执行保留硬 60 秒限制。真实 SQLite 测试覆盖默认偏好、新建与重复收藏、暂停后撤销及重开、拒绝暂停评分和恢复。
- `pnpm test:e2e`：Edge 57 项通过，覆盖批量选择、排序和搜索、逐词失败重试、保存与语音失败的独立反馈、暂停音频重试及评分资格刷新；界面测试使用原生接口 fixture，不证明原生存储。
- `node apps/desktop/e2e/native-management-check.mjs`：Windows x64 release 和隔离数据库副本验证原生设置、暂停、队列过滤、重启、恢复，以及新收藏自动建卡和重复保存。测试未触碰用户库，进程已停止。

原生脚本未验证模型实际生成或播放音频、禁网、真实批量部分失败及其他目标平台；语音失败与批量部分失败由浏览器 fixture 覆盖。旧库迁移测试使用构造的 schema，不能代替历史版本二进制升级测试。完整 M2 与发行验收仍属于 #22 和后续里程碑。

## M2 评分撤销（issue #18）

2026-10-29 独立复核：以用户确认的 `75ba287...59620bb` 为基点分别执行规范与规格审查，未发现确定的实现缺陷；补齐规格审查发现的测试缺口。公开存储检查现在验证评分后保存的完整笔记记录与设置在撤销后保留，以及两个 SQLite 连接中的旧撤销目标拒绝覆盖后续评分。界面新增新卡 / 分钟卡 / 长期卡撤销后继续评分，以及重载后保留撤销重试身份。检查与限制见 [复核记录](evidence/issue-18/closeout-2026-10-29.md)。例句编辑尚属 #20，本票不声称已验证其撤销交互。

补测提交前复验：后端预编译后 53 项通过，保留硬 60 秒执行限制；Edge 完整 49 项通过。lint、typecheck、build、Rust 格式和 locked workspace 编译及文档本地链接检查通过。

2026-10-29：隔离 Windows x64 原生窗口通过评分、重启、撤销和继续评分；检查使用合成数据库副本，不触及用户库。存储回归覆盖新卡、分钟步骤、长期评分、重启、提交失败、重复撤销、跨学习日及 UTC 到东京的额度归属。Edge 45 项通过，包含响应丢失和明确拒绝后的重试；浏览器替身不证明原生持久化。lint、typecheck、build、Rust 格式与 workspace 编译通过；后端先预编译再执行全套测试，保留 60 秒超时。Windows 原生无 bundle 构建及隔离演示通过。

旧版本评分可能只保存队列数字位置，无法可靠重建已变化的原顺序。用户同意此时撤销进度和额度、按当前规则重建队列，并提示顺序可能变化；构造的 schema 5 文件升级、跨日撤销及重启重试通过。该旧库不是由历史二进制生成，原生旧快照、跨时区失败路径及其他目标平台尚未实机验收；不据此宣称 M2 或发行就绪。

## M2 每日设置与复习队列（issue #17）

2026-10-29 验收复核：规范和规格审查发现队列检查延长未播放音频的缓存寿命，以及评分前的数值位置不足以支持后续撤销恢复。修复后，Windows 原生隔离测试确认队列检查不更新访问时间、实际音频读取会更新访问时间，设置和两张队列卡重启恢复；存储回归确认评分事务保存评分前的卡片顺序。此前两项检查均先复现失败。复核后 `pnpm lint`、`pnpm typecheck`、`pnpm build`、`cargo fmt --all --check`、`cargo check --locked --workspace`、`pnpm desktop:build --no-bundle`、预编译后 `pnpm test:rust`（43 项）通过；Edge 42 项在修复前通过，修复未改前端行为。真实撤销及保留后续编辑仍由 #18 验证。完整队列快照随评分次数和队列长度增长，尚未测量大词库成本。

2026-10-29：schema 5 增量保存每日设置及队列顺序；分钟卡优先，新旧卡按实际额度穿插，并在评分事务中保存队列上下文。音频不可用的听音卡不阻塞识义卡；播放失败可跳过而不评分。

- `pnpm lint`、`pnpm typecheck`、`pnpm build`、`cargo fmt --all --check`、`cargo check --locked --workspace` 和 `pnpm desktop:build --no-bundle`：通过。构建保留既有 Zod / Rollup 注释警告和 Rust 增量缓存硬链接回退警告。
- `pnpm rust:test:build` 后 `pnpm test:rust`：43 项通过，包含固定时间下的额度、等待、同词暂缓、混排、上调额度及数据库重启恢复；测试执行保留 60 秒上限。
- `pnpm test:e2e`：Edge 42 项通过，包含设置保存失败和听音播放失败后跳过；浏览器使用原生接口 fixture，不证明原生持久化。
- `pnpm wasm:build` 后 `node scripts/check-wasm.mjs`：WASM ABI 实际执行通过，不作为排程一致性验收。
- `node apps/desktop/e2e/native-queue-check.mjs`：Windows x64 release 在隔离合成数据库副本上启动真实窗口；通过原生 IPC 保存设置，重启后设置及一张队列卡保持一致。测试进程已停止，未修改用户学习数据。

本次原生检查未覆盖多卡混排、原生播放失败或禁网；多卡顺序由 SQLite 测试覆盖。跨重启撤销属于后续 #18，其他平台和发行验收尚未验证。构建使用 Node.js 24.22.1 和 Python 3.14.10；资源准备输出中的 Python 3.12.13 是既有缓存清单的生成版本，不代表本次执行版本。

## M2 单卡评分与事务恢复（issue #16）

2026-10-29：schema 4 增量保存卡片进度、评分前状态、排程结果和评分上下文。识义卡翻面及听音卡成功播放并翻面后可评分；同词暂缓、每日额度、到期和时钟倒退在资格查询及评分事务内检查。失败保留操作身份以便重试，明确拒绝可刷新状态。此前 #15 尚无复习队列，完整队列及撤销分别由后续工单实现。

- `pnpm lint`、`pnpm typecheck`、`pnpm desktop:build --no-bundle`、`cargo check --locked --workspace`、`cargo fmt --all --check`：通过。
- `pnpm rust:test:build`、`pnpm test:rust`：通过，包括事务失败回滚、重试、时区变化、额度和同词暂缓测试；测试执行沿用 60 秒上限。
- `pnpm scheduling:build`、`pnpm test:scheduling`、`node scripts/check-wasm.mjs`：通过；71 个固定向量及 4096 个校准向量的原生与 WASM 结果一致。
- `pnpm test:e2e`：Edge 37 项通过；浏览器测试模拟原生接口，不证明原生持久化。
- `node apps/desktop/e2e/native-rating-check.mjs`：Windows x64 release 隔离副本中，真实窗口翻面并评分，重启后显示未到期；原生 IPC 回读进度、同操作重试及同词卡暂缓均通过。使用 #15 测试数据库副本，未修改用户学习数据，测试进程已停止。

原生界面本轮仅实测识义卡的“记得”路径；其他评分、听音门槛及失败路径由自动化测试覆盖。其他平台、完整复习队列、撤销和发行验收不在本次验证范围。

## M2 单个生词双卡与原生语音（issue #15）

2026-10-29 人工验收补记：用户确认交付音频无问题，新生成 `école` 的听感通过。下文“待用户确认”保留为技术验收时的记录；完整桌面禁网、连续 24 小时及其他平台等限制不因此改变。实现提交为 `e544dd9`，下一条主线为 #16 的四档评分、真实排程与事务恢复。

2026-10-29：#15 的实现及 Windows x64 技术验收已完成。schema 3 增量保留真实 schema 2 生词记录的完整公开内容，不批量建卡；加入复习通过立即事务创建两卡，重复及重启重试保持身份。听音卡接入真实 Supertonic 3 / 目标 48 kbit/s Ogg Opus，缺失和失败可重试，识义卡不受语音失败影响。卡面、机器合成标识、0.6 倍重播、播放请求取消、七天缓存清理和持久诊断已接入。

| 检查 | 结果 |
| --- | --- |
| `pnpm lint` / `pnpm typecheck` / `pnpm build` | 通过；保留 Vite 对依赖 PURE 注释的警告 |
| `pnpm test:e2e`，已安装 Edge | 28 项通过，其中复习界面 10 项；包含关闭后重开取消旧播放、成功重播后再次加入复用音频 |
| `pnpm rust:test:build` 后 `pnpm test:rust` | 29 项通过；预编译独立，执行阶段硬 60 秒超时保持不变 |
| `cargo fmt --all --check`、锁定离线全 workspace Clippy / 编译、`pnpm desktop:build --no-bundle` | 通过；保留增量缓存硬链接回退复制警告 |
| `pnpm wasm:build` 后 `node scripts/check-wasm.mjs` | 实际 WASM ABI 运行通过，schema 版本为 3；不当作新增 FSRS 验收 |
| `pnpm test:resources` / `pnpm test:tooling`，配置真实语音输入 | 7 项资源及 12 项工具检查通过，语音打包集成项未跳过 |
| `pnpm test:speech:cleanup`，配置最新 release | 4 项 Windows 原生检查通过：启动清理、占用 / 无关 / 非规范名保留、目录联接保护、锁协调、无 stderr 的持久诊断及重启重试 |
| `node apps/desktop/e2e/native-speech-check.mjs …` | 真实 WebView2 播放、正面隐藏、翻面不重播、0.6 倍重播、跳过及两卡 / 生词 / 音频重启恢复通过；损坏缓存、并发生成及失败 / 成功替换通过 |
| 便携包及独立音频解码 | 新语音开发 ZIP 165,205,335 字节，逐项校验 1,072 个文件；新 `école` 音频经 opusinfo / opusdec 确认有效，留给用户试听 |

测试使用隔离的合成生词库，没有修改用户学习数据；任务启动的测试进程已关闭。规范与规格两项代码审查未发现剩余缺陷，之前的播放失败标志、持久日志和废弃临时文件问题已修复并通过回归。命令、哈希、截图与试听文件见 [#15 完整证据](evidence/issue-15/README.md)。

限制：新音频的主观听感由用户确认；本轮原生窗口未施加网络限制，不能把实播结果描述为完整桌面禁网验收，既有 #14 禁网证据单独保留。缓存文件符号链接实测因权限跳过，目录联接保护已通过；连续运行 24 小时触发、跨进程同时生成、长时间播放清理及不支持硬链接的文件系统未测。其他平台、干净机器及 M5 完整版 / Lite 发行仍待后续；评分 / 排程持久化属于 #16。

### #15 实施过程（2026-10-28，历史）

以下“未完成”和“待验证”描述保留为当时的过程记录，当前结论以上节为准。

本轮补充 Windows x64 语音开发包入口：固定资源和编码器校验后打包，归档内容、许可、CRC 及不含 `Lexique-data` 的集成检查通过；损坏清单会失败并保留已有文件，原无语音打包测试 7 项通过。复习界面新增播放失败后再次加入的重新生成路径，浏览器完整测试 26 项、`pnpm lint`、`pnpm typecheck`、`cargo fmt --all --check`、桌面 `cargo check` / Clippy 和 Rust 测试通过。Windows 原生清理测试单独运行 3 项通过；并行运行时两项因 WebView2 临时日志尚未释放导致测试目录清理报错。

开发包隔离副本通过真实桌面 IPC 验证：损坏缓存可重新生成，同一进程内并发生成返回一致结果，禁用运行库后重新生成失败且旧文件不变，恢复运行库后可替换。测试使用独立测试数据库并保留音频供用户试听。现读取完成即释放音频文件句柄，播放使用内存中的副本；跨进程同时生成及长时间播放期间的清理仍需原生验证，24 小时定时触发尚未实测。开发包不是正式发行验收，实际音质由用户试听确认。

识义卡和听音卡翻面共用摘要、缺例句提示及按需展开的笔记和来源；听音卡缺少展开内容的回归测试先失败，修复后 `e2e/review.spec.ts` 的 5 项通过。`pnpm typecheck`、`pnpm lint` 和 `pnpm desktop:build --no-bundle` 通过；release 构建保留 Vite 对依赖注释的警告。

后续缓存恢复切片：界面调用 `speech_cached` 恢复音频，重复加入时复用已有音频；Edge fixture 的复习卡 6 项通过，`pnpm typecheck` 与离线 `cargo check --locked --workspace` 通过。缓存原生重启、并发、重解析点和不支持硬链接的文件系统尚未验收；目前无七天清理，不得视为 #15 完成。

本次接入 Windows 启动及每 24 小时的七天清理，并保留已读取音频的文件句柄直至进程退出。Windows release 的隔离启动测试 2 项通过：过期音频删除，新文件、占用文件及无关文件保留；语音目录指向目录联接时不清理目标。`cargo fmt --all --check`、`cargo check --locked --offline --workspace` 和 `pnpm desktop:build --no-bundle` 通过；编译保留缓存硬链接回退警告。24 小时周期、清理失败重试、生成 / 播放过程中的并发和完整发行资源仍未验收。

随后新增隔离失败路径：语音目录不可用时应用仍初始化，错误写入标准错误；修复目录后重启可清理过期文件。Windows 原生清理测试共 3 项通过；`pnpm rust:test:build` 后 `pnpm test:rust` 的 29 项测试通过，执行阶段保留 60 秒硬超时。24 小时周期、实际生成 / 播放期间并发和完整发行资源仍未验收。

继续补充缓存缺失后的恢复测试：已有两卡且音频缺失时，再次加入触发语音生成并显示可用，识义卡仍可见。Edge 中 `e2e/review.spec.ts` 7 项通过，`pnpm lint`、`pnpm typecheck` 通过；此测试模拟原生接口，不证明清理或真实播放。

Windows release 使用 `.tooling/issue-15-cleanup-retry-20261028/` 中独立测试库和开发暂存资源：启动时删除预置的过期音频，原生界面保留识义卡并显示听音卡待生成；再次加入后重新生成 6,709 字节的 Ogg，界面显示可用，数据库回读仍为原识义卡和听音卡 ID。打开听音卡后将测试文件访问时间设为八天前，第二个实例启动清理后文件仍在；测试进程已关闭。这不证明实际声音输出、禁网或标准发行包。

隔离目录 `.tooling/issue-15-native-cache-check/` 的 Windows x64 release 实测：打开既有测试词条并重新加入复习后，界面显示语音已生成，专用目录出现 9,311 字节的 `1.ogg`。关闭并保持启动会话存活后重启，不再次生成即可显示语音已生成；本次未测实际声音输出或断网行为。测试进程已关闭。

Windows x64 在 `.tooling/issue-15-native-check-20261028/` 使用独立数据目录启动 release 样例：首次窗口持续黑屏；以 `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--disable-gpu` 重启后界面正常。通过真实原生接口查找并保存 `école`、创建两卡、生成语音；听音正面未显示词头，打开后未显示播放错误，翻面显示释义、缺例句提示和笔记来源入口。未测量实际音频输出，也未限制网络，不能据此声称原生实播或离线验收通过。测试进程已关闭，黑屏根因尚未确定。浏览器测试使用原生接口 fixture，不证明实际播放。音频缓存及重启恢复、七天清理、正式资源集成及发行许可仍未完成；本票不能验收或视为完成。

本文件保留各阶段历史结果；较早记录中“待验收”或“未完成”的描述以当时为准。#13 后续独立规格审查及 #14 最终准入决定分别见[收尾记录](evidence/issue-13/closeout-2026-10-28.md)和[语音证据](evidence/issue-14/README.md)。

## Windows ARM64 开发样例交叉构建

2026-10-26：在 Windows x64 主机交叉构建 `aarch64-pc-windows-msvc` release 成功；构建前端时 TypeScript 和 Vite 构建通过。使用 `scripts/package_portable.py --architecture arm64` 生成开发样例并校验 10 个包内文件，ZIP 为 4,904,286 字节，SHA-256 `afcc8733be720dc66c35b8b964dc913e970e84b3c0522650ce2f7967dab1af8b`；包含现有词典小样，不含 TTS 或个人运行数据。七项隔离打包测试和两项构建参数测试通过。实际 ARM64 启动、WebView2、词典与语音均未验收，该包不是正式发行或 Lite 版。

2026-10-28 收尾复验：`pnpm test:tooling` 的两项构建参数及七项隔离打包测试通过；回读已有 ZIP 的 SHA-256、CRC 和清单中九个文件哈希均匹配。未重新构建或在 ARM64 设备上执行该样例。

日期：2026-10-24。此文件只记录本仓库骨架检查，不继承历史原型的通过结论。

## 可重复检查

| 范围 | 命令 / 证据 |
| --- | --- |
| 前端约束 | `pnpm lint` / `pnpm typecheck` / `pnpm build` |
| 浏览器 | `pnpm test:e2e`；三语、导航、主题、收纳、手机宽度、页面错误；报告与截图输出到忽略目录 |
| SQLite | `pnpm rust:test:build` 后 `pnpm test:rust`；重复初始化保留数据、高版本拒绝、迁移失败回滚 |
| Rust | `pnpm rust:fmt` / `pnpm rust:check`，CI 额外运行 Clippy |
| WASM | `pnpm wasm:build` 后 `node scripts/check-wasm.mjs`，实际装载并调用编译产物 |
| 原生目标 | CI 的 Windows / macOS × x64 / ARM64 `cargo check` |

## 早先骨架检查结果

本机 Windows 验证：`pnpm lint`、`pnpm typecheck`、`pnpm build`、Rustfmt、WASM release 编译与实际执行检查均通过。使用已安装的 Microsoft Edge 运行 2 项完整浏览器 E2E，通过；检查产生桌面与手机截图，结束后测试服务已停止。

SQLite 集成测试的本机预编译未通过：环境缺少 MSVC `link.exe`，因此没有执行后端测试，也没有宣称原生桌面已启动或已构建成功。对应集成测试与四目标原生编译将在 GitHub Actions 中运行，实际状态以该提交的检查结果为准。

构建显示 Zod 依赖的两条 Rollup 注释解析警告；构建退出码为 0，未关闭或降级警告。pnpm 11 构建脚本白名单已使用 `allowBuilds`，只放行 esbuild。

## Node.js 安全升级后的环境复核

复核日期：2026-10-24。

- 现有 nvm-windows 1.2.2 将 Node.js v24.15.0 升级并切换至 v24.22.1 x64；`nvm current`、`node --version` 和 `C:\Program Files\nodejs` 链接均确认生效。该版本为 [2026-10-21 官方安全更新](https://nodejs.org/en/blog/release/v24.22.1)，修复高危 DNS / TLS / Brotli 等问题；旧版本仍留在 nvm 中，不再是当前版本。
- 新 `node.exe` SHA-256 为 `d702daa815eabbff763c7d3808e235cf031cf2ca70211b56847bc72f51e240e5`，与官方发布值一致；Authenticode 显示 OpenJS Foundation 签名有效。
- npm 11.19.1、Corepack 和仓库锁定的 pnpm 11.25.0 可用。`pnpm install --frozen-lockfile`、`pnpm lint`、`pnpm typecheck`、`pnpm build` 均退出 0；锁文件未改变。构建仍有上述两条 Zod / Rollup 注释警告。
- 使用已安装 Edge 执行 `pnpm test:e2e`，2 项通过；仅验证界面骨架与浏览器开发服务。
- `tauri info` 确认 WebView2 155.0.4283.62；未检测到 MSVC / Windows SDK，PATH 中没有 Rust / Cargo / rustup。在用户 `.rustup` 中直接执行遗留二进制得到 Rust / Cargo 1.91.1，未满足项目锁定的 1.99.1，且该工具链没有 WASM 目标。因此未运行当前原生构建、SQLite 后端测试或 WASM 重建。
- 正常权限下 npm、Rust 1.99.1 manifest 和 crates.io 官方端点均返回 HTTP 200。受限工具环境会出现 DNS 错误，不能据此判定主机断网。

上述缺项已在下面的原生环境补齐中处理；保留此段检查时的结果，以区分升级前后证据。

## 原生开发环境补齐与实测

日期：2026-10-24。使用官方安装器，rustup 安装器 SHA-256 与官方值一致，Build Tools 安装器 Microsoft 签名有效。Build Tools 安装退出码为 0，无需重启。

| 工具 | 已安装版本 / 组件 |
| --- | --- |
| rustup / Rust / Cargo | 1.29.2 / 1.99.1 / 1.99.1；项目工具链自动生效，用户 PATH 已加入 `.cargo/bin` |
| Rust 组件 | rustfmt、Clippy；`wasm32-unknown-unknown`、`x86_64-pc-windows-msvc`、`aarch64-pc-windows-msvc` |
| Microsoft Build Tools | Visual Studio 2022 Build Tools 17.14.42；MSVC 14.44.35207，包含 x64 / x86 和 ARM64 工具 |
| Windows SDK | 10.0.26100.0 目标头文件、库及资源编译器 |
| 既有运行环境 | Node.js 24.22.1、pnpm 11.25.0、WebView2 155.0.4283.62 |

实测通过：

- `pnpm rust:fmt`。
- `cargo clippy --locked -p lexique-core -p lexique-storage --all-targets -- -D warnings`。
- `pnpm rust:test:build` 与 `pnpm test:rust`：3 项存储集成测试通过；0 项单元 / 文档测试不计作额外测试通过。
- `pnpm wasm:build` 与 `node scripts/check-wasm.mjs`：实际编译、装载并调用当前版本 ABI；不代表 FSRS 验收。
- `pnpm rust:check`：普通终端环境无需手动加载 VS Developer Shell 即可通过；当前旧会话临时补入新安装的 Cargo PATH。
- `cargo check --locked --workspace --target aarch64-pc-windows-msvc`：Windows ARM64 编译检查通过，未在 ARM64 硬件运行。
- `pnpm --filter @lexique/desktop exec tauri build --debug --no-bundle`：x64 调试程序成功构建，包含前端生产构建；仍有前述两条 Zod / Rollup 注释警告。
- 实际启动 `target/debug/lexique-desktop.exe`，原生窗口加载 `http://tauri.localhost/`，显示“本地存储已就绪”；用户应用数据目录创建 12,288 字节的 `lexique.sqlite3`，文件头检查通过，标准错误日志为空。测试窗口正常关闭，数据库保留；这不代表笔记业务的重启恢复验收。
- `tauri info` 的环境项全部通过；`vswhere` 确认 x64、ARM64、SDK 组件已注册，安装完整。

重新打开终端后会读取持久化的 Rust PATH；旧终端或继承旧环境的 Codex 进程可能需要重启。开发命令仍为 `pnpm desktop`。本轮不修改应用业务代码或锁文件；安装器与临时日志位于已忽略的 `.tooling/`。

## M0 资源小样探查

日期：2026-10-24。详情与来源见[资源小样调查](resource-feasibility.md)。这是独立资源探查，没有新增应用业务接口，也没有改变工程依赖或项目锁文件。

- 从用户指定的 mdx.mdict.org 获取 328,668 字节的 MDX 1.2 样本；头部长度 / ADLER32 校验通过。隔离 Cargo 工具固定 `mdictlib = 0.2.8` 并启用 LZO，编译退出 0；逐条解码 18,085 条正文并完成真实查询，执行退出 0。原始 HTML 未被执行。内容再分发许可尚未核实，不能默认随包发行。
- FreeDict 法英 0.4.1、法中 2025.11.23 的 Dictd 包下载成功，本地 SHA-256 已记录。标准库脚本只读取归档、不提取执行；全部索引行的偏移 / 长度检查及正文 UTF-8 解码通过，分别得到 8,505 / 10,947 条词条记录。未对照官方发布校验值，未完成结构化转换或准入。
- Python 首次输出受终端 GBK 对 `œ` 的显示限制失败；显式 `-X utf8` 后重跑成功，源文件没有改写。完整脚本、断言、源包和日志保存在忽略目录中。
- 已读取 FreeMdict 指定目录及两个子目录，尚未下载其大型全集或 MDD。TTS 仅核验官方文档和模型元数据，没有下载权重、生成 / 播放音频或接受 Pocket 的访问条件。
- 用户确认 TTS 拆为三票后，七份任务模板、父规格引用及依赖检查通过。已发布为 [#1](https://github.com/zardonc/lexique/issues/1) 的七张原生子任务 #2–#8，均为 open / ready-for-agent；GitHub 逐张回读正文与本地批准稿一致，父规格正文未变。原生依赖仅为 #8 blocked by #2，父页面显示七张子任务且只有 #8 为 blocked。页面凭证已保存；本轮没有执行这些任务的功能实现。

## 未包含的验收

应用尚未接入真实词典、完整变位、TTS、FSRS、第三方接口、Google 授权、Sheets 同步或浏览器学习流程；没有验证完整离线资源包、签名安装器、四架构安装或生产运行。独立资源解码和 CI 原生编译检查也不能替代这些验收。

## M0 默认词典小样准入（issue #2）

日期：2026-10-24。本票新增独立资源构建 / 查询入口，不改变上述应用接入边界。完整来源、原始许可、覆盖缺项、查询契约、体积和重建步骤见 [FreeDict 法中小样](../resources/freedict-fra-zho-2025.11.23/README.md)。

- 已从 FreeDict 官方获取法中 2025.11.23 原始 TEI 源包及发布 SHA-512，实算一致；保留未改动源包、包内 CC BY-SA 3.0 正文、Karl Bartel / FreeDict / WikDict / Wiktionary / DBnary 归属与转换、再分发说明。源码 MIT 不覆盖词典数据。
- Python 3.12.13 / SQLite 3.50.4 将全部 10,947 条记录转换为只读查询的 SQLite，保留 10,777 个原始键、同键记录及完整原始词条片段。先前 Dictd 的 10,589 个不同规范化索引键不等于原始词头数。`vacances` 存在；`sommes`、`yeux`、`chevaux` 等缺项未补造；弯撇号不改写为 ASCII。
- `pnpm test:resources`：6 项通过；包含经公开查询逐条比对所有源记录的元素 / 属性 / 文本、真实重音 / 撇号 / œ / 中文、多记录、无结果、错误 / 损坏 / 篡改资源、校验失败、保留既有输出和用户库、查询前后资源不变、连续两次生成逐字节一致。此前每条新增行为均先观察对应检查失败，再实现通过。
- `python -X utf8 scripts/dictionary_sample.py build .tooling/dictionary-admission/result`：通过；公开查询已实际读取生成库。构建及查询只使用本地输入，不调用网络。源包 464,676 字节、TEI 4,346,160、SQLite 5,062,656、完整压缩资源载荷 1,341,112、安装载荷逻辑字节 5,551,649；详细 SHA-256、计量组成和环境限制在小样记录 / `report.json`。
- `pnpm lint`、`pnpm typecheck`、`pnpm build`：全部通过；保留既有两条 Zod / Rollup 注释警告。没有新增应用依赖或改变锁文件。
- `pnpm rust:test:build` 后 `pnpm test:rust`：3 项存储集成测试通过，原有硬性 60 秒执行限制不变。受限环境报告增量缓存硬链接失败后复制的警告，未抑制；本票未修改 Rust 源码。
- `pnpm test:e2e`：受限环境中首次运行卡住，已明确终止本次 Playwright 及其 Vite 子进程树，未当作通过；正常权限下使用 Edge 重跑，2 项通过，Playwright 管理的开发服务已停止。这验证既有界面回归，不证明原生词典集成。
- 本票源码 / 文档差异和本地 Markdown 链接检查通过。上游 `COPYING` 保持字节原样；`git diff --check` 对其第 2 行 `=======` 标题误报冲突标记，另报告原始结尾空行。这两项作为保留第三方许可原文的已知例外，其余本票差异检查通过。
- 独立规范审查与规格审查各 0 项问题。新增 Windows / Linux / macOS 资源测试 CI 配置，远端尚未运行；本机只验证 Windows x64。未新增或重建 CodeGraph 索引。

本次准入限于该固定开发小样及其附带分发义务，不表示完整默认词典、桌面查词 #8、原生 / 四架构运行、首装断网安装或正式发行已验收；未更改用户学习数据库，也未冻结全部后续资源格式。

## M0–M1 原生小样离线查词（issue #8）

日期：2026-10-25。此节更新上述“应用尚未接入真实词典”的历史边界：现在仅固定 FreeDict 法中小样已接入原生窗口；完整默认词典、笔记、变位、TTS 与发行验收仍未完成。系统、版本、资源 / 可执行文件哈希、禁网方法、逐项观察与原生截图见 [Windows 验证证据](evidence/issue-8/README.md)。

- `pnpm lint`、`pnpm typecheck`、`pnpm build`：通过；保留既有两条 Zod / Rollup 注释警告。
- `pnpm test:e2e`：4 项通过，包含浏览器明确边界、平台 IPC 响应校验、真实来源字段、无结果 / 缺失 / 损坏 / 查询失败清除旧结果，以及既有三语、主题、导航与手机宽度回归。IPC 替身仅用于浏览器边界检查，不作为原生成功证据。
- `pnpm test:resources`：7 项通过；新增重复准备仅复用已验证生成物、损坏时失败且保留原件的检查。源包、内容与转换的既有完整比对仍通过。
- `pnpm rust:fmt`、`pnpm rust:check`、`cargo clippy --offline --locked -p lexique-core -p lexique-storage -p lexique-dictionary --all-targets -- -D warnings`：通过。受限文件系统的缓存硬链接警告仍可能出现，未抑制。
- `pnpm rust:test:build` 预编译后 `pnpm test:rust`：2 项词典集成测试与 3 项存储集成测试通过；执行约 1.5 秒，既有硬性 60 秒执行限制保留。覆盖真实 SQLite / 原始拼写 / 多记录 / 无匹配 / 注入字符串 / 无效输入 / 缺失 / 损坏 / 读取失败 / 替换恢复；逐字节确认资源和既有用户库不变且没有 SQLite sidecar。
- `pnpm desktop:build --debug --no-bundle`：通过。原生窗口在无互联网能力的 AppContainer 中首次启动并完成真实查词，应用和 WebView 后代令牌已核查；正常关闭退出 0 后在同等禁网环境重启，查询相同词条成功。原生资源错误分支已实际替换本次生成副本验证，结束后恢复原哈希，用户库哈希不变。
- TDD 记录：公开 Rust 查询入口先编译失败，准备入口先导入失败，页面先因查词输入缺失失败；实现后通过。损坏资源带匹配测试哈希的回归先得到错误分类，补充 SQLite 文件头校验后通过。浏览器 IPC 测试曾因缺少实际 `isTauri` 标志进入预览，修正测试的系统边界后通过。
- 独立规范审查 0 项、规格审查 0 项；差异及本次 Markdown 本地链接检查通过。验证启动的应用均已退出，Playwright 管理的服务已停止；既有未提交调查文档保留且未纳入本次提交。

这次只证明本机 Windows x64 开发构建的小样查询；没有重跑其他平台运行、完整词库 / 三语覆盖、WASM、安装器、缺 WebView2 的离线依赖安装、笔记恢复或正式发行。原生查词不接受用户资源路径、不执行原始 HTML；仅新增本地 main 窗口的查词命令，没有开放文件、shell、HTTP 插件或远程窗口权限。CI 已为资源准备与 Rust 词典测试补齐配置，本票提交尚未在远端执行。

## Windows portable 交付（issue #8 补充）

日期：2026-10-25。本节补充前述 debug 验收：Windows release 现按用户要求构建免安装 portable ZIP，运行数据、WebView profile、临时文件及框架配置 / 缓存 / 日志均定位到 `.exe` 同级 `Lexique-data/`。来源、尺寸、最终二进制与 ZIP 哈希、截图及逐项观察见 [portable 交付证据](evidence/issue-8/README.md#windows-portable-交付补充)。

- `pnpm desktop:portable`：通过；locked release 构建，无安装器，10 个交付文件逐项回读校验，ZIP 为 4,972,902 字节。重复打包和只遗留 `.zip.sha256` 均明确拒绝覆盖；旧交付保留。
- `pnpm lint`、`pnpm typecheck`、构建中的 `pnpm build`：通过；`pnpm test:resources` 7 项、`pnpm test:e2e` 4 项通过。保留原有 Zod / Rollup 注释与终端颜色环境警告。
- `pnpm rust:fmt`、`pnpm rust:check`、`cargo clippy --offline --locked --workspace --all-targets -- -D warnings`：通过；入口修正后再次通过格式与全 workspace Clippy。保留受限文件系统缓存硬链接复制警告。
- `pnpm rust:test:build` 后 `pnpm test:rust`：2 项词典与 3 项存储测试通过，执行仍受硬性 60 秒限制。
- 原生回归先确认旧构建不生成 exe 同级数据库；最终 ZIP 在带中文 / 空格路径、不同工作目录、外部 WebView 环境变量覆盖条件下禁网启动并查询成功。关闭退出 0 后整目录移动、同等禁网重启仍可查词并退出 0；实际应用及两轮各六个 WebView 后代令牌已检查。无法创建便携目录时退出 101，不改旧库、不回退；旧用户库哈希和旧 WebView 文件元数据保持不变。
- 独立规范审查发现校验 sidecar 可被覆盖，规格审查发现继承的 WebView 数据目录可逸出；两项已修复并以公开打包入口 / 实际 native 进程验证，独立复查各剩余 0 项。差异、链接与远端 CI 结果在交付时另行核对；原有未提交的资源调查文档保持原样。

这仅交付 Windows 11 x64 小样 portable，不证明签名、缺 WebView2 的干净机器、四目标发行、完整资源、笔记或同步。操作系统自己的诊断 / 安全记录不受应用目录配置控制；程序不写系统安装目录，不自动迁移旧 AppData。验证启动的原生进程和 Playwright 服务均已关闭。CI 已新增 `windows-x64-portable` ZIP 产物；远端结果见 issue #8 最终交付记录。

## M1 生词记录与独立笔记（issue #10）

日期：2026-10-25。此节更新上述笔记未实现的历史边界：Windows x64 开发构建现已完成三层记录、显式保存、幂等重试、版本冲突、搜索排序与重启恢复。完整命令、哈希、故障观察及截图见 [M1 验证证据](evidence/issue-10/README.md)。

- `pnpm lint`、`pnpm typecheck`、`pnpm build`，Rust 格式 / workspace 编译 / Clippy，以及 WASM 编译与实际装载均通过；既有警告保留。
- `pnpm rust:test:build` 后 `pnpm test:rust`：15 项通过，包含真实文件库重开、并发 / 幂等 / 冲突、写入与提交失败、增量迁移；执行硬限仍为 60 秒。资源检查 7 项、Edge E2E 13 项通过。
- 最终 locked debug 构建在 AppContainer 禁网环境真实重启，身份、来源、摘要、换行多语言笔记和排序偏好恢复。原生关闭三种选择及保存失败不退出均实测；词典缺失 / 校验失败时仍可编辑保存笔记，摘要不变。测试仅使用隔离副本，原始资源和已有学习数据保留。
- 独立审查的未确认保存保护与 macOS 菜单 Quit / Cmd+Q 问题已修复并复查。macOS Dock Quit 仍可能绕过未保存提示，须在该平台验收前处理；macOS 专属代码未在本机编译或运行。

本轮仅 Windows x64 开发构建验收；其他架构 / 平台、远端 CI、release portable 重打及正式发行未验证。没有新增草稿、删除、FSRS、同步、TTS 或服务。验证进程均已关闭；本轮开始前的文档编辑未纳入本次实现提交。

## M1 Windows 复盘修复与 macOS 阻塞

日期：2026-10-25。Windows 可执行的复盘项已修复：公开原生构建入口默认 locked，portable 复用同一入口；独立打包测试与构建参数测试接入 CI，交付说明更新 M1 笔记能力。新包输出到隔离目录，旧交付与既有学习数据保留。详见 [Windows 后续证据](evidence/issue-10/windows-follow-up.md)。

- 工具测试 7 项、资源检查 7 项、Edge E2E 13 项、预编译后的 Rust 测试 15 项通过；lint / typecheck 与 release 内嵌前端生产构建通过。后端执行硬限仍为 60 秒。
- Windows x64 locked release portable 成功生成并逐项校验；真实禁网窗口验证 Alt+F4、标题栏关闭、继续编辑、保存失败不退出、恢复数据库后重试，以及多语言笔记与排序的重启恢复。Windows ARM64 workspace 编译通过，但无该架构实机运行证据。
- macOS 专属编译、菜单 Quit / Cmd+Q / Dock Quit 及原生数据恢复集中标记为 **blocked — 等待 macOS 执行环境**。退出矩阵、已知 Dock Quit 缺陷与解除阻塞条件见 [macOS 清单](macos-m1-blockers.md)。Windows 结果不替代其验收。

本轮未推送、未运行远端 CI；没有验证 macOS / Windows ARM64 实机、缺 WebView2 干净机器、签名或正式发行。验证仅使用隔离合成数据，原有未提交文档修改保持原样。

## M2 固定 FSRS 与原生 / 实际 WASM（issue #13）

日期：2026-10-26。共享核心候选使用固定 `fsrs-rs 6.6.2` 官方默认参数执行四档评分、初学 / 重学步骤与日历排程，不含优化或间隔扰动。Windows x64 原生 CLI 和无宿主导入的实际 WASM 通过同一公开入口执行，最终一致性准入 **blocked**：两个合法反例出现一天排程差异。来源、许可、完整命令、输入契约、误差与产物体积见 [#13 证据](evidence/issue-13/README.md)。

- 首轮 69 个固定向量及 4,096 个固定种子校准向量通过；独立审查扩大输入后发现浮点误差跨过取整边界。反例已加入固定集，最终 71+4,096 对照有 2 项严格排程不一致，`pnpm test:scheduling` 退出 1 并保留完整报告；没有放宽日期 / 整数间隔比较或将失败改为通过。
- 5 项核心集成测试、既有 15 项 Rust 测试通过；先 `pnpm rust:test:build`，再执行 `pnpm test:rust`，硬限仍为 60 秒。lint / typecheck / build、Rust 格式、workspace check / Clippy、资源 7 项、工具 7 项及 Edge E2E 13 项均通过。
- Windows ARM64 workspace 编译通过，尚无实机运行证据。本机 Python Store 别名失效导致首次资源准备 / workspace check 失败；仅在验证进程中将已有 Python 3.12.13 目录加入 PATH 后重跑通过，没有修改全局设置。首次 WASM 构建暴露上游随机数依赖限制，最终使用其明确报错的 unsupported 后端；实际产物没有宿主导入。

未接入学习数据存储、评分 UI 或 TTS，未更改数据库 schema。#13 保持未通过，依赖它的后续集成继续阻塞；没有换库、补丁替换上游算法或改变已确认取整规则。TTS 仍由 #14 单独准入，生成后需人工试听校验。macOS 实机、Windows ARM64 实机、远端 CI、完整离线资源及发行仍未验证；本次不据此关闭 M2 或解除其他里程碑门槛。

### Issue #13 数学兼容修复

日期：2026-10-26。本节更新上段历史阻塞结论。优先检查依赖与开发环境后，以零第三方依赖的最小程序确认 Windows / WASM 的标准 `exp` / `powf` 相差 1 ULP；固定包哈希、隔离 target 重建、Debug / Release 及 Rust 1.91.1 交叉检查没有发现应用代码、缓存或配置导致偏差的证据。完整假设、探针与原失败结果见 [诊断](evidence/issue-13/math-diagnosis.md)。

保留 FSRS 6.6.2 原公式与官方参数，显式 vendored 补丁只将标量调度路径的数学函数统一为固定 libm 0.2.16，关闭其默认 features；源码审计仅 manifest / model.rs 与官方包不同。模型标识为 `fsrs-rs/6.6.2+libm-0.2.16`，新增回归拒绝未打补丁的历史版本。当前 Windows x64 release 原生 / 实际 WASM 的 71+4,096 向量通过、记忆差为零，原两条反例继续要求整数排程严格相等。Rust 格式、workspace Clippy、预编译后 21 项 Rust 测试和 Windows ARM64 workspace 编译通过。

本次未改 UI，未重跑前端 / E2E 或重打 portable；macOS / ARM64 实机、远端 CI 仍未运行。没有修改用户数据、迁移存储或集成 TTS；#14 的语音准入及人工试听仍独立保留，不能据此宣称完整 M2 通过。上游标准数学的独立诊断命令仍在本机退出 1，这是刻意保留的原因复现，正式 `pnpm test:scheduling` 已通过。

### Issue #13 验收收尾

2026-10-28：五项技术验收通过。重新构建 Windows x64 原生 / 实际 WASM，71 + 4,096 条向量排程严格一致、记忆差为零，产物哈希与既有机器报告相同。锁定安装、lint / typecheck / build、Rust 格式 / workspace check / Clippy、预编译后 24 项 Rust、7 项资源、9 项工具和当前工作区 18 项 Edge E2E 通过；runner 硬 60 秒限制不变，Windows ARM64 仅编译通过。

独立规范审查无需处理问题；独立规格子审查三次限流失败，由主代理完成逐项规格复核，未冒称独立审查通过。完整来源 / 依赖回读、命令与限制见[收尾记录](evidence/issue-13/closeout-2026-10-28.md)。本次只提交 #13 文档，保留已有 #14 编辑；不改用户库、不启动后续实现。远端 CI、macOS / ARM64 实机及发行验收仍未运行，当前分支未推送。Playwright 服务已停止。

## M2 单词离线语音探针（issue #14）

2026-10-27：历史语音验证包固定的 Python 3.12.10 缺少后续安全修复，已将 Supertonic 准备脚本和 CI 改为 Python 3.14.10；旧包仅留作历史证据。v7 六项打包检查及三项失败路径检查通过，受限进程实际生成并调用播放 38 条音频；远端 CI 与干净机器未验证。版本及限制见[试验记录](evidence/issue-14/supertonic-trial.md)。

本机使用 v7 的 Python 3.14.10 另运行 `test_dictionary_sample.py` 和 `test_package_portable.py`，各七项通过；这不代替 CI 的其他系统运行。旧 Kokoro 探针的历史资源清单仍指向 Python 3.12.10，不应作为当前安全发行材料。

同日从 v7 ZIP 解压副本在受限 Windows x86-64 进程新生成 38 条 WAV 并调用播放，逐文件哈希匹配，前后联网探测均被拒绝。Eigen 5.0.1 官方源码归档哈希与上游声明一致，但实际 wheel 使用的 Eigen 版本及分发义务仍未确认；见[试验记录](evidence/issue-14/supertonic-trial.md)和[许可说明](evidence/issue-14/supertonic-licenses/ATTRIBUTION.md)。

2026-10-27 用户反馈试听 v7 生成音频，男女声均正常；未逐项说明文件、设备或速度，不能据此宣称全部 38 条逐条验收或干净机器已验证。

同日核验 Eigen 3.4.0 官方源码归档 SHA-256 与上游声明相同；该归档包含 LGPL 文件，并提供 `EIGEN_MPL2_ONLY` 编译限制。实际 wheel 采用的版本、源码范围和编译选项尚未证明，不能据此认定其发行义务已闭合。

2026-10-27：补充固定 CPython 3.14.10 官方许可文档中的 Expat 和 Unicode 通知，v7 缺失该文档的回归检查失败，v8 六项打包检查通过。NumPy wheel 元数据所列 17 项许可文件在 v7 中均存在；CPython 的 `libtommath.dll`、OpenSSL 归属以及 sherpa-onnx wheel 的实际构建依赖仍需核查，v8 尚未实播。详见[许可说明](evidence/issue-14/supertonic-licenses/ATTRIBUTION.md)。

同日 v8 准备目录在受限进程重新生成并调用播放 38 条 WAV，前后联网探测均被拒绝；没有从 ZIP 解压运行，也未人工试听 v8 样本。

2026-10-27 用户反馈 v8 生成音频的男女声均正常；未逐项说明试听文件、设备或播放速度。

同日复核 `libtommath.dll`：CPython 配置将其关联 Tcl/Tk，但嵌入包中未发现 Tcl/Tk DLL 或根目录 PE 文件的直接导入。官方 `cpython-bin-deps` 的同名 DLL 与嵌入包的 DLL 大小及 Git blob 均不同，不能按相同二进制认定；动态加载及确切来源仍未排除或确认。详见[许可说明](evidence/issue-14/supertonic-licenses/ATTRIBUTION.md)。

后续复核证实上述文件差异来自官方 Authenticode 签名：验证签名有效，去除证书表并还原 PE 相关字段后与 CPython 上游二进制 Git blob 一致。Tcl 源码的 LibTomMath 许可加入 v9，新增检查在 v8 失败、v9 六项通过；该记录不证明动态加载情况或全部运行库义务。后续 v9 准备目录的生成与播放见下方当前状态。

日期：2026-10-26。Kokoro v1.0 int8 / ff_siwis、kokoro-onnx 0.6.1、eSpeak NG 1.52.0
组合在 Windows x64 的受限禁网进程中完成 17 个准确输入的真实生成、有效 WAV 回读及同步
Windows 播放调用。完整独立 Python 运行包也重跑通过，不借用开发虚拟环境或预生成音频。
模型、声线、前端、运行库、对应源码、许可与下载校验清单、可试听音频和性能报告见
[#14 证据](evidence/issue-14/README.md)。

准入仍 **blocked — 待修正后的完整样本试听验收**。用户认可声音可继续评估，并要求先修复
孤立 y；已将其词汇读音显式修正为 /i/，保留原前端音素、修正标记与前后音频。
播放 API 成功不证明声学输出
或法语质量，未知词 G2P 推断也不代表正确发音。首轮两项短句因前端分隔符被严格拒绝，
已保留失败证据并显式处理；其他未知音素继续报错，不静默丢弃。

完整压缩 / 安装量及依赖明细见该票 measurements；单独语音包不使用 WebView2，不能据其体积
推断正式桌面首装总量。缺 VC++ 的干净机器、其他三个目标、产品语音 IPC / UI 及 M5 未验收。
18 项 Python、lint / typecheck / build、21 项 Rust、71+4,096 调度向量、工具与 13 项 Edge E2E
回归通过，具体命令和环境失败重跑见 [检查记录](evidence/issue-14/checks.md)。未改用户库或产品依赖。

## Piper 语音偏好设置

2026-10-26：新增三语设置页，选择法语女声／男声、正常／慢速，默认女声／正常；通过受限桌面 IPC 保存到现有 SQLite 元数据。不存在的配置使用默认值，非法配置保持错误。无 schema 迁移、新依赖、学习数据重写或音频删除。规则见[语音偏好](speech-preferences.md)。

- `pnpm lint`、`pnpm typecheck`、`pnpm build`：通过。首次 lint 发现测试夹具复杂度超限，提取默认回复后复验通过。
- `pnpm rust:test:build` 后 `pnpm test:rust`：23 项通过，执行保持硬性 60 秒上限；新增 2 项覆盖默认值、数据库重开恢复、其他偏好保留、非法字段和损坏配置。
- `cargo fmt --all --check`、`cargo check --locked --workspace`：通过，包含 Windows x64 桌面命令与权限生成。
- `pnpm test:e2e`（本机 Edge）：17 项通过，新增 4 项覆盖保存／重新进入、失败重试、非法响应／重新读取和三语手机布局；桌面与手机截图已检查。
- 首次受限环境浏览器检查无法连接本机服务，已停止并在允许回环连接的环境重跑。初版顺序回复夹具被 React 严格模式的重复读取消耗；改为持有保存值的夹具后通过。没有据此修改生产读取行为。

浏览器测试模拟 IPC，不证明原生窗口完整端到端链路；Rust 测试证明实际 SQLite 文件的重开恢复。未重打 portable 或做原生窗口交互、其他平台运行、TTS 合成／播放／缓存、慢速倍率听感校准。偏好设置可用不表示模型已获默认包再分发准入或完整 M2 已通过。


## Piper MLS 双说话人准入与偏好兼容

2026-10-26：替代模型选择 `fr_FR-mls-medium`，源说话人 `3698/F` → 推理编号 `1`、`1840/M` → `0`。依据固定官方模型许可指引、从零训练模型卡及 MLS 来源，按 CC BY 4.0 路径继续准入；[补查记录](evidence/issue-14/replacement-admission.md)明确许可解释边界，[原始材料](evidence/issue-14/mls-licenses/ATTRIBUTION.md)保留来源、署名、全文和哈希。不是完整引擎 GPL 分发义务或正式发行验收。

设置保存稳定的 `female`／`male`，兼容旧 `siwis`／`tom` 对应的男女声及原语速；没有数据迁移或音频删除。新增回归先复现旧模型标识泄漏（断言失败），再加入 Rust 反序列化别名并通过。

- `pnpm lint`、`pnpm typecheck`、`pnpm build`：通过。
- `pnpm rust:test:build` 后 `pnpm test:rust`：24 项通过；硬性 60 秒执行超时保持，首次编译独立。
- `cargo fmt --all --check`、`cargo check --locked --workspace`：通过；仅文件系统硬链接缓存回退复制警告。
- `pnpm test:e2e`（本机 Edge）：17 项通过；设置页的选择、保存失败、重新进入和响应校验沿用既定边界。
- 原始许可归档 SHA-256、模型权重／配置哈希与上游固定快照校验通过。

独立合成探针及四组合结果见[试听页](evidence/issue-14/mls-samples/listen.html)。正常时长系数 `1.0`、慢速 `1.3`，两档均保留模型声学噪声 `0.333`、将时长扰动 `noise_w_scale` 固定为 `0`。校准后 72 个 WAV 有效，36 对样本逐条慢速更长；女声／男声总时长倍率为 `1.276`／`1.275`。同一受限进程前后各两次联网探测返回 WinError 10013，72 次同步 Windows PlaySound 调用完成。首轮及校准轮合计生成播放 144 次；不把播放调用完成当作人工听感验收。首轮默认时长噪声产生部分慢速样本反而更短，不能据总时长断言通过速度验收；保留原始报告作校准对照。裸文本 `y` 仍为字母名，另有显式 `[[i]]` 音素对照，不能把该对照当作未改文本的正确合成。

未接入产品 TTS 合成／播放／缓存，浏览器 IPC 模拟不等于原生完整链路；未重打 portable、计量最终整包或运行其他三个目标。双声线音质、慢速清晰度及显式 /i/ 听感仍待人工验收。


### MLS 人工验收失败

2026-10-26：用户实际试听上述四组合，确认全部不是正确法语词读音，而是混乱人声且音质不佳。撤回 MLS 候选，不认定 TTS 质量准入或 #14 完成。72 个有效 WAV、36 对时长检查及播放成功只是容器／调用证据，未覆盖真实可懂度；后续替换需重新试听。原始失败样本和报告保留，不以调速校准通过覆盖人工否决。


### Supertonic 3 替代小批筛查

2026-10-26：MLS 已因人工试听失败撤回。Supertonic 3 int8 / F1 女声、M1 男声 × 正常 1.0／慢速 0.8，在禁网受限 Windows x64 进程生成、回读并调用播放 26 个 WAV；前后联网探测均为 WinError 10013。12 对法语样本逐条慢速更长；不认定声音质量或完整 #14 验收通过。固定资源哈希、原始许可证、模型／代码许可区分见[执行记录](evidence/issue-14/supertonic-trial.md)。

语言参数原生绑定 getter 返回副本：CLI 断言先复现失败，再改完整字典赋值、保留断言并重跑通过。Whisper tiny 本地对 28 个样本作识别对照，支持 MLS 目标句偏离严重及新组合例句有所改善；短词、y／i 识别不稳定，不代替人工。ASR 归一化的重音／œ 自检先失败后修复，最终全量对照通过运行。仅本地试验环境新增两个 sherpa-onnx 包，未改产品依赖。

本轮未再改前端／Rust 源码；沿用本轮此前 lint、typecheck、build、24 项 Rust 及 17 项 Edge E2E 通过记录。独立脚本校验、音频哈希与浏览器试听页另行检查。未重打 portable、未通过完整包体／四平台／实际产品音频链路；短词与 y 的质量及人工试听仍是明确门槛。

补充检查：26 个试听控件在浏览器均可加载有效时长且没有自动播放；独立逐文件哈希、PCM、12 对时长、许可证原始字节及本地链接检查通过。手写改动通过 Git 空白检查；逐字归档的 CC BY 全文与 Piper VOICES 原件各自带末尾空行，保持上游字节及哈希，不为消除该提示修改原文。

### 慢速与字母名试听候选

用户确认原英语与法语音频可用，要求慢速更慢，并倾向单独字母 `y` 使用字母名。本轮通过 `probe_supertonic.py` 在 Windows x64 受限进程生成并回读 32 个 WAV，包含男女声、正常 `1.0`、候选慢速 `0.6`、英语及 `i grec`；32 次同步播放调用完成，前后两次网络探测均为 WinError 10013。逐文件 SHA-256 核对通过；16 对相同输入的慢速时长均大于正常速度，试听页 17 个本地引用均存在，`git diff --check` 通过。[报告与试听](evidence/issue-14/supertonic-slower/listen.html)记录本轮结果。

`0.6` 和 `i grec` 的听感尚待用户确认；此试验没有实现产品 TTS、字母语境识别或音频缓存，也没有完成原 #14 全量样本和跨平台验收。

后续用户要求播放时变速：试听页常速和慢速使用同一个正常生成的音频，慢速播放倍率设为 `0.6`。设置保存原有 `normal`／`slow` 值并明确显示倍率；这不表示应用已经具备语音生成或播放能力。前述 `speed=0.6` 合成样本仅作历史记录保留。

验证：`pnpm run lint`、`pnpm run typecheck`、`pnpm run build` 通过；Windows Edge 下 `pnpm --filter @lexique/desktop exec playwright test e2e/speech.spec.ts e2e/speech-playback.spec.ts` 的 5 项通过，覆盖设置保存恢复、失败处理及试听页同文件的实际播放倍率。试听探针的 Python 语法检查通过；未为调速重新生成音频。

用户随后确认 `0.6` 倍播放清晰自然，`i grec` 可作为单独字母 `y` 的读音；该确认不扩展为完整单词集、产品合成与播放或发行验收。

完整输入补测：Windows x64 禁网受限进程下运行探针，原定 17 项、英语和字母名分别由男女声以正常速度生成并同步调用播放，共 38 个 WAV；网络探测前后各两次均为 WinError 10013，峰值工作集 314,765,312 字节。逐文件 SHA-256、原始输入顺序及倍率核对通过，削波计数均为零。浏览器检查[试听页](evidence/issue-14/supertonic-full/listen.html)的 38 个控件，切换慢速后播放倍率均为 0.6，资源未报缺失。CLI 对缺失资源及已有输出目录均非零退出，已有输出被保留。完整样本的读音仍需人工试听；独立候选的完整发行包、其他平台及产品链路仍未验收。

用户随后确认完整音频样本没有问题。使用现有 FFmpeg 7.1 将 38 个常速 WAV 分别编码为 Ogg Opus 24 和 48 kbit/s（libopus、voip），共 76 个文件。原始文件共 5,467,662 字节，两种 Opus 分别共 145,852 和 286,705 字节，节省 97.33% 和 94.76%。所有文件均解码为非静音 PCM，重采样回 44.1 kHz 的帧数与源文件相差 0 至 1 帧。浏览器逐个加载 76 个 Opus 文件并成功以 0.6 倍播放一个 24 kbit/s 文件；[试听对照](evidence/issue-14/supertonic-opus/listen.html)供人工比较音质。未验证其他平台的播放器，也未将编解码接入产品或确认 FFmpeg 随产品分发的许可。

用户最初确认上述 Opus 24 和 48 kbit/s 的音质可接受。2026-10-27 进一步复听后，用户明确指出 24 kbit/s 在同一句内声音时轻时响，48 kbit/s 没有这一问题，并确定应用新生成语音默认采用 Ogg Opus 48 kbit/s；此前优先评估 24 kbit/s 的建议不再适用。新设备试听常速 / 0.6 倍慢速播放正常，但产品编码与其他目标平台兼容性、发行许可仍未验证。

追加 CLI 回归：4 项通过，包括在受限 Windows x64 进程重新生成并播放 38 个音频，以及缺失资源、损坏资源和既有输出的失败检查。运行库 wheel 已按 PyPI 摘要核对；六个候选原始下载文件合计 196,958,331 字节，但完整桌面包及许可尚未核算，详见[追加记录](evidence/issue-14/supertonic-trial.md)。

### Supertonic 独立验证包

2026-10-27 当前状态：v9 准备目录在受限 Windows x86-64 进程生成并调用播放 38 条音频，用户确认 v9 声音正确；反馈未逐项列出试听文件、设备或播放速度。v9 补充 LibTomMath 许可，六项打包检查通过，ZIP 为 159,650,967 字节，解压逻辑大小为 240,989,324 字节；并非从 ZIP 解压后执行。本票剩余的来源和再分发准入缺口见[当前准入决定](evidence/issue-14/README.md#当前准入决定)。完整桌面包体和其他三个目标实机验证留待后续发行验收，不因其未完成阻塞 #14。

后续在不生成音频的前提下增加 hclust 许可及版权说明，并构建 v11 验证包。v11 与 v9 的模型及运行时逐文件哈希一致，包 CRC、六项打包检查及四项无音频探针检查通过；真实合成测试按用户要求跳过。v11 ZIP 为 159,653,525 字节，解压逻辑大小 240,995,571 字节，哈希及差异见[试验记录](evidence/issue-14/supertonic-trial.md)。上游 wheel 的实际构建依赖、选项和量化发布物的完整转换来源仍未证实，不能据此宣称 #14 通过。

进一步补充 Darts 和 ThreadPool 上游通知及模型静态对照后，v15 ZIP 为 159,665,936 字节、解压逻辑大小 241,018,108 字节，1,048 个文件的清单及 CRC 检查通过；与已试听的 v9 有 1,016 个模型和运行时文件哈希一致。六项打包检查、四项无音频探针检查及五项模型比较测试通过，真实合成项按用户要求跳过。比较仅支持部分同名权重具有数值关系，未证明完整模型转换及 wheel 的实际构建依赖；详见[试验记录](evidence/issue-14/supertonic-trial.md)及[静态对照](evidence/issue-14/supertonic-static-comparison.md)。#14 的资源准入仍未完成。

2026-10-28：按用户提供的研究资料补充 Eigen 5.0.1 上游源码归档中的 MPL、BSD 及许可概览原文，并将其加入资源来源清单；归档和 24 项许可材料的 SHA-256 校验通过。打包失败路径测试四项通过，两个需要现成包的检查本次跳过；未重建 v15、未生成或播放音频。源码许可证据不能证明 wheel 使用的 Eigen 版本或其完整依赖，资源准入仍待核实。

同日补齐 Eigen 两版候选许可，构建 v16：ZIP 159,715,206 字节，解压逻辑大小 241,154,340 字节，1,060 文件；CRC 和六项打包检查通过，1,016 个模型 / 运行时文件与 v9 哈希一致。四项无音频探针检查、五项静态比较测试通过；真实合成按用户要求跳过。OpenSSL 两个 DLL 与固定 CPython 二进制依赖逐字节相同；其全部第三方通知和 sherpa-onnx wheel 实际依赖仍未完整审计，见[许可记录](evidence/issue-14/supertonic-licenses/ATTRIBUTION.md)。

同日固定提交的独立源码 checkout 完成 Windows x64 Release C API 构建；编译日志保留 C4804、C4068 警告。新 DLL 的 `.text` 哈希和 `.rdata` 长度与 v9 不同，不能作为已试听 wheel 的构建证明，亦未执行该新 DLL 的音频验证。构建依赖和日志见[受控构建记录](evidence/issue-14/controlled-build-20261028.md)；v16 不含该后续记录。

用户随后确认：接受 v9 wheel 构建证明缺失及量化校准过程未独立复现为 #14 已知限制，按候选依赖保守归档通知，并将 OpenRAIL-M 协议及机器合成披露纳入 M2 / M5 验收。v16-revised 包 159,725,647 字节、解压 241,177,527 字节、1,067 文件，六项打包、四项无音频探针及五项静态比较通过，1,016 个模型 / 运行库文件与 v9 一致；未再次合成音频。项目准入决定不等同完整依赖证明或正式发行法律批准。详见[准入记录](evidence/issue-14/README.md)。

2026-10-26：精简重复运行库后的 Windows x64 v2 ZIP 为 158,381,008 字节，解压文件逻辑总量 238,534,083 字节；1,026 项文件哈希及 ZIP CRC 通过。解压副本在受限进程重生成 38 条并调用播放，前后联网探测返回 WinError 10013；五项打包检查通过。完整产品体积、干净机器、其他平台及全部传递依赖许可仍未验收，详见[执行记录](evidence/issue-14/supertonic-trial.md)。

ONNX Runtime 1.28.2 DLL 与上游 MT Release 版本逐字节哈希一致，已收录其许可及第三方通知。加入许可材料的 v3 ZIP 为 158,414,965 字节、解压文件逻辑总量 238,868,060 字节，1,028 项清单哈希与 ZIP CRC 通过；v3 未重跑解压播放。其他依赖的许可仍需确认。

固定模型、运行时与许可材料的独立 Windows x64 验证包已构建：压缩 166,691,207 字节、文件逻辑总量 261,179,033 字节；ZIP CRC 和 1,031 项文件哈希通过。包内运行时在受限进程重生成 38 条并完成同步播放调用，前后联网探测均被拒绝。打包失败路径测试 4 项通过。命令、摘要和局限见[执行记录](evidence/issue-14/supertonic-trial.md)。

这不是完整桌面包，不包括系统 VC++、WebView2、词典或应用；未验证 ZIP 解压到干净机器、其他架构、完整传递许可及产品集成。完整版 / Lite 分发和 Lite 模型下载均尚未实现，#14 不宣称完成。

2026-10-27：补充 KissFFT 固定上游版权声明和 BSD-3-Clause 文本；Windows x86-64 v5 验证包 ZIP CRC、1,034 项清单哈希与五项打包测试通过，新增许可检查在 v4 先失败。v5 尚未实播，源码依赖声明不等于 wheel 的实际链接清单；详细数据见[执行记录](evidence/issue-14/supertonic-trial.md)。

2026-10-27：补充固定 kaldifst 和 OpenFST 许可、版权及作者文件；Windows x86-64 v6 验证包五项打包测试通过，新增文件检查在 v5 先失败。v6 未实播，实际二进制依赖与完整分发义务仍待核验；详细数据见[执行记录](evidence/issue-14/supertonic-trial.md)。

同日复测 v6 ZIP 解压副本：受限 Windows x86-64 进程新生成 38 条 WAV 并完成同步播放调用，音频哈希均匹配，生成前后两地址联网探测均为 WinError 10013。v6 新生成音频未获人工试听验收，实际二进制依赖和其他平台仍未核验；详见[执行记录](evidence/issue-14/supertonic-trial.md)。

同日用户确认 Windows ARM64、macOS x86-64 和 macOS ARM64 留待后续验证，不阻塞 #14；正式桌面发行仍须满足四目标要求。v6 Python 扩展及 C API DLL 自报同一 `2.0.1` 版本与 `af2d8d81` 源码提交前缀；完整依赖和发行许可不能由自报版本代替，详见[来源记录](evidence/issue-14/supertonic-licenses/ATTRIBUTION.md)。

2026-10-27：新增 portable 语音临时目录与定期清理的产品要求及后续接入验收项；当前没有产品语音生成或清理实现，本次不声称已验证文件存放、过期清理或移动后重试。macOS portable 路径与四架构实机验证仍待落实。

## 共享例句（issue #20）

2026-10-29：`pnpm lint`、`pnpm typecheck`、`pnpm build`、`pnpm rust:fmt`、`pnpm rust:check`、`pnpm rust:test:build` 和 `pnpm test:rust` 通过。存储测试覆盖例句保存、冲突、重试、历史操作兼容和请求字段校验。Windows x64 使用 `pnpm desktop:build --no-bundle` 构建后，通过 `apps/desktop/e2e/native-examples-check.mjs` 在合成数据库副本中编辑两条例句并重启，确认例句与复习卡进度恢复。

Obscura MCP 使用模拟桌面接口检查了识义卡展示；这不是原生卡片全流程验证。新增 Playwright 浏览器测试尚未运行，本机缺少对应的 Playwright 浏览器；未验证其他平台、完整 portable 包或真实音频播放。

2026-10-30 补充：例句组现可明确关联收藏时摘要中的一个释义；旧记录保持未指定，不推断原意。`pnpm lint`、`pnpm typecheck`、`pnpm build`、`pnpm rust:fmt`、`pnpm rust:check`、`pnpm rust:test:build`、`pnpm test:rust` 通过；使用项目 `target/playwright-browsers` 中的 Playwright Chromium、固定 UTC 时区运行 `pnpm test:e2e -- --workers=2`，63 项通过，覆盖两类卡的多义词选择。Windows x64 release 原生检查使用合成数据库副本，验证所选释义、两条例句及复习进度重启恢复。未验证其他平台、完整 portable 包或真实音频播放；浏览器测试使用模拟 IPC，原生重启检查不覆盖完整原生复习流程。

同日审查修复：新增或修改非空例句须选择释义，未关联的旧例句可原样保留并单独编辑笔记；冲突面板显示当前保存的例句释义。`pnpm lint`、`pnpm typecheck`、`pnpm build`、`pnpm rust:fmt`、`pnpm rust:check`、`pnpm rust:test:build` 和 `pnpm test:rust` 通过；固定 UTC 的 `pnpm test:e2e -- --workers=2` 64 项通过。浏览器测试使用模拟 IPC；本轮未重跑原生窗口、其他平台或真实音频播放。

后续修复：两类卡默认展示按顺序首个非空例句，并按该例句标记缺项；其余非空例句仍可展开。`pnpm lint`、`pnpm typecheck`、`pnpm build` 和固定 UTC 的 `pnpm test:e2e -- --workers=2` 通过（65 项）。浏览器测试使用模拟 IPC，本轮未重跑原生窗口或真实音频播放。

## TTS 模型生成的独立优化实验

2026-11-01：按用户要求脱离具体业务场景，使用现有 Supertonic 3 int8 / sherpa-onnx 2.0.1 / ONNX Runtime 1.28.2，在 Windows x64 / Ryzen 9 8945HS / 32 GB RAM 比较模型复用、推理线程、空转、共享并发、独立进程及合成步数。12 个性能目录、137 次配置轮次均在各自 60 秒硬限内完成；2,574 个 Ogg Opus 的身份、大小及元数据检查，2,538 个 PCM 的实际哈希及数值回读通过。2,124 个固定种子输出匹配可用串行参考（包括参考本身）；当前随机输出不作相等断言。7 项输入 / 坏资源失败检查通过，请求 CUDA 的现有包明确回退 CPU。

持续生成推荐一个常驻模型、2 推理线程、最多 12 路共享调用、关闭空转、保留 8 步。规范审查先复现并修正存活进程内存读取失败被忽略的问题，6 项回归通过；原始 122 轮保留历史采样限制，新增 15 轮使用失败即拒绝的采样确认。后者固定 24 词三轮中位数为 23.35 → 5.58 秒；完整进程为 25.85 → 9.84 秒，启动 / 预热收益单独解释。推荐组采样私有提交 / 工作集峰值分别为 1,220.2 / 600.9 MiB，提交量不等于驻留 RAM。12 路并非所有负载都严格最快，八进程本次热批量略快但私有提交约为共享推荐组的五倍；单条延迟优先的已测推荐是 4 线程串行。4 / 2 步分别有 1 / 11 个输入的 PCM 峰值超过满幅，人工音质未确认；没有采纳减少步数。

命令、逐轮时间、CPU / 同时采样内存、实际编码试听和限制见[模型生成实验](evidence/tts-generation/README.md)。本轮没有改应用 / IPC / 缓存 / 学习数据、增加依赖或替换运行库；没有重新运行 TS、Rust、产品原生 UI 或四平台发行验收，不扩大 #24 原有多硬件证据或既有离线 / 播放验收。

2026-11-01 追加单词缓存与 GPU 独立验证：公开词典 8,047 个合格单词中均匀抽取 128 个，男女声各生成一份，实际平均约 8.8 KB Ogg / 11.1 KB 本机文件分配量；单声线 1 万词约 88 MB 文件内容 / 111 MB 文件分配，两声线约 222 MB 文件分配，不含模型和文件系统元数据。应用仍只缓存单词，例句语音未实现，现有七天未访问清理策略未改。

隔离副本换入官方 sherpa-onnx 2.0.1 / ORT 1.28.2 CUDA 13 DLL 并提供固定 NVIDIA 运行库；RTX 4060 Laptop / 驱动 621.01 上四个子模型 profile 均出现真实 CUDA 节点，仍有 CPU 量化运算和复制。24 词、三轮交叉对照中，串行 CPU 28.19 → CUDA 12.85 秒，完整子进程 30.85 → 16.20 秒；12 路 CPU / CUDA 热批量均约 5.65 秒，GPU 没有稳定的额外批量吞吐收益，额外卷积参数亦无稳定优势。GPU 时间有波动，输出与 CPU 数值不同，人工音质未验收；实验 DLL 全量净增约 1.89 GB，不是最小产品依赖或最终包体。

38 个成功试验、893 个计量和 306 个预热 Ogg / PCM 的回读核验、504 个确认输出的重复一致性、CUDA 共享 / 串行对应哈希、发布归档和 NVIDIA 文件摘要通过；CPU-only runtime 请求 CUDA 的合法音频被 runner 明确拒绝为 GPU 成功，父 CLI 按预期退出 1。新增脚本自检及既有 6 项内存采样回归通过。完整记录见 [GPU 与缓存实测](evidence/tts-generation/gpu-cache.md)。本轮只新增实验脚本 / 证据并复用现有试验入口，未改变产品依赖、GPU 选择、缓存、学习数据或准入；TS、Rust、产品 UI、其他硬件 / 平台、首条冷启动 GPU 和正式发行未重跑。

## 普通主动式变位表达（#41）

2026-11-06：#41 后续补齐固定 Lefff 句法、助动词／代词专用准入、h/y 省音及有独立引用的罕见／历史词形。全部 7,826 个身份的 23 区块报告无缺失或未确认项：7,687 个普通动词准入，139 个仅保留源词形；过去复合时 7,681 个支持、145 个明确不可用。来源政策与不可用清单保留，不把单一词典遗漏视为语法缺陷。

22 项公开 CLI 测试、前端检查、锁定 Rust 编译／预编译后的 60 秒硬限后端执行、资源及工具测试通过。修正 `ayons chu` 多解释的测试选择器并补充罕见词形后，最终浏览器整套 105 项全部通过（54.9 秒）。Windows x64 发布构建及真实原生查找、23 区块、歧义高亮、性数备选、罕见／历史标签、明确不可用、资源失败与正常重启通过；学习数据、语音设置和选定音频字节保留。

完整来源、独立期待、命令和测量见 [#41 证据](evidence/issue-41/README.md)。数据库 123,875,328 字节仍低于 128 MiB 硬限；反查索引 17,252,352 字节超过旧 16 MiB 证据预算，完成资源使用显式 18 MiB 预算。最终首进程 CLI 1,737 ms 超过原 1,700 ms 预算，热测中位 1,453 ms；前版实测首进程曾达 3,297 ms，原测量一并保留；不将其标为全部延迟通过。依用户要求，本轮跳过新鲜离线包及其他 OS／架构验证，不宣布正式离线发行就绪。

## 离线英语释义（issue #42）

2026-11-06：FreeDict French-English 0.4.1 的实际固定来源、完整原文与 GPL-2.0-or-later 通知接入独立释义字段；动词标题与歧义候选均显示英语释义。7,826 个身份中 1,377 个有释义，共 1,953 义项；其余 6,449 个明确不可用，不影响变位表。普通／代词用法及来源义项独立保留，英语覆盖与 23 区块覆盖分别统计。

lint、类型检查、Windows x64 release／前端构建、Rust 格式／编译及预编译后的 138 项后端测试通过；后端执行保留 60 秒硬限。整套浏览器 105 项通过（50.6 秒），词典资源 7 项及构建／portable 工具检查通过；缺少独立验收的语音输入时跳过一项语音包集成检查。真实 Windows 原生窗口验证候选／标题释义、不可用状态、计数／导航、资源失败和正常重启，保留合成学习记录、进度、语音设置与选定音频字节。

24 项公开 CLI 测试通过（295.716 秒）；Git 暂存字节独立重建得到相同数据库。来源、结果及成本见 [#42 证据](evidence/issue-42/README.md)。数据库 124,735,488 字节低于 128 MiB 硬限；六次独立 CLI 进程均低于 1,700 ms 证据预算，首进程 1,642 ms，热测中位 1,645 ms，文件系统为热缓存。英语为部分覆盖。依用户要求跳过新鲜离线包及其他 OS／架构验收，不宣布正式离线发行就绪。

## 代词式表达与配合（issue #43）

2026-11-06：固定来源准入 2,955 个独立代词式身份，覆盖 2,954 个原形；7,826 个普通身份保留。适用 23 区块、人称限制和逐项排除报告无缺失／未确认项。主语配合、依宾语语境配合与不变分词分别展示；性数、vous／集合 on、en 和命令式备选可单独匹配。英语为独立部分覆盖：10,781 个身份中 1,493 个有释义，共 2,089 义项。

lint、类型检查、Windows x64 release／生产前端构建、Rust 格式／锁定编译及预编译后的 138 项后端测试通过；后端执行保留 60 秒硬限。整套浏览器 106 项通过（43.9 秒），词典资源 7 项及工具检查通过；一项需要独立语音资源的包集成检查跳过。真实 Windows 原生窗口验证普通／代词身份、语境标签、配合备选、23 区块、精确高亮、资源失败与正常重启，保留合成学习记录、两卡进度、语音偏好与三份音频字节。

30 项公开 CLI 测试通过（612.479 秒），Git 暂存字节导出 42 个输入独立重建得到相同数据库。数据库 250,650,624 字节低于 256 MiB 原生快照上限，反查索引 35,909,632 字节低于显式 36 MiB 证据预算；增加的存储成本来自可单独反查的完整代词式备选。来源、公开 CLI 结果及原有 1,700 ms 延迟预算实测见 [#43 证据](evidence/issue-43/README.md)。依用户要求跳过新鲜离线包及其他 OS／架构验收，不宣布正式离线发行就绪。

六次独立 CLI 进程均超过原 1,700 ms 预算：首进程 10,333 ms，后五次中位 2,485 ms、最大 2,546 ms。文件系统为热缓存，计时包含 CLI 的完整资源校验，不代表原生 IPC 延迟；此预算未通过，单独保留为性能限制。
