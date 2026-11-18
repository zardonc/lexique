# Issue #84 independent review candidates

**Status: pending. No independent reviewer has accepted this batch.**

There are 50 proposed senses, 200 field candidates and 50 usage candidates in
the [monolithic field-review format](../../../resources/core-pilot-79/field-review.json).
Batch sense IDs are `b84-151-nom` through the row-specific `b84-200-ver`;
all are prefixed `b84-`. Each candidate carries its exact proposed text,
source ID, fixed revision, contributor-history link and one-based source excerpt.
The `accepted_text` and `accepted_usage` keys are existing format slots; their
quality statuses are **pending**, and `reviewer` is null. They do not represent
accepted text. Field rights, source selected-text review and sense independent
review statuses are also pending.

The [source selections](source-research.md) and [capture index](source-index.json)
identify the inputs for a separate reviewer. Raw sources and API responses are
at `C:\Users\inthe\IdeaProjects\lexique-issue-84\target\issue-84\sources`.
The complete current curation diff is retained at
`target/issue-84/scoped-content.diff`; the focused test/converter diff is
`target/issue-84/scoped-tests.diff`. Baseline: `42c7205`, plus the coordinator's
integration infrastructure at `b8931d4` before delivery.

The reviewer must independently assess the French definition, IPA context,
English explanation, Chinese explanation and grammatical/domain exclusions for
every selected sense. Specific source-taxonomy decisions concern
`autre/PRO:ind` (row 155), `est-ce que/ADV` (163) and `seul/NOM` (181).
Check the uncountable singular and approximation uses of `quelque` separately,
and retain consonant-only pronunciation scopes for the selected determiners and
numeral. Selected-span rights and applicable article import notices need their
own decisions. No historical or unrelated specialist sense replaces a frozen
target.

Author source matching, membership checks and the refusal of unreviewed inputs
are execution evidence only. They are not independent language quality,
learner acceptance, admitted-resource or native application evidence. The
coordinator allocates a fresh reviewer after integrating the workers' records.
