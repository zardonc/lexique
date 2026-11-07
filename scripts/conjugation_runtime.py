"""Decode compact simple-form models and construct admitted conjugation cells."""

import json
import struct
import unicodedata
import zlib

from conjugation_generation import BLOCKS, COMPOUND, SIMPLE, SUBJECTS, expressions, ordinary_constructions, query_base
from conjugation_pronominal import pronominal_cells


def normalize_query(query):
    return " ".join(unicodedata.normalize("NFC", query).lower().replace("'", "’").split())


def matches_query(form, expressions, query):
    return any(normalize_query(expression) == query for expression in (form, *expressions))


def load_models(connection):
    models = {}
    for model, payload in connection.execute("SELECT id, cells FROM models ORDER BY id"):
        try:
            cells = json.loads(payload) if isinstance(payload, str) else None
        except ValueError as error:
            raise ValueError(f"Invalid conjugation model: {model}") from error
        if (type(model) is not int or model < 0 or not isinstance(cells, list) or not cells
                or any(not isinstance(cell, list) or len(cell) != 7
                       or type(cell[0]) is not int or cell[0] < 0
                       or not isinstance(cell[1], str) or not isinstance(cell[2], str)
                       or cell[3] not in (*SIMPLE, "W", None)
                       or cell[4] not in (None, "1", "2", "3")
                       or cell[5] not in (None, "s", "p")
                       or cell[6] not in (None, "m", "f") for cell in cells)):
            raise ValueError(f"Invalid conjugation model: {model}")
        models[model] = cells
    if not models:
        raise ValueError("Conjugation resource has no simple-form models")
    return models


def candidate_lemmas(connection, query, models):
    probes = {query, query_base(query)}
    words = query.split()
    if words:
        probes.add(words[-1])
    for probe in tuple(probes):
        if probe.startswith(("m’", "t’", "s’")):
            probes.add(probe[2:])
        for ending in ("-toi", "-nous", "-vous", "-t’en", "-nous-en", "-vous-en"):
            if probe.endswith(ending):
                probes.add(probe[:-len(ending)])
    probes.discard("")
    compatible = set()
    model_cuts = {}
    for model, cells in models.items():
        suffixes = {(cell[0], normalize_query(cell[1])) for cell in cells}
        model_cuts[model] = {cut for cut, _ in suffixes}
        for cut, suffix in suffixes:
            for probe in probes:
                if probe.endswith(suffix):
                    compatible.add((probe[:len(probe) - len(suffix)], model, cut))
    stems = sorted({stem for stem, _, _ in compatible})
    bases = set()
    if stems:
        placeholders = ",".join("?" for _ in stems)
        for stem, model, cut, lemma in connection.execute(
                f"SELECT stem, model, cut, lemma FROM stems WHERE stem IN ({placeholders})", stems):
            if model not in models or cut not in model_cuts[model] or not isinstance(lemma, str) or not lemma:
                raise ValueError("Invalid conjugation stem reference")
            if (stem, model, cut) in compatible:
                bases.add(lemma)
    identities = {row[0] for row in connection.execute("SELECT lemma FROM verbs WHERE lemma = ?", (query,))}
    if bases:
        placeholders = ",".join("?" for _ in bases)
        identities.update(row[0] for row in connection.execute(
            f"SELECT lemma FROM verbs WHERE base IN ({placeholders})", sorted(bases)))
    return sorted(identities)


