# #66：词典例句摘录与逐句出处

实现：当前词典来源在摘录开始时固定，用户手动粘贴法语和可用中文，明确加入共享编辑草稿并保存。空主例句优先，部分／完整主例句保持；更多例句最多 100 组，各段最多 20,000 Unicode 字符。三项相同定位既有项，译文或完整出处不同保留。逐句来源和原文重开在编辑器及两类卡背共用；旧未知来源保持未知。

schema 18 非破坏增加主例句出处，更多例句保留可选来源；保存仍使用原有事务、修订和操作回放。新出处由原生精确词典查询构造并核对，已保存出处不要求重新打开资源。

## 验证

| 命令 | 结果 |
| --- | --- |
| `pnpm lint`、`pnpm typecheck`、`pnpm build` | passed |
| `pnpm test:e2e notebook example-speech example-excerpts dictionary-tabs --workers=4` | passed，41 项（包含 7 项摘录检查） |
| `pnpm test:e2e example-excerpts --workers=2` | passed，8 项，包含新词明确保存及保持收藏偏好 |
| `pnpm rust:test:build` | passed，先单独编译 |
| `pnpm test:rust` | passed，保留硬性 60 秒执行上限 |
| `pnpm rust:fmt`、`pnpm rust:check` | passed |

存储检查包含 schema 17 接入后的未知来源、来源参与保存操作身份及两个来源的文件关闭后回读。界面检查包含空／部分／完整主例句、追加、重复焦点、不同译文／来源、词典切换、Unicode 上限、容量、失败保留及既有独立笔记的离开保护。

## 原生执行入口

从 `apps/desktop` 执行：

```powershell
node e2e/native-example-excerpts-check.mjs <speech-package> <fresh-executable> <new-target/issue-66/destination>
```

该脚本仅创建独立目录和合成数据，使用真实 IPC／SQLite；从新可执行文件旁复制配套词典，构建已有卡片、评分历史和选定语音。接着降为 schema 17 后重启、执行两个词典的摘录与明确保存／重复定位、拒绝伪造新出处、修订冲突及旧操作回放；关闭后移除资源，再次打开／编辑／保存及重启回读，并比较学习与选定语音表。执行阶段保留既有 60 秒硬上限。

集成分支 Windows x64 使用 `pnpm desktop:build --no-bundle` 新构建，以上入口以既有 `target/pr23-native-speech-20261101` 语音资源和全新 `target/issue-66/native-1` 执行：passed。schema 17 升级、两词典来源、明确保存／重复定位、伪造出处拒绝、修订冲突、旧操作回放、资源移除后的编辑和多次正常退出重启回读均通过；双卡、评分历史、进度及选定音频的真实 SQLite 快照保持。完整日志和结果在忽略的 `target/`。

完整集成浏览器 E2E：`pnpm test:e2e`，225 passed。同一新构建以 `target/issue-66/native-combined-2` 再次执行上述入口：passed，新增「摘录 → 明确保存 → 共用入口实际播放」检查；播放前后保存内容及学习快照保持。schema 18 的朗读缓存／缺模型检查也以 `target/issue-65/native-combined` 通过。

原生检查使用 WebView offline 和本地合成；本次未执行整机禁网、其他平台实机或正式发行验收。
