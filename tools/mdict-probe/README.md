# MDX/MDD compatibility probe

Independent Windows-tested resource reader for [issue #3](https://github.com/zardonc/lexique/issues/3). It extends the original MDX 1.2 probe with duplicate-preserving queries, paired MDD reads, wire checksum verification and finite read limits. It uses `mdictlib = 0.2.8` with `lzo` (`lzokay = 2.0.1`, MIT), and `serde_json = 1.0.154` (MIT OR Apache-2.0), locked in [Cargo.lock](Cargo.lock). The tool is a separate Cargo workspace.

```powershell
pnpm mdict:test:build
pnpm test:mdict
cargo fmt --manifest-path tools/mdict-probe/Cargo.toml --check
cargo clippy --locked --manifest-path tools/mdict-probe/Cargo.toml --all-targets -- -D warnings
cargo run --locked --release --manifest-path tools/mdict-probe/Cargo.toml -- tools/mdict-probe/tests/fixtures/v2-utf8-zlib.mdx être double missing --mdd tools/mdict-probe/tests/fixtures/v2-utf8-zlib.mdd '\theme.css' '\img\œil.png' '\audio\bonjour.wav' '\missing.png'
```

Arguments are `MDX [QUERY ...] [--mdd MDD [RESOURCE_KEY ...]]`. JSON reports header labels, physical entry counts, exact duplicate counts, decoded byte totals and every matched ordinal/key. `basis` distinguishes exact matches from the parser's header-normalized fallback. A missing key has an empty match list. All records and supplied MDD resources are read before a success report; payloads remain in memory. The command emits no dictionary body or media file. MDX and MDD errors exit nonzero without partial success output.

The shared test runner retains its hard 60-second execution deadline; compile first. CI separately checks this workspace's formatting, Clippy, compilation and tests.

## Read limits

Each file is limited to 128 MiB and 1,000,000 entries. Header XML is limited to 1 MiB; compressed and decompressed blocks to 16 MiB each; a materialized record to 8 MiB; parser working memory to 256 MiB per handle. Aggregate decoded MDX keys are limited to 64 MiB, and MDX bodies or MDD payloads to 512 MiB per file. Other parser ceilings retain `mdictlib` defaults. Checksums use `ChecksumPolicy::Verify` for both files. These are probe admission limits, not product performance budgets.

## Evidence and limits

The [compatibility report](../../docs/evidence/issue-3/README.md) records the real French corpus, licensed fixture pairs, failure checks and remaining product integration decisions. The tool does not render HTML, resolve redirect chains, play media, create indexes on disk, or read/write user learning data. Four-platform execution and production import remain separate work.
