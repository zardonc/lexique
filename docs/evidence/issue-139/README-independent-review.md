# Issue #139 independent field and selected-rights review

**50 exact groups / 50 everyday selected uses / 200 fields accepted.** There
are **198 unchanged fields, two Chinese replacements and zero unresolved
fields**. Rows 2901–2950 retain every headword, Lexique POS subclass, original
order and both frequency strings. The denominator remains 4,087 groups /
3,773 headwords. No exception, spelling substitution or runtime alias is admitted.

Reviewer: **Independent semantic and selected-rights reviewer #139; separate
from author and coordinator**. This is a separate agent review; learner
acceptance remains a separate gate. The [ticket](https://github.com/zardonc/lexique/issues/139),
parent #78, complete reviewer guide, pipeline notes, project guidance, glossary,
resource ADRs and coding standards were read from the supplied checkout/context.

- Author commit: `29e1d2f1bc8b7e26306332015173027897cc7bc2`.
- Author merge: `c3c385a16a590709f80b0f027f6fb29de66448cf`.
- Review base after merging integration: `7b619a70ffce54fa4c48ee79cf21fab1a5808358`.
- Integration merged for final verification: `8f5c761762d7e0a674ed0fb94b08e10888343ed0`.
- Checkout: `C:/Users/inthe/IdeaProjects/lexique-spec-130-139/review-139`.
- Branch: `codex/d2-review-139`.
- Author captures: `C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/author-evidence/issue-139`.
- Reviewer audits/logs: `target/issue-139-review/` in the review checkout.
- Retained archive: `C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/reviewer-evidence/issue-139/issue-139-review.zip`.

[Authored snapshots](authored/snapshot.json) preserve all 15 original committed
author files. [Accepted groups](groups.json), [readable content](review-candidates.md)
and [field review](field-review.json) retain exact proposed/accepted text,
fixed source/revision/span, individual field reasons and separate quality/rights
decisions. All 50 uses and source-spelling relations have independent reasons.

| Row / field | Accepted text | Fixed lexical basis |
| --- | --- | --- |
| 2905 `dresser/VER`, Chinese | 训练动物，使其学会特定行为；及物用法 dresser un animal。 | Revision 40049123, animal subsense 104 and behavioural-training parent 100 |
| 2937 `célébrer/VER`, Chinese | 举行庆典或仪式来庆祝、纪念某个事件或其周年；及物用法。 | Revision 39000544, event/recurrence definition 16 |

The [replacement ledger](accepted-replacements.json) preserves both exact
proposals and reasons. English/Chinese remain Lexique explanations of the
selected French meanings, under CC BY-SA 4.0, rather than upstream bilingual quotes.

Every POS/use was read independently. `dealer` selects the masculine drug-seller
noun and `/di.lœʁ/`; `elle-même` selects the exact feminine singular emphatic
personal pronoun; lowercase `bible` selects a physical printed copy. `vertu`
retains its singular restriction, `gang` its masculine criminal-band sense,
and `milliard/NOM` its numerical-noun frame and explicit 1,000,000,000 meaning.
Temporary `muet` and causative/transitive `évacuer`, `préoccuper` and `ralentir`
remain distinct from their other uses. Short source fragments at `soumettre`
lines 33–34 additionally bind its recipient construction; these are grammatical
support, not displayed examples. An independent original-HTML read of Le Robert
confirms the selected study meaning of `technologie` in teaching vocabulary;
its external prose is not redistributed.

## Source and selected-rights evidence

All 50 retained French article bytes equal their first-party API revision
content literally. All 200 selected field spans and the recorded supporting
spans match the raw text; French language/POS frames match every frozen group.
Complete contributor pagination has six responses in one finished chain,
2,666 named article memberships and 368 anonymous article counts. These are
article credits, not unique people or sentence authorship. No lexical article
was recaptured during review. Policy/template captures match their API text
after separately recorded CRLF-to-LF normalization.

All 50 used [source-context nodes](source-context.json) carry root-level
`selected_text_rights_status=passed_at_selected_text_boundary`, the actual
review reason and reviewer. [Sources](sources.json), [rights](rights.json),
[excerpts](source-excerpts.json), [capture records](source-captures.json),
[selected rights](selected-rights.md), [notices](NOTICES.md) and [COPYING](COPYING)
retain the selected-text obligations and exact source identities.

The 31 DAF8 article import notices retain separate Académie française original
credit and the established original collective-work basis: final publication
1935 gives economic expiry 2006-01-01 under L123-3. The current selected spans
were reviewed independently; an article marker does not prove sentence authorship
or license a later editorial/electronic work. `portefeuille`'s CFC chart import
at line 62 is excluded from the wallet selected at 49. Wallet and `injuste`
images/captions are excluded. `muet`'s inline TLFi IPA reference and
`portefeuille`'s ninth-edition bibliographic reference are explicitly retained;
no modern external dictionary definition is selected through these references.

## Verification and limitations

[Verification](verification.json) records the focused tests, independent byte/span,
contributor and rights-material audit, documentation check and diff check.
Tests cover exact membership/frequencies, all four fields and uses, original
proposals, accepted text/source/rights consistency, root source-context rights
metadata, both replacements and exact reuse against author base `d3725aa`.
The base resource is `core-129-v2` with 2,435 admitted groups and zero #139 reuse.

Fresh supplemental Legifrance requests returned HTTP 403. The retained policy
and established project original-work basis remain the cited legal evidence;
no new official-law capture or new jurisdictional finding is claimed.
[Admission](admission.json) distinguishes 200 independently accepted fields from
zero newly admitted canonical fields and 200 pending canonical fields.
Combined conversion, actual packaged obligations, native lookup/save/reopen,
learner-data preservation, learner acceptance, other platforms, offline and
release gates remain coordinator owned and unrun by this reviewer. Frontend
and Rust checks are unrun for this evidence/Python-test-only change.
