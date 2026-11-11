"""Verify or rebuild the supplemental FreeDict MDX from its pinned TEI source."""

import argparse
import datetime
import hashlib
from html import escape
import importlib.util
import json
from pathlib import Path
import shutil
import sys
import tarfile
from unittest.mock import patch
import xml.etree.ElementTree as ET

from conjugation_meanings import NS, load_meanings

ROOT = Path(__file__).resolve().parents[1]
RESOURCE = Path("resources/freedict-fra-eng-mdx-0.4.1")


def verify(root=ROOT, source=None):
    directory = root / RESOURCE
    metadata = json.loads((directory / "manifest.json").read_text(encoding="utf-8"))
    source = source or root / "resources/freedict-fra-eng-0.4.1"
    _, provenance = load_meanings(source)
    data = (directory / metadata["file"]).read_bytes()
    if (len(data) != metadata["bytes"]
            or hashlib.sha256(data).hexdigest() != metadata["sha256"]
            or metadata["source_sha512"] != provenance["source_sha512"]
            or metadata["source"]["license"] != provenance["license"]
            or metadata["source"]["source_url"] != provenance["source_url"]):
        raise ValueError("Bundled FreeDict MDX or its source differs from the admitted resource")
    for notice in ("AUTHORS", "COPYING"):
        if (directory / notice).read_bytes() != (source / notice).read_bytes():
            raise ValueError(f"Bundled FreeDict MDX notice differs from its source: {notice}")
    return metadata


class Rows:
    def __init__(self, rows):
        self.rows = rows

    def __len__(self):
        return len(self.rows)

    def items(self):
        return self.rows


def build(output, writer):
    metadata = verify()
    source = ROOT / "resources/freedict-fra-eng-0.4.1"
    _, provenance = load_meanings(source)
    with tarfile.open(source / provenance["source_file"], mode="r:xz") as archive:
        root = ET.fromstring(archive.extractfile("fra-eng/fra-eng.tei").read())
    entries = root.findall("t:text/t:body/t:entry", NS)
    assert root.find("t:teiHeader/t:fileDesc/t:publicationStmt/t:date", NS).get("when") == "2018-08-11"
    rows = []
    for ordinal, entry in enumerate(entries, 1):
        key = entry.findtext("t:form/t:orth", namespaces=NS)
        assert key
        parts = [f"<h2 lang='fr'>{escape(key)}</h2>"]
        for pronunciation in entry.findall("t:form/t:pron", NS):
            parts.append(f"<p>{escape(' '.join(pronunciation.itertext()).strip())}</p>")
        grammar = entry.find("t:gramGrp", NS)
        if grammar is not None:
            parts.append(f"<p>{escape(' '.join(grammar.itertext()).strip())}</p>")
        senses = entry.findall("t:sense", NS)
        parts.append("<ol>" + "".join(f"<li>{escape(' '.join(sense.itertext()).strip())}</li>" for sense in senses) + "</ol>")
        parts.append(f"<p>FreeDict fra-eng 0.4.1 — source entry {ordinal}; GPL-2.0-or-later.</p>")
        rows.append((key, "".join(parts)))
    assert len(rows) == 8505 and len({key for key, _ in rows}) == 8417
    spec = importlib.util.spec_from_file_location(
        "freedict_writer", writer, submodule_search_locations=[str(writer.parent)])
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    description = ("FreeDict French-English 0.4.1, original content 2018-08-11; "
                   "Horst Eyermann, Michael Bunk, John Darrington and the FreeDict project. "
                   "Data GPL-2.0-or-later. https://freedict.org/; original source and AUTHORS/COPYING accompany this file. "
                   "Complete TEI records rendered as HTML; original translations retained; conversion 2026-11-10.")
    mdx = module.MDictWriter(Rows(rows), title="FreeDict français–anglais 0.4.1",
                            description=description, version="2.0", encoding="utf8", compression_type=2)
    output.mkdir(parents=True, exist_ok=False)
    destination = output / metadata["file"]
    conversion_date = datetime.date(2026, 11, 10)
    with patch.object(module.datetime, "date") as writer_date, destination.open("xb") as stream:
        writer_date.today.return_value = conversion_date
        mdx.write(stream)
    assert hashlib.sha256(destination.read_bytes()).hexdigest() == metadata["sha256"]
    shutil.copytree(source, output / "source/freedict-fra-eng-0.4.1")
    (output / "manifest.json").write_text(json.dumps(metadata, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    return metadata


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--writer", type=Path, help="Pinned mdict-utils writemdict.py (commit recorded in manifest)")
    args = parser.parse_args()
    if args.output and not args.writer:
        parser.error("--output requires --writer")
    result = build(args.output, args.writer) if args.output else verify()
    print(json.dumps({"status": "passed", "file": result["file"],
                      "bytes": result["bytes"], "records": result["records"]}))


if __name__ == "__main__":
    main()
