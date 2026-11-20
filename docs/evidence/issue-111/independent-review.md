# Issue #111 independent semantic and selected-rights review

**50 exact uses and 200 fields accepted unchanged; zero unresolved fields.**
Every proposed FR/IPA/EN/ZH field was read against its selected fixed raw
source, French grammatical frame and same-use restrictions. All frozen rows
1501–1550, subclasses, frequencies and identities remain intact. The denominator
is 4,087 groups. No annotation, specialist exception or runtime alias is approved.

Ticket [#111](https://github.com/zardonc/lexique/issues/111), parent
[#78](https://github.com/zardonc/lexique/issues/78). Author SHA:
`9aca23191000a915a757ed9999646d9b24c55099`. Reviewed pinned base:
`b3560857e61286a61cbf88a1aabf2072fe8ae11e`, after the author's integration merge.
Owning reviewer checkout: `C:/Users/inthe/IdeaProjects/lexique-review-111`,
branch `codex/d2-review-111`. The reviewer did not author the proposals.
The review covers this evidence directory and the focused resource test.

## Decisions

[Field review](field-review.json) records each accepted text alongside its
original proposal, a field-specific reason, a selected-rights reason and
independence. It also records all 50 reviewed identities, use decisions and
spelling decisions. [Original candidates](review-candidates.json) retain the
entire author's groups with their pending statuses. No accepted wording needed
replacement. Mechanical source comparison establishes fixed provenance; the
semantic verdict comes from reading the actual fields and source context.

- `telle/ADJ:ind` is explicitly attested before a noun in `tel` lines 72–74,
  while the exact flexion article supplies feminine singular and `/tɛl/`.
  Its generic `tel#fr-adj` link alone does not establish the indefinite subclass;
  the separate indefinite section and `telle ville` example do.
- `pilote/ADJ` is limited to postnominal `projet pilote`. French Wikipedia
  revision 223861804, line 3 supplies the small-scale feasibility trial;
  frozen Lexique supplies ADJ. The French dictionary supplies component IPA,
  and has no adjective section. No generic free-standing adjective is claimed.
- `niveau`, `tableau`, `toit`, `tombe` and `montre` retain ordinary height,
  painting, roof, grave and portable-watch senses despite domain indexing.
  `faible/NOM` has general machine/beam/joist support at raw line 236; it is
  not confined to the adjacent sword example.
- `vierge/ADJ` selects a person's sexual inexperience. `vierge/NOM` keeps
  the source's female and with-a-man restrictions in all three meanings.
  `couple/NOM` selects the masculine romantic/marital/sexual pair without
  importing the separate feminine noun or a sex-pairing restriction.
- `malheureux`, `tendre`, `envers`, `italien`, `double`, `futur` and `faible`
  retain their separately reviewed POS senses. `italien/NOM` is the language;
  `double/NOM` is exactly twice the amount; `futur/NOM` is temporal future.
- Selected verbs retain their transitive arguments and exclusions. `bénir`
  keeps its religious context; `fusil` avoids a rifled-barrel-only restriction;
  `montre` keeps carried or worn portability rather than wrist-only scope.
- Every IPA is an explicit French recorded variant for the frozen spelling.
  `/bys/` is the bus noun, `/ɑ̃.tʁe.ne/`, `/ɡɑ.ʃe/`, `/ʁə.pɑ/` and
  `/lɔʁ.skə/` are selected recorded variants. No liaison, elision or generic
  morphological query alias is inferred.

EN/ZH are accurate new explanatory translations of the same selected French
uses. They are not upstream bilingual quotations. All binding-only spelling
decisions retain `alias_admission=not_admitted`; the canonical alias set remains.
[Selected-rights review](selected-rights-review.md) separately accepts the actual
selected text boundaries and retains their redistribution obligations.

## Evidence and verification

Raw captures/logs: `C:/Users/inthe/IdeaProjects/lexique-batch-111/target/issue-111/`.
Reviewer evidence: `C:/Users/inthe/IdeaProjects/lexique-review-111/target/issue-111-review/`.
Platform: Windows x64, PowerShell 7, uv-pinned CPython 3.14.10.

- `capture-audit.json` / `capture-audit.log`: all 43 complete wikitext files
  equal their fixed API revision content, with matching page/revision IDs,
  titles, timestamps and last editors. All 200 primary field spans and their
  supporting spans match the raw lines. Seven contributor parts cover all
  43 registries and terminate pagination. Per-source comparisons and raw
  SHA-256 values are also retained in [source context](source-context.json).
- `groups-01.txt` through `groups-10.txt`, `frames-*.txt` and `boundary-*.txt`:
  bounded projections used for independent reading; complete captures stay on disk.
- `baseline.log`: pinned Python author's resource-boundary baseline, 2 tests passed.
- `focused.log`: `uv run --python 3.14.10 python -B -X utf8 -m unittest discover -s scripts -p test_core_batch_111.py`, 4 tests passed.
- `pilot.log`: the same pinned Python command with `-p test_core_pilot.py`,
  15 existing pilot tests passed.
- The focused test uses `core_pilot.load_inputs` on a temporary resource copy,
  with the 50-row seam and unchanged canonical alias relations. Five mutation
  checks reject changed accepted text, source line, usage, pending rights and
  a new alias even when review/checksum fixtures are updated coherently.
- `docs.log` and `diff-check.log`: repository documentation links/format and
  scoped whitespace diff checks passed. Retained reviewer patch:
  `review.patch`, from `git diff b3560857e61286a61cbf88a1aabf2072fe8ae11e -- docs/evidence/issue-111 scripts/test_core_batch_111.py`.

## Limitations

The review reads fixed supplied captures; no independent live refetch is claimed.
Contributor registries extend beyond the selected revisions and are not proof
of selected-sentence authorship. DAF8 uses the established original-edition
basis; no live statute refresh is claimed. Canonical conversion, actual combined
notices, packaged output, native lookup/save/reopen and release admission remain
parent-owned pending. The temporary validator seam proves resource contract
acceptance, not those platform/product gates. No app or native suite was run.
