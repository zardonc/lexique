# Grammalecte verb metadata

Grammalecte 2.3.0 by Olivier R., pinned from the
[GitHub mirror](https://github.com/Pofilo/grammalecte/tree/08511c222029b3be20bf547563315d10fb48d44b)
of the [upstream project](https://grammalecte.net/).
See [provenance](provenance.json), [upstream attribution](upstream-README.txt),
the unmodified [GPL version 3](LICENSE-GPL3), and the original
[decoder reference](conj.py). No upstream Python or JavaScript code is executed.

The archive contains the complete original `conj_data.json` bytes, including
all 8,403 verb identities and raw type flags. Lexique's MIT conversion code
projects metadata and explicit conjugation forms onto existing Lefff identities.
The format-3 build adds ordinary compound and gerund constructions (2026-11-06);
conjugation_generation.py is the corresponding conversion source. No extra
verb identities are admitted. Generated forms retain GPL-3.0-only attribution. Linguistic metadata derived from this source
remains GPL-3.0-only. Lefff forms remain separately governed by LGPL-LR.
The resource combines these separately attributed tables; it does not relicense
either source or change the application's MIT source license.

Redistribution retains both resources' notices, license texts, modification
dates, original legible source, and conversion/correction source. This directory
supplies the complete compressed original JSON and reference decoder under GPL 3;
the build copies these materials beside the generated database. A future product
package must retain them and the corresponding conversion source.

## Metadata interpretation

The upstream raw type's first character supplies group; positions 2, 3, 4, and 6
indicate intransitive, transitive, indirect-transitive, and impersonal uses.
Positions 7/8 explicitly indicate être/avoir. Upstream `zz` means incomplete
auxiliary information; it never implies a default avoir. Essential pronominal
entries without ordinary-use flags have an inapplicable ordinary auxiliary,
not an invented active paradigm. Missing entries remain unconfirmed.

The upstream internal auxiliary group 0 maps to traditional group 3 as required
by #38. [Cited corrections](corrections.json) retain original flags and explain
the traditional classification of aller, additional ordinary auxiliaries, and
the unresolved homographic groups of ressortir. These annotations contain
original factual summaries, not copied dictionary definitions or tables.
Dual auxiliaries without validated sense mappings remain labelled unconfirmed.
Admission does not certify unresolved source claims or a complete paradigm;
the build reports unresolved coverage separately.
