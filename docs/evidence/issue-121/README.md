# Issue #121 author package

All **50 exact frozen groups at rows 2001–2050** have one selected use and
**200 proposed FR/IPA/EN/ZH fields**. None of these exact groups is already
admitted in the baseline `core-119-v1` inputs: **zero reused groups/fields**.
The denominator remains **4,087 groups**. There are no missing authored
fields, substitutions, aliases or new annotation/specialist exceptions.
All 200 new fields await independent content and selected-rights review.

The [ticket](https://github.com/zardonc/lexique/issues/121) and
[parent spec](https://github.com/zardonc/lexique/issues/78) define the batch.
The parent’s Testing Decisions authorize the existing resource/admission seam.
The author owns this evidence directory and
[test_core_batch_121.py](../../../scripts/test_core_batch_121.py).

| File | Content |
| --- | --- |
| [groups.json](groups.json) | Exact rows, spellings, POS subclasses, frequencies, selected uses and four fields |
| [reuse.json](reuse.json) | Baseline canonical comparison; no reused group |
| [sources.json](sources.json) | 48 fixed French revisions, attribution, contribution licences and original-work layers |
| [source-excerpts.json](source-excerpts.json) | 200 exact field bindings and language/POS/construction frames |
| [source-context.json](source-context.json) | Selected locations, import/reference boundaries, full named registries and anonymous counts |
| [source-captures.json](source-captures.json) | Retained raw/API/request paths, complete contributor pagination and fixed-revision replay URLs |
| [field-review.json](field-review.json) | 200 field proposals, 50 use records and 50 exact-spelling records for the separate reviewer |
| [rights.json](rights.json) | Per-field rights proposals, exact source spans, authors, original-work basis, modifications and obligations |
| [admission.json](admission.json) | Proposed/accepted counts, review priorities and parent-owned gates |
| [source-research.md](source-research.md) | All selected uses, fixed sources and construction/register decisions |
| [rights-proposal.md](rights-proposal.md) | Contribution, original-work and read-only validation boundaries |
| [NOTICES.md](NOTICES.md) | Source attribution, modification and original-work credits |
| [COPYING](COPYING) | Complete CC BY-SA 4.0 legal text |

## Verification and reproduction

The [focused check](../../../scripts/test_core_batch_121.py) passes **three tests**:
exact frozen membership/four fields, complete source/right/review handoff,
and actual first-party source bytes plus contributor continuation/registries.
TDD logs are [membership RED](../../../target/issue-121/membership-red.log),
[membership GREEN](../../../target/issue-121/membership-green.log),
[rights RED](../../../target/issue-121/rights-red.log), and
[final GREEN](../../../target/issue-121/green.log).

The [reproduction log](../../../target/issue-121/reproduction.log) records
byte-for-byte reconstruction of **48 source wikitext bodies and nine JSON
evidence files**. The [live fixed replay](../../../target/issue-121/live-fixed-reproduction.log)
also reproduces all 48 article bodies through five pinned revision-ID requests.
Contributor capture terminates after **seven parts**, retaining **3,336 named
article-contributor memberships** across 48 articles, anonymous counts and
complete history links. Names are counted per article, not as globally distinct people.

```powershell
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-121/capture_validation.py
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-121/prepare_candidates.py
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-121/finish_author_package.py
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-121/write_docs.py
uv run --python 3.14.10 --no-project python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_121.py -v
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-121/reproduce.py
uv run --python 3.14.10 --no-project python -B -X utf8 target/issue-121/refetch_fixed.py
pwsh -NoProfile -File scripts/check-docs.ps1
git diff --cached --check
```

Checkout: `C:/Users/inthe/IdeaProjects/lexique-batch-121`, branch
`codex/d2-content-121`. The clean initial head
`b91c4729a0a1c93baa58f86ed666b173fb2fc253` descended from integration
`codex/d2-content-120-129`. The latest integration tip is merged before handoff.
All source, authority, request, preparation and verification files remain in
this checkout’s ignored `target/issue-121/` for parent archival.
The [final verification record](../../../target/issue-121/final-verification.json)
records the delivered heads, commands and outcomes.

## Limitations and review priorities

- **2036 promis/NOM:** a familiar fiancé noun, qualified as dated or regional.
  Current Larousse collège records the regional alternative; Wiktionnaire and
  Usito mark it dated. Regional modern-use eligibility needs the independent
  reviewer’s judgement. No named region or ordinary nationwide frequency is claimed.
- **2039 b/NOM:** exact lowercase French letter name, masculine and invariable,
  supported by the source’s `lettre` section; /be/ names the letter rather than
  giving its consonant sound. The separate `b…` abbreviation is excluded.
- **2041 enculé/ADJ:** the vulgar, insulting modifier is restricted to
  `enculé de + nom`. The source explicitly assigns adjectival value to this
  construction within its noun section. Standalone noun/sexual participle uses are excluded.
- **2024 chèque/NOM:** current banking facts validate the payment instrument.
  The source’s old `au porteur` description is not generalised to every cheque.

Independent content/rights acceptance, canonical conversion and combined notices,
actual native lookup/save/reopen, learner acceptance and platform/offline/release
checks remain separate parent gates. The author changed no canonical resource,
application code or learner data and ran no native, frontend or release checks.
