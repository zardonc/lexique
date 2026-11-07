"""Project verb translation senses from the pinned FreeDict French-English source."""

from collections import defaultdict
import hashlib
import io
import json
from pathlib import Path
import tarfile
import xml.etree.ElementTree as ET

RESOURCE = Path(__file__).resolve().parents[1] / "resources/freedict-fra-eng-0.4.1"
NS = {"t": "http://www.tei-c.org/ns/1.0"}


def load_meanings(directory=RESOURCE):
    directory = Path(directory)
    expected = json.loads((RESOURCE / "provenance.json").read_text(encoding="utf-8"))
    if json.loads((directory / "provenance.json").read_text(encoding="utf-8")) != expected:
        raise ValueError("English meaning provenance differs from pinned FreeDict")
    for name, digest in expected["files"].items():
        if hashlib.sha256((directory / name).read_bytes()).hexdigest() != digest:
            raise ValueError(f"English meaning source SHA-256 mismatch: {name}")
    archive = (directory / expected["source_file"]).read_bytes()
    if hashlib.sha512(archive).hexdigest() != expected["source_sha512"]:
        raise ValueError("English meaning release checksum mismatch")
    published = (directory / (expected["source_file"] + ".sha512")).read_text().split()
    if published != [expected["source_sha512"], expected["source_file"]]:
        raise ValueError("English meaning published checksum mismatch")
    with tarfile.open(fileobj=io.BytesIO(archive), mode="r:xz") as package:
        tei = package.extractfile("fra-eng/fra-eng.tei").read()
        if package.extractfile("fra-eng/COPYING").read() != (directory / "COPYING").read_bytes():
            raise ValueError("English meaning license differs from original")
    entries = ET.fromstring(tei).findall("t:text/t:body/t:entry", NS)
    if len(entries) != expected["entries"]:
        raise ValueError("English meaning source entry count mismatch")
    meanings = defaultdict(list)
    for ordinal, entry in enumerate(entries, 1):
        if entry.findtext("t:gramGrp/t:pos", namespaces=NS) not in ("v", "vi", "vt"):
            continue
        headword = entry.findtext("t:form/t:orth", namespaces=NS)
        if not headword:
            raise ValueError(f"Missing English meaning headword: {ordinal}")
        lemma, use = headword, "ordinary"
        for prefix in ("se ", "s'", "s’"):
            if headword.startswith(prefix):
                lemma, use = headword[len(prefix):], "pronominal"
                break
        # The source's taire translations describe se taire, not ordinary taire.
        if headword == "taire":
            continue
        for index, sense in enumerate(entry.findall("t:sense", NS), 1):
            glosses = [" ".join(q.itertext()).strip() for q in sense.findall("t:cit[@type='trans']/t:quote", NS)]
            if not glosses:
                continue
            # This source sense belongs to retentir, not retenir.
            if headword == "retenir" and index == 6 and glosses == ["thunder"]:
                continue
            if any(not g or any(ord(c) < 32 for c in g) for g in glosses):
                raise ValueError(f"Invalid English meaning translation: {ordinal}/{index}")
            labels = [" ".join(u.itertext()).strip() for u in sense.findall("t:usg", NS)]
            meanings[lemma].append({"source_entry": ordinal, "source_sense": index, "headword": headword,
                                    "use": use, "labels": labels, "glosses": glosses})
    return dict(meanings), expected


def verb_meanings(lemma, verb, meanings):
    pronominal = verb.get("pronominal")
    key = ("en " if pronominal["with_en"] else "") + pronominal["base"] if pronominal else lemma
    senses = [s for s in meanings.get(key, [])
              if s["use"] == "pronominal" or not pronominal and verb["ordinary_status"] == "supported"]
    return {"status": "available" if senses else "unavailable", "senses": senses}
