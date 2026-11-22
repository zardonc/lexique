# Separate independent review handoff: #131

Review the sourced author package for [issue #131](https://github.com/zardonc/lexique/issues/131) against [parent #78](https://github.com/zardonc/lexique/issues/78), especially its resource/admission and independent-quality testing decisions. This is an author handoff, not an independent acceptance report.

## Exact input and execution boundary

- Author checkout: `C:/Users/inthe/IdeaProjects/lexique-spec-130-139/ticket-131`, branch `codex/d2-content-131`.
- Original diff base: `e4005d868880dac12c84c412773049eea737bc67`. The author commit hash and merged integration tip are supplied with delivery; archived `delivery.json` records the actual source/test identities.
- Review committed changes with `git diff e4005d868880dac12c84c412773049eea737bc67..<author-commit> -- docs/evidence/issue-131 scripts/test_core_batch_131.py`. The complete patch and focused test patch are retained in the archive.
- Stable source/execution evidence: `C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/author-evidence/issue-131`. Original ignored evidence remains in the author checkout. The admission’s `author_evidence_root` and source-capture fallback resolve the archive if the worktree is later cleaned.
- Host boundary: Windows x64, PowerShell 7, Python through uv. No native application execution or independent quality review is claimed.

Read [CODING_STANDARDS.md](../../../CODING_STANDARDS.md) for reviewer input and evidence rules. Work only on this ticket’s evidence and focused test. The coordinator owns global resource assembly and native integration.

## Reviewable content

[source-research.md](source-research.md) presents the 50 four-field proposals. [groups.json](groups.json) carries exact frozen identities, source bindings, selected use and excluded alternatives. [field-review.json](field-review.json) supplies each original `proposed_text`, exact source and supporting construction spans, with null reviewer and pending acceptance. [rights.json](rights.json), [selected-rights.md](selected-rights.md), [source-context.json](source-context.json), [source-excerpts.json](source-excerpts.json) and [source-captures.json](source-captures.json) preserve rights, attribution and actual raw-capture proof.

Preserve the author commit and original proposals before applying accepted replacements. Record actual reviewer identity, independence from drafting, exact accepted text, quality/rights decisions, reasons and affected sense IDs. A changed field requires its own decision; no previous ticket quality review applies to these 200 fields.

## Points requiring semantic and source judgement

- Verify all 50 exact groups and one modern everyday sense per group, with four fields for the same selected use. There are no owner-approved exceptions in this ticket.
- Review French `bye` and `bah` as interjections within Lexique `ONO`, including register and the chosen IPA variants.
- Review `amen` using the **French** interjection frame in English Wiktionary, the resolved prayer-ending equivalent and the corroborating same-form French IPA. Retain the differing adverb label in French Wiktionnaire as evidence; an automated frame check does not decide POS equivalence.
- Review plural possessive `nôtres` through the explicit `les nôtres` agreement/meaning frame. Confirm /notʁ/ applies to the frozen component, and preserve the required determiner construction. Do not admit a general query alias or use the archaic possessive adjective as the group’s meaning.
- Confirm transitivity, `parvenir à`, h muet in `huile`, masculine/feminine alternatives in person nouns and all intentionally limited meanings. The judicial `divorce` and agricultural `culture` proposals remain scoped to their selected senses.
- Assess every selected source/import boundary independently. Confirm DAF8’s original collective-work basis and the Larousse edition/original-sentence proposal. Godefroy, Rivarol, TLFi and neighbouring quoted examples are not general permission to import external prose.

## Commands and integration

Run `uv run --no-project python -B -X utf8 scripts/test_core_batch_131.py` for the author resource/admission check. It checks fixed source spans, continuation chains, exact frozen groups and honest pending states. It does not certify semantics or rights.

After independent acceptance and coordinator assembly, run `uv run --no-project python -B -X utf8 scripts/test_core_batch_131.py --admitted` against the real resource. This currently fails because the baseline has no admitted #131 groups. The coordinator must separately reproduce conversion and notices, verify all 50 native lookups, trusted representative save/reopen and preservation of existing learning data before recording native completion.

The default author check intentionally enforces the immutable pending proposal package. Preserve it when writing reviewed records; the coordinator may advance the test to the accepted evidence shape as in #120–#129 once the real independent decisions exist. Native checks and independent decisions remain pending in [admission.json](admission.json).
