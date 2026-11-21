# Issue #128 independent field and selected-rights review

**50/50 modern selected uses and 200/200 fields accepted; 197 unchanged and
three exact construction replacements; zero unresolved independent fields.**
All 189 selected language/POS/form/construction frames and 47 source contexts
pass at their selected boundaries. No annotation, specialist sense or runtime
alias is admitted. Frozen rows 2351–2400, spelling, POS subclass, frequency
strings and order remain; the denominator is 4,087 groups.

Reviewer: **Independent semantic and selected-rights reviewer #128**, separate
from the content author and parent integration owner. This is independent
content/rights review; human learner acceptance remains separate.

Reviewed author commit: `b70a64eac359b7da97ab7192816e207843d6a056`,
[PR #193](https://github.com/zardonc/lexique/pull/193),
[issue #128](https://github.com/zardonc/lexique/issues/128), parent
[#78](https://github.com/zardonc/lexique/issues/78). Review checkout:
`C:/Users/inthe/IdeaProjects/lexique-review-128`, `codex/d2-review-128`.
Clean integration `55971e6301c930cb381a6ef67349eece0de35269` was merged before
review; the latest integration is merged before final reporting, with its exact
SHA in the [verification record](../../../target/issue-128-review/verification.json).
The reviewed changes are committed author content plus the owned review diff;
the retained [review patch](../../../target/issue-128-review/review.patch) and
[test patch](../../../target/issue-128-review/tests.patch) identify that diff.

The captured spec and issue were read at
`C:/Users/inthe/IdeaProjects/lexique-spec-120-129/target/implement-120-129/`.
The independently reviewed [#119 schema](../issue-119/independent-review.md)
and accepted #123 field/rights evidence at
`C:/Users/inthe/IdeaProjects/lexique-review-123/docs/evidence/issue-123/`
were used as review precedents, with actual #128 boundaries judged separately.

## Field and use decisions

Every actual FR/IPA/EN/ZH proposal was read against its fixed French lexical
definition, applicable pronunciation, POS/form frame, constructions and
adjacent senses. [Field review](field-review.json) is
`separate_independent_field_review`, with exact proposals, accepted texts,
individual semantic/rights reasons, 50 use decisions and 50 spelling decisions.
[Original candidates](review-candidates.json) preserve all authored fields and
pending states. [Authored admission](admission-author-proposal.json) preserves
the original zero independent acceptance; the [live ledger](admission.json)
records 50 eligible groups / 200 eligible fields and zero independent missing
or pending fields. Canonical admission remains 0 newly admitted / 200 pending
fields until parent conversion, notices and native execution pass.

For row **2375 `sacrer/VER`**, the three meaning fields add the attested
intransitive/`contre` construction from fixed definition line **26**. The IPA
`/sa.kʁe/` is unchanged. [Exact replacements](accepted-replacements.json)
retain both the proposals and complete accepted wording:

| Field | Added construction |
| --- | --- |
| French | S’emploie sans complément ou avec sacrer contre quelqu’un ou quelque chose. |
| English | Used without an object or with sacrer contre quelqu’un/quelque chose (to swear at someone or something). |
| Chinese | 不接宾语，或用 sacrer contre quelqu’un/quelque chose（咒骂某人或某事）。 |

The contemporary selected route is explicitly Québec; the source's alternate
`vieilli` route outside it does not become an unqualified everyday sense.
`lutter contre`, `digne de` + noun/infinitive and `satisfaire à` remain explicit
in all three meaning fields. Lowercase noun `japonais` is the language with SOV
order, separate from its adjective and capitalized person noun. Noun `géant`
is a very tall person; noun `original` is the exemplar for copies. `pêche` is
the peach fruit, `donne` is card dealing, and `figure` is the human face.
These scoped common uses do not require exceptions or additional senses.

All 50 isolated IPA fields fit their exact source spelling and form. Recorded
variants for `jouet`, `moto`, `pressé`, `passeport` and `élevé` are not combined
into unsupported pronunciations. `vif` uses only the phonetic substring at
line 308, bound to the French adjective frame. [Selected rights](selected-rights.md)
records the original-work/import, media, quotation and attribution boundaries.

## Verification and retained evidence

The approved resource/admission seam is exercised by
[the focused tests](../../../scripts/test_core_batch_128.py). They verify all
50 exact frozen identities and frequencies, all 200 accepted texts and rights,
the original proposals, the actual live counts, all source frames, and the
selected construction/IPA boundaries. Canonical overlap accepts identical
records, including the converter's normalization of the sense acceptance-state
spelling, and rejects differing content or provenance. Parent canonical counts
can advance to completion without freezing authored pending counts.

- Construction [RED](../../../target/issue-128-review/construction-red.log)
  and [GREEN](../../../target/issue-128-review/construction-green.log).
- Independent admission [RED](../../../target/issue-128-review/admission-red.log)
  and [GREEN](../../../target/issue-128-review/admission-green.log).
- Field admission states [RED](../../../target/issue-128-review/field-state-red.log)
  and [GREEN](../../../target/issue-128-review/field-state-green.log).
- [Full GREEN](../../../target/issue-128-review/full-green.log): four tests.
- [Fixed-source byte audit](../../../target/issue-128-review/fixed-source-byte-audit.log):
  all 47 complete raw articles equal both current-capture and exact-revision
  recapture API bytes; all 200 fields and 189 support lines match; six
  contributor continuation chains end completely; five legal captures exist.
- [Isolated overlap/negative checks](../../../target/issue-128-review/overlap-and-mutations.log):
  50 identical overlaps pass; changed canonical text/span, removed construction,
  pending field rights, pending support frame, pending original-work layer and
  false pending counts fail; later canonical completion counts pass.
- [Documentation checks](../../../target/issue-128-review/docs-check.log) and
  [diff checks](../../../target/issue-128-review/diff-check.log).

Complete evidence remains under ignored `target/issue-128-review/`. Raw/current/
fixed API captures and rights authorities are retained in the specified author
archive, `C:/Users/inthe/IdeaProjects/lexique-batch-128/target/issue-128`;
the integration `target/issue-128` is the alternate archive. No live requests or
hot retries were made; retained exact captures respect the upstream 429 limit.

## Remaining gates

Canonical resource assembly, actual combined packaged notices and obligations,
native lookup/save/reopen, learner acceptance, offline/platform execution and
formal release readiness remain parent-owned and were not run here. Only
`docs/evidence/issue-128/**`, `scripts/test_core_batch_128.py` and ignored
`target/issue-128-review/**` were changed for this review.
