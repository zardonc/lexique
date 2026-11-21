# Issue #122 independent field and selected-use review

**50 uses / 200 fields accepted: 193 unchanged fields, seven exact field
replacements, one exact use replacement and zero unresolved fields.** All
frozen rows 2051–2100 retain exact headword/POS, frequencies and ordering.
Zero reuse, annotation exceptions or runtime aliases; denominator **4,087 groups**.

Reviewer: Independent semantic and selected-rights reviewer #122, separate from
the content author and parent integration owner. This separate review session
provides independent content evidence; human learner acceptance is a parent gate.

Inputs: [#122](https://github.com/zardonc/lexique/issues/122), full
[#78 specification](https://github.com/zardonc/lexique/issues/78), current
AGENTS.md, DEV-SCOPE.md, LOCAL-DEV-ENV.md, CODING_STANDARDS.md, product/roadmap,
resource-admission guidance and resource/licensing ADRs. Captured spec/ticket
root: `C:/Users/inthe/IdeaProjects/lexique-spec-120-129/target/implement-120-129/`.
The spec Testing Decisions approve the resource/admission seam used here.

Review checkout: `C:/Users/inthe/IdeaProjects/lexique-review-122`, branch
`codex/d2-review-122`. Authored package: `435b0000b431bf18fe21a05752523242a0dc7750`;
final author head `5d12853`; review diff base
`7937180a4ab592cb4d0e8f0e7af4cffd80cb74e8`.
The review covers committed changes in `docs/evidence/issue-122/**` and
`scripts/test_core_batch_122.py` only. Ignored reviewer logs are retained locally.

## Field and use decisions

Every actual FR/IPA/EN/ZH proposal was read against its fixed French lexical
definition, language/POS frame, isolated pronunciation, adjacent meanings and
construction/spelling evidence. Per-field and per-use independent reasons,
accepted text, original proposal and exact fixed span appear in
[field-review.json](field-review.json); the original group objects remain
unchanged in [review-candidates.json](review-candidates.json).

| Row/use | Exact accepted change | Reason |
| --- | --- | --- |
| 2080 `mince/ONO`, EN | `Darn! A euphemistic exclamation used instead of merde! (shit!).` | Grammatical indefinite article before the consonant sound in euphemistic |
| 2096 `race/NOM`, FR | `En élevage, subdivision d’animaux d’une même espèce ou sous-espèce, obtenue en lignée pure et définie par des caractères morphologiques ou des performances économiques.` | Selected source line 72 qualifies performances as économiques |
| 2096 `race/NOM`, EN | `In animal breeding, a breed: a subdivision within a species or subspecies, obtained as a pure lineage and defined by physical characteristics or economic performance.` | Same economic-performance criterion |
| 2096 `race/NOM`, ZH | `在动物育种中，品种：同一物种或亚种内，经纯系繁育形成、按形态特征或经济性能界定的动物群体。` | Same economic-performance criterion |
| 2100 `reçu/NOM`, FR | `Quittance sous seing privé par laquelle on reconnaît avoir reçu une somme d’argent.` | Source line 26 requires private signature |
| 2100 `reçu/NOM`, EN | `A privately signed receipt acknowledging that a sum of money has been received.` | Same private-signature restriction |
| 2100 `reçu/NOM`, ZH | `私人签署的收据；书面确认已收到一笔钱款的凭证。` | Same private-signature restriction |
| 2100 `reçu/NOM`, use | `Masculine noun; privately signed written acknowledgement of money received.` | Selected use states the same restriction as all meaning fields |

The other 193 fields and 49 uses are correct for their selected scope.
`bel` and `nouvel` retain the masculine-singular pre-noun vowel/silent-h
construction and their own IPA; `allo` and `revolver` have exact French
alternative-spelling records. These are source bindings, with no runtime aliases.
`mince/ONO` uses the article-wide pronunciation record, not an adjective meaning.
`assister` selects transitive help rather than `assister à` attendance; only
the IPA substring in the pronunciation template is imported. `venu/ADJ`
retains qualified development/production use, with the adjective agreement
table as its phonetic source. `race` selects the everyday animal-breed use
with its stated criteria. Register and grammatical restrictions remain explicit.

## Standards

Owned scope, frozen identities and original proposals are preserved. Tests
exercise the approved admission boundary using actual reviewed records and raw
captures. Canonical overlap compares already installed content rather than
requiring the shard to be absent. No shared/native code, dependency or learner
data changes are part of this review. No remaining standards finding was found.

## Spec

All 50 uses and 200 fields have independent quality and selected-rights decisions;
seven fields and one use now retain the accepted grammar/restrictions. The live
ledger has zero independent pending or missing fields. The authored zero-accepted,
200-pending ledger is retained in [author-admission.json](author-admission.json).
Canonical conversion/package and native acceptance remain separate parent gates.

## Verification

Six focused tests pass. Each of the three corrections and independent acceptance
has a retained RED/GREEN slice at the approved seam. Fixed-source audit verifies
52 articles byte for byte against first-party API content, all 200 selected and
supporting spans, exact title/revision/timestamp and eight complete contributor
responses. The terminal main/support responses have no continuation.

Reviewer root: `C:/Users/inthe/IdeaProjects/lexique-spec-120-129/target/implement-120-129/reviewer-evidence/issue-122/issue-122-review/`.
Files: `source-audit.json`, `source-audit.log`, `red-mince.log`, `green-mince.log`,
`red-race.log`, `green-race.log`, `red-recu.log`, `green-recu.log`,
`red-acceptance.log`, `green-acceptance.log`, `focused-green.log`,
`owned-docs-check.json`, `review.patch`, `tests.patch` and `final-verification.json`.
Author raw root: `C:/Users/inthe/IdeaProjects/lexique-spec-120-129/target/implement-120-129/author-evidence/issue-122/`.

## Limitations

The focused tests validate reviewed inputs and admission records; the canonical
converter is not run here. Actual combined resource/package notices, native
lookup/save/reopen, human learner acceptance, whole-machine offline execution,
other-platform execution and formal release readiness remain parent-owned.
