"""Source-admitted pronominal identities; independent rules, no upstream code execution."""

from collections import Counter, defaultdict
import json
import re

from conjugation_generation import BLOCKS, COMPOUND, SUBJECTS, elides, expressions, source_form


def reflexive(form, h_aspire, person=None, number=None, imperative=False, with_en=False):
    pn = f"{person}{number}"
    if imperative:
        return form + ({"2s": "-t’en", "1p": "-nous-en", "2p": "-vous-en"} if with_en else
                       {"2s": "-toi", "1p": "-nous", "2p": "-vous"})[pn]
    pronoun = {"1s": "me", "2s": "te", "1p": "nous", "2p": "vous"}.get(pn, "se")
    if pronoun in ("me", "te", "se") and (with_en or elides(form, h_aspire)):
        return pronoun[0] + "’" + ("en " if with_en else "") + form
    return pronoun + " " + ("en " if with_en else "") + form


def pronominal_metadata(lemma, verb, data, with_en=False):
    code = data["lVtyp"][data["dVerb"][lemma][0]] if lemma in data["dVerb"] else "_________"
    frames = [e for e in data["syntax"].get(lemma, []) if "@pron" in e["macros"].split(",")
              or re.search(r"seréfl|seréc", e["frame"])]
    if (with_en and (code[5] != "e" or lemma == "découdre")
            or not with_en and code[5] not in "qpruv" and not frames):
        return None
    if with_en:
        frames = []
    current = [e for e in frames if e["file"] == frames[0]["file"]] if frames else []
    direct = any(re.search(r"[,<]Obj:[^,>]*(?:seréfl|seréc)", e["frame"]) for e in current)
    indirect = any(re.search(r"[,<]Obj(?:à|de):[^,>]*(?:seréfl|seréc)", e["frame"]) for e in current)
    obligatory = current and all("@pron" in e["macros"].split(",") for e in current)
    object_frame = any(re.search(r"[,<]Obj:", e["frame"]) for e in current)
    agreement = "invariable" if lemma in {"plaire", "complaire", "déplaire", "rire"} or indirect and not direct and not object_frame or with_en and code[7] != "e" else (
        "subject" if (code[5] in "pre" or obligatory) and not object_frame else "contextual")
    persons = ["3s"] if code[6] == "m" or current and all("Suj:" not in e["frame"] for e in current) else list(SUBJECTS)
    defined_persons = {f"{p}{n}" for e in current for t, p, n in e["slots"] if t in ("P", "I", "F", "C", "J", "S", "T") and p and n}
    if defined_persons:
        persons = [p for p in persons if p in defined_persons]
    if code[5] == "r":
        persons = [p for p in persons if p not in ("1s", "2s")]
    if persons == ["3s"]:
        agreement = "invariable"
    labels = ["pronominal"] + (["impersonal"] if persons == ["3s"] else [])
    labels.extend(label for label in verb["usage_labels"] if label in {"historical", "rare", "falconry_participle"})
    return {**verb, "auxiliary_status": "supported", "auxiliaries": [{"lemma": "être", "use": "pronominal"}],
            "usage_labels": labels, "pronominal": {
                "base": lemma, "agreement": agreement, "persons": persons,
                "with_en": with_en, "reciprocal": code[5] == "r",
                "source": {"grammalecte_type": code, "lefff_entries": [{"file": e["file"], "line": e["line"],
                            "entry": e["entry"]} for e in frames]}}}


