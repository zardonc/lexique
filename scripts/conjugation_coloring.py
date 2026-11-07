"""Linguistic spans over complete forms, separate from compact storage models."""

import json
from pathlib import Path

from conjugation_generation import COMPOUND

RULES = json.loads(Path(__file__).with_name("conjugation_endings.json").read_text(encoding="utf-8"))


def verbal_word(form, start, end, verb, imperative):
    if "pronominal" in verb:
        prefixes = ("m’en ", "t’en ", "s’en ", "nous en ", "vous en ") if verb["pronominal"]["with_en"] else ("m’", "t’", "s’", "me ", "te ", "se ", "nous ", "vous ")
        for prefix in prefixes:
            if form[start:end].startswith(prefix):
                start += len(prefix)
                break
        if imperative:
            for suffix in ("-nous-en", "-vous-en", "-t’en", "-toi", "-nous", "-vous"):
                if form[start:end].endswith(suffix):
                    end -= len(suffix)
                    break
    if start >= end:
        raise ValueError(f"Empty conjugation verbal part: {form}")
    return [start, end]


def form_spans(cell, verb):
    form, tense = cell["form"], cell["tense"]
    start = 3 if tense in ("GP", "GPP") else 0
    if tense in COMPOUND:
        main = [form.rindex(" ") + 1, len(form)]
        auxiliary = verbal_word(form, start, main[0] - 1, verb, tense == "YP")
        verbal = [[auxiliary[0], main[1]]] if form[auxiliary[1]:main[0]] == " " else [auxiliary, main]
    else:
        main = verbal_word(form, start, len(form), verb if tense != "K" else {}, tense == "Y")
        verbal = [main]
    return verbal, main


def inflection(cell, verb, main):
    if verb["group_status"] != "supported" or verb["group"] not in (1, 2):
        return None
    tense = "K" if cell["tense"] in COMPOUND else "G" if cell["tense"] == "GP" else cell["tense"]
    base = verb.get("pronominal", {}).get("base", cell["lemma"])
    family = RULES["lexical"].get(base, {}).get(tense, RULES["groups"][str(verb["group"])].get(tense, RULES["common"].get(tense, {})))
    if cell["tense"] in COMPOUND or tense in ("W", "G"):
        slot = "_"
    elif tense == "K":
        slot = (cell["gender"] or "m") + (cell["number"] or "s")
    else:
        slot = (cell["person"] or "") + (cell["number"] or "")
    endings = family.get(slot, [])
    word = cell["form"][main[0]:main[1]]
    if source_only(cell, verb, main):
        return None
    ending = next((value for value in endings if word.endswith(value)), None)
    if endings and ending is None:
        raise ValueError(f"Unvalidated regular conjugation ending: {cell['lemma']} {cell['tense']} {cell['form']}")
    return [main[1] - len(ending), main[1]] if ending else None


def source_only(cell, verb, main):
    base = verb.get("pronominal", {}).get("base", cell["lemma"])
    return (cell["tense"] in ("P", "I", "J", "F", "C", "S", "T", "Y") and (cell["person"] is None or cell["number"] is None)
            or cell["form"][main[0]:main[1]] in RULES["source_only_participles"].get(base, []))


def coloring(cell, verb):
    form = cell["form"]
    verbal, main = form_spans(cell, verb)
    ending = inflection(cell, verb, main)
    result = {}
    for text in dict.fromkeys([form, *cell["expressions"]]):
        if not text.endswith(form):
            raise ValueError(f"Conjugation expression does not preserve its form: {text}")
        offset = len(text) - len(form)
        result[text] = {"verbal": [[start + offset, end + offset] for start, end in verbal],
                        "main": [position + offset for position in main],
                        "ending": [position + offset for position in ending] if ending else None}
    return result


def coloring_report(rows, verbs):
    groups = {str(group): {"identities": sum(verb["group"] == group for verb in verbs.values()), "cells": 0, "endings": 0} for group in (1, 2, 3)}
    unconfirmed = set()
    retained_source_only = set()
    for row in rows:
        cell = dict(zip(("source_line", "form", "lemma", "raw_tag", "tense", "person", "number", "gender"), row[:8]))
        verb = verbs[cell["lemma"]]
        _, main = form_spans(cell, verb)
        ending = inflection(cell, verb, main)
        if str(verb["group"]) in groups:
            counts = groups[str(verb["group"])]
            counts["cells"] += 1
            counts["endings"] += ending is not None
        if verb["group"] in (1, 2) and ending is None:
            (retained_source_only if source_only(cell, verb, main) else unconfirmed).add((cell["lemma"], cell["tense"], cell["form"]))
    return {"groups": groups, "regular_unconfirmed": sorted(unconfirmed), "retained_source_only": sorted(retained_source_only)}
