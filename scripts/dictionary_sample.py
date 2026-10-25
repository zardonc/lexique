"""Build and query the admitted FreeDict sample, offline, using Python 3.12+ stdlib."""

import argparse
import gzip
import hashlib
import io
import json
import platform
import re
import shutil
import sqlite3
import tarfile
import xml.etree.ElementTree as ET
from pathlib import Path

RESOURCE = Path(__file__).resolve().parents[1] / "resources/freedict-fra-zho-2025.11.23"
PROVENANCE = json.loads((RESOURCE / "provenance.json").read_text(encoding="utf-8"))
SOURCE = RESOURCE / PROVENANCE["source_file"]
CONVERSION_VERSION = 1
APPLICATION_ID = 0x4C585244
NS = {"t": "http://www.tei-c.org/ns/1.0"}


def source_records(source=SOURCE):
    """Validate the pinned archive before parsing; never extract or execute its files."""
    with Path(source).open("rb") as stream:
        archive = stream.read(1_000_001)
    if len(archive) > 1_000_000 or hashlib.sha512(archive).hexdigest() != PROVENANCE["source_sha512"]:
        raise ValueError("Source SHA-512 does not match the pinned FreeDict release")
    published = (RESOURCE / (SOURCE.name + ".sha512")).read_text(encoding="utf-8").split()
    if published != [PROVENANCE["source_sha512"], SOURCE.name]:
        raise ValueError("Published FreeDict checksum differs from provenance")
    with tarfile.open(fileobj=io.BytesIO(archive), mode="r:xz") as package:
        def read_member(name):
            member = package.getmember(name)
            if not member.isfile() or member.size > 8_000_000:
                raise ValueError(f"Invalid source member: {name}")
            return package.extractfile(member).read()

        tei = read_member("fra-zho/fra-zho.tei")
        license_text = read_member("fra-zho/COPYING")
    if license_text != (RESOURCE / "COPYING").read_bytes():
        raise ValueError("Original license differs from retained COPYING")
    root = ET.fromstring(tei)
    entries = root.findall("t:text/t:body/t:entry", NS)
    fragments = re.findall(r"<entry>.*?</entry>", tei.decode("utf-8"), re.DOTALL)
    if len(entries) != 10947 or len(fragments) != len(entries):
        raise ValueError("Unexpected TEI entry structure or record count")
    records = []
    for ordinal, (entry, fragment) in enumerate(zip(entries, fragments, strict=True), 1):
        key = entry.findtext("t:form/t:orth", namespaces=NS)
        if not key or ET.fromstring(fragment).findtext("form/orth") != key:
            raise ValueError(f"Invalid TEI key at source record {ordinal}")
        records.append({"ordinal": ordinal, "key": key, "entry_xml": fragment})
    return records, tei, archive


class Dictionary:
    """Exact, case-sensitive, read-only lookup; no results is [], corruption raises."""

    def __init__(self, database):
        self.connection = sqlite3.connect(Path(database).resolve().as_uri() + "?mode=ro", uri=True)
        self.connection.row_factory = sqlite3.Row
        try:
            self.connection.execute("PRAGMA trusted_schema = OFF")
            if self.connection.execute("PRAGMA integrity_check").fetchall()[0][0] != "ok":
                raise ValueError("Dictionary resource failed SQLite integrity check")
            if (self.connection.execute("PRAGMA application_id").fetchone()[0] != APPLICATION_ID
                    or self.connection.execute("PRAGMA user_version").fetchone()[0] != CONVERSION_VERSION):
                raise ValueError("Unsupported dictionary resource format")
            metadata = json.loads(self.connection.execute("SELECT provenance FROM resource").fetchone()[0])
            if metadata != PROVENANCE:
                raise ValueError("Dictionary resource provenance differs from the admitted sample")
            if self.connection.execute("SELECT count(*), count(DISTINCT key) FROM entries").fetchone()[:] != (10947, 10777):
                raise ValueError("Dictionary resource record count differs from the admitted sample")
            records = [dict(row) for row in self.connection.execute(
                "SELECT ordinal, key, entry_xml FROM entries ORDER BY ordinal"
            )]
            if content_checksum(records) != PROVENANCE["records_sha256"]:
                raise ValueError("Dictionary resource content checksum differs from the admitted sample")
        except Exception:
            self.connection.close()
            raise

    def lookup(self, key):
        return [dict(row) for row in self.connection.execute(
            "SELECT ordinal, key, entry_xml FROM entries WHERE key = ? ORDER BY ordinal", (key,)
        )]

    def __enter__(self):
        return self

    def __exit__(self, *_):
        self.connection.close()


