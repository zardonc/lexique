"""Project ordinary frames and explicit availability from pinned Lefff syntax.

This reads data, not the upstream compiler. Original bytes accompany each build.
"""

from collections import defaultdict
from functools import cache
import gzip
import hashlib
import json
from pathlib import Path
import re
import xml.etree.ElementTree as ET

RESOURCE = Path(__file__).resolve().parents[1] / "resources/lefff-syntax"


def slots(tag):
    return {(tense, person, number) for tense in tag if tense in "PFIJCYSTKGW"
            for person in ([c for c in tag if c in "123"] or [None])
            for number in ([c for c in tag if c in "sp"] or [None])}


def load_syntax(directory):
    directory = Path(directory)
    expected = json.loads((RESOURCE / "provenance.json").read_text(encoding="utf-8"))
    provenance = json.loads((directory / "provenance.json").read_text(encoding="utf-8"))
    if provenance != expected:
        raise ValueError("Syntactic provenance differs from pinned Lefff")
    for name, digest in expected["files"].items():
        if hashlib.sha256((directory / name).read_bytes()).hexdigest() != digest:
            raise ValueError(f"Syntactic source SHA-256 mismatch: {name}")
    tables = {t.attrib["name"]: t for t in ET.fromstring((directory / "morpho.fr").read_bytes()).findall("table")}

    @cache
    def available(name):
        name = name.split(":")[0]
        table = tables[name]
        result = set()
        for inherit in table.findall("like"):
            inherited = set(available(inherit.attrib["name"]))
            for exception in inherit.attrib.get("except", "").split():
                excluded = slots(exception)
                inherited = {s for s in inherited if not any(all(e is None or e == v for e, v in zip(x, s)) for x in excluded)}
            result.update(inherited)
        for form in table.iter("form"):
            result.update(slots(form.attrib.get("tag", "")))
        return result

    manual = defaultdict(set)
    for line in (directory / "v_new.mf").read_text(encoding="utf-8").splitlines():
        fields = line.split("\t")
        if len(fields) >= 3:
            manual[fields[1]].update(slots(fields[2]))
    entries = defaultdict(list)
    for name in ("v_new.ilex", "v.ilex", "v-phd.ilex"):
        for line_number, raw in enumerate(gzip.decompress((directory / (name + ".gz")).read_bytes()).splitlines(), 1):
            # One legacy comment is Latin-1; the three data fields are UTF-8.
            fields = [part.decode("utf-8") for part in raw.split(b"\t")[:3]]
            if len(fields) != 3 or fields[0].startswith("#"):
                continue
            entry, pattern, description = fields
            lemma = entry.split("___")[0]
            parts = description.split(";")
            if len(parts) != 6:
                raise ValueError(f"Unsupported syntactic entry: {name}:{line_number}")
            if parts[2] != "v":
                continue
            _, surface, _, frame, macros, constructions = parts
            flags = set(macros.split(","))
            ordinary = "@pron" not in flags
            active = set(constructions.split(","))
            impersonal = any(c.startswith("%actif_impersonnel") for c in active) and "%actif" not in active
            aux = ["avoir", "être"] if "@être_possible" in flags else ["être"] if "@être" in flags else ["avoir"]
            use = "transitive" if re.search(r"[,<]Obj:", frame) else "indirect_transitive" if re.search(r"[,<]Obj[àde]+:", frame) else "intransitive"
            defined = manual[lemma] if pattern == "0" else available(pattern)
            entries[lemma].append({"file": name, "line": line_number, "entry": entry, "pattern": pattern,
                                   "frame": frame, "macros": macros, "constructions": constructions,
                                   "ordinary": ordinary, "impersonal": impersonal, "auxiliaries": aux,
                                   "use": use, "slots": sorted(defined, key=str)})
    for frames in entries.values():
        # Prefer current ordinary frames; absence of that use permits older positive evidence.
        ordinary_file = next((e["file"] for e in frames if e["ordinary"]), None)
        for entry in frames:
            entry["ordinary"] = entry["ordinary"] and entry["file"] == ordinary_file
    return dict(entries), provenance


def apply_syntax(lemma, result, entries):
    source = entries.get(lemma, [])
    if lemma in {"revoici", "revoilà", "voici", "voilà", "uw", "uwSe", "_error"}:
        # Non-infinitival presentatives and upstream sentinels stay queryable.
        result.update(ordinary_status="not_applicable", auxiliary_status="not_applicable", auxiliaries=[])
        result["usage_labels"].append("non_ordinary_source_identity")
        return result
    if not source:
        return result
    result["source"] = {**(result["source"] or {}), "syntax": {"resource": "lefff-syntax", "entries": [
        {k: v for k, v in e.items() if k in ("file", "line", "entry", "pattern", "frame", "macros", "constructions")} for e in source]}}
    ordinary = [e for e in source if e["ordinary"]]
    if not ordinary:
        result.update(ordinary_status="not_applicable", auxiliary_status="not_applicable", auxiliaries=[])
        result["usage_labels"] = ["pronominal_only"]
        return result
    result.update(ordinary_status="supported", auxiliary_status="supported")
    if result["group"] is None and result["correction"] is None:
        groups = {1 if e["pattern"].startswith("v-er:") or e["pattern"] == "v-er" else 2 if e["pattern"] == "v-ir2" else 3 for e in ordinary}
        if len(groups) == 1:
            result.update(group=groups.pop(), group_status="supported")
    previous_uses = set(result["usage_labels"]) & {"transitive", "indirect_transitive", "intransitive"}
    result["usage_labels"] = sorted({e["use"] for e in ordinary} | previous_uses)
    if all(e["impersonal"] for e in ordinary):
        result["usage_labels"].append("impersonal")
    result["usage_labels"].extend((result["correction"] or {}).get("usage_labels", []))
    auxiliaries = {(a, e["use"]) for e in ordinary for a in e["auxiliaries"]}
    # Older usage labels enrich an admitted auxiliary, never introduce one.
    if any(aux == "avoir" for aux, _ in auxiliaries):
        auxiliaries.update(("avoir", use) for use in previous_uses if use in {"transitive", "indirect_transitive"})
    if not result["correction"] or "auxiliaries" not in result["correction"]:
        result["auxiliaries"] = [{"lemma": a, "use": "; ".join(sorted({use for aux, use in auxiliaries if aux == a}))}
                                 for a in sorted({aux for aux, _ in auxiliaries})]
    return result