def _verb_record(connection, lemma):
    row = connection.execute(
        "SELECT base, model, source_lines, information, h_aspire, slots FROM verbs WHERE lemma = ?", (lemma,)).fetchone()
    if row is None:
        return None
    base, model, anchors, payload, h_aspire, slots = row
    try:
        information = json.loads(zlib.decompress(payload)) if isinstance(payload, bytes) else None
    except (ValueError, zlib.error) as error:
        raise ValueError(f"Invalid compressed conjugation verb metadata: {lemma}") from error
    if (not isinstance(base, str) or not base or not isinstance(anchors, bytes)
            or type(h_aspire) is not int or h_aspire not in (0, 1)
            or type(slots) is not int or slots < 1 or not isinstance(information, dict)
            or information.get("ordinary_status") not in ("supported", "not_applicable", "unconfirmed")
            or information.get("auxiliary_status") not in ("supported", "not_applicable", "unconfirmed")
            or not isinstance(information.get("usage_labels"), list)
            or any(not isinstance(label, str) for label in information["usage_labels"])
            or not isinstance(information.get("auxiliaries"), list)
            or any(not isinstance(auxiliary, dict) or auxiliary.get("lemma") not in ("avoir", "être")
                   or not isinstance(auxiliary.get("use"), str) for auxiliary in information["auxiliaries"])
            or not isinstance(information.get("blocks"), dict)
            or "correction" not in information
            or any(information["blocks"].get(code) not in ("supported", "not_applicable", "unconfirmed", "missing")
                   for code in BLOCKS)):
        raise ValueError(f"Invalid conjugation verb metadata: {lemma}")
    correction = information.get("correction")
    if (correction is not None and not isinstance(correction, dict)
            or isinstance(correction, dict) and any(
                not isinstance(correction[key], list) or any(not isinstance(pn, str) or pn not in SUBJECTS for pn in correction[key])
                for key in ("persons", "compound_imperative_persons") if key in correction)):
        raise ValueError(f"Invalid conjugation correction metadata: {lemma}")
    pronominal = information.get("pronominal")
    if model is None:
        if (base == lemma or anchors or not isinstance(pronominal, dict)
                or pronominal.get("base") != base
                or pronominal.get("agreement") not in ("invariable", "subject", "contextual")
                or not isinstance(pronominal.get("persons"), list) or not pronominal["persons"]
                or any(not isinstance(pn, str) or pn not in SUBJECTS for pn in pronominal["persons"])
                or type(pronominal.get("with_en")) is not bool or type(pronominal.get("reciprocal")) is not bool):
            raise ValueError(f"Invalid pronominal metadata: {lemma}")
    elif base != lemma or type(model) is not int or pronominal is not None:
        raise ValueError(f"Invalid ordinary conjugation model reference: {lemma}")
    return base, model, anchors, information, bool(h_aspire), slots


def _simple_cells(lemma, record, models):
    if record is None or record[0] != lemma or record[1] not in models:
        raise ValueError(f"Missing ordinary conjugation model: {lemma}")
    _, model, anchors, _, _, _ = record
    templates = models[model]
    if len(anchors) != 4 * len(templates):
        raise ValueError(f"Conjugation source-line anchors differ from model: {lemma}")
    rows = []
    for cell, (line,) in zip(templates, struct.iter_unpack("<I", anchors)):
        cut, suffix, tag, tense, person, number, gender = cell
        if cut > len(lemma) or line < 1:
            raise ValueError(f"Invalid conjugation model or source-line anchor: {lemma}")
        form = (lemma[:-cut] if cut else lemma) + suffix
        if not form:
            raise ValueError(f"Empty conjugation model form: {lemma}")
        rows.append((line, form, lemma, tag, tense, person, number, gender))
    return rows


def _validate_construction(construction):
    if not isinstance(construction, dict) or set(construction) != {"avoir", "être"}:
        raise ValueError("Invalid conjugation construction metadata")
    for auxiliary in construction.values():
        if not isinstance(auxiliary, dict) or set(auxiliary) != set(COMPOUND.values()):
            raise ValueError("Invalid conjugation auxiliary construction metadata")
        for tense, forms in auxiliary.items():
            persons = {""} if tense in ("W", "G") else set(SUBJECTS)
            if not isinstance(forms, dict) or set(forms) != persons or any(not isinstance(form, str) for form in forms.values()):
                raise ValueError("Invalid conjugation auxiliary forms")


def table_cells(connection, lemma, models, construction, cache):
    _validate_construction(construction)
    record = _verb_record(connection, lemma)
    if record is None:
        return []
    base, model, _, information, h_aspire, slots = record
    if base not in cache:
        base_record = record if model is not None else _verb_record(connection, base)
        cache[base] = _simple_cells(base, base_record, models)
    simple = cache[base]

    def auxiliary_form(auxiliary, tense, pn):
        return construction[auxiliary][tense][pn or ""]

    if model is None:
        cells = pronominal_cells(lemma, simple, information, h_aspire, auxiliary_form)
    else:
        ordinary = simple + ordinary_constructions(simple, information, auxiliary_form)
        impersonal = "impersonal" in information["usage_labels"]
        cells = [(*row, json.dumps(expressions(row[1], row[4], row[5], row[6], row[7], impersonal, h_aspire)
                                  if information["ordinary_status"] == "supported" else [row[1]], ensure_ascii=False))
                 for row in ordinary]
    if len(cells) != slots:
        raise ValueError(f"Conjugation logical cell count mismatch: {lemma}")
    return cells