def content_checksum(records):
    content = json.dumps(records, ensure_ascii=False, separators=(",", ":")).encode("utf-8")
    return hashlib.sha256(content).hexdigest()


def build(output, source=SOURCE):
    """Create a new directory only; never overwrite resources or user data."""
    records, tei, archive = source_records(source)
    if content_checksum(records) != PROVENANCE["records_sha256"]:
        raise ValueError("Converted content checksum differs from the admitted sample")
    output = Path(output)
    output.mkdir(parents=True, exist_ok=False)
    database = output / "dictionary.sqlite3"
    connection = sqlite3.connect(database)
    try:
        with connection:
            connection.executescript(f"""
                PRAGMA application_id = {APPLICATION_ID};
                PRAGMA user_version = {CONVERSION_VERSION};
                CREATE TABLE resource (provenance TEXT NOT NULL);
                CREATE TABLE entries (
                    ordinal INTEGER PRIMARY KEY, key TEXT NOT NULL COLLATE BINARY,
                    entry_xml TEXT NOT NULL
                );
                CREATE INDEX entries_key ON entries(key);
            """)
            connection.execute("INSERT INTO resource VALUES (?)", (json.dumps(PROVENANCE, ensure_ascii=False),))
            connection.executemany("INSERT INTO entries VALUES (:ordinal, :key, :entry_xml)", records)
    finally:
        connection.close()
    (output / SOURCE.name).write_bytes(archive)
    for name in ("COPYING", "provenance.json", SOURCE.name + ".sha512"):
        shutil.copyfile(RESOURCE / name, output / name)
    payload = sorted(output.iterdir())
    bundle = output / "sample.tar.gz"
    with bundle.open("wb") as compressed:
        with gzip.GzipFile(filename="", fileobj=compressed, mode="wb", mtime=0) as gzip_stream:
            with tarfile.open(fileobj=gzip_stream, mode="w", format=tarfile.USTAR_FORMAT) as package:
                for path in payload:
                    content = path.read_bytes()
                    member = tarfile.TarInfo(path.name)
                    member.size = len(content)
                    member.mode = 0o644
                    package.addfile(member, io.BytesIO(content))
    report = {
        "resource_id": PROVENANCE["resource_id"], "conversion_version": CONVERSION_VERSION,
        "records": len(records), "distinct_keys": len({record["key"] for record in records}),
        "source_archive_bytes": len(archive), "source_tei_bytes": len(tei),
        "sqlite_bytes": database.stat().st_size,
        "sqlite_sha256": hashlib.sha256(database.read_bytes()).hexdigest(),
        "installed_payload_bytes": sum(path.stat().st_size for path in payload),
        "compressed_download_bytes": bundle.stat().st_size,
        "bundle_sha256": hashlib.sha256(bundle.read_bytes()).hexdigest(),
        "python_version": platform.python_version(), "sqlite_version": sqlite3.sqlite_version,
    }
    (output / "report.json").write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    return report


def prepare(output):
    """Reuse a verified generated directory; damaged outputs require explicit repair."""
    output = Path(output)
    if not output.exists():
        return build(output)
    with Dictionary(output / "dictionary.sqlite3"):
        pass
    for name in (SOURCE.name, "COPYING", "provenance.json", SOURCE.name + ".sha512"):
        if (output / name).read_bytes() != (RESOURCE / name).read_bytes():
            raise ValueError(f"Generated resource differs from its source: {name}")
    report = json.loads((output / "report.json").read_text(encoding="utf-8"))
    for filename, field in (("dictionary.sqlite3", "sqlite_sha256"), ("sample.tar.gz", "bundle_sha256")):
        if hashlib.sha256((output / filename).read_bytes()).hexdigest() != report[field]:
            raise ValueError(f"Generated resource checksum mismatch: {filename}")
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("build").add_argument("output", type=Path)
    commands.add_parser("prepare").add_argument("output", type=Path)
    query = commands.add_parser("query")
    query.add_argument("database", type=Path)
    query.add_argument("key")
    arguments = parser.parse_args()
    if arguments.command in ("build", "prepare"):
        result = build(arguments.output) if arguments.command == "build" else prepare(arguments.output)
    else:
        with Dictionary(arguments.database) as dictionary:
            result = dictionary.lookup(arguments.key)
    print(json.dumps(result, ensure_ascii=False, indent=2))
    return 0 if arguments.command != "query" or result else 2


if __name__ == "__main__":
    raise SystemExit(main())
