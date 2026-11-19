# #98 bare-l annotation evidence

The owner approved frozen rows **859 `l/ART:def`** and **860 `l/PRO:per`** as
zero-field annotation exceptions on **2026-11-18, America/Vancouver**.
The exact approval reply, effect, date and source pointer are in
[owner-approval.json](owner-approval.json); the preserved records are in
[annotations.json](annotations.json).

Lexique 3.83 separately attests bare `l/ART:def` at TSV line **75462** and
bare `l/PRO:per` at line **75464**, plus apostrophized `l'/ART:def` at line
**75466** and `l'/PRO:per` at line **75468**. Fixed French and English bare-`l`
entries provide a French letter/symbol, while the French `l'` entry redirects
to `l’`, whose article and personal-pronoun definitions retain the apostrophe.
The reason for the bare-form annotations is unproved. The precise selected
source excerpts, fixed URLs and original TSV records are in
[literal-l-evidence.json](literal-l-evidence.json).

All 50 frozen identities and both frequency strings remain exact. Each annotation
has zero senses, zero admitted fields and four missing fields; annotations are
counted separately from content and retain the 4,087-group denominator.
There is no new alias. Existing `l'` resource records and the literal-`l`
negative Rust regression in `crates/dictionary/tests/core_pilot.rs:213` are unchanged.
Other 48 groups have proposed four-field content pending independent acceptance.
