# Compact conjugation storage

The accepted refactor keeps the conjugation lookup and resource CLI interfaces while replacing format 5's materialized cells with shared simple-form suffix models. The original format-5 database and report remain in `target/conjugation-refactor/baseline-v5/` for exhaustive equivalence checks; resource preparation uses a new format-6 directory and never changes learning data.

## Representation

- `models(id, cells)` stores an ordered JSON array of simple-cell tuples: `[cut, suffix, raw_tag, tense, person, number, gender]`. `cut` counts Unicode scalars removed from the lemma; the remaining prefix plus `suffix` reproduces the original form. Templates retain variants, duplicate source interpretations and original insertion order. Storage suffixes are not linguistic ending annotations for #45.
- `verbs(lemma, base, model, source_lines, information, meanings, h_aspire, slots)` stores one record per ordinary or pronominal identity. Ordinary identities own a model and little-endian unsigned 32-bit source-line anchors aligned with its cells. Pronominal identities refer to the ordinary `base`, have a null model and empty anchors. `information` is UTF-8 JSON compressed with zlib; `meanings` remains independent JSON. The pinned metadata and block statuses retain their existing response shape. `slots` counts logically generated cells, not physical rows.
- `stems(stem, model, cut, lemma)` is a `WITHOUT ROWID` table with that composite primary key. It stores distinct lemma prefixes for each model's cuts. Reverse lookup strips supported model suffixes, finds compatible stems, expands candidate ordinary/pronominal identities and verifies the complete original query. It stores no surface-form or compound-expression index.
- `metadata(provenance, slots, supplement, verbs, english, construction)` retains the admitted source identities/counts and the simple auxiliary forms used by the existing construction rules. `verbs_base` is the only additional identity index.

Compounds, gerunds, subjects, reflexive expressions and gender/number alternatives are constructed from the existing admitted rules at lookup time. Auxiliary forms come from the pinned Grammalecte patterns. Lefff source cells, explicit corrections, use labels, English senses and pronominal restrictions are preserved; this is a representation change, not a new admission policy.

## Exactness and tests

Preserve every cell field, source-line anchor, expression alternative, table metadata, logical interpretation and stable ordering. Source lines are provenance rather than unique cell identities. Stable table ordering remains `tense, person, number, gender, source_line`, with nulls first and original insertion order breaking ties. Matches retain the original CLI ordering; final candidate acceptance compares the full query with a generated form or expression, never a token alone.

Tests use the public resource CLI and native conjugation lookup. The requested full-corpus comparison regenerates all 1,451,539 format-5 cells and compares them with the preserved database, including duplicates and ordered expression arrays. Focused native tests independently verify construction, complete-expression candidate lookup, ambiguity, ordering and resource failures. Existing browser/native checks cover the unchanged desktop interface and learning-data preservation. Backend compilation remains separate from the runner's hard 60-second execution limit.

## Integration

The format-6 build and native corpus comparison reproduce all 1,451,539 cells and 10,781 identities exactly. The database is 10,969,088 bytes (10.46 MiB), with 170 models and 24,307 stem references; the stem table occupies 733,184 bytes. Fresh native IPC measured 33.8 ms first and 33.3 ms warm median. Fresh CLI processes measured 1730.9 ms first and 1457.5 ms warm median; the first process misses the existing 1,700 ms budget. Commands, the matched TTS development package, executed identities and limitations are recorded in the [verification report](evidence/conjugation-storage/README.md).

Update preparation, native resource reading, release copying, portable validation, fixtures and resource reports together. Keep original resources, separate licenses and corresponding converter source in packages. Format 6 with typographic query support is prepared in `target/desktop-conjugation-v6-query/`; the original `target/desktop-conjugation-v6/` remains intact with its corresponding converters. Older formats remain available as evidence and are rejected by the new runtime rather than silently migrated.

The refactor precedes #44–#49. [#44 query normalization](evidence/issue-44/README.md) applies consistently before suffix candidate lookup and full-expression verification while preserving the format-6 database bytes and identity contract. #45 must supply independently validated linguistic ending spans rather than interpreting storage suffixes as endings. #49 must use and identify the compact resource in combined fresh offline acceptance. Record actual resource/index/package sizes and cold/warm timings; earlier format-5 cost evidence remains historical.
