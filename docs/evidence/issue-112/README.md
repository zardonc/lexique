# Issue #112 authored content

All **50 frozen rows 1551–1600** retain their exact headwords, POS subclasses,
frequencies and order. Each has one fully authored selected sense with
**FR/IPA/EN/ZH: 50 senses / 200 fields**, with no missing authored fields or
unresolved exact groups. **Zero fields are independently accepted** in this
author package. The global denominator remains **4,087 groups**.

The [ticket](https://github.com/zardonc/lexique/issues/112) and
[parent spec](https://github.com/zardonc/lexique/issues/78) define the scope.
[Source research](source-research.md) explains the selected-use boundaries.
[Rights evidence](rights.json) records the author proposal and the
[redistribution notices](NOTICES.md). Separate independent quality and
selected-span rights review remains pending for the parent.

## Handoff files

| File | Content |
| --- | --- |
| [groups.json](groups.json) | Exact frozen groups and complete authored senses |
| [sources.json](sources.json) | 46 fixed French source revisions, attribution and modifications |
| [field-review.json](field-review.json) | 200 proposed fields, 50 uses and 50 spelling relations; independent states pending |
| [source-context.json](source-context.json) | French/POS frames, construction evidence, full contributor registry and original-work boundaries |
| [source-excerpts.json](source-excerpts.json) | Exact one-based source lines, IPA positions and selected construction span |
| [rights.json](rights.json) | Per-field author rights basis, fixed policy spans and redistribution obligations |
| [source-captures.json](source-captures.json) | Fixed API/raw paths and complete seven-part contributor pagination |
| [reuse.json](reuse.json) | Canonical rows checked; no assigned group already admitted or rewritten |
| [admission.json](admission.json) | 50 authored groups / 200 fields; zero accepted fields; parent gates pending |
| [COPYING](COPYING) | Complete CC BY-SA 4.0 legal text |

## Author verification

The parent’s Testing Decisions authorize the existing resource boundary used by
[the focused check](../../../scripts/test_core_batch_112.py). It checks frozen
membership and frequencies against the accepted manifest, four-field content,
fixed bindings, attribution/rights records, construction restrictions, explicit
reuse and honest pending independence. Author preparation also compares each
complete captured API revision with its wikitext and checks French/POS frames.
These checks are author evidence, not independent semantic or rights review.

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-112/prepare_candidates.py
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-112/prepare_rights.py
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_112.py
git diff --cached --check
```

Preparation uses the retained captures without live source access. The missing
package and missing rights-record checks recorded expected RED results; the
complete package recorded GREEN with two tests. Logs and source captures stay
under `C:/Users/inthe/IdeaProjects/lexique-batch-112/target/issue-112/`, including
`red.log`, `rights-red.log`, `prepare.log`, `rights-prepare.log`, `green.log`,
`links.log`, and the recorded Git state. The captured ticket/spec originals
remain under `C:/Users/inthe/IdeaProjects/lexique-spec-110-119/target/implement-110-119/`.

Author checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-112`, branch
`codex/d2-content-112`, initial base
`fa4ea3eab1558fbede04b50cf0cd15f05bdf26dc` on integration
`codex/d2-content-110-119`. The integration tip is merged before handoff;
the retained Git state identifies that exact tip and tested commit.

## Remaining gates

Independent selected-field quality, spelling/construction and rights review
remains pending for all 200 fields. Combined conversion, packaged notices and
actual native lookup/save/reopen are parent-owned and were not run here.
The established DAF8 original-edition rights basis is reused; this author did
not refresh the live statute. Learner acceptance, whole-core completion and
release/platform gates remain separate.
