"""Write original lookup fixtures with the admitted mdict-utils writer."""

import argparse
import datetime
import json
from pathlib import Path
import sys
from unittest.mock import patch


class Rows:
    def __init__(self, rows):
        self.rows = rows

    def __len__(self):
        return len(self.rows)

    def items(self):
        return self.rows


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--writer-parent", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    sys.path.insert(0, str(args.writer_parent.resolve()))
    from writer_support.writemdict import MDictWriter

    verbs = [("être", "<p>to be</p>"), ("suivre", "<p>to follow</p>"),
             ("se laver", "<p>to wash oneself</p>"), ("s'abstenir", "<p>to abstain</p>"),
             ("été", "<p>summer (noun)</p>"), ("double", "<p>first sense</p>"),
             ("double", "<p>second sense</p>")]
    updated = [(key, "<p>to be, revised</p>" if key == "être" else body) for key, body in verbs]
    cases = {"verbs.mdx": verbs, "verbs-updated.mdx": updated,
             "other.mdx": [("être", "<p>another dictionary</p>"), ("double", "<p>another sense</p>")],
             "formatting.mdx": [("école", """<style>body{color:#123456}.format{font-weight:bold;background:url(https://example.invalid/mdx.png)}</style>
<p style="color:#123456;font-weight:bold;background-image:url(https://example.invalid/mdx.png)">French sample: école</p><script>window.__TAURI_INTERNALS__.invoke('speech_preferences',{preferences:{voice:'male',wpm:100}});parent.__dictionary_escaped=true</script>
<img src="https://example.invalid/mdx.png" onerror="parent.__dictionary_escaped=true"><a href="https://example.invalid/leave">stay here</a>
<iframe src="https://example.invalid/frame"></iframe>""")],
             "response-count.mdx": [("many", "<p>one physical sense</p>")] * 201,
             "response-bytes.mdx": [("huge", "<p>" + "x" * (2 * 1024 * 1024) + "</p>")] * 2}
    args.output.mkdir(parents=True, exist_ok=False)
    report = []
    fixture_date = datetime.date(2026, 11, 9)
    for name, rows in cases.items():
        path = args.output / name
        writer = MDictWriter(Rows(rows), title="Lexique lookup fixture", description="Original project test data, MIT",
                             version="2.0", encoding="utf8", compression_type=2, block_size=4 * 1024 * 1024)
        with patch("writer_support.writemdict.datetime.date") as date, path.open("xb") as output:
            date.today.return_value = fixture_date
            writer.write(output)
        report.append({"file": name, "bytes": path.stat().st_size, "records": len(rows)})
    print(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
