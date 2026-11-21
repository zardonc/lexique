"""Check #128 at the approved resource/admission boundary."""
import csv
import json
from pathlib import Path
from urllib.parse import parse_qs, urlparse
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-128"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
# Exact membership from the self-contained #128 ticket.
MEMBERSHIP = [
    ("semblant", "NOM"), ("net", "ADJ"), ("géant", "ADJ"), ("géant", "NOM"),
    ("pointer", "VER"), ("empereur", "NOM"), ("passeport", "NOM"), ("torturer", "VER"),
    ("info", "NOM"), ("jouet", "NOM"), ("central", "ADJ"), ("pressé", "ADJ"),
    ("inscrire", "VER"), ("lutter", "VER"), ("épuiser", "VER"), ("ascenseur", "NOM"),
    ("digne", "ADJ"), ("saloperie", "NOM"), ("garage", "NOM"), ("mandat", "NOM"),
    ("moto", "NOM"), ("vif", "ADJ"), ("figure", "NOM"), ("rapprocher", "VER"),
    ("sacrer", "VER"), ("chute", "NOM"), ("unir", "VER"), ("japonais", "ADJ"),
    ("japonais", "NOM"), ("creuser", "VER"), ("satisfaire", "VER"), ("transmettre", "VER"),
    ("attente", "NOM"), ("donne", "NOM"), ("exécuter", "VER"), ("gant", "NOM"),
    ("élevé", "ADJ"), ("accomplir", "VER"), ("décrocher", "VER"), ("fièvre", "NOM"),
    ("révolution", "NOM"), ("prononcer", "VER"), ("pêche", "NOM"), ("existence", "NOM"),
    ("impressionner", "VER"), ("océan", "NOM"), ("original", "ADJ"), ("original", "NOM"),
    ("vilain", "ADJ"), ("méthode", "NOM"),
]
# Independently read selected definition/IPA locations, in frozen row order.
SELECTED_LINES = [
    (8, 7), (40, 20), (10, 9), (99, 94), (41, 40), (10, 9), (9, 7), (7, 6), (11, 10), (9, 8),
    (10, 9), (22, 10), (7, 6), (24, 10), (27, 6), (12, 11), (10, 9), (8, 7), (10, 9), (21, 9),
    (11, 10), (24, 308), (22, 14), (9, 6), (26, 6), (12, 11), (7, 6), (10, 9), (128, 127), (8, 7),
    (26, 6), (10, 6), (11, 10), (10, 9), (7, 6), (14, 13), (10, 9), (9, 6), (7, 6), (10, 9),
    (42, 13), (7, 6), (13, 12), (8, 7), (17, 6), (11, 10), (11, 10), (112, 111), (10, 9), (10, 9),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch128Tests(unittest.TestCase):
    def test_all_reviewed_fields_and_selected_rights_pass_before_canonical_assembly(self):
        review = read("field-review.json")
        self.assertEqual(review["kind"], "separate_independent_field_review")
        self.assertIs(review["independence"], True)
        self.assertIs(review["independent_acceptance"], True)
        self.assertTrue(review["reviewer"])
        fields = {(f["sense_id"], f["field"]): f for f in review["fields"]}
        rights = read("rights.json")
        eligible = {(f["sense_id"], f["field"]): f for f in rights["fields"]}
        proposals = {(g["senses"][0]["sense_id"], name): f["text"]
                     for g in read("review-candidates.json") for name, f in g["senses"][0]["fields"].items()}
        self.assertEqual(len(fields), 200)
        self.assertEqual(set(fields), set(eligible))
        self.assertEqual(set(fields), set(proposals))
        usages = {u["sense_id"]: u for u in review["usage"]}
        self.assertEqual(len(usages), 50)
        for group, (definition_line, ipa_line) in zip(read("groups.json"), SELECTED_LINES, strict=True):
            sense = group["senses"][0]
            self.assertIn(sense["independent_review"], ("passed", "passed_after_exact_accepted_replacements"))
            self.assertEqual(sense["use"], usages[sense["sense_id"]]["accepted_usage"])
            self.assertEqual(sense["source_headword"], usages[sense["sense_id"]]["source_headword"])
            self.assertEqual(usages[sense["sense_id"]]["everyday_sense_status"], "passed")
            for name, field in sense["fields"].items():
                with self.subTest(row=group["frozen_row"], field=name):
                    key = sense["sense_id"], name
                    decision, right = fields[key], eligible[key]
                    self.assertEqual(field["independent_review"], "passed")
                    self.assertEqual(field["rights_eligibility_status"], "passed_at_selected_text_boundary")
                    self.assertEqual(field["text"], decision["accepted_text"])
                    self.assertEqual(right["accepted_text"], decision["accepted_text"])
                    self.assertEqual(decision["proposed_text"], proposals[key])
                    self.assertEqual(decision["accepted_text_quality_status"], "passed")
                    self.assertEqual(decision["rights_eligibility_status"], "passed_at_selected_text_boundary")
                    self.assertEqual(right["independent_review"], "passed_at_selected_text_boundary")
                    self.assertEqual(decision["reviewer"], review["reviewer"])
                    self.assertEqual(right["reviewer"], review["reviewer"])
                    self.assertTrue(decision["review_reason"])
                    self.assertTrue(decision["selected_text_rights_reason"])
                    self.assertEqual(field["source_span"]["line_1_based"],
                                     ipa_line if name == "ipa" else definition_line)
                    self.assertEqual(decision["exact_source"]["source_id"], field["source_id"])
                    self.assertEqual(decision["exact_source"]["line_1_based"], field["source_span"]["line_1_based"])
                    self.assertEqual(decision["exact_source"]["reviewed_excerpt"], field["source_span"]["raw_source_excerpt"])
                    for layer in right["original_historical_dictionary_layers"]:
                        self.assertEqual(layer["independent_review"], "passed_at_original_work_boundary")
        self.assertEqual(len(rights["construction_sources"]), 189)
        for frame in rights["construction_sources"]:
            self.assertEqual(frame["independent_review"], "passed_for_source_binding_only")
            self.assertEqual(frame["reviewer"], review["reviewer"])
            self.assertTrue(frame["independent_rights_reason"])
            for layer in frame["original_historical_dictionary_layers"]:
                self.assertEqual(layer["independent_review"], "passed_at_original_work_boundary")
        for source in rights["sources"]:
            self.assertEqual(source["independent_review"], "passed_at_selected_text_boundary")
        for context in read("source-context.json")["current_content"]:
            self.assertEqual(context["selected_text_rights_status"], "passed_at_selected_text_boundary")
            for layer in context["original_historical_dictionary_layers"]:
                self.assertEqual(layer["independent_review"], "passed_at_original_work_boundary")
                self.assertTrue(layer["independent_rights_reason"])
        admission = read("admission.json")
        self.assertEqual((admission["independently_accepted_new_groups"],
                          admission["independently_accepted_new_fields"]), (50, 200))
        self.assertEqual(admission["pending_independent_review_fields"], 0)
        self.assertEqual(admission["pending_independent_rights_fields"], 0)
        self.assertEqual(admission["pending_independent_source_frames"], 0)
        self.assertEqual(admission["unresolved_field_blockers"], [])
        self.assertEqual(review["unresolved_fields"], [])
        self.assertEqual((review["unchanged_fields"], review["replaced_fields"]), (197, 3))
        changed = {(f["sense_id"], f["field"]) for f in review["fields"]
                   if f["proposed_text"] != f["accepted_text"]}
        self.assertEqual(changed, {("b128-2375-sacrer-ver-selected", name)
                                   for name in ("french_definition", "english", "chinese")})

    def test_quebec_swearing_preserves_its_indirect_construction_in_every_meaning(self):
        sense = next(g for g in read("groups.json") if g["frozen_row"] == 2375)["senses"][0]
        expected = {
            "french_definition": "Au Québec : jurer, notamment en employant les jurons religieux typiques du Québec ; peut aussi exprimer la surprise, la joie ou la douleur. S’emploie sans complément ou avec sacrer contre quelqu’un ou quelque chose.",
            "english": "In Québec, to swear, especially using the characteristic religious swearwords; this can also express surprise, joy or pain. Used without an object or with sacrer contre quelqu’un/quelque chose (to swear at someone or something).",
            "chinese": "魁北克用法：说脏话，尤指使用当地特有的宗教咒骂词；也可表达惊讶、喜悦或疼痛。不接宾语，或用 sacrer contre quelqu’un/quelque chose（咒骂某人或某事）。",
        }
        for name, text in expected.items():
            with self.subTest(field=name):
                self.assertEqual(sense["fields"][name]["text"], text)

    def test_exact_ticket_groups_have_complete_reviewed_senses_and_identical_canonical_overlap(self):
        groups = read("groups.json")
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        self.assertEqual([g["frozen_row"] for g in groups], list(range(2351, 2401)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
                encoding="utf-8-sig", newline="") as handle:
            frozen = list(csv.DictReader(handle))
        for group in groups:
            with self.subTest(row=group["frozen_row"]):
                row = frozen[group["frozen_row"] - 1]
                self.assertEqual((group["headword"], group["lexique_cgram"]),
                                 (row["word"], row["lexique_cgram"]))
                for frequency in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                    self.assertEqual(group[frequency], row[frequency])
                self.assertEqual(len(group["senses"]), 1)
                sense = group["senses"][0]
                self.assertTrue(sense["use"].strip())
                self.assertIsNone(sense["cefr"])
                self.assertEqual(set(sense["fields"]), set(LANGUAGES))
                for name, field in sense["fields"].items():
                    self.assertEqual(field["language"], LANGUAGES[name])
                    self.assertTrue(field["text"].strip())
                    self.assertTrue(field["modification"].strip())
        prior = json.loads((ROOT / "resources/core-pilot-79/curated-inputs.json").read_text(encoding="utf-8"))
        canonical = {g["frozen_row"]: g for g in prior["groups"]}
        for group in groups:
            if group["frozen_row"] in canonical:
                reviewed = json.loads(json.dumps(group))
                # Canonical assembly normalizes only this accepted-state spelling.
                for sense, admitted in zip(reviewed["senses"], canonical[group["frozen_row"]]["senses"], strict=True):
                    self.assertIn(admitted["independent_review"], ("passed", "passed_after_exact_accepted_replacements"))
                    sense["independent_review"] = admitted["independent_review"]
                self.assertEqual(reviewed, canonical[group["frozen_row"]])
        reuse = read("reuse.json")
        self.assertEqual((reuse["reused_groups"], reuse["reused_fields"], reuse["groups"]), (0, 0, []))
        admission = read("admission.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual(admission["independently_accepted_new_fields"], 200)
        self.assertEqual(admission["pending_independent_review_rows"], [])
        self.assertTrue(admission["independent_reviewer"])
        authored = read("admission-author-proposal.json")
        self.assertEqual(authored["independently_accepted_new_fields"], 0)
        self.assertEqual(authored["pending_independent_review_fields"], 200)
        self.assertEqual(admission["newly_admitted_fields"] + admission["canonical_pending_fields"], 200)

    def test_fixed_source_rights_and_constructions_match_separate_review(self):
        rights = read("rights.json")
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        captures = read("source-captures.json")
        archive = Path(captures["capture_root"]).parents[1]
        contexts = {s["id"]: s for s in read("source-context.json")["current_content"]}
        raw = {}
        self.assertEqual(len(sources), 47)
        self.assertEqual(len(captures["sources"]), 47)
        for capture in captures["sources"]:
            source = sources[capture["source_id"]]
            self.assertEqual(source["title"], capture["title"])
            self.assertEqual(source["revid"], capture["revid"])
            self.assertIn("oldid=" + str(source["revid"]), source["fixed_url"])
            self.assertTrue(source["timestamp"])
            self.assertTrue(source["history_url"])
            self.assertTrue(contexts[source["id"]]["current_contributors"])
            for layer in source["original_historical_dictionary_layers"]:
                self.assertEqual(layer["independent_review"], "passed_at_original_work_boundary")
                self.assertTrue(layer["legal_basis_url"])
                if layer["original_authors"] == "Académie française":
                    self.assertTrue(layer["conservative_expiry"].startswith("2006-01-01"))
            raw_path = archive / capture["raw_path"]
            # Source captures are retained in the author archive, outside tracked candidate data.
            if raw_path.exists():
                data = json.loads((archive / capture["api_path"]).read_text(encoding="utf-8-sig"))
                page = next(p for p in data["query"]["pages"] if p["pageid"] == capture["pageid"])
                revision = next(r for r in page["revisions"] if r["revid"] == capture["revid"])
                self.assertEqual(revision["timestamp"], source["timestamp"])
                self.assertEqual(raw_path.read_bytes(), revision["slots"]["main"]["content"].encode("utf-8"))
                raw[source["id"]] = raw_path.read_text(encoding="utf-8").splitlines()
        self.assertTrue(captures["contributor_registry"]["pagination_complete"])
        self.assertFalse(captures["contributor_registry"]["selected_sentence_authorship_proved"])
        parts = [archive / p for p in captures["contributor_registry"]["parts"]]
        self.assertEqual(len(parts), 6)
        if all(path.exists() for path in parts):
            names, continuation = {}, {}
            for path in parts:
                query = parse_qs(urlparse(path.with_suffix(".request.txt").read_text(encoding="utf-8")).query)
                for name, value in continuation.items():
                    self.assertEqual(query[name], [str(value)])
                response = json.loads(path.read_text(encoding="utf-8"))
                self.assertNotIn("error", response)
                for page in response["query"]["pages"]:
                    names.setdefault(page["title"], set()).update(p["name"] for p in page.get("contributors", []))
                continuation = response.get("continue", {})
            self.assertFalse(continuation)
            self.assertEqual(set(names), {s["title"] for s in sources.values()})
            for source in sources.values():
                self.assertEqual(contexts[source["id"]]["current_contributors"], sorted(names[source["title"]]))
        reviews = read("field-review.json")
        proposals = {(f["sense_id"], f["field"]): f for f in rights["fields"]}
        bindings = {(f["sense_id"], f["field"]): f for f in read("source-excerpts.json")["fields"]}
        fields = {(f["sense_id"], f["field"]): f for f in reviews["fields"]}
        self.assertEqual(len(proposals), 200)
        self.assertEqual(set(proposals), set(bindings))
        self.assertEqual(set(proposals), set(fields))
        self.assertIs(reviews["independent_acceptance"], True)
        self.assertTrue(reviews["reviewer"])
        self.assertEqual(rights["independent_rights_fields_accepted"], 200)
        self.assertEqual((EVIDENCE / "COPYING").read_text(encoding="utf-8").rstrip(),
                         (ROOT / rights["complete_legal_text"]).read_text(encoding="utf-8").rstrip())
        self.assertTrue(rights["authority_captures"])
        self.assertTrue(rights["redistribution_obligations"])
        for group in read("groups.json"):
            sense = group["senses"][0]
            self.assertIn(sense["independent_review"], ("passed", "passed_after_exact_accepted_replacements"))
            for name, field in sense["fields"].items():
                with self.subTest(row=group["frozen_row"], field=name):
                    key = sense["sense_id"], name
                    span, binding = field["source_span"], bindings[key]
                    self.assertEqual(binding["reviewed_excerpt"], span["raw_source_excerpt"])
                    self.assertEqual(binding["line_1_based"], span["line_1_based"])
                    self.assertEqual(binding["source_id"], field["source_id"])
                    self.assertEqual(binding["revid"], sources[field["source_id"]]["revid"])
                    self.assertEqual(fields[key]["accepted_text"], field["text"])
                    self.assertEqual(proposals[key]["exact_source_span"], span)
                    self.assertEqual(proposals[key]["source_id"], field["source_id"])
                    for required in ("author_rights_basis", "selected_boundary", "attribution", "modification"):
                        self.assertTrue(proposals[key][required])
                    self.assertEqual(proposals[key]["independent_review"], "passed_at_selected_text_boundary")
                    self.assertEqual(proposals[key]["reviewer"], reviews["reviewer"])
                    self.assertEqual(field["content_license"], "CC-BY-SA-4.0")
                    if field["source_id"] in raw:
                        self.assertEqual(span["raw_source_excerpt"], raw[field["source_id"]][span["line_1_based"] - 1])
                    if name == "ipa":
                        self.assertEqual(field["text"], "/" + binding["ipa_variant_excerpt"] + "/")
                        column = binding["ipa_variant_column_1_based"] - 1
                        self.assertEqual(binding["reviewed_excerpt"][column:column + len(binding["ipa_variant_excerpt"])],
                                         binding["ipa_variant_excerpt"])
        for item in read("source-excerpts.json")["supporting_evidence"]:
            if item["source_id"] in raw:
                self.assertEqual(item["reviewed_excerpt"], raw[item["source_id"]][item["line_1_based"] - 1])
        self.assertEqual(len(reviews["usage"]), 50)
        self.assertEqual(len(reviews["spelling_relations"]), 50)
        for item in reviews["usage"]:
            self.assertTrue(item["construction_and_flexion_evidence"])
            self.assertTrue(item["author_construction_and_flexion_interpretation"])
            self.assertEqual(item["accepted_usage_quality_status"], "passed")
            self.assertEqual(item["accepted_usage"], item["proposed_usage"])
        for relation in reviews["spelling_relations"]:
            self.assertEqual(relation["relation_role"], "source_binding_only")
            self.assertEqual(relation["alias_admission"], "not_admitted")
            self.assertEqual(relation["frozen_headword"], relation["source_headword"])
        groups = {g["frozen_row"]: g["senses"][0] for g in read("groups.json")}
        # Selected restrictions from the frozen first-party source, not unrestricted synonyms.
        for name in ("french_definition", "english", "chinese"):
            self.assertIn("Québec" if name != "chinese" else "魁北克", groups[2375]["fields"][name]["text"])
            self.assertIn("satisfaire à", groups[2381]["fields"][name]["text"])
        self.assertEqual(groups[2372]["fields"]["ipa"]["text"], "/vif/")
        self.assertEqual(groups[2384]["fields"]["ipa"]["source_span"]["line_1_based"], 9)
        self.assertIn("SOV", bindings[(groups[2379]["sense_id"], "french_definition")]["reviewed_excerpt"])
        self.assertEqual(read("admission.json")["pending_independent_review_fields"], 0)


if __name__ == "__main__":
    unittest.main()
