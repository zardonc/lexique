# Issue #148 independent review

**50 groups / 50 everyday uses / 200 fields accepted.** There are **199 unchanged
fields, one Chinese replacement and zero unresolved fields or uses**. Every
French definition, IPA, English explanation, Chinese explanation and selected
use was independently inspected against the retained fixed source text.
Frozen rows 3351–3400, exact headwords/POS subclasses, frequencies and order
are preserved. No new annotation, specialist exception or runtime alias is
admitted; the fixed denominator remains 4,087 groups.

Reviewer: **Independent field/use/selected-rights reviewer #148
(`codex/d2-review-148`), separate from the content author and coordinator**.
This is independent agent review; learner acceptance remains a separate gate.
[Issue #148](https://github.com/zardonc/lexique/issues/148) and
[parent #78](https://github.com/zardonc/lexique/issues/78) were read from their
exact cached JSON in the current context, alongside the reviewer guide,
pipeline notes, repository guidance, glossary/resource ADRs and TDD skill.

## Review inputs

- Author head: `088cb317482b218b9827913bdc82382daed9a7b3`.
- Fresh review base: `b85dfe73f9db3285588d7349d18f0a3922e33da6`.
- Integration merged before delivery: `9bcc0d6f30e5e3c9aeea7076795e497510f38a26`.
- Checkout: `C:/Users/inthe/IdeaProjects/lexique-d2-140-149/review-148`.
- Branch: `codex/d2-review-148`.
- Author captures/helpers: `C:/Users/inthe/IdeaProjects/lexique-d2-140-149/context/author-evidence/issue-148`.
- Full independent audit archive: `C:/Users/inthe/IdeaProjects/lexique-d2-140-149/context/reviewer-evidence/issue-148`.
- Write scope: `docs/evidence/issue-148/**` and `scripts/test_core_batch_148.py`.

[Authored snapshots](authored/snapshot.json) preserve the exact original JSON,
prose, notices and licence before accepted edits. Prose snapshots use `.md.txt`
to preserve their original relative-link bytes. [Field review](field-review.json)
retains all 200 exact proposed/accepted texts, source spans, independent reasons
and distinct quality/rights decisions. All 50 usage and spelling records have
specific construction, gender, inflection and everyday-use reasons.

## Accepted content

| Field | Accepted text | Fixed selected basis |
| --- | --- | --- |
| 3356 `tonton/NOM`, Chinese | 叔叔、伯伯、舅舅、姑父或姨父的儿童亲昵称呼（阳性名词）。 | French `tonton`, revision 38674954, line 10: child-language `oncle`; noun/IPA at 9 |

The [replacement record](accepted-replacements.json) retains the proposal,
accepted text, exact source and review reason. Other fields and all selected
uses remain authored as proposed.

`petit-déjeuner` and `etc` retain their exact frozen spellings with evidenced
noun/adverb source relations. `yeah/ONO` uses the actual French interjection,
with `/jɛ/`. `écrouler/VER` retains exclusively pronominal `s’écrouler` and
isolated infinitive IPA. `brute` selects printed standard `/bʁyt/` separately
from the inaccurate noun-head notation. `bouleverser` retains painful emotion,
`football` the goalkeeper exception, `insecte` adult anatomy, and `relax` both
relaxed and relaxing with invariable agreement. `pervers/NOM` retains an
attributed evaluative sexuality use; neighbouring religion, psychology and
PNL assertions supply no field.

Common body-part, material, aircraft, stage-magic and game vocabulary remains
ordinary content. Source thematic labels do not authorize specialist exceptions,
and no exception is required for these selected everyday uses. No pending owner
choice was identified.

## Source and selected-rights audit

All **53 fixed article/API UTF-8 byte pairs**, **200 selected field spans** and
**206 supporting-span occurrences** passed. Exact French language, POS, head,
pronunciation and construction frames were inspected. **12 completed contributor
chains / 12 responses**, **3,193 named article credits** and **646 anonymous
counts** match the records. Registries and complete history links provide article
attribution; they do not identify the author of each selected sentence.

Thirty-one DAF8 import notices retain the original institutional collective-work
credit and established **2006-01-01** expiry, separately from CC BY-SA 4.0 Wiki
transcription and Lexique adaptation/translation layers. Exact source, imported
original-expression and excluded quotation/reference decisions are in
[rights.json](rights.json), [source context](source-context.json) and
[selected-rights.md](selected-rights.md). Retained policy/template text matches
its API after newline normalization; article bytes were compared literally.
No fresh source request was needed.

## Verification and limits

Authorized seam: resource/admission records against exact frozen membership,
independently reviewed content, fixed source spans and selected rights.

```powershell
uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_148.py
uv run --python 3.14.10 python -B -X utf8 target/issue-148-review/audit.py
pnpm test:docs
git diff --check
```

The focused Chinese regression failed before its correction. The separate
review-admission check failed on author-only proposals before accepted records
were installed. Seven focused tests pass on the reviewed shard, checking exact
membership/frequencies, source bytes/spans, complete contributor chains,
spelling/pronominal boundaries, original proposals, and all 200 accepted
field/source/rights records and 50 uses. Full RED/GREEN, source, documentation,
diff and integration-merge logs are retained in the archive with the actual
review commit and command results in `final-verification.json`.

Canonical conversion/assembly, actual combined packaged obligations, all-group
native lookup/save/reopen, learner acceptance, global coverage/size and remaining
offline/platform/release gates are coordinator-owned and unrun here. Frontend,
Rust and browser checks are unrun because no such implementation changed.
No services were started. No push, PR or tracker operation was performed.
