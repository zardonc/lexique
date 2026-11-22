"""Check #132 at the approved resource/admission boundary."""
import csv
import json
from pathlib import Path
from urllib.parse import parse_qs, urlparse
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-132"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
MEMBERSHIP = [
    ("honorer", "VER"), ("poète", "NOM"), ("royal", "ADJ"), ("bienvenue", "NOM"),
    ("pilule", "NOM"), ("repousser", "VER"), ("facture", "NOM"), ("classique", "ADJ"),
    ("ceinture", "NOM"), ("oublie", "NOM"), ("kilo", "NOM"), ("volontaire", "ADJ"),
    ("gris", "ADJ"), ("soucier", "VER"), ("proposition", "NOM"), ("exposer", "VER"),
    ("grouiller", "VER"), ("intervenir", "VER"), ("moderne", "ADJ"), ("instruction", "NOM"),
    ("jus", "NOM"), ("améliorer", "VER"), ("compliquer", "VER"), ("compagnon", "NOM"),
    ("cercueil", "NOM"), ("indice", "NOM"), ("bourrer", "VER"), ("comédie", "NOM"),
    ("rock", "NOM"), ("rumeur", "NOM"), ("véhicule", "NOM"), ("cassé", "ADJ"),
    ("transporter", "VER"), ("jouir", "VER"), ("saurer", "VER"), ("phrase", "NOM"),
    ("navire", "NOM"), ("cabinet", "NOM"), ("final", "ADJ"), ("tenue", "NOM"),
    ("fonction", "NOM"), ("quinze", "ADJ:num"), ("étendre", "VER"), ("tasse", "NOM"),
    ("nazi", "NOM"), ("roue", "NOM"), ("plante", "NOM"), ("uniquement", "ADV"),
    ("sel", "NOM"), ("douze", "ADJ:num"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


def retained_path(name):
    path = ROOT / name
    if path.exists():
        return path
    return Path(read("admission.json")["author_evidence_root"]).joinpath(*Path(name).parts[2:])


class Batch132Tests(unittest.TestCase):
    def test_exact_frozen_groups_have_a_resource_handoff(self):
        groups = read("groups.json")
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        self.assertEqual([g["frozen_row"] for g in groups], list(range(2551, 2601)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
                encoding="utf-8-sig", newline="") as handle:
            frozen = list(csv.DictReader(handle))
        for group in groups:
            row = frozen[group["frozen_row"] - 1]
            self.assertEqual((group["headword"], group["lexique_cgram"]),
                             (row["word"], row["lexique_cgram"]))
            for frequency in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                self.assertEqual(group[frequency], row[frequency])
        self.assertEqual(read("admission.json")["scope_denominator_groups"], 4087)

    def test_four_accepted_fields_match_independent_review_sources_and_rights(self):
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        rights = {(f["sense_id"], f["field"]): f for f in read("rights.json")["fields"]}
        reviews = {(f["sense_id"], f["field"]): f for f in read("field-review.json")["fields"]}
        excerpts = {(f["sense_id"], f["field"]): f for f in read("source-excerpts.json")["fields"]}
        contexts = {s["id"]: s for s in read("source-context.json")["current_content"]}
        self.assertEqual(len(rights), 196)
        self.assertEqual(set(rights), set(reviews))
        self.assertEqual(set(rights), set(excerpts))
        originals = {(s["sense_id"], name): field for g in read("authored/groups.json")
                     for s in g["senses"] for name, field in s["fields"].items()}
        replacements = {(r["sense_id"], r["field"]): r for r in read("accepted-replacements.json")}
        self.assertEqual(read("review-candidates.json"), read("authored/review-candidates.json"))
        self.assertEqual(read("authored/groups.json"), read("review-candidates.json"))
        reviewer = read("field-review.json")["reviewer"]
        self.assertTrue(reviewer)
        for group in read("groups.json"):
            if group["frozen_row"] == 2560:
                self.assertEqual(group["senses"], [])
                continue
            self.assertEqual(len(group["senses"]), 1)
            sense = group["senses"][0]
            self.assertTrue(sense["use"].strip())
            self.assertIsNone(sense["cefr"])
            self.assertEqual(sense["source_headword"], group["headword"])
            self.assertEqual(sense["independent_review"], "passed")
            self.assertEqual(set(sense["fields"]), set(LANGUAGES))
            for name, field in sense["fields"].items():
                with self.subTest(row=group["frozen_row"], field=name):
                    key = sense["sense_id"], name
                    source = sources[field["source_id"]]
                    self.assertEqual(field["language"], LANGUAGES[name])
                    self.assertTrue(field["text"].strip())
                    self.assertTrue(field["modification"].strip())
                    self.assertEqual(field["content_license"], "CC-BY-SA-4.0")
                    self.assertEqual(field["independent_review"], "passed")
                    self.assertEqual(field["rights_eligibility_status"], "passed_at_selected_text_boundary")
                    self.assertEqual(field["text"], reviews[key]["accepted_text"])
                    self.assertEqual(reviews[key]["proposed_text"], originals[key]["text"])
                    self.assertEqual(reviews[key]["reviewer"], reviewer)
                    self.assertIs(reviews[key]["independence"], True)
                    self.assertEqual(reviews[key]["accepted_text_quality_status"], "passed")
                    self.assertEqual(reviews[key]["rights_eligibility_status"], "passed_at_selected_text_boundary")
                    self.assertTrue(reviews[key]["review_reason"].strip())
                    self.assertTrue(reviews[key]["selected_text_rights_reason"].strip())
                    if key in replacements:
                        self.assertEqual(replacements[key]["proposed_text"], originals[key]["text"])
                        self.assertEqual(replacements[key]["accepted_text"], field["text"])
                        self.assertEqual(replacements[key]["exact_source"], reviews[key]["exact_source"])
                    else:
                        self.assertEqual(field["text"], originals[key]["text"])
                    self.assertEqual(excerpts[key]["revid"], source["revid"])
                    self.assertEqual(excerpts[key]["reviewed_excerpt"], field["source_span"]["raw_source_excerpt"])
                    self.assertEqual(rights[key]["exact_source_span"], field["source_span"])
                    self.assertEqual(rights[key]["independent_review"], "passed_at_selected_text_boundary")
                    self.assertEqual(rights[key]["accepted_text"], field["text"])
                    self.assertEqual(rights[key]["reviewer"], reviewer)
                    self.assertEqual(rights[key]["source_revision"], source["revid"])
                    self.assertEqual(contexts[source["id"]]["selected_text_rights_status"], "passed_at_selected_text_boundary")
                    self.assertEqual(contexts[source["id"]]["reviewer"], reviewer)
                    for layer in rights[key]["original_historical_dictionary_layers"]:
                        expiry = "2006-01-01" if layer["original_authors"] == "Académie française" else "1952-01-01"
                        self.assertTrue(layer["conservative_expiry"].startswith(expiry))
                        self.assertEqual(layer["independent_review"], "passed_at_original_work_boundary")
                    self.assertEqual(excerpts[key]["accepted_text"], field["text"])
                    self.assertEqual(excerpts[key]["independent_review"], "passed")
                    self.assertEqual(sources[field["source_id"]]["selected_text_review"], "passed_at_selected_text_boundary")
                    for required in ("author_rights_basis", "selected_boundary", "attribution", "modification"):
                        self.assertTrue(rights[key][required])
                    self.assertIn("oldid=" + str(source["revid"]), source["fixed_url"])
                    self.assertTrue(source["history_url"])
        review = read("field-review.json")
        self.assertTrue(review["independent_acceptance"])
        self.assertEqual(review["schema"], read("authored/field-review.json")["schema"])
        self.assertEqual(len(review["usage"]), 49)
        self.assertEqual(len(review["spelling_relations"]), 50)
        self.assertTrue(all(r["alias_admission"] == "not_admitted" for r in review["spelling_relations"]))
        for usage in review["usage"]:
            sense = next(s for g in read("groups.json") for s in g["senses"]
                         if s["sense_id"] == usage["sense_id"])
            self.assertEqual(usage["accepted_usage"], sense["use"])
            self.assertEqual(usage["accepted_usage_quality_status"], "passed")
            self.assertEqual(usage["everyday_sense_status"], "passed")
            self.assertEqual(usage["reviewer"], reviewer)
            self.assertTrue(usage["review_reason"].strip())
        admission = read("admission.json")
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (49, 196))
        self.assertEqual((admission["independently_accepted_new_groups"], admission["independently_accepted_new_fields"]), (49, 196))
        self.assertEqual((admission["unchanged_accepted_fields"], admission["replacement_fields"]), (192, 4))
        self.assertEqual((admission["missing_fields"], admission["canonical_pending_fields"]), (4, 196))
        self.assertEqual(admission["newly_admitted_fields"], 0)
        self.assertEqual(admission["missing_admitted_fields"], 200)
        self.assertEqual(admission["pending_independent_review_fields"], 0)
        self.assertEqual(admission["pending_independent_rights_fields"], 0)
        self.assertEqual(read("rights.json")["independent_rights_fields_accepted"], 196)
        self.assertEqual(admission["native_checks"], "integration_owned_not_run")
        self.assertEqual((read("reuse.json")["reused_groups"], read("reuse.json")["reused_fields"]), (0, 0))
        self.assertEqual((EVIDENCE / "COPYING").read_text(encoding="utf-8").rstrip(),
                         (ROOT / "resources/core-pilot-79/COPYING").read_text(encoding="utf-8").rstrip())

    def test_only_approved_oublie_annotation_has_zero_fields_and_keeps_rejected_proposal(self):
        admission = read("admission.json")
        self.assertEqual(admission["author_unresolved_content_rows"], [])
        self.assertEqual(admission["annotation_exception_groups"], 1)
        self.assertEqual(admission["specialist_exception_groups"], 0)
        self.assertEqual(admission["unresolved_field_blockers"], [])
        self.assertEqual(admission["missing_annotation_fields"], 4)
        self.assertEqual(admission["pending_independent_review_rows"], [])
        annotation = read("groups.json")[9]
        self.assertEqual((annotation["headword"], annotation["lexique_cgram"]), ("oublie", "NOM"))
        self.assertEqual(annotation["status"], "approved_annotation_exception")
        self.assertEqual(annotation["senses"], [])
        self.assertEqual((annotation["admitted_fields"], annotation["missing_fields"]), (0, 4))
        self.assertEqual(annotation["owner_approval"]["exact_decision"], "批准仅 oublie/NOM 为零字段注记")
        self.assertEqual((annotation["owner_approval"]["approved_by"], annotation["owner_approval"]["date"],
                          annotation["owner_approval"]["ticket"]), ("project_owner", "2026-11-21", 132))
        self.assertIn(annotation["owner_approval"]["exact_decision"], (EVIDENCE / "owner-approval.md").read_text(encoding="utf-8"))
        rejected = read("rejected-old-use.json")
        self.assertEqual(rejected["status"], "rejected_old_use_proposal_not_admitted")
        sense = rejected["group"]["senses"][0]
        self.assertEqual(len(sense["fields"]), 4)
        self.assertEqual(sense["independent_review"], "pending")
        for name, restriction in (("french_definition", "vieilli"), ("english", "old-fashioned"), ("chinese", "旧时")):
            self.assertIn(restriction, sense["fields"][name]["text"])
        for index, restrictions in ((13, ("Se soucier de", "de", "se soucier de")),
                                    (33, ("Jouir de", "de", "jouir de")),
                                    (34, ("préalablement salés", "salted beforehand", "预先盐腌")),
                                    (43, ("anse", "handle", "把手"))):
            fields = read("groups.json")[index]["senses"][0]["fields"]
            for name, text in zip(("french_definition", "english", "chinese"), restrictions, strict=True):
                self.assertIn(text, fields[name]["text"])

    def test_retained_first_party_sources_and_complete_contributor_pagination(self):
        captures = read("source-captures.json")
        self.assertTrue(all(retained_path(c["api_path"]).exists() for c in captures["sources"]),
                        "Retained #132 source archive is required for independent verification")
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        contexts = {s["id"]: s for s in read("source-context.json")["current_content"]}
        raw = {}
        self.assertEqual(len(captures["sources"]), 50)
        for capture in captures["sources"]:
            data = json.loads(retained_path(capture["api_path"]).read_text(encoding="utf-8-sig"))
            page = next(p for p in data["query"]["pages"] if p["pageid"] == capture["pageid"])
            revision = next(r for r in page["revisions"] if r["revid"] == capture["revid"])
            source = sources[capture["source_id"]]
            self.assertEqual((source["title"], source["timestamp"]), (page["title"], revision["timestamp"]))
            body = retained_path(capture["raw_path"]).read_bytes()
            self.assertEqual(body, revision["slots"]["main"]["content"].encode("utf-8"))
            raw[source["id"]] = body.decode("utf-8").splitlines()
        names, anonymous, continuation = {}, {}, {}
        for name in captures["contributor_registry"]["parts"]:
            path = retained_path(name)
            query = parse_qs(urlparse(path.with_suffix(".request.txt").read_text(encoding="utf-8")).query)
            for key, value in continuation.items():
                self.assertEqual(query[key], [str(value)])
            data = json.loads(path.read_text(encoding="utf-8"))
            self.assertNotIn("error", data)
            for page in data["query"]["pages"]:
                names.setdefault(page["title"], set()).update(p["name"] for p in page.get("contributors", []))
                if "anoncontributors" in page:
                    anonymous[page["title"]] = page["anoncontributors"]
            continuation = data.get("continue", {})
        self.assertFalse(continuation)
        self.assertEqual(set(names), {s["title"] for s in sources.values()})
        for source in sources.values():
            self.assertEqual(contexts[source["id"]]["current_contributors"], sorted(names[source["title"]]))
            self.assertEqual(contexts[source["id"]]["anonymous_contributors_count"], anonymous.get(source["title"], 0))
        expected_pos = {"VER": "verbe", "NOM": "nom", "ADJ": "adjectif", "ADV": "adverbe", "ADJ:num": "adjectif numéral"}
        for group in read("groups.json"):
            if group["frozen_row"] == 2560:
                self.assertEqual(group["senses"], [])
                continue
            for name, field in group["senses"][0]["fields"].items():
                lines = raw[field["source_id"]]
                span = field["source_span"]
                self.assertEqual(span["raw_source_excerpt"], lines[span["line_1_based"] - 1])
                before = lines[:span["line_1_based"]]
                self.assertEqual(next(l for l in reversed(before) if l.startswith("== ")), "== {{langue|fr}} ==")
                pos_line = next(l for l in reversed(before) if l.startswith("=== {{S|"))
                self.assertIn("{{S|" + expected_pos[group["lexique_cgram"]] + "|fr", pos_line)
                if name == "ipa":
                    self.assertIn(field["text"].strip("/"), span["raw_source_excerpt"])
        for binding in read("source-excerpts.json")["supporting_evidence"]:
            self.assertEqual(binding["reviewed_excerpt"], raw[binding["source_id"]][binding["line_1_based"] - 1])

    def test_vocal_rock_scope_and_sentence_explanation_are_explicit(self):
        rock, phrase = read("groups.json")[28]["senses"][0], read("groups.json")[35]["senses"][0]
        for name, restriction in (("french_definition", "rock chanté"), ("english", "Vocal rock"),
                                  ("chinese", "有演唱的摇滚乐")):
            self.assertIn(restriction, rock["fields"][name]["text"])
        self.assertIn("vocal rock", rock["use"])
        self.assertIn("一个或多个分句", phrase["fields"]["chinese"]["text"])
        self.assertEqual({(r["frozen_row"], r["field"]) for r in read("accepted-replacements.json")},
                         {(2579, "french_definition"), (2579, "english"), (2579, "chinese"), (2586, "chinese")})


if __name__ == "__main__":
    unittest.main()
