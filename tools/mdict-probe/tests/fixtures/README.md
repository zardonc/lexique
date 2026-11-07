# Original MDX/MDD test fixtures

All dictionary text, CSS, the one-pixel PNG and the short PCM WAV are original project test data under the repository's [MIT licence](../../../../LICENSE). Every MDX variant has ten physical records, including two distinct `double` records, three accent variants, two apostrophe spellings, `œil`, a raw redirect and an entry referencing the three paired resources. Each matching MDD has those three exact resource byte sequences. The WAV is a test signal.

[manifest.json](manifest.json) records file sizes, hashes and format parameters. Six pairs cover v1.2 UTF-8/UTF-16 uncompressed, v2 UTF-8 uncompressed, v2 UTF-8/UTF-16 zlib and v2 UTF-8 with encrypted indexes. The MDD key encoding is always UTF-16LE, as required by the format. `unsupported-v3-header.mdx` has a valid checksum and v3 header declarations over a v2 payload; it tests header rejection and is not a real v3 compatibility sample.

The independent writer is [mdict-utils](https://github.com/liuyug/mdict-utils/tree/64e15b99aca786dbf65e5a2274f85547f8029f2e), MIT, at commit `64e15b99aca786dbf65e5a2274f85547f8029f2e`; its [licence](https://github.com/liuyug/mdict-utils/blob/64e15b99aca786dbf65e5a2274f85547f8029f2e/LICENSE) applies to the writer. The parser under test did not create these fixtures. Writer source stays in an ignored build directory; ordinary tests use only the checked-in fixtures. The generator fixes the header date to 2026-11-07 and refuses to overwrite existing paired files.

To fetch the pinned writer and regenerate into a fresh directory from the repository root:

```powershell
$writerParent = 'target/mdict-fixture-source'
$writerDirectory = Join-Path $writerParent 'writer_support'
New-Item -ItemType Directory -Path $writerDirectory -Force | Out-Null
foreach ($writerFile in @('writemdict.py', 'ripemd128.py', 'pureSalsa20.py')) {
    $writerContent = gh api "repos/liuyug/mdict-utils/contents/mdict_utils/base/${writerFile}?ref=64e15b99aca786dbf65e5a2274f85547f8029f2e" | ConvertFrom-Json
    if ($LASTEXITCODE -ne 0) { throw 'Cannot fetch pinned writer' }
    [IO.File]::WriteAllBytes((Join-Path $writerDirectory $writerFile), [Convert]::FromBase64String($writerContent.content))
}
uv run --python 3.14 python -X utf8 tools/mdict-probe/tests/make_fixtures.py --writer-parent $writerParent --output target/mdict-fixtures-recreated
```

Original third-party dictionary files and text are excluded from these fixtures. The [corpus evidence](../../../../docs/evidence/issue-3/README.md) records their separate authorization status.
