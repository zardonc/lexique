# Supplemental French-English MDX

FreeDict French-English 0.4.1 provides 8,505 physical records and 8,417 unique keys. Original content: **2018-08-11**. Conversion: **2026-11-10**. It is a small supplemental dictionary with known gaps and translation defects, not the complete default dictionary or a CEFR word list.

The 273,758-byte MDX uses version 2.0, UTF-8 and zlib without encryption or MDD media. Lexique reads it with its existing bounded MDX reader and isolates the HTML. The bundled dictionary ID is `builtin-fra-eng`; user imports remain separate.

Source: [official FreeDict 0.4.1 release](https://download.freedict.org/dictionaries/fra-eng/0.4.1/). Copyright belongs to Horst Eyermann, Michael Bunk, John Darrington and the FreeDict project; retain the complete accompanying AUTHORS and COPYING. The dictionary content remains **GPL-2.0-or-later**. Lexique and its conversion script remain MIT.

Modification, 2026-11-10: complete TEI entries rendered as HTML and written to MDX. Original spellings, translations and duplicate entries are retained; entry footers identify the original TEI record. No lexical corrections, example enrichment or media are added. The manifest records the original archive SHA-512, converted bytes, content date and writer commit.

The repository keeps the complete original archive, provenance and notices in `resources/freedict-fra-eng-0.4.1/`. Portable packages already include that same source for conjugation meanings in `conjugation/freedict-fra-eng-0.4.1/`; the MDX reuses it. MDX notices and the converter are in `dictionary/freedict-fra-eng-mdx/`.

## Verify and rebuild

Python 3.12+ is sufficient for verification. In a repository checkout, run:

```powershell
python -B -X utf8 scripts/freedict_mdx.py
```

To rebuild from a portable package, use a new workspace. Copy this MDX folder to `resources/freedict-fra-eng-mdx-0.4.1/`, the complete `conjugation/freedict-fra-eng-0.4.1/` folder to `resources/freedict-fra-eng-0.4.1/`, this folder's `freedict_mdx.py` and `conjugation/conjugation_meanings.py` to `scripts/`, and this folder's `LICENSE-MIT` to the workspace root as `LICENSE`.

Use the MIT [mdict-utils writer at commit 64e15b99aca786dbf65e5a2274f85547f8029f2e](https://github.com/liuyug/mdict-utils/tree/64e15b99aca786dbf65e5a2274f85547f8029f2e). Keep its `writemdict.py`, `ripemd128.py` and `pureSalsa20.py` together in their original `mdict_utils` folder, along with the upstream MIT license. The script loads that pinned writer directly; no Python package installation is required.

```powershell
python -B -X utf8 scripts/freedict_mdx.py --output target/rebuilt-freedict-mdx --writer PATH_TO_MDICT_UTILS/writemdict.py
```

The output directory must be new. The converter checks the record counts and requires the rebuilt MDX to match the manifest byte for byte; complete original source is copied beside the rebuilt file. Original resource folders and user data remain untouched.
