# #14 检查记录

2026-10-26，Windows x64。所有命令仅改变本票独立输出及构建 / 测试生成物，未访问学习数据库。

| 命令 / 检查 | 结果 |
| --- | --- |
| CLI 回归，先于 `speech_probe.py` 实现 | 预期红灯：脚本不存在；随后实现通过 |
| `python -X utf8 -m unittest discover -s scripts -p 'test_*.py' -v`，配置 `LEXIQUE_SPEECH_BUNDLE` | 18 项通过，含真实模型 / 独立 Python / 禁网 CLI；无跳过 |
| `pnpm lint` / `pnpm typecheck` / `pnpm build` | 通过；构建有既有 Zod PURE 注释警告 |
| `pnpm rust:test:build` | 通过，编译独立于执行预算 |
| `pnpm test:rust` | 21 项通过，保持已有硬 60 秒超时；有既有增量缓存硬链接警告 |
| `pnpm test:scheduling` | 71 固定 + 4,096 校准向量原生 / 实际 WASM 严格一致 |
| `pnpm test:tooling` | 2 项 Node + 5 项 portable 检查通过 |
| `pnpm test:e2e`，Edge | 13 项通过；只证明既有浏览器行为未回归 |
| 17 样本禁网生成、WAV 回读、同步 Windows 播放 | 通过技术检查；孤立 y 已显式修正为 /i/，最终音质未确认 |
| 开发虚拟环境与独立 CPython 包重复生成 | y 修正前 17 项哈希相同；修正后仅 y 变化，其余16项相同，无缓存读取 |
| 完整 offline ZIP 构建 | 逐下载 SHA-256、逐文件 inventory、ZIP CRC 检查通过；最终体积另记录 |
| 微软 VC++ 离线安装器 | Authenticode Valid，签名者 Microsoft Corporation；未执行安装 |

code-review 技能分别审查规范与 #14 规格，发现前端可能被环境变量覆盖 / 系统回退、
文档误写 510 音素上限。前者新增 CLI 红绿回归并固定 DLL / 数据树 / 版本，后者修正为
上游分块行为；两项复审均已解决，无残留可执行发现。当前音质阻塞不算技术测试通过。
用户随后要求先修正 y：真实 CLI 回归先复现 `iɡʁˈɛk != i`，再以准确小写词汇输入的
显式发音修正得到绿灯；保留前端原音素、修正标记及前后试听样本。

环境问题及修正：

- Python 的系统 Store 别名不可用，改用既有 Python 3.12.13；交付包使用固定官方 CPython 3.12.10 embed。
- 沙箱默认 TEMP 子目录不允许 phonemizer 创建文件；仅给本票进程设置 TEMP / TMP 到 `.tooling/issue-14` 后复跑。
- 初次 `pnpm typecheck` 受 Corepack 下载的 DNS 限制而失败；使用本票 `.tooling/corepack` 缓存准备锁定 pnpm 后通过。
- 直接 `node --test scripts/test-build-desktop.mjs` 缺少 pnpm 上下文而失败；按测试要求运行 `pnpm test:tooling` 后通过。
- 沙箱内浏览器 E2E 无法完成本地服务握手，终止该次任务后在普通进程权限下重跑通过。
  此重跑只用于浏览器回归；语音离线验收始终在禁网受限进程内。
- CIM 查询被拒绝；CPU 名称从只读处理器注册表、物理内存从 GlobalMemoryStatusEx 取得。
  注册表枚举同时有类型转换诊断，未将查询失败解读为硬件缺失。

未执行：远端 CI、三种其他平台运行、干净机器 VC++ 安装、缺 WebView2 首装、桌面语音 IPC / UI、
主观音质认可、完整四目标资源包、签名 / 公证 / M5。未接受 Pocket 访问条件，也未下载三套候选。
进程均同步结束；Playwright 管理自己的 Vite 服务。没有改变系统网络、安装依赖或停止无关服务。
