"""Check independently accepted #135 evidence; --admitted checks canonical content."""
import csv
import json
from pathlib import Path
import sys
import unittest
from urllib.parse import parse_qs, urlparse

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-135"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
MEMBERSHIP = [
    ("ruiner", "VER"), ("répandre", "VER"), ("normalement", "ADV"),
    ("ongle", "NOM"), ("électrique", "ADJ"), ("brancher", "VER"),
    ("liaison", "NOM"), ("marchand", "NOM"), ("section", "NOM"),
    ("transport", "NOM"), ("entraînement", "NOM"), ("douer", "VER"),
    ("réfléchi", "ADJ"), ("fumée", "NOM"), ("plomb", "NOM"),
    ("dégoûtant", "ADJ"), ("prime", "NOM"), ("tapis", "NOM"),
    ("version", "NOM"), ("ho", "ONO"), ("cieux", "NOM"),
    ("positif", "ADJ"), ("abuser", "VER"), ("cabine", "NOM"),
    ("gai", "ADJ"), ("hauteur", "NOM"), ("blé", "NOM"),
    ("dégueulasse", "ADJ"), ("oignon", "NOM"), ("écrit", "NOM"),
    ("laid", "ADJ"), ("couche", "NOM"), ("auquel", "PRO:int"),
    ("auquel", "PRO:rel"), ("reparler", "VER"), ("collier", "NOM"),
    ("équipage", "NOM"), ("disparu", "NOM"), ("illégal", "ADJ"),
    ("soupçonner", "VER"), ("vain", "ADJ"), ("bibliothèque", "NOM"),
    ("conférence", "NOM"), ("persuader", "VER"), ("reprocher", "VER"),
    ("étonnant", "ADJ"), ("incendie", "NOM"), ("religieux", "ADJ"),
    ("âgé", "ADJ"), ("semer", "VER"),
]
ADMITTED = "--admitted" in sys.argv
if ADMITTED:
    sys.argv.remove("--admitted")


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


def retained_path(name):
    parts = name.replace("\\", "/").split("/")
    archive = Path(read("admission.json")["author_evidence_root"])
    if archive.is_dir():
        return archive.joinpath(*parts[parts.index("issue-135") + 1:])
    path = Path(name)
    if path.is_file():
        return path
    local = ROOT / path
    if local.is_file():
        return local
    if "issue-135" in parts:
        return archive.joinpath(*parts[parts.index("issue-135") + 1:])
    return archive.joinpath(*parts[2:])


