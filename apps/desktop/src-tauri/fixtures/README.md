# Note import dictionary fixture

`nfd-headword.mdx` contains one original entry with the decomposed key
`e\u0301le\u0300ve` and an original HTML paragraph. The content uses the
repository's [MIT licence](../../../../LICENSE). It tests an installed custom
dictionary whose headword is NFC-equivalent to the imported `élève` while
preserving the dictionary's original lookup spelling.

The independent writer is mdict-utils at
[`64e15b99aca786dbf65e5a2274f85547f8029f2e`](https://github.com/liuyug/mdict-utils/tree/64e15b99aca786dbf65e5a2274f85547f8029f2e),
MIT. Fetch its writer files as described in the
[MDX fixture instructions](../../../../tools/mdict-probe/tests/fixtures/README.md).
From the repository root, with that `writer_support` package on `PYTHONPATH`,
this reproduces the fixture into an absent destination:

```python
from datetime import date
from pathlib import Path
from unittest.mock import patch
from writer_support.writemdict import MDictWriter

destination = Path('apps/desktop/src-tauri/fixtures/nfd-headword.mdx')
writer = MDictWriter(
    {'e\u0301le\u0300ve': '<p>Original decomposed-headword fixture.</p>'},
    title='Lexique NFC identity fixture', description='Original Lexique test data, MIT',
    version='2.0', encoding='utf8', compression_type=0,
    encrypt_index=False, block_size=128, is_mdd=False)
with patch('writer_support.writemdict.datetime.date') as fixture_date, destination.open('xb') as output:
    fixture_date.today.return_value = date(2026, 11, 20)
    writer.write(output)
assert 0 < destination.stat().st_size < 16384
```
