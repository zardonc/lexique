"""Ordinary active constructions from pinned, separately licensed source forms.

The suffix-coded representation is documented by Grammalecte's bundled conj.py.
No upstream executable code is imported. See docs/evidence/issue-41/README.md.
"""

import json
import re
from collections import Counter, defaultdict

SIMPLE = {"G": (":P", 0), "K": (":Q", 1), "P": (":Ip", 2), "I": (":Iq", 3),
          "J": (":Is", 4), "F": (":If", 5), "C": (":K", 6), "S": (":Sp", 7),
          "T": (":Sq", 8), "Y": (":E", 9)}
COMPOUND = {"PC": "P", "PQP": "I", "PA": "J", "FA": "F", "CP": "C",
            "SP": "S", "SQP": "T", "YP": "Y", "WP": "W", "KP": "G", "GPP": "G"}
BLOCKS = (*SIMPLE, "W", *COMPOUND, "GP")
SUBJECTS = {"1s": ["je"], "2s": ["tu"], "3s": ["il", "elle", "on"],
            "1p": ["nous"], "2p": ["vous"], "3p": ["ils", "elles"]}


def source_form(data, lemma, tense, who, model=None):
    if tense == "W":
        return lemma
    family, index = SIMPLE[tense]
    pattern = data["lTags"][data["dVerb"][model or lemma][1]][index]
    suffix = data["dPatternConj"][family][pattern].get(who, "")
    if not suffix:
        return ""
    cut = ord(suffix[0]) - ord("0")
    return (lemma[:-cut] if cut else lemma) + suffix[1:]


def elides(form, h_aspire):
    return bool(re.match("[aàâeéèêëiîïoôœuùûü]", form)) or form.startswith("h") and not h_aspire or form.startswith("y") and not re.match("y[aeiouy]", form)


def expressions(form, tense, person, number, gender, impersonal, h_aspire):
    if tense in ("Y", "YP", "W", "WP", "G", "K", "KP", "GP", "GPP"):
        return [form]
    if impersonal and (person, number) != ("3", "s"):
        return [form]
    pronouns = ["il"] if impersonal and person == "3" and number == "s" else SUBJECTS.get(f"{person}{number}", [])
    result = []
    for subject in pronouns:
        if gender == "f" and subject in ("il", "ils") or gender == "m" and subject in ("elle", "elles"):
            continue
        vowel = elides(form, h_aspire)
        verbal = "j’" + form if subject == "je" and vowel else subject + " " + form
        result.append(verbal)
        if tense in ("S", "T", "SP", "SQP"):
            result.append(("qu’" if verbal.startswith(("il", "elle", "on")) else "que ") + verbal)
    return result


def query_base(query):
    return re.sub(r"^(?:(?:que |qu’)?(?:je |j’|tu |il |elle |on |nous |vous |ils |elles ))", "", query)