class Batch135Tests(unittest.TestCase):
    def test_canonical_use_status(self):
        usage = read("field-review.json")["usage"]
        self.assertEqual([record["frozen_row"] for record in usage], list(range(2701, 2751)))
        for record in usage:
            with self.subTest(row=record["frozen_row"]):
                self.assertEqual(record["accepted_text_quality_status"], "passed")
                self.assertEqual(record["rights_eligibility_status"], "passed_at_selected_text_boundary")
                self.assertTrue(record["reviewer"])
                self.assertTrue(record["review_reason"])
                self.assertEqual(record["accepted_usage_quality_status"], "passed")
                self.assertEqual(record["everyday_sense_status"], "passed")
                self.assertEqual(record["everyday_sense_reason"], record["review_reason"])

    def test_exact_ticket_resource_groups(self):
        path = (ROOT / "resources/core-pilot-79/curated-inputs.json"
                if ADMITTED else EVIDENCE / "groups.json")
        if ADMITTED:
            from core_pilot import load_inputs
            data = load_inputs(path.parent)[0]
        else:
            data = json.loads(path.read_text(encoding="utf-8"))
        groups = [g for g in (data["groups"] if ADMITTED else data)
                  if 2701 <= g["frozen_row"] <= 2750]
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        self.assertEqual([g["frozen_row"] for g in groups], list(range(2701, 2751)))

    def test_accepted_fields_preserve_frozen_frequencies_and_parent_gates(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
                encoding="utf-8-sig", newline="") as handle:
            scope = list(csv.DictReader(handle))
        self.assertEqual(len(scope), 4087)
        for group in read("groups.json"):
            with self.subTest(row=group["frozen_row"]):
                frozen = scope[group["frozen_row"] - 1]
                self.assertEqual((group["headword"], group["lexique_cgram"]),
                                 (frozen["word"], frozen["lexique_cgram"]))
                for name in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                    self.assertEqual(group[name], frozen[name])
                self.assertEqual(len(group["senses"]), 1)
                sense = group["senses"][0]
                self.assertTrue(sense["use"].strip())
                self.assertTrue(sense["excluded_uses"].strip())
                self.assertIsNone(sense["cefr"])
                self.assertEqual(sense["independent_review"], "passed")
                self.assertEqual(set(sense["fields"]), set(LANGUAGES))
                for name, field in sense["fields"].items():
                    self.assertEqual(field["language"], LANGUAGES[name])
                    self.assertTrue(field["text"].strip())
                    self.assertTrue(field["modification"].strip())
                    self.assertEqual(field["independent_review"], "passed")
        reuse = read("reuse.json")
        self.assertEqual((reuse["reused_groups"], reuse["reused_fields"], reuse["groups"]), (0, 0, []))
        admission = read("admission.json")
        self.assertEqual(admission["author_evidence_root"],
                         "C:/Users/inthe/IdeaProjects/lexique-spec-130-139/context/author-evidence/issue-135")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual((admission["independently_accepted_new_groups"],
                          admission["independently_accepted_new_fields"]), (50, 200))
        self.assertTrue(admission["independent_reviewer"])
        self.assertEqual(admission["pending_independent_review_rows"], [])
        self.assertEqual(admission["pending_independent_review_fields"], 0)
        self.assertEqual(admission["pending_independent_rights_fields"], 0)
        self.assertEqual(admission["newly_admitted_fields"], 0)
        self.assertEqual(admission["missing_admitted_fields"], 200)
        self.assertEqual(admission["missing_authored_fields"], 0)
        self.assertEqual(admission["native_checks"], "integration_owned_not_run")

    def test_captured_revisions_and_complete_attribution_chains(self):
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        contexts = {s["id"]: s for s in read("source-context.json")["current_content"]}
        captures = read("source-captures.json")
        self.assertEqual(len(captures["sources"]), 51)
        for capture in captures["sources"]:
            response = json.loads(retained_path(capture["api_path"]).read_text(encoding="utf-8"))
            page = next(p for p in response["query"]["pages"] if p["pageid"] == capture["pageid"])
            revision = next(r for r in page["revisions"] if r["revid"] == capture["revid"])
            source = sources[capture["source_id"]]
            self.assertEqual((source["title"], source["revid"], source["timestamp"]),
                             (page["title"], revision["revid"], revision["timestamp"]))
            self.assertEqual(retained_path(capture["raw_path"]).read_bytes().decode("utf-8"),
                             revision["slots"]["main"]["content"])
            self.assertIn("oldid=" + str(revision["revid"]), source["fixed_url"])
            self.assertTrue(source["attribution"])
            self.assertTrue(source["history_url"])
            self.assertEqual(source["selected_text_review"], "passed_at_selected_text_boundary")
            self.assertEqual(contexts[source["id"]]["selected_text_rights_status"],
                             "passed_at_selected_text_boundary")
            self.assertEqual(source["independent_reviewer"], read("admission.json")["independent_reviewer"])
        names, anonymous = {}, {}
        paths = [retained_path(p) for p in captures["contributor_registry"]["parts"]]
        for prefix in ("contributors-", "contributors-support-"):
            chain = sorted((p for p in paths if p.name.removeprefix(prefix).split(".")[0].isdigit()),
                           key=lambda p: int(p.name.removeprefix(prefix).split(".")[0]))
            self.assertTrue(chain)
            continuation = {}
            for path in chain:
                query = parse_qs(urlparse(path.with_suffix(".request.txt").read_text(encoding="utf-8")).query)
                for key, value in continuation.items():
                    self.assertEqual(query[key], [str(value)])
                response = json.loads(path.read_text(encoding="utf-8"))
                self.assertNotIn("error", response)
                for page in response["query"]["pages"]:
                    names.setdefault(page["title"], set()).update(p["name"] for p in page.get("contributors", []))
                    if "anoncontributors" in page:
                        anonymous[page["title"]] = page["anoncontributors"]
                continuation = response.get("continue", {})
            self.assertFalse(continuation)
        self.assertEqual(set(names), {s["title"] for s in sources.values()})
        for source in sources.values():
            self.assertEqual(contexts[source["id"]]["current_contributors"], sorted(names[source["title"]]))
            self.assertEqual(contexts[source["id"]]["anonymous_contributors_count"], anonymous.get(source["title"], 0))
        self.assertFalse(captures["contributor_registry"]["selected_sentence_authorship_proved"])

    def test_fields_bind_exact_french_pos_ipa_rights_and_original_proposals(self):
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        raw = {c["source_id"]: retained_path(c["raw_path"]).read_text(encoding="utf-8").splitlines()
               for c in read("source-captures.json")["sources"]}
        rights = read("rights.json")
        proposals = {(r["sense_id"], r["field"]): r for r in rights["fields"]}
        reviews = {(r["sense_id"], r["field"]): r for r in read("field-review.json")["fields"]}
        excerpts = {(r["sense_id"], r["field"]): r for r in read("source-excerpts.json")["fields"]}
        self.assertEqual(len(proposals), 200)
        self.assertEqual(set(proposals), set(reviews))
        self.assertEqual(set(proposals), set(excerpts))
        self.assertTrue(read("field-review.json")["independent_acceptance"])
        authored = {(s["sense_id"], name): field for g in read("authored/commit-5a2275d/groups.json")
                    for s in g["senses"] for name, field in s["fields"].items()}
        pos_names = {"VER": "verbe", "NOM": "nom", "ADJ": "adjectif", "ADV": "adverbe",
                     "ONO": "interjection", "PRO:int": "pronom interrogatif", "PRO:rel": "pronom relatif"}
        for group in read("groups.json"):
            sense = group["senses"][0]
            for name, field in sense["fields"].items():
                with self.subTest(row=group["frozen_row"], field=name):
                    lines = raw[field["source_id"]]
                    span = field["source_span"]
                    self.assertEqual(span["raw_source_excerpt"], lines[span["line_1_based"] - 1])
                    before = lines[:span["line_1_based"]]
                    language = next(line for line in reversed(before) if line.startswith("== "))
                    self.assertEqual(language.replace(" ", ""), "=={{langue|fr}}==")
                    pos = next(line for line in reversed(before) if line.startswith("=== {{S|"))
                    self.assertIn("{{S|" + pos_names[group["lexique_cgram"]] + "|fr", pos)
                    if name == "ipa":
                        self.assertIn(field["text"].strip("/"), span["raw_source_excerpt"])
                    else:
                        self.assertTrue(span["raw_source_excerpt"].startswith("# "))
                    key = (sense["sense_id"], name)
                    proposal, review, excerpt = proposals[key], reviews[key], excerpts[key]
                    self.assertEqual(proposal["exact_source_span"], span)
                    self.assertEqual(proposal["source_revision"], sources[field["source_id"]]["revid"])
                    self.assertEqual(review["proposed_text"], authored[key]["text"])
                    self.assertEqual(review["accepted_text"], field["text"])
                    self.assertEqual(excerpt["reviewed_excerpt"], span["raw_source_excerpt"])
                    self.assertEqual(review["accepted_text_quality_status"], "passed")
                    self.assertEqual(review["rights_eligibility_status"], "passed_at_selected_text_boundary")
                    self.assertEqual(review["reviewer"], read("admission.json")["independent_reviewer"])
                    self.assertTrue(review["independence"])
                    self.assertTrue(review["review_reason"])
                    self.assertTrue(review["selected_text_rights_reason"])
                    self.assertEqual(proposal["independent_review"], "passed_at_selected_text_boundary")
                    self.assertEqual(proposal["reviewer"], review["reviewer"])
                    self.assertEqual(proposal["accepted_text"], field["text"])
                    self.assertTrue(proposal["author_rights_basis"])
                    self.assertTrue(proposal["attribution"])
                    self.assertEqual(field["content_license"], "CC-BY-SA-4.0")
        for support in read("source-excerpts.json")["supporting_evidence"]:
            line = raw[support["source_id"]][support["line_1_based"] - 1]
            column = support["column_1_based"] - 1
            self.assertEqual(line[column:column + len(support["reviewed_excerpt"])], support["reviewed_excerpt"])
        self.assertEqual(rights["independent_rights_fields_accepted"], 200)
        for record in rights["sources"]:
            self.assertEqual(record["independent_review"], "passed_at_selected_text_boundary")
            self.assertEqual(record["original_historical_dictionary_layers"],
                             sources[record["source_id"]]["original_historical_dictionary_layers"])
            for layer in record["original_historical_dictionary_layers"]:
                self.assertEqual(layer["independent_review"], "passed_original_collective_work_boundary")
                self.assertIn("2006-01-01", layer["conservative_expiry"])
        self.assertIn("2006-01-01", rights["original_daf8_basis_scope"])
        self.assertEqual((EVIDENCE / "COPYING").read_bytes().rstrip(),
                         (ROOT / "resources/core-pilot-79/COPYING").read_bytes().rstrip())

    def test_scoped_special_forms_and_excluded_external_import(self):
        groups = {g["frozen_row"]: g for g in read("groups.json")}
        for row in (2733, 2734):
            for name in ("french_definition", "english", "chinese"):
                text = groups[row]["senses"][0]["fields"][name]["text"]
                self.assertIn({"french_definition": "masculine singulière", "english": "singular", "chinese": "阳性单数"}[name], text)
        self.assertNotEqual(groups[2733]["senses"][0]["sense_id"], groups[2734]["senses"][0]["sense_id"])
        for name in ("french_definition", "english", "chinese"):
            self.assertIn("être doué de", groups[2712]["senses"][0]["fields"][name]["text"])
            self.assertIn("sous d’autres cieux", groups[2721]["senses"][0]["fields"][name]["text"])
        self.assertEqual(groups[2712]["headword"], "douer")
        self.assertEqual(groups[2712]["senses"][0]["fields"]["ipa"]["text"], "/dwe/")
        self.assertEqual(groups[2721]["headword"], "cieux")
        self.assertIn("pluriel", groups[2721]["senses"][0]["fields"]["french_definition"]["text"])
        self.assertIn("vulgar", groups[2728]["senses"][0]["fields"]["english"]["text"])
        contexts = read("source-context.json")["current_content"]
        couche = next(c for c in contexts if c["title"] == "couche")
        marker = next(m for m in couche["external_reference_markers"] if "Import:CFC" in m["raw_reference"])
        self.assertIn("unselected", marker["selected_boundary"])
        self.assertTrue(all(location["line_1_based"] != 86 for location in couche["selected_locations"]))
        for authority in read("rights.json")["authority_captures"]:
            path = retained_path(authority["path"])
            self.assertTrue(path.is_file())
            if authority.get("pages"):
                response = json.loads(path.read_text(encoding="utf-8"))
                for record in authority["pages"]:
                    page = next(p for p in response["query"]["pages"] if p["pageid"] == record["pageid"])
                    revision = next(r for r in page["revisions"] if r["revid"] == record["revid"])
                    self.assertEqual(retained_path(record["raw_path"]).read_bytes().decode("utf-8"),
                                     revision["slots"]["main"]["content"])

    def test_exact_replacements_uses_and_modern_corroboration(self):
        groups = {g["frozen_row"]: g for g in read("groups.json")}
        originals = {g["frozen_row"]: g for g in read("authored/commit-5a2275d/groups.json")}
        replacements = {(r["frozen_row"], r["field"]): r for r in read("accepted-replacements.json")}
        changed = {(row, name) for row,g in groups.items()
                   for name,field in g["senses"][0]["fields"].items()
                   if field["text"] != originals[row]["senses"][0]["fields"][name]["text"]}
        self.assertEqual(changed, {(row, name) for row in (2712,2721)
                                  for name in ("french_definition","english","chinese")})
        self.assertEqual(set(replacements), changed)
        for (row,name), replacement in replacements.items():
            field = groups[row]["senses"][0]["fields"][name]
            self.assertEqual(replacement["proposed_text"], originals[row]["senses"][0]["fields"][name]["text"])
            self.assertEqual(replacement["accepted_text"], field["text"])
            self.assertEqual(replacement["accepted_source_span"], field["source_span"])
            self.assertTrue(replacement["reason"])
        usage = {r["sense_id"]: r for r in read("field-review.json")["usage"]}
        self.assertEqual(len(usage), 50)
        for row,g in groups.items():
            s = g["senses"][0]
            record = usage[s["sense_id"]]
            self.assertEqual(record["proposed_usage"], originals[row]["senses"][0]["use"])
            self.assertEqual(record["accepted_usage"], s["use"])
            self.assertEqual(record["accepted_text_quality_status"], "passed")
            self.assertEqual(record["excluded_uses"], s["excluded_uses"])
            self.assertTrue(record["review_reason"])
        spelling = read("field-review.json")["spelling_relations"]
        self.assertEqual(len(spelling), 50)
        for record in spelling:
            self.assertEqual(record["independent_review"], "passed")
            self.assertEqual(record["frozen_headword"], groups[record["frozen_row"]]["headword"])
            self.assertEqual(record["alias_admission"], "not_admitted")
        modern = read("modern-use-evidence.json")
        self.assertEqual({r["frozen_row"] for r in modern["uses"]}, {2712,2721})
        self.assertFalse(modern["external_prose_selected_for_redistribution"])
        for record in modern["uses"]:
            self.assertEqual(record["quality_status"], "passed_for_scoped_modern_construction")
            self.assertFalse(record["exception_requested_or_assumed"])
            self.assertTrue(record["reason"])
            for capture in record["captures"]:
                self.assertEqual(capture["status"], 200)
                self.assertEqual(Path(capture["archived_raw_path"]).stat().st_size, capture["bytes"])


if __name__ == "__main__":
    unittest.main()