def pronominal_cells(headword, rows, metadata, h_aspire, auxiliary_form):
    info = metadata["pronominal"]
    cells = []
    seen = set()

    def add(line, form, tense, person=None, number=None, gender=None, context=False, collective=False):
        identity = (form, tense, person, number, gender)
        if identity in seen:
            return
        seen.add(identity)
        exp = expressions(form, tense, person, number, None if context else gender, False, False)
        if collective or info["reciprocal"] and (person, number) == ("3", "s"):
            exp = [e for e in exp if e.startswith(("on ", "qu’on "))]
        if info["persons"] == ["3s"] and person:
            exp = [e for e in exp if e.startswith(("il ", "qu’il "))]
        cells.append((line, form, headword, "pronominal/v1/" + info["agreement"], tense, person, number, gender,
                      json.dumps(exp, ensure_ascii=False)))

    for row in rows:
        line, form, _, _, tense, person, number, gender = row[:8]
        if tense in COMPOUND or tense in ("GP", None):
            continue
        if tense not in ("W", "G", "K") and f"{person}{number}" not in SUBJECTS:
            continue
        if person and f"{person}{number}" not in info["persons"]:
            continue
        if tense == "Y" and f"{person}{number}" not in ("2s", "1p", "2p"):
            continue
        if tense == "K" and info["agreement"] == "invariable" and (gender, number) not in (("m", "s"), (None, None)):
            continue
        if tense != "K":
            form = reflexive(form, h_aspire, person, number, tense == "Y", info["with_en"])
        add(line, form, tense, person, number, gender)
        if tense == "G":
            add(line, "en " + form, "GP")
    participles = [r for r in rows if r[4] == "K"]
    imperative_persons = {f"{r[5]}{r[6]}" for r in cells if r[4] == "Y"}
    for tense, base in COMPOUND.items():
        for pn in [None] if base in ("W", "G") else info["persons"]:
            if base == "Y" and pn not in imperative_persons:
                continue
            prefix = auxiliary_form("être", base, pn)
            if not prefix:
                continue
            for pp in participles:
                line, form, _, _, _, _, number, gender = pp[:8]
                if info["agreement"] == "invariable":
                    if (gender, number) not in (("m", "s"), (None, None)):
                        continue
                elif gender is None or number is None:
                    continue
                elif info["agreement"] == "subject" and pn and pn not in ("2p", "3s") and number != pn[1]:
                    continue
                verbal = reflexive(prefix, False, pn[0] if pn else None, pn[1] if pn else None,
                                   base == "Y", info["with_en"])
                add(line, ("en " if tense == "GPP" else "") + verbal + " " + form, tense,
                    pn[0] if pn else None, pn[1] if pn else None,
                    gender if info["agreement"] != "invariable" else None, info["agreement"] == "contextual",
                    info["agreement"] == "subject" and pn == "3s" and number == "p")
    return cells


def generate_pronominal(rows, verbs, data):
    by_lemma = defaultdict(list)
    for row in rows:
        by_lemma[row[2]].append(row)
    generated, identities, excluded, admissions = [], {}, [], {}
    coverage = {code: Counter() for code in BLOCKS}
    for lemma, verb in verbs.items():
        admitted = False
        for with_en in (False, True):
            metadata = pronominal_metadata(lemma, verb, data, with_en)
            if metadata is not None:
                admitted = True
                admissions[reflexive(lemma, lemma in data["h_aspire"], with_en=with_en)] = (lemma, metadata)
        if not admitted:
            excluded.append({"lemma": lemma, "reason": "no_explicit_pronominal_use"})
    for headword, (lemma, metadata) in admissions.items():
        info = metadata["pronominal"]
        cells = pronominal_cells(headword, by_lemma[lemma], metadata, lemma in data["h_aspire"],
                                 lambda aux, tense, pn: source_form(data, aux, tense, ":" + pn if pn else ":P"))
        defined = {tuple(s) for e in data["syntax"].get(lemma, []) for s in e["slots"]}
        statuses = {}
        for code in BLOCKS:
            if any(r[4] == code for r in cells):
                statuses[code] = "supported"
            elif defined and (code not in COMPOUND and not any(s[0] == ("G" if code == "GP" else code) for s in defined)
                              or code in COMPOUND and not any(s[0] == "K" for s in defined)
                              or code == "YP" and not any(r[4] == "Y" for r in cells)):
                statuses[code] = "not_applicable"
            else:
                statuses[code] = "missing"
        if len(info["persons"]) < 6:
            for code in [c for c in BLOCKS if c not in ("W", "WP", "K", "G", "GP", "KP", "GPP")]:
                for pn in SUBJECTS:
                    if pn not in info["persons"]:
                        statuses[f"{code}/{pn}"] = "not_applicable"
        if info["persons"] == ["3s"]:
            statuses.update(Y="not_applicable", YP="not_applicable")
        metadata["blocks"] = statuses
        identities[headword] = metadata
        generated.extend(cells)
        for code in BLOCKS:
            coverage[code][statuses[code]] += 1
    return generated, identities, {"identities": len(identities), "base_lemmas": len({v["pronominal"]["base"] for v in identities.values()}), "excluded": excluded,
                                    "construction_exclusions": [{"lemma": "s’en découdre", "reason": "non_reflexive_en_locution"}],
                                    "incomplete": [{"lemma": lemma, "blocks": [k for k, v in verb["blocks"].items() if v in ("missing", "unconfirmed")]}
                                                   for lemma, verb in identities.items() if any(v in ("missing", "unconfirmed") for v in verb["blocks"].values())],
                                    "blocks": {code: dict(counts) for code, counts in coverage.items()}}