def generate(rows, verbs, data):
    by_lemma = defaultdict(list)
    for row in rows:
        by_lemma[row[2]].append(row)
    generated = []
    coverage = {code: Counter() for code in BLOCKS}
    for lemma, verb in verbs.items():
        original = by_lemma[lemma]
        cells = list(original)
        seen = {(r[1], *r[4:]) for r in cells}
        impersonal = "impersonal" in verb["usage_labels"]
        admitted = verb["ordinary_status"] == "supported"
        correction = verb["correction"] or {}
        model = correction.get("model")
        allowed_persons = correction.get("persons", ["3s"] if impersonal else list(SUBJECTS))
        syntax = [e for e in data["syntax"].get(lemma, []) if e["ordinary"]]
        defined = {tuple(s) for e in syntax for s in e["slots"]}

        def add(form, tense, person=None, number=None, gender=None, provenance="grammalecte"):
            identity = (form, tense, person, number, gender)
            if form and identity not in seen:
                seen.add(identity)
                cell = (original[0][0], form, lemma, provenance, tense, person, number, gender)
                cells.append(cell)
                generated.append(cell)

        if admitted:
            for form in correction.get("forms", []):
                add(form["form"], form["tense"], form.get("person"), form.get("number"), form.get("gender"),
                    "reference/" + form.get("use", "rare"))
            # These literal source-pattern forms were omitted by the old compiled lexicon.
            if lemma in {"déluger", "frimasser", "mouillasser", "pluvioter"}:
                add(lemma[:-2] + "ant", "G", provenance="lefff-syntax/morpho.fr/v-er/G")
            for tense in SIMPLE if lemma in data["dVerb"] or model else ():
                if tense == "K":
                    for gender in "mf":
                        for number in "sp":
                            add(source_form(data, lemma, tense, f":{gender}:{number}", model), tense,
                                number=number, gender=gender)
                elif tense == "G":
                    add(source_form(data, lemma, tense, ":P", model), tense)
                else:
                    for person_number in SUBJECTS:
                        if person_number in allowed_persons:
                            add(source_form(data, lemma, tense, ":" + person_number, model), tense,
                                person_number[0], person_number[1])
            for row in list(cells):
                if row[4] == "G":
                    add("en " + row[1], "GP", provenance="ordinary/v1/gerund")
            if verb["auxiliary_status"] == "supported":
                participles = [r for r in cells if r[4] == "K"]
                for auxiliary in verb["auxiliaries"]:
                    aux = auxiliary["lemma"]
                    for tense, base in COMPOUND.items():
                        persons = [None] if base in ("W", "G") else list(SUBJECTS)
                        for pn in persons:
                            if pn and pn not in allowed_persons:
                                continue
                            # A missing imperative is not licensed by an auxiliary imperative.
                            if base == "Y" and pn not in correction.get("compound_imperative_persons", []) and not any(r[4] == "Y" and f"{r[5]}{r[6]}" == pn for r in cells):
                                continue
                            prefix = source_form(data, aux, base, ":" + pn if pn else ":P")
                            if not prefix:
                                continue
                            for pp in participles:
                                _, form, _, _, _, _, number, gender = pp
                                if aux == "avoir" and (gender not in (None, "m") or number not in (None, "s")):
                                    continue
                                if aux == "être" and pn and number != pn[1] and pn != "2p":
                                    continue
                                # Only explicitly dimensioned agreement, never guessed from K.
                                if aux == "être" and (gender is None or number is None):
                                    continue
                                add(("en " if tense == "GPP" else "") + prefix + " " + form,
                                    tense, pn[0] if pn else None, pn[1] if pn else None,
                                    gender if aux == "être" else None,
                                    "ordinary/v1/" + aux + "/" + auxiliary["use"])
        statuses = {}
        for code in BLOCKS:
            if any(r[4] == code for r in cells):
                state = "supported"
            elif verb["ordinary_status"] == "not_applicable":
                state = "not_applicable"
            elif admitted and (impersonal and code in ("Y", "YP")
                               or code in SIMPLE and not any(s[0] == code for s in defined)
                               or code == "GP" and not any(s[0] == "G" for s in defined)
                               or code in COMPOUND and not any(s[0] == "K" for s in defined)
                               or code == "YP" and not any(s[0] == "Y" for s in defined)):
                state = "not_applicable"
            elif not admitted or code in COMPOUND and verb["auxiliary_status"] != "supported":
                state = "unconfirmed"
            else:
                state = "missing"
            statuses[code] = state
        if admitted:
            actual = {(r[4], r[5], r[6]) for r in cells}
            for code, person, number in defined:
                if code in ("P", "I", "J", "F", "C", "S", "T", "Y") and person and number:
                    if not impersonal or (person, number) == ("3", "s") and code != "Y":
                        if (code, person, number) not in actual:
                            statuses[code] = "missing"
        if admitted and impersonal:
            for code in ("P", "PC", "I", "PQP", "J", "PA", "F", "FA", "C", "CP", "S", "SP", "T", "SQP", "Y", "YP"):
                for pn in SUBJECTS:
                    if pn != "3s":
                        statuses[f"{code}/{pn}"] = "not_applicable"
            for code in ("Y", "YP"):
                statuses[code] = "not_applicable"
        if admitted and "persons" in correction:
            for code in ("P", "PC", "I", "PQP", "J", "PA", "F", "FA", "C", "CP", "S", "SP", "T", "SQP"):
                for pn in SUBJECTS:
                    if pn not in allowed_persons:
                        statuses[f"{code}/{pn}"] = "not_applicable"
        statuses.update(dict.fromkeys(correction.get("unavailable_cells", []), "not_applicable"))
        verb["blocks"] = statuses
        for code in BLOCKS:
            coverage[code][statuses[code]] += 1
    all_rows = rows + generated
    return [(*r, json.dumps(expressions(r[1], r[4], r[5], r[6], r[7],
                                      "impersonal" in verbs[r[2]]["usage_labels"], r[2] in data["h_aspire"])
                           if verbs[r[2]]["ordinary_status"] == "supported" else [r[1]], ensure_ascii=False))
            for r in all_rows], {code: dict(counts) for code, counts in coverage.items()}
