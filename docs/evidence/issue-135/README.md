# D2 issue #135: author candidates and review handoff

**50 exact groups, 50 selected senses and 200 four-field author candidates** cover
frozen rows 2701–2750 from [issue #135](https://github.com/zardonc/lexique/issues/135)
under [parent #78](https://github.com/zardonc/lexique/issues/78). All proposed
fields have fixed source spans, applicable article attribution and rights
proposals. **Zero new groups or fields are independently accepted or admitted.**
Independent quality, selected-span rights, combined conversion/packaging and
native lookup/save/reopen remain pending.

## Content and coverage

[Groups](groups.json) retain exact spellings, POS subclasses, frozen order and
both manifest frequency strings. There is no reuse, substitution, new exception
or lookup alias. Each selected sense has FR/IPA/EN/ZH for its explicit use;
CEFR remains unknown. English and Chinese are Lexique-authored explanations,
not quotations from a bilingual dictionary. [Original author proposals](authored/)
retain the wording and pending decisions for a separate reviewer.

| Batch measure | Count |
| --- | ---: |
| Frozen groups / distinct headwords | 50 / 49 |
| Proposed senses / complete authored fields | 50 / 200 |
| Missing authored fields | 0 |
| Independently accepted new groups / fields | 0 / 0 |
| Pending quality fields / pending selected-rights fields | 200 / 200 |
| Missing admitted batch fields | 200 |
| Fixed global denominator | 4,087 |

The unchanged `core-129-v2` baseline has 2,435 admitted groups / 9,740 fields,
leaving 1,652 groups without admitted content against that denominator. These
proposals do not increase its counters. The existing 15 zero-field annotations,
two specialist approvals, 15 spelling relations / 11 aliases and learner data
remain under their existing records and gates.

`auquel/PRO:int` and `auquel/PRO:rel` remain separate masculine-singular à + lequel
uses, with separate source sections and agreement tables. `hauteur` retains
aspirated h; `dégueulasse` retains vulgar register; `abuser` selects the informal
intransitive “go too far” use. Pronunciations select one recorded variant;
variants remain visible in the captures. `étonnant` uses its adjective agreement
template for IPA rather than borrowing its verb section.

## Source and rights package

[Capture inventory](source-captures.json), [source identities](sources.json),
[context and contributors](source-context.json), [exact selected excerpts](source-excerpts.json),
[field/use/spelling proposals](field-review.json), [rights proposals](rights.json),
[selected-rights scope](selected-rights.md), [notices](NOTICES.md) and [legal text](COPYING)
form the author package. There are 51 fixed French articles: 49 ticket headwords
plus `ciel` and `doué` as support. Six primary and one support contributor API
responses retain complete continuation chains, 3,037 article-credit name entries
and the returned anonymous counts. Those are article credits, not proof of
selected-sentence authorship.

Retained source/API/request files, author recipes and check logs are under
`C:/Users/inthe/IdeaProjects/lexique-spec-130-139/ticket-135/target/issue-135`.
The complete owned archive is
`C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/author-evidence/issue-135`.
These research captures include unselected text and are not learner-content
redistribution. The admission record supplies both roots; the focused check can
read archived captures when its original target directory is absent.

The package reuses unchanged licence/policy captures from the retained #129
author archive, with their actual version/time and reuse origin. Policy raw text
is materialized directly from the retained revision API. The CFC import-template
capture is new and versioned. No review identity or accepted decision is invented.

## Verification and reproduction

The approved resource seam detects missing membership, mismatched POS/source
frames, empty or misbound fields, incomplete contributor pagination and false
acceptance. The initial canonical check found none of the 50 groups admitted;
its expected failure is retained in `target/issue-135/RED-admission.log`.
Author resource checks and exact byte reproduction are recorded in
[final verification](final-verification.json).

```powershell
uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_135.py
uv run --python 3.14.10 python -B -X utf8 target/issue-135/reproduce.py
pnpm test:docs
git diff --check
```

`reproduce.py` uses fixed captures and compares the complete authored JSON and
notices byte-for-byte. It writes author output: run only in an author reproduction
checkout before accepted replacements exist. Restore archived helpers under
that checkout’s own `target/issue-135` first; running them directly from the
external archive would resolve the wrong repository root. No fresh source
fetch is needed or used for reproduction.

The additional `--admitted` test mode checks the canonical resource for all 50
groups. It remains an expected failure until independent review and combined
admission; a passing author check does not turn that gate green.

## Pending gates and review attention

The independent reviewer must assess the actual FR/EN/ZH language, chosen IPA,
source/import boundaries, register, construction and everyday-use eligibility
for every field. In particular, `douer/VER` is restricted to natural/providential
endowment, with participial support; that support does not substitute an adjective
for the frozen verb. `cieux/NOM` proposes figurative lands/climates through the
fixed `ciel` definition, exact plural relation and `cieux` pronunciation. Their
eligibility remains explicit review attention, with no owner exemption assumed.

The coordinator owns accepted replacements, combined resource/notices, converter
output and sizes, a matching native executable, every-group lookup/trusted save,
normal restart/reopen, and prior learning-state preservation. No native execution,
assembled resource, learner acceptance, whole-machine offline result, other
platform validation or release readiness is claimed by this package.
