"""Read pinned linguistic metadata; no upstream conjugator code is executed."""

from collections import Counter
import gzip
import hashlib
import json
from pathlib import Path

RESOURCE = Path(__file__).resolve().parents[1] / "resources/grammalecte-2.3.0"
AUXILIARY_REFERENCE = "https://vitrinelinguistique.oqlf.gouv.qc.ca/24584/la-grammaire/le-verbe/auxiliaires/tableau-synthese-de-lemploi-des-auxiliaires-avoir-et-etre"


def load_metadata(directory):
    directory = Path(directory)
    expected = json.loads((RESOURCE / "provenance.json").read_text(encoding="utf-8"))
    provenance = json.loads((directory / "provenance.json").read_text(encoding="utf-8"))
    if provenance != expected:
        raise ValueError("Verb metadata provenance differs from pinned Grammalecte")
    for name, digest in expected["files"].items():
        if hashlib.sha256((directory / name).read_bytes()).hexdigest() != digest:
            raise ValueError(f"Verb metadata SHA-256 mismatch: {name}")
    raw = gzip.decompress((directory / "conj_data.json.gz").read_bytes())
    if hashlib.sha256(raw).hexdigest() != expected["source_sha256"]:
        raise ValueError("Verb metadata uncompressed SHA-256 mismatch")
    data = json.loads(raw)
    corrections = json.loads((directory / "corrections.json").read_text(encoding="utf-8"))
    if (not isinstance(data, dict) or not isinstance(data.get("dVerb"), dict)
            or not isinstance(data.get("lVtyp"), list)
            or len(data["dVerb"]) != expected["verbs"]):
        raise ValueError("Incomplete verb metadata")
    return data, corrections, provenance


def verb_metadata(lemma, data, corrections):
    result = {"group": None, "group_status": "unconfirmed", "auxiliaries": [],
              "auxiliary_status": "unconfirmed", "ordinary_status": "unconfirmed",
              "usage_labels": [], "source": None, "correction": None}
    if lemma not in data["dVerb"]:
        return result
    code = data["lVtyp"][data["dVerb"][lemma][0]]
    result["source"] = {"resource": "grammalecte-2.3.0", "entry": f"dVerb/{lemma}",
                        "raw_type": code}
    # Grammalecte's group 0 denotes auxiliaries; M3 uses the traditional three groups.
    result.update(group=3 if code[0] == "0" else int(code[0]), group_status="supported")
    result["usage_labels"] = [label for index, flag, label in (
        (2, "i", "intransitive"), (3, "t", "transitive"),
        (4, "n", "indirect_transitive"), (6, "m", "impersonal")) if code[index] == flag]
    if any(code[index] == flag for index, flag in ((2, "i"), (3, "t"), (4, "n"))):
        result["ordinary_status"] = "supported"
    elif code[5] in "pr":
        result["ordinary_status"] = "not_applicable"
        result["auxiliary_status"] = "not_applicable"
        result["usage_labels"].append("pronominal_only")
    if result["ordinary_status"] == "supported":
        result["auxiliaries"] = [{"lemma": auxiliary, "use": "ordinary active; sense mapping unconfirmed"}
                                 for index, flag, auxiliary in ((8, "a", "avoir"), (7, "e", "être"))
                                 if code[index] == flag]
        if len(result["auxiliaries"]) == 1:
            result["auxiliary_status"] = "supported"
            result["auxiliaries"][0]["use"] = "ordinary active"
    correction = corrections.get(lemma)
    if correction:
        result["correction"] = correction
        for key in ("group", "group_status", "auxiliaries"):
            if key in correction:
                result[key] = correction[key]
        if "auxiliaries" in correction:
            result["auxiliary_status"] = "supported"
    if not result["auxiliaries"] and result["ordinary_status"] == "supported" and (code[3] == "t" or code[4] == "n"):
        result["auxiliaries"] = [{"lemma": "avoir", "use": "transitive" if code[3] == "t" else "indirect_transitive"}]
        result["auxiliary_status"] = "supported"
        result["correction"] = {
            "note": "Ordinary transitive constructions take avoir; other uses remain unconfirmed when upstream auxiliary flags are incomplete.",
            "references": [AUXILIARY_REFERENCE],
        }
    if lemma in {"entrer", "rentrer", "retourner", "sortir", "ressortir", "tomber"}:
        result["auxiliaries"] = [{"lemma": "avoir", "use": "indirect_transitive" if lemma == "ressortir" else "transitive"},
                                 {"lemma": "être", "use": "intransitive"}]
        result["auxiliary_status"] = "supported"
        result["correction"] = {**(result["correction"] or {}),
                                "note": "Auxiliary depends on ordinary use; avoir for transitive and être for intransitive movement.",
                                "references": [AUXILIARY_REFERENCE]}
    return result


def metadata_report(verbs):
    return {
        "verbs": len(verbs),
        **{field: dict(sorted(Counter(v[field] for v in verbs.values()).items()))
           for field in ("group_status", "auxiliary_status", "ordinary_status")},
        "groups": dict(sorted(Counter(str(v["group"]) for v in verbs.values()).items())),
        "missing_lemmas": sorted(lemma for lemma, verb in verbs.items() if verb["source"] is None),
        "unconfirmed_auxiliary_lemmas": sorted(lemma for lemma, verb in verbs.items()
                                               if verb["auxiliary_status"] == "unconfirmed"),
        "corrections": sorted(lemma for lemma, verb in verbs.items() if verb["correction"]),
        "excluded_source_rows": 0,
        "forms": "Unfiltered Lefff source rows; unresolved or malformed-looking forms retain provenance.",
    }
