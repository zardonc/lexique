# 原创查词夹具

本目录的正文均为项目原创测试文字，采用仓库 [MIT 许可](../../../../../LICENSE)。`verbs.mdx` 与更新版覆盖普通／代词式原形、同形名词及重复记录；`other.mdx` 用于多词典切换。`response-count.mdx` 含 201 个同键物理记录；`response-bytes.mdx` 含两份超过合计 4 MiB 的正文，用于明确的响应上限检查。

文件由 [make-fixtures.py](../../../../../docs/evidence/issue-9/make-fixtures.py) 使用 [#3 已固定的独立 mdict-utils writer](../../../../../tools/mdict-probe/tests/fixtures/README.md) 创建，格式为 v2／UTF-8／zlib，日期固定 2026-11-09。解析器未参与生成。按 #3 获取固定 writer 后，可生成到新目录：

```powershell
python -X utf8 docs/evidence/issue-9/make-fixtures.py --writer-parent target/mdict-fixture-source --output target/issue-9/fixtures-recreated
```
