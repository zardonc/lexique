"""Check #145 at the spec-approved resource/admission boundary."""
import csv
import json
from pathlib import Path
from urllib.parse import parse_qs, urlparse
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-145"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
# Literal membership from ticket #145, preserving grammatical subclasses.
MEMBERSHIP = [
    ("ressortir", "VER"), ("kidnapper", "VER"), ("marchandise", "NOM"),
    ("mouton", "NOM"), ("telles", "ADJ:ind"), ("aîné", "NOM"),
    ("circuler", "VER"), ("tigre", "NOM"), ("adopter", "VER"),
    ("terreur", "NOM"), ("légume", "NOM"), ("impôt", "NOM"),
    ("stade", "NOM"), ("claquer", "VER"), ("orphelin", "ADJ"),
    ("boue", "NOM"), ("fonds", "NOM"), ("conserver", "VER"),
    ("pompier", "NOM"), ("plainte", "NOM"), ("répétition", "NOM"),
    ("commissariat", "NOM"), ("mépriser", "VER"), ("pigeon", "NOM"),
    ("tandis que", "CON"), ("vote", "NOM"), ("micro", "NOM"),
    ("plaie", "NOM"), ("connu", "ADJ"), ("disposition", "NOM"),
    ("tels", "ADJ:ind"), ("symbole", "NOM"), ("travailleur", "NOM"),
    ("maigre", "ADJ"), ("taré", "NOM"), ("out", "ADJ"),
    ("conquérir", "VER"), ("déguiser", "VER"), ("certes", "ADV"),
    ("drame", "NOM"), ("actuel", "ADJ"), ("domestique", "NOM"),
    ("portée", "NOM"), ("mâle", "NOM"), ("promotion", "NOM"),
    ("enregistrement", "NOM"), ("particulièrement", "ADV"),
    ("extrême", "ADJ"), ("saut", "NOM"), ("autoroute", "NOM"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch145Tests(unittest.TestCase):
    def test_culinary_bulk_use_retains_the_herb_and_spice_distinction(self):
        sense = next(g for g in read("groups.json") if g["frozen_row"] == 3211)["senses"][0]
        fields = sense["fields"]
        self.assertIn("quantité substantielle", fields["french_definition"]["text"])
        self.assertIn("substantial quantities", fields["english"]["text"])
        self.assertIn("较大量", fields["chinese"]["text"])
        self.assertIn("spices and herbs", sense["use"])

    def test_accepted_emotion_uses_only_the_conventional_selected_phrase(self):
        sense = next(g for g in read("groups.json") if g["frozen_row"] == 3210)["senses"][0]
        for name in ("french_definition", "english", "chinese"):
            self.assertEqual(sense["fields"][name]["source_span"]["raw_source_excerpt"],
                             "[[épouvante]], [[crainte]] [[violent]]e.")

    def test_bodily_wound_explanations_use_current_anatomical_wording(self):
        sense = next(g for g in read("groups.json") if g["frozen_row"] == 3228)["senses"][0]
        self.assertEqual(sense["fields"]["french_definition"]["text"], "Lésion des tissus du corps.")
        self.assertEqual(sense["fields"]["english"]["text"], "Wound; an injury to body tissue.")
        self.assertEqual(sense["fields"]["chinese"]["text"], "创伤或伤口；身体组织受到的损伤（阴性名词）。")

    def test_motorway_crossing_restriction_is_at_road_level(self):
        sense = next(g for g in read("groups.json") if g["frozen_row"] == 3250)["senses"][0]
        self.assertIn("sans intersections à niveau", sense["fields"]["french_definition"]["text"])
        self.assertIn("no at-grade intersections", sense["fields"]["english"]["text"])

    def test_exact_frozen_groups_have_complete_resource_records(self):
        if (EVIDENCE / "groups.json").exists():
            groups = read("groups.json")
        else:
            canonical = json.loads((ROOT / "resources/core-pilot-79/curated-inputs.json").read_text(encoding="utf-8"))
            groups = [g for g in canonical["groups"] if 3201 <= g["frozen_row"] <= 3250]
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP,
                         "The frozen #145 resource slice lacks complete batch content")
        self.assertEqual([g["frozen_row"] for g in groups], list(range(3201, 3251)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(encoding="utf-8-sig", newline="") as handle:
            frozen = list(csv.DictReader(handle))
        self.assertEqual(len(frozen), 4087)
        for group in groups:
            with self.subTest(row=group["frozen_row"]):
                row = frozen[group["frozen_row"] - 1]
                self.assertEqual((group["headword"], group["lexique_cgram"]), (row["word"], row["lexique_cgram"]))
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

    def test_rights_proposals_and_review_gates_are_explicit(self):
        rights = read("authored/rights.json")
        admission = read("authored/admission.json")
        review = read("authored/field-review.json")
        self.assertEqual(len(rights["fields"]), 200)
        self.assertEqual(rights["independent_rights_fields_accepted"], 0)
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual((admission["independently_accepted_new_groups"], admission["independently_accepted_new_fields"]), (0, 0))
        self.assertEqual(admission["missing_admitted_fields"], 200)
        self.assertEqual(admission["pending_independent_review_fields"], 200)
        self.assertEqual(admission["pending_independent_rights_fields"], 200)
        self.assertEqual(admission["unresolved_field_blockers"], [])
        self.assertFalse(review["independent_acceptance"])
        self.assertIsNone(review["reviewer"])
        self.assertEqual((len(review["fields"]), len(review["usage"]), len(review["spelling_relations"])), (200, 50, 50))
        for field in review["fields"]:
            self.assertEqual(field["accepted_text_quality_status"], "pending")
            self.assertIsNone(field["reviewer"])
        expected = {(g["senses"][0]["sense_id"], name): field for g in read("authored/groups.json")
                    for name, field in g["senses"][0]["fields"].items()}
        self.assertEqual({(f["sense_id"], f["field"]) for f in rights["fields"]}, set(expected))
        for field in rights["fields"]:
            proposed = expected[field["sense_id"], field["field"]]
            self.assertEqual(field["exact_source_span"], proposed["source_span"])
            self.assertEqual(field["license"], "CC-BY-SA-4.0")
            self.assertTrue(field["attribution"])
            self.assertTrue(field["modification"])
            self.assertEqual(field["independent_review"], "pending")
            self.assertIsNone(field["reviewer"])

    def test_accepted_fields_uses_and_selected_rights_match_the_review(self):
        groups = read("groups.json")
        review, rights, admission = read("field-review.json"), read("rights.json"), read("admission.json")
        reviewer = review["reviewer"]
        self.assertTrue(review["independent_acceptance"])
        self.assertTrue(review["independence"])
        self.assertIn("reviewer #145", reviewer)
        self.assertEqual(review["unresolved_fields"], [])
        self.assertEqual(rights["independent_rights_fields_accepted"], 200)
        self.assertEqual((admission["independently_accepted_new_groups"], admission["independently_accepted_new_fields"]), (50, 200))
        self.assertEqual((admission["missing_admitted_fields"], admission["pending_independent_review_fields"], admission["pending_independent_rights_fields"]), (0, 0, 0))
        self.assertIn("pending_coordinator", admission["native_checks"])
        self.assertIn("pending_coordinator", admission["conversion_and_packaging"])
        expected = {(g["senses"][0]["sense_id"], name): field for g in groups
                    for name, field in g["senses"][0]["fields"].items()}
        authored = {(g["senses"][0]["sense_id"], name): field for g in read("authored/groups.json")
                    for name, field in g["senses"][0]["fields"].items()}
        decisions = {(r["sense_id"], r["field"]): r for r in review["fields"]}
        legal = {(r["sense_id"], r["field"]): r for r in rights["fields"]}
        self.assertEqual(set(expected), set(decisions))
        self.assertEqual(set(expected), set(legal))
        for key, field in expected.items():
            with self.subTest(key=key):
                quality, eligibility, proposal = decisions[key], legal[key], authored[key]
                self.assertEqual(quality["accepted_text"], field["text"])
                self.assertEqual(quality["proposed_text"], proposal["text"])
                self.assertEqual(quality["proposed_source_span"], proposal["source_span"])
                self.assertEqual(eligibility["accepted_text"], field["text"])
                self.assertEqual(eligibility["exact_source_span"], field["source_span"])
                self.assertEqual(quality["exact_source"]["reviewed_excerpt"], field["source_span"]["raw_source_excerpt"])
                self.assertEqual(quality["exact_source"]["column_1_based"], field["source_span"]["column_1_based"])
                self.assertEqual(field["independent_review"], "passed")
                self.assertEqual(quality["accepted_text_quality_status"], "passed")
                self.assertEqual(field["rights_eligibility_status"], "passed_at_selected_text_boundary")
                self.assertEqual(eligibility["independent_review"], field["rights_eligibility_status"])
                self.assertEqual(quality["reviewer"], reviewer)
                self.assertEqual(eligibility["reviewer"], reviewer)
                self.assertTrue(quality["review_reason"])
                self.assertEqual(quality["selected_text_rights_reason"], eligibility["selected_text_rights_reason"])
                self.assertTrue(eligibility["independent_original_span_assessment"])
        usage = {r["sense_id"]: r for r in review["usage"]}
        spelling = {r["sense_id"]: r for r in review["spelling_relations"]}
        self.assertEqual((len(usage), len(spelling)), (50, 50))
        for group in groups:
            sense = group["senses"][0]
            decision = usage[sense["sense_id"]]
            self.assertEqual(decision["accepted_usage"], sense["use"])
            self.assertEqual(decision["everyday_sense_status"], "passed")
            self.assertEqual(decision["reviewer"], reviewer)
            self.assertTrue(decision["everyday_sense_reason"])
            self.assertEqual(spelling[sense["sense_id"]]["alias_admission"], "not_admitted")
            self.assertEqual(spelling[sense["sense_id"]]["independent_review"], "passed")
        replacements = read("accepted-replacements.json")
        changes = {(r["sense_id"], r["field"]): r for r in replacements["changed_fields"]}
        self.assertEqual(set(changes), {k for k in expected if expected[k]["text"] != authored[k]["text"]
                                      or expected[k]["source_span"] != authored[k]["source_span"]})
        self.assertEqual(sum(expected[k]["text"] != authored[k]["text"] for k in expected), 11)
        self.assertEqual(sum(expected[k]["source_span"] != authored[k]["source_span"] for k in expected), 6)
        for key, record in changes.items():
            self.assertEqual(record["proposed_text"], authored[key]["text"])
            self.assertEqual(record["accepted_text"], expected[key]["text"])
            self.assertEqual(record["proposed_source_span"], authored[key]["source_span"])
            self.assertEqual(record["accepted_source_span"], expected[key]["source_span"])
        original_scope = rights["independent_original_work_assessment"]
        self.assertEqual(original_scope["historical_expressive_replacement"]["status"], "full_proposed_exposition_not_cleared_or_admitted")
        self.assertIn("France-only", rights["original_daf8_basis_scope"])

    def test_selected_spans_and_contributors_match_fixed_captures(self):
        captures = read("source-captures.json")
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        contexts = {s["id"]: s for s in read("source-context.json")["current_content"]}
        capture_root = Path(captures["capture_root"])
        self.assertEqual(str(capture_root), read("admission.json")["author_evidence_root"])
        self.assertEqual((capture_root.name, capture_root.parent.name), ("issue-145", "author-evidence"))

        def retained(name):
            return capture_root.joinpath(*Path(name).parts[2:])

        self.assertEqual(len(captures["sources"]), 51)
        raw = {}
        for capture in captures["sources"]:
            data = json.loads(retained(capture["api_path"]).read_text(encoding="utf-8-sig"))
            page = next(p for p in data["query"]["pages"] if p["pageid"] == capture["pageid"])
            revision = next(r for r in page["revisions"] if r["revid"] == capture["revid"])
            body = revision["slots"]["main"]["content"]
            self.assertEqual((page["title"], revision["timestamp"]), (capture["title"], capture["timestamp"]))
            self.assertEqual(body, retained(capture["raw_path"]).read_text(encoding="utf-8"))
            source = next(s for s in sources.values() if s["pageid"] == capture["pageid"])
            self.assertEqual(source["revid"], capture["revid"])
            self.assertEqual(parse_qs(urlparse(source["fixed_url"]).query)["oldid"], [str(capture["revid"])])
            raw[source["id"]] = body.splitlines()
        excerpts = read("source-excerpts.json")
        self.assertEqual(len(excerpts["fields"]), 200)
        for record in [*excerpts["fields"], *excerpts["supporting_evidence"]]:
            line = raw[record["source_id"]][record["line_1_based"] - 1]
            column = record["column_1_based"] - 1
            self.assertEqual(line[column:column + len(record["reviewed_excerpt"])], record["reviewed_excerpt"])
            self.assertFalse(record["reviewed_excerpt"].lstrip().startswith("#*"))
            self.assertEqual(record["revid"], sources[record["source_id"]]["revid"])
        by_field = {(r["sense_id"], r["field"]): r for r in excerpts["fields"]}
        self.assertEqual(len(by_field), 200)
        pos_frames = {"NOM": "nom", "VER": "verbe", "ADJ": "adjectif", "ADJ:ind": "adjectif indéfini",
                      "CON": "conjonction", "ADV": "adverbe"}
        for group in read("groups.json"):
            sense = group["senses"][0]
            for name, field in sense["fields"].items():
                exact = by_field[sense["sense_id"], name]
                self.assertEqual(field["source_id"], exact["source_id"])
                self.assertEqual(field["source_span"]["raw_source_excerpt"], exact["reviewed_excerpt"])
                self.assertEqual(field["source_span"]["line_1_based"], exact["line_1_based"])
                preceding = raw[exact["source_id"]][:exact["line_1_based"]]
                language = next(line for line in reversed(preceding) if line.startswith("== "))
                self.assertRegex(language, r"^==\s*\{\{langue\|fr\}\}\s*==$")
                if name != "ipa":
                    pos = next(line for line in reversed(preceding) if line.startswith("=== {{S|"))
                    self.assertIn("{{S|" + pos_frames[group["lexique_cgram"]] + "|fr", pos)
                else:
                    self.assertEqual(field["text"].strip("/"), exact["ipa_variant_excerpt"])
                    column = exact["ipa_variant_column_1_based"] - 1
                    self.assertEqual(exact["reviewed_excerpt"][column:column+len(exact["ipa_variant_excerpt"])], exact["ipa_variant_excerpt"])
        names, anonymous, chains = {}, {}, {}
        for name in captures["contributor_registry"]["parts"]:
            path = retained(name)
            data = json.loads(path.read_text(encoding="utf-8-sig"))
            prefix, part = path.name.removesuffix(".api.json").rsplit("-", 1)
            chains.setdefault(prefix, {})[int(part)] = (path, data)
            for page in data["query"]["pages"]:
                names.setdefault(page["title"], set()).update(p["name"] for p in page.get("contributors", []))
                if "anoncontributors" in page:
                    anonymous[page["title"]] = page["anoncontributors"]
        self.assertEqual(set(chains), {f"contributors-new-{n}" for n in range(1, 7)})
        for parts in chains.values():
            self.assertEqual(sorted(parts), list(range(1, len(parts) + 1)))
            for part, (path, data) in parts.items():
                continuation = data.get("continue")
                if part == len(parts):
                    self.assertFalse(continuation)
                else:
                    self.assertTrue(continuation)
                    query = parse_qs(urlparse(parts[part+1][0].with_suffix(".request.txt").read_text(encoding="utf-8")).query)
                    for key, value in continuation.items():
                        self.assertEqual(query[key], [str(value)])
        self.assertEqual(set(names), {s["title"] for s in sources.values()})
        for context in contexts.values():
            self.assertEqual(context["current_contributors"], sorted(names[context["title"]]))
            self.assertEqual(context["current_registry_names"], len(names[context["title"]]))
            self.assertEqual(context["anonymous_contributors_count"], anonymous.get(context["title"], 0))
        self.assertFalse(captures["contributor_registry"]["selected_sentence_authorship_proved"])

    def test_inflected_indefinite_adjectives_and_french_out_keep_scope(self):
        groups = {g["frozen_row"]: g for g in read("groups.json")}
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        support = read("source-excerpts.json")["supporting_evidence"]
        for row, headword, number in ((3205, "telles", "Féminin pluriel"), (3231, "tels", "Masculin pluriel")):
            group = groups[row]
            sense = group["senses"][0]
            self.assertEqual((group["headword"], group["lexique_cgram"]), (headword, "ADJ:ind"))
            self.assertEqual(sense["source_headword"], "tel")
            self.assertEqual(sense["fields"]["ipa"]["text"], "/tɛl/")
            self.assertEqual(sources[sense["fields"]["ipa"]["source_id"]]["title"], headword)
            rows = [r for r in support if r["sense_id"] == sense["sense_id"]]
            self.assertTrue(any("{{S|adjectif indéfini|fr}}" in r["reviewed_excerpt"] for r in rows))
            self.assertTrue(any(number in r["reviewed_excerpt"] and "tel#fr-adj" in r["reviewed_excerpt"] for r in rows))
            self.assertIn(number.lower(), sense["fields"]["french_definition"]["text"].lower())
        out = groups[3236]
        sense = out["senses"][0]
        self.assertEqual((out["headword"], out["lexique_cgram"]), ("out", "ADJ"))
        self.assertEqual(sense["fields"]["ipa"]["text"], "/awt/")
        self.assertIn("Démodé", sense["fields"]["french_definition"]["source_span"]["raw_source_excerpt"])
        rows = [r for r in support if r["sense_id"] == sense["sense_id"]]
        self.assertTrue(any(r["reviewed_excerpt"] == "== {{langue|fr}} ==" for r in rows))
        self.assertTrue(any("{{S|adjectif|fr}}" in r["reviewed_excerpt"] for r in rows))
        self.assertTrue(any("{{invariable|fr}}" in r["reviewed_excerpt"] for r in rows))

    def test_notices_and_original_proposals_accompany_content(self):
        names = ("groups.json", "field-review.json", "rights.json", "sources.json", "source-context.json", "source-excerpts.json", "admission.json")
        for name in names:
            self.assertTrue((EVIDENCE / "authored" / name).is_file())
        notices = (EVIDENCE / "NOTICES.md").read_text(encoding="utf-8")
        self.assertEqual((EVIDENCE / "COPYING").read_bytes().rstrip(), (ROOT / "resources/core-pilot-79/COPYING").read_bytes().rstrip())
        for source in read("sources.json")["sources"]:
            self.assertIn(source["fixed_url"], notices)
            self.assertIn(source["history_url"], notices)
        selected_rights = (EVIDENCE / "selected-rights.md").read_text(encoding="utf-8")
        self.assertIn("2006-01-01", selected_rights)
        self.assertIn("CC BY-SA 4.0", selected_rights)
        self.assertIn("CFC", selected_rights)


if __name__ == "__main__":
    unittest.main()
