# Issue #161 author evidence

Frozen rows **4001–4050** retain all **50 exact groups** and their frequency strings.
The author shard proposes **50 senses / 200 four-field bindings** from **50 fixed
French Wiktionnaire articles**, with complete contributor pagination and retained
API/raw bytes. No author content blocker or new exception is recorded.
Independent quality, everyday-use, spelling/construction and selected-expression
rights review remains pending: **0 independently accepted groups / fields**,
**200 fields missing from admission**. The global denominator remains **4,087**.

[Groups](groups.json), [sources](sources.json), [exact spans](source-excerpts.json),
[source and contributor context](source-context.json), [capture pointers](source-captures.json),
[field/use review draft](field-review.json), [rights proposals](rights.json),
[admission counts](admission.json), [reuse](reuse.json) and
[content blockers](content-blockers.json) form the reviewable shard.
The [preserved author package](authored/README.md) retains the proposed values.
[NOTICES](NOTICES.md) and [COPYING](COPYING) accompany the adaptations.

## Selected restrictions and source boundaries

- `ci/ADV` selects modern compound use, including `ci-joint` and `ci-dessous`;
  isolated archaic use is excluded. Fixed spelling remains `ci`; no alias is admitted.
- `couché/ADJ` selects lying in bed. Its IPA comes from the adjective's own
  agreement template, not the verb-participle section.
- `manifester/VER` selects intransitive participation in a demonstration;
  `hypocrite/NOM` retains the person noun. `hache` retains h aspiré and
  `hypocrite` h muet.
- `migraine` selects the explicitly hyperbolic severe-headache use; `vaurien`
  selects the milder informal rascal use. `caporal` retains the exact French
  Army/Air Force rank hierarchy and NATO OR-3 code.
- `marron/ADJ` selects the colour frame with both sourced agreement possibilities.
  `tombeau` preserves the professional above-ground monument / underground
  vault distinction in all three semantic fields.
- `fidélité` selects loyalty to duties, affections and commitments. Its CFC
  cartographic definition at line 40 is excluded. DAF8 original-work layers
  receive separate attribution and the established original collective-work basis;
  bibliography and import markers do not prove selected-sentence authorship.
  New expression-level rights decisions remain pending.

## Verification and handoff

Applied the approved #78 resource-artifact testing seam using
`mattpocock-skills:tdd`; native application checks remain coordinator-owned.

- `uv run python scripts/test_core_batch_161.py`: **passed**, five checks of
  exact membership/four fields, raw API bytes/POS/IPA spans/contributor chains,
  pending rights/quality counts, restricted uses/excluded import boundaries,
  and preserved author package/notices/legal text.
- The retained TDD logs record initial failing membership, rights and notices
  checks followed by their passing author-shard checks.
- `pnpm test:docs`: **passed**, no document/link problems.
- `git diff --cached --check`: **passed**.
- Canonical conversion, combined packaging and native lookup/save/reopen:
  **not run in this author checkout**. This shard changes no runtime or canonical input.

Execution evidence, original sources, legal authorities, preserved package and
delivery commit pointer reside at
`C:/Users/inthe/IdeaProjects/lexique-d2-160-162/context/author-evidence/issue-161`.
Tests resolve this recorded absolute root; they require the archive to be present.
Checkout base: `0963287ea5ed593cc84215377d62e6db1a072566` on Windows x64.
The final author SHA and full patch are retained in that archive for the
independent reviewer and integration coordinator.
