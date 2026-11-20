# Issue #119 independent selected-rights review

**196 new selected fields pass their independent rights review.** The four
reused `côte/NOM` fields retain their existing accepted records and rights.
Reviewer: Independent semantic and selected-rights reviewer #119, separate
from the author and parent integration owner. Author head:
`d4abf52426449dd5601e989640dc91649e6196f9`.

The raw audit compares all **52 selected source articles** (51 French
Wiktionnaire and the French spelling section of English Wiktionary `heu`)
with their retained first-party API revision, title and timestamp. Every
field and French POS/form frame matches its raw source line. All returned
named registries and anonymous counts match the paginated responses; the
last main French, French-support and English responses have no continuation.
An omitted anonymous count is zero. Article histories retain anonymous
attribution. Registries can contain later contributors and do not identify
the author of each selected sentence.

The **28 DAF8-marked articles** retain separate original credit to
**Académie française, Dictionnaire de l’Académie française, eighth edition,
1932–1935**. The established [#99 original-work basis](../issue-99/source-context.json)
uses [CPI L123-3](https://www.legifrance.gouv.fr/loda/article_lc/LEGIARTI000006278941/2026-02-20):
seventy years from the January following collective publication. Final 1935
publication gives conservative expiry **2006-01-01**. The authored
`1966-01-01` proposals are retained as `author_proposed_conservative_expiry`
beside the reviewed date in [sources](sources.json),
[context](source-context.json) and [rights](rights.json). The original
author-proposed global basis is retained separately in `rights.json`.
Import/reference markers do not establish sentence-level origin; original
credit is retained conservatively at the marked edition boundary.

Fixed Wiktionnaire/Wiktionary contributor transcription/adaptation and
Lexique scoped FR adaptations and EN/ZH explanations retain **CC BY-SA 4.0**.
The complete [legal text](COPYING), sections 2(a), 3(a), 3(b) and 4, permits
sharing, adaptation and database extraction with attribution, source/licence
identification, modification notices, ShareAlike and no additional downstream
restrictions. Fixed Wiktionnaire copyright policy revision 38903168 and DAF8
import-template revision 22113837 distinguish externally included material.
Wikimedia Terms section 7 supplies article/history attribution and preserves
import-specific obligations; the retained policy/terms/CC authority files
remain at the author evidence root listed below.

The `pistolet` CFC import at line 53 is attached to a cartographic definition;
the firearm definition at line 17 and IPA at line 16 are outside that boundary.
English `heu` revision 92930671 line 74 is a French alternative-spelling
template. Its TLFi reference is bibliography, not imported definition prose.
The selected raw boundaries contain lexical meanings, recorded IPA and
French grammar/form evidence. No attributed literary/news example, media,
etymology or other-language meaning supplies an accepted field. The
`dans le civil` example establishes only the short construction; its quoted
prose is not copied into the accepted meanings.

Each new field in [rights.json](rights.json) has its accepted text,
independent reviewer, exact original source span and specific boundary
reason. Original proposals remain in [field-review.json](field-review.json)
and [review-candidates.json](review-candidates.json). Attribution and
modification records remain complete, including the five accepted wording
replacements. No additional source or runtime alias is admitted.

Raw sources and authorities:
`C:/Users/inthe/IdeaProjects/lexique-batch-119/target/issue-119/`.
Reviewer comparison:
[source-audit.json](../../../target/issue-119-review/source-audit.json).
Future integration archive:
`target/implement-110-119/author-evidence/issue-119`.

The assembled converter output, actual combined notices and other parent
admission/native/release gates remain unrun. Live statute retrieval was not
repeated; the original edition basis and corrected arithmetic were reviewed
against the cited established primary-source record.
