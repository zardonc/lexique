"""Integration checks through the resource build and read-only query interfaces."""

import hashlib
import json
import sqlite3
import subprocess
import sys
import tarfile
import tempfile
import unittest
import xml.etree.ElementTree as ET
from contextlib import closing
from pathlib import Path

from dictionary_sample import SOURCE, Dictionary, build, prepare


class DictionarySampleTests(unittest.TestCase):
    def test_prepare_reuses_only_verified_generated_resources(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "sample"
            first = prepare(output)
            database = output / "dictionary.sqlite3"
            before = database.stat().st_mtime_ns
            self.assertEqual(prepare(output), first)
            self.assertEqual(database.stat().st_mtime_ns, before)
            database.write_bytes(b"corrupt")
            with self.assertRaises(sqlite3.DatabaseError):
                prepare(output)
            self.assertEqual(database.read_bytes(), b"corrupt")

    def test_build_preserves_real_unicode_and_same_key_records(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "sample"
            report = build(output)
            self.assertEqual(report["records"], 10947)
            self.assertEqual(report["distinct_keys"], 10777)
            database = output / "dictionary.sqlite3"
            before = hashlib.sha256(database.read_bytes()).hexdigest()
            with Dictionary(database) as dictionary:
                self.assertIn("学校", dictionary.lookup("école")[0]["entry_xml"])
                self.assertIn("眼睛", dictionary.lookup("œil")[0]["entry_xml"])
                self.assertEqual(len(dictionary.lookup("or")), 3)
                self.assertTrue(dictionary.lookup("Côte d’Ivoire"))
                self.assertTrue(dictionary.lookup("vacances"))
                self.assertEqual(dictionary.lookup("sommes"), [])
                self.assertEqual(dictionary.lookup("' OR 1=1 --"), [])
            self.assertEqual(hashlib.sha256(database.read_bytes()).hexdigest(), before)

    def test_rejects_invalid_resources_and_preserves_existing_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            missing = root / "missing.sqlite3"
            with self.assertRaises(sqlite3.OperationalError):
                Dictionary(missing)
            self.assertFalse(missing.exists())
            corrupt = root / "corrupt.sqlite3"
            corrupt.write_bytes(b"not a SQLite resource")
            with self.assertRaises(sqlite3.DatabaseError):
                Dictionary(corrupt)
            output = root / "sample"
            build(output)
            database = output / "dictionary.sqlite3"
            before = database.read_bytes()
            with self.assertRaises(FileExistsError):
                build(output)
            self.assertEqual(database.read_bytes(), before)
            # A structurally valid SQLite file with lost records must not look like "no matches".
            with closing(sqlite3.connect(database)) as connection:
                with connection:
                    connection.execute("DELETE FROM entries")
            with self.assertRaisesRegex(ValueError, "record count"):
                with Dictionary(database):
                    pass

    def test_rejects_unverified_source_before_creating_output(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "changed.tar.xz"
            source.write_bytes(SOURCE.read_bytes() + b"modified")
            with self.assertRaisesRegex(ValueError, "SHA-512"):
                build(root / "output", source=source)
            self.assertFalse((root / "output").exists())

    def test_rejects_altered_content_and_unrelated_learning_database(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            learning = root / "learning.sqlite3"
            with closing(sqlite3.connect(learning)) as connection:
                connection.execute("CREATE TABLE user_notes (text TEXT)")
            original = learning.read_bytes()
            with self.assertRaisesRegex(ValueError, "format"):
                with Dictionary(learning):
                    pass
            self.assertEqual(learning.read_bytes(), original)
            output = root / "sample"
            build(output)
            database = output / "dictionary.sqlite3"
            with closing(sqlite3.connect(database)) as connection:
                with connection:
                    connection.execute("UPDATE entries SET entry_xml = '<entry/>' WHERE ordinal = 1")
            with self.assertRaisesRegex(ValueError, "content checksum"):
                with Dictionary(database):
                    pass

    def test_query_command_distinguishes_no_match_from_success_and_corruption(self):
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "sample"
            build(output)
            database = output / "dictionary.sqlite3"
            command = [sys.executable, "-X", "utf8", str(Path(__file__).with_name("dictionary_sample.py")),
                       "query", str(database)]
            found = subprocess.run([*command, "œil"], capture_output=True, text=True, encoding="utf-8", check=False)
            self.assertEqual(found.returncode, 0, found.stderr)
            self.assertIn("眼睛", json.loads(found.stdout)[0]["entry_xml"])
            missing = subprocess.run([*command, "sommes"], capture_output=True, text=True, encoding="utf-8", check=False)
            self.assertEqual(missing.returncode, 2, missing.stderr)
            self.assertEqual(json.loads(missing.stdout), [])
            database.write_bytes(b"broken")
            corrupt = subprocess.run([*command, "œil"], capture_output=True, text=True, encoding="utf-8", check=False)
            self.assertEqual(corrupt.returncode, 1)
            self.assertIn("DatabaseError", corrupt.stderr)

    def test_every_source_record_survives_query_and_build_is_reproducible(self):
        with tarfile.open(SOURCE) as source:
            root = ET.fromstring(source.extractfile("fra-zho/fra-zho.tei").read())
        namespace = {"t": "http://www.tei-c.org/ns/1.0"}
        expected = {}
        for ordinal, entry in enumerate(root.findall("t:text/t:body/t:entry", namespace), 1):
            key = entry.findtext("t:form/t:orth", namespaces=namespace)
            expected.setdefault(key, []).append((ordinal, entry))
        with tempfile.TemporaryDirectory() as directory:
            first = Path(directory) / "first"
            second = Path(directory) / "second"
            report = build(first)
            build(second)
            with Dictionary(first / "dictionary.sqlite3") as dictionary:
                for key, entries in expected.items():
                    actual = dictionary.lookup(key)
                    self.assertEqual([row["ordinal"] for row in actual], [ordinal for ordinal, _ in entries])
                    for row, (_, entry) in zip(actual, entries, strict=True):
                        # Compare every element, attribute and text independently of serialization/prefixes.
                        parsed = ET.fromstring(row["entry_xml"])
                        source_values = [(node.tag.split("}")[-1], node.attrib, node.text, node.tail)
                                         for node in entry.iter()]
                        query_values = [(node.tag, node.attrib, node.text, node.tail) for node in parsed.iter()]
                        source_values[0] = (*source_values[0][:3], None)
                        self.assertEqual(query_values, source_values, key)
            for name in ("dictionary.sqlite3", "sample.tar.gz", "report.json"):
                self.assertEqual((first / name).read_bytes(), (second / name).read_bytes(), name)
            with tarfile.open(first / "sample.tar.gz") as bundle:
                self.assertEqual(set(bundle.getnames()), {
                    "dictionary.sqlite3", "COPYING", "provenance.json", SOURCE.name, SOURCE.name + ".sha512"
                })
                self.assertEqual(report["installed_payload_bytes"], sum(member.size for member in bundle.getmembers()))
                for member in bundle.getmembers():
                    self.assertEqual(bundle.extractfile(member).read(), (first / member.name).read_bytes())
            self.assertEqual(report["compressed_download_bytes"], (first / "sample.tar.gz").stat().st_size)
            self.assertEqual(json.loads((first / "report.json").read_text(encoding="utf-8")), report)


if __name__ == "__main__":
    unittest.main()
