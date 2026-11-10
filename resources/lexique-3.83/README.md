# Lexique 3.83 frequency resource

Original data: Boris New and Christophe Pallier, Lexique 3.83. Corpus contributors,
publications and the original version statement are preserved in
`UPSTREAM-README.txt`. The fixed source, file checksums and documentation commit
are in [provenance.json](provenance.json).

The data and derived SQLite database use **CC BY-SA 4.0**. The complete license and
warranty disclaimer are in [COPYING](COPYING). Attribution, source links and the
modification notice accompany the database. Application and converter code use
the separate MIT license, included as `LICENSE-MIT` in the prepared resource.

`Lexique383.tsv.gz` preserves the complete original TSV bytes. Conversion uses
`freqlemfilms2`, the film/television subtitle lemma/POS frequency per million
words. It groups by NFC lemma and original `cgram`, requires one consistent
finite nonnegative value per group, and adds each POS group once per lemma.
It retains all groups, including lemmas without an `islem=1` row. Missing,
invalid or conflicting groups yield an unknown total with per-POS reasons.
Eight source rows have no POS; their totals stay unknown and their original
frequency values are retained in the details.
Novel frequencies are not mixed into this metric.

The resource keeps original spellings and accents. Search keys use NFC and
lowercase. An exact canonical lemma uses its own statistics; a surface form
maps only when the source supplies one possible lemma. Ambiguous surface forms
remain unknown and preserve their candidates. The application thresholds
100/10 and the high/medium/low colors are product rules, independent of CEFR.

The resource contains no learning records, notes, cards, progress or audio.
It accompanies both the full and Lite packages for first-run offline lookup.
