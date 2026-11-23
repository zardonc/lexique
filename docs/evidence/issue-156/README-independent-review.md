# Independent review of #156

All 200 proposed fields and 50 selected uses were inspected against the retained
fixed French sources. **196 unchanged fields / 49 groups are independently
accepted; four ya/NOM fields and one use remain unresolved.** Selected-text
rights pass for all 200 fields / 50 uses. Two original-work boundary records
were corrected; no field wording changed.

Reviewer: the separate #156 field/use/rights reviewer in
`C:/Users/inthe/IdeaProjects/lexique-d2-150-159/review-156`, branch
`codex/d2-review-156`. This review is separate from drafting; human learner
acceptance was not performed.

Ticket/spec: [#156](https://github.com/zardonc/lexique/issues/156) and
[#78](https://github.com/zardonc/lexique/issues/78). Original author commit:
`a40ecb51abf607e0a69fd1899ccc1fdb4fa8a377`; author head:
`b33db9fef0cd4266bbdc9a727551b2bd2b5397ae`; review base: `3729d677`.
Reviewed diff: `git diff 574e4a4...b33db9f -- docs/evidence/issue-156
scripts/test_core_batch_156.py`. Original proposals and test are preserved
byte-for-byte under [authored/](authored/README.md.txt). Markdown snapshots use
the `.md.txt` suffix so their original relative links remain documentary text.

[Field review](field-review.json) contains 200 individual linguistic reasons,
exact proposed/accepted text, fixed spans, reviewer identity and separate rights
decisions. Its 50 use reasons cover POS, inflection, constructions, excluded
senses and contemporary interpretation. [Replacements](accepted-replacements.json)
is empty. Source/context/excerpt/rights/capture records retain the existing
schemas and actual article/contributor origins.

The [source audit](source-audit.json) checked 50 literal API/raw UTF-8 byte
matches, 200 selected field spans, 190 unique supporting spans, 1,540 span
occurrences, seven contributor responses in five complete chains, 2,508 named
article memberships and 382 anonymous counts. Counts are article attribution,
not sentence authorship; checks are separate from editorial judgments.

The [original-work review](selected-original-work-review.json) separates
mouchoir's Quillet 1934 inspiration and DAF9 bibliography from its selected
cloth/cellulose description. Primary BnF metadata identifies the 1934 collective
publication; its original collective term ends 2005-01-01. Valet's Quillet 1979
reference receives no blanket clearance: the primary Académie eighth edition,
1935, tome 2 p.705 independently contains the selected card expression.
Original facsimiles and sentence authorship were not verified. Selected
adaptation, contributor credit, legal text and modification obligations remain
distinct from unselected later editions, quotations and media.

## ya/NOM

The exact masculine slang knife noun **ya /ja/** is lexically supported by fixed
revision 39362597, primary [Bob](https://www.languefrancaise.net/Bob/39498) and
[Cordial](https://www.cordial.fr/dictionnaire/definition/ya.php). Bob is marked
not revised and supplies a 1956 literary quotation and a 2002 dictionary source.
Its [register code 10](https://www.languefrancaise.net/Aide/Registre) means strongly
argotic on an approximate social-register scale, not everyday frequency.
Cordial's primary page was retrieved with HTTP 200 but has no dated use example.
A 2020 Expressio comment contains a contemporary metalinguistic enumeration and
constructed wordplay using this noun: recent playful use, not ordinary everyday
support.

[The decision](ya-modern-use-review.json) keeps everyday eligibility unresolved.
The four texts/IPA are semantically supported candidates with no accepted text.
No spelling replacement, annotation exception or owner approval is introduced.
Everyday support is unproved; the noun is not declared obsolete or unused.
External dictionary prose, the Dard quotation and the Expressio joke are not
learner fields.

## Validation and limits

- `uv run --no-project python -B -X utf8 scripts/test_core_batch_156.py`: eight
  tests passed, including frozen membership/frequencies, four-field/rights
  alignment, captures, preserved proposals and real converter rejection of
  author-only/unresolved quality decisions.
- Focused missing-boundary and pending-acceptance failures plus passing checks
  are retained in the external proof archive.
- `pnpm test:docs`: 554 documents, 4,259 local links, zero problems.
  `git diff --cached --check`: passed. Final commands and SHA are retained in
  the external handoff.
- Canonical conversion, combined notices, native lookup/save/reopen, upgrade
  preservation, learner acceptance and offline/platform checks were not run.
  Frontend/Rust code did not change.

Execution proof/helpers/corroboration are archived at
`C:/Users/inthe/IdeaProjects/lexique-d2-150-159/context/reviewer-evidence/issue-156`.
The canonical baseline remains `core-149-v1`, 3,431 groups, nineteen annotations
and the 4,087-group denominator. Zero new canonical fields are admitted; #156
remains incomplete while ya everyday support and parent integration gates remain
unresolved.
