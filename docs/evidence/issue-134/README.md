# Issue #134: frozen groups 2651–2700

**50 exact groups, 50 selected senses and 200 sourced author proposals** are
ready for separate review. The immutable 4,087-group denominator, exact POS
subclasses, spelling, row order and both original frequency strings are retained.
No group is omitted or exempted. No previously admitted group is counted again.

| Resource/admission measure | Author package |
| --- | ---: |
| Frozen groups | 50 |
| Authored selected senses | 50 |
| Complete proposed fields | 200 |
| Fixed French source articles | 51 |
| Reused admitted groups / fields | 0 / 0 |
| Missing authored fields | 0 |
| Independently accepted new groups / fields | 0 / 0 |
| Fields pending independent quality / rights review | 200 / 200 |
| Missing admitted fields | 200 |
| Author-unresolved content rows | 0 |
| Global target groups / headwords | 4,087 / 3,773 |

The [ticket #134](https://github.com/zardonc/lexique/issues/134) and
[parent #78](https://github.com/zardonc/lexique/issues/78) define the resource
and native seams. Fresh worker-guide, pipeline-note, ticket, parent, repository
scope/product/roadmap/readiness/domain and resource ADR readings preceded this
package. The clean assigned branch `codex/d2-content-134` was fast-forwarded to
integration base `6b9db47664581bfa26f2abaf112a6b33c2d0c78c` before authoring.
Only this evidence directory and
[scripts/test_core_batch_134.py](../../../scripts/test_core_batch_134.py) are
scoped tracked changes.

## Reviewable content and evidence

- [Readable proposals](review-candidates.md) and [groups.json](groups.json):
  exact four-field text and complete construction/register/excluded-use scope.
- [field-review.json](field-review.json): 200 proposed fields, 50 usage records
  and 50 spelling/inflection relationships, with reviewer null and acceptance
  pending. Author checks do not supply independent language or rights review.
- [source-captures.json](source-captures.json), [source-context.json](source-context.json)
  and [source-excerpts.json](source-excerpts.json): fixed article/API revisions,
  literal field/support spans, contributor pagination and attribution boundaries.
- [rights.json](rights.json), [selected-rights.md](selected-rights.md),
  [NOTICES.md](NOTICES.md) and [COPYING](COPYING): scoped author rights proposals,
  original-work credits and actual redistribution obligations.
- [admission.json](admission.json) and [reuse.json](reuse.json): exact counts,
  pending gates and comparison with unchanged `core-129-v2` canonical inputs.
- [authored originals](authored/): all nine original JSON proposals preserved
  before separate review; accepted replacements must not overwrite them.
- [verification.json](verification.json): actual author checks and evidence
  locations, with coordinator-owned gates recorded separately.

French definitions are scoped adaptations; English and Chinese are new Lexique
explanations supported by the selected French semantics, not upstream bilingual
quotations. Their actual language accuracy remains for the separate reviewer.
CEFR is unknown throughout.

## Selected boundaries

`lesquels/PRO:int` and `lesquels/PRO:rel` retain separate masculine plural
identities, the corresponding `lequel` source section and exact-form /le.kɛl/.
`bijoux/NOM` retains plural identity, attested nominal `bijou` binding and its own
/bi.ʒu/. These source bindings add no runtime aliases.

`nègre/NOM` selects the racial designation for comprehension of offensive
language. All three meaning fields retain the insulting/pejorative and
dated/vulgar restrictions from lexical line 39. The fixed article’s 2020
metalinguistic discussion at line 55 supplies contemporary review context;
its prose is unselected. This is an ordinary candidate with pending everyday-use
review, not a specialist or annotation exemption.

`arrestation` selects the applicable /a.ʁɛs.ta.sjɔ̃/ explicitly recorded at
pronunciation line 57. The source lexical header’s `arrêtation` and missing /s/
are retained in the source/context evidence but excluded from learner fields.
`tester` selects the modern object-taking test definition at line 36; its
inconsistent intransitive header tag is recorded for review. `piper` selects
physical dice manipulation with /pi.pe/, separate from computing /paj.pe/.
`voile` is the feminine boat sail; `orange` is the feminine fruit.

Thirty-five article-level DAF8 imports retain original Académie française
credits and the already independently reviewed original-work basis. That
historical review does not review any new #134 field. `légende`’s CFC cartographic
subsense at line 18 is explicitly excluded; narrative line 28 is selected.

## Reproduction and author validation

The stable retained author archive is
`C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/author-evidence/issue-134/`.
It contains all raw/API/request captures, the two completed contributor chains,
reused rights-authority captures, exact `selection.py` and adapted preparation,
rights, notice and reproduction helpers. The source index identifies every
capture and fixed revision. Helpers were adapted from the established #129
author recipe after checking its current schema and hardcoded paths.
The original ignored `target/issue-134/` cache is also retained. Admission and
capture-root metadata identify the stable archive; resource/source checks read
the archive directly, so removing the original cache later will not break review.

From the ticket worktree:

```powershell
uv run --python 3.14.10 python -B -X utf8 scripts/test_core_batch_134.py
uv run --python 3.14.10 python -B -X utf8 C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/author-evidence/issue-134/reproduce.py
pnpm test:docs
git diff --check
```

Reproduction writes the root proposal JSON from retained captures and compares
all nine files byte for byte. Use author output for that command; accepted
reviewer output requires its own preserved inputs/reproduction procedure.
The focused check inspects delivered resource/admission records, literal source
bindings, actual contributor continuations, retained originals and selected
boundary cases. It does not import preparation helpers or mock a reviewer.

Expected RED evidence is retained under ignored `target/issue-134/RED.log`:
the baseline canonical slice lacked all 50 groups. The second focused RED
record, `RED-author-preservation.log`, covers absent preserved author proposals.
Passing results and full logs are listed in [verification.json](verification.json).
Captured HTTP 429 failures remain in ignored cache logs; completed fixed captures
are used for reproduction without further source requests. All source/API/request,
rights, author/helper and check logs are also preserved in the stable archive.

## Pending coordinator gates

Separate quality, selected-span rights, construction/spelling and everyday-use
review remain pending for all 200 new fields. The coordinator owns accepted
replacements, combined canonical assembly/conversion and packaged notices,
actual separate review, all-group native lookup/save/reopen and preservation
acceptance, learner acceptance and issue/PR operations. This author package
changes no shared resource, converter, native runner or user learning data and
does not establish release readiness.
