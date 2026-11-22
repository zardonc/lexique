"""Check #137 at the approved resource/admission boundary."""
import csv
import json
from pathlib import Path
import sys
from urllib.parse import parse_qs, urlparse
import unittest

ROOT = Path(__file__).resolve().parents[1]
ROWS = set(range(2801, 2851))
EVIDENCE = ROOT / "docs/evidence/issue-137"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
MEMBERSHIP = [
    ("sauce", "NOM"),
    ("tunnel", "NOM"),
    ("déclencher", "VER"),
    ("drap", "NOM"),
    ("expression", "NOM"),
    ("conduite", "NOM"),
    ("surveillance", "NOM"),
    ("contrer", "VER"),
    ("illusion", "NOM"),
    ("scénario", "NOM"),
    ("union", "NOM"),
    ("rideau", "NOM"),
    ("communiquer", "VER"),
    ("territoire", "NOM"),
    ("poésie", "NOM"),
    ("tentative", "NOM"),
    ("profondément", "ADV"),
    ("pénétrer", "VER"),
    ("secouer", "VER"),
    ("renseignement", "NOM"),
    ("fréquenter", "VER"),
    ("cingler", "VER"),
    ("franchir", "VER"),
    ("évanouir", "VER"),
    ("entretien", "NOM"),
    ("soutien", "NOM"),
    ("boucle", "NOM"),
    ("caractère", "NOM"),
    ("emballer", "VER"),
    ("alliance", "NOM"),
    ("hâte", "NOM"),
    ("négocier", "VER"),
    ("démolir", "VER"),
    ("as", "NOM"),
    ("jury", "NOM"),
    ("planquer", "VER"),
    ("barre", "NOM"),
    ("misère", "NOM"),
    ("clairement", "ADV"),
    ("concevoir", "VER"),
    ("fumier", "NOM"),
    ("production", "NOM"),
    ("comité", "NOM"),
    ("navrer", "VER"),
    ("fameux", "ADJ"),
    ("auto", "NOM"),
    ("mamie", "NOM"),
    ("bâtir", "VER"),
    ("trahison", "NOM"),
    ("canapé", "NOM"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


def retained_path(name):
    path = Path(name)
    if not path.is_absolute():
        path = ROOT / path
    if path.is_file():
        return path
    parts = name.replace("\\", "/").split("/")
    prefix = parts.index("issue-137")
    return Path(read("admission.json")["author_evidence_root"]).joinpath(*parts[prefix + 1:])


class ReviewedShard137Tests(unittest.TestCase):
    def test_chinese_land_area_and_grandmother_scope_match_selected_french(self):
        groups = {g['frozen_row']: g for g in read('groups.json')}
        for row, expected in ((2814, '地域；边界或明确或不明确的一片土地。'),
                              (2847, '奶奶；外婆，亲昵、口语或儿童用语中的“祖母或外祖母”。')):
            with self.subTest(row=row):
                self.assertEqual(groups[row]['senses'][0]['fields']['chinese']['text'], expected)

    def test_exact_frozen_groups_have_four_reviewed_fields_and_honest_gates(self):
        groups = read("groups.json")
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        self.assertEqual([g["frozen_row"] for g in groups], list(range(2801, 2851)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(encoding="utf-8-sig", newline="") as handle:
            frozen = list(csv.DictReader(handle))
        for group in groups:
            row = frozen[group["frozen_row"] - 1]
            self.assertEqual((group["headword"], group["lexique_cgram"], group["subtitle_lemma_per_million"], group["subtitle_pos_per_million"]), tuple(row.values()))
            self.assertEqual(len(group["senses"]), 1)
            sense = group["senses"][0]
            self.assertTrue(sense["use"])
            self.assertIsNone(sense["cefr"])
            self.assertEqual(set(sense["fields"]), set(LANGUAGES))
            self.assertEqual(sense["independent_review"], "passed_after_exact_accepted_replacements")
            for name, field in sense["fields"].items():
                self.assertEqual(field["language"], LANGUAGES[name])
                self.assertTrue(field["text"].strip())
                self.assertTrue(field["modification"].strip())
                self.assertEqual(field["independent_review"], "passed")
        admission = read("admission.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual((admission["independently_accepted_new_groups"], admission["independently_accepted_new_fields"]), (50, 200))
        self.assertEqual((admission["pending_independent_review_fields"], admission["pending_independent_rights_fields"]), (0, 0))
        self.assertEqual(admission["pending_independent_review_rows"], [])
        self.assertTrue(admission["independent_reviewer"])
        self.assertEqual((admission["annotation_exception_groups"], admission["specialist_exception_groups"]), (0, 0))
        self.assertTrue(read("field-review.json")["independent_acceptance"])
        self.assertEqual(read("field-review.json")["kind"], "separate_independent_field_review")
        self.assertTrue(read("field-review.json")["independence"])
        self.assertEqual(read("field-review.json")["unresolved_fields"], [])
        self.assertEqual(read("reuse.json")["reused_fields"], 0)

    def test_exact_proposals_accepted_fields_and_distinct_rights_decisions_align(self):
        reviewer = read("admission.json")["independent_reviewer"]
        originals = {(f["sense_id"], f["field"]): f for f in read("authored/field-review.json")["fields"]}
        reviews = {(f["sense_id"], f["field"]): f for f in read("field-review.json")["fields"]}
        rights = {(f["sense_id"], f["field"]): f for f in read("rights.json")["fields"]}
        self.assertEqual(len(reviews), 200)
        self.assertEqual(len(read("rights.json")["fields"]), 200)
        self.assertEqual(len(read("field-review.json")["fields"]), 200)
        self.assertEqual(set(reviews), set(originals))
        self.assertEqual(set(reviews), set(rights))
        changes = {}
        for group, authored in zip(read("groups.json"), read("authored/groups.json"), strict=True):
            self.assertEqual(group["frozen_row"], authored["frozen_row"])
            sense, proposed = group["senses"][0], authored["senses"][0]
            self.assertEqual(sense["sense_id"], proposed["sense_id"])
            for name, field in sense["fields"].items():
                key = sense["sense_id"], name
                record, legal, original = reviews[key], rights[key], originals[key]
                self.assertEqual(record["proposed_text"], original["proposed_text"])
                self.assertEqual(record["proposed_text"], proposed["fields"][name]["text"])
                self.assertEqual(record["exact_source"], original["exact_source"])
                self.assertEqual(record["supporting_sources"], original["supporting_sources"])
                self.assertEqual(record["accepted_text"], field["text"])
                self.assertEqual(legal["accepted_text"], field["text"])
                self.assertEqual(legal["exact_source_span"], field["source_span"])
                self.assertEqual(record["accepted_text_quality_status"], "passed")
                self.assertEqual(record["rights_eligibility_status"], "passed_at_selected_text_boundary")
                self.assertEqual(legal["independent_review"], "passed_at_selected_text_boundary")
                self.assertTrue(record["independence"])
                self.assertTrue(record["review_reason"])
                self.assertTrue(record["selected_text_rights_reason"])
                self.assertEqual(record["selected_text_rights_reason"], legal["selected_text_rights_reason"])
                self.assertEqual(record["reviewer"], reviewer)
                self.assertEqual(legal["reviewer"], reviewer)
                if record["proposed_text"] != record["accepted_text"]:
                    changes[key] = record
        replacements = {(f["sense_id"], f["field"]): f for f in read("accepted-replacements.json")}
        self.assertEqual(set(changes), set(replacements))
        self.assertEqual({(f["frozen_row"], f["field"]) for f in replacements.values()}, {(2814, "chinese"), (2847, "chinese")})
        for key, replacement in replacements.items():
            for name in ("proposed_text", "accepted_text", "review_reason", "reviewer"):
                self.assertEqual(replacement[name], reviews[key][name])
        self.assertEqual(read("admission.json")["replacement_fields"], len(changes))
        self.assertEqual(read("admission.json")["unchanged_accepted_fields"], 198)
        self.assertEqual(read("rights.json")["independent_rights_fields_accepted"], 200)
        self.assertEqual(read("source-excerpts.json"), read("authored/source-excerpts.json"))

    def test_fixed_sources_and_every_field_span_match_actual_captures(self):
        captures = read("source-captures.json")
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        raw = {}
        self.assertEqual(len(captures["sources"]), 53)
        self.assertEqual(len(sources), 53)
        for capture in captures["sources"]:
            data = json.loads(retained_path(capture["api_path"]).read_text(encoding="utf-8-sig"))
            page = next(p for p in data["query"]["pages"] if p.get("pageid") == capture["pageid"])
            revision = next(r for r in page["revisions"] if r["revid"] == capture["revid"])
            body = retained_path(capture["raw_path"]).read_bytes().decode("utf-8")
            self.assertEqual(body, revision["slots"]["main"]["content"])
            source = sources[capture["source_id"]]
            self.assertEqual((source["title"], source["revid"], source["timestamp"]), (page["title"], revision["revid"], revision["timestamp"]))
            self.assertIn("oldid=" + str(source["revid"]), source["fixed_url"])
            raw[source["id"]] = body.splitlines()
        self.assertEqual(set(sources), set(raw))
        fields = {(f["sense_id"], f["field"]): f for f in read("source-excerpts.json")["fields"]}
        rights = {(f["sense_id"], f["field"]): f for f in read("rights.json")["fields"]}
        proposals = {(f["sense_id"], f["field"]): f for f in read("field-review.json")["fields"]}
        self.assertEqual(len(fields), 200)
        self.assertEqual(set(fields), set(rights))
        self.assertEqual(set(fields), set(proposals))
        for group in read("groups.json"):
            sense = group["senses"][0]
            for name, field in sense["fields"].items():
                key = (sense["sense_id"], name)
                span = field["source_span"]
                self.assertEqual(span["raw_source_excerpt"], raw[field["source_id"]][span["line_1_based"] - 1])
                self.assertEqual(field["content_license"], "CC-BY-SA-4.0")
                self.assertEqual(rights[key]["exact_source_span"], span)
                self.assertTrue(rights[key]["author_rights_basis"])
                self.assertEqual(proposals[key]["accepted_text"], field["text"])
                self.assertEqual(fields[key]["source_id"], field["source_id"])
                self.assertEqual(fields[key]["reviewed_excerpt"], span["raw_source_excerpt"])
                self.assertEqual(proposals[key]["exact_source"]["source_id"], field["source_id"])
                self.assertEqual(proposals[key]["exact_source"]["revid"], sources[field["source_id"]]["revid"])
                lines = raw[field["source_id"]][:span["line_1_based"]]
                language = next(l for l in reversed(lines) if l.startswith("== "))
                self.assertEqual(language, "== {{langue|fr}} ==")
                pos = next(l for l in reversed(lines) if l.startswith("=== {{S|"))
                expected = {"NOM": "nom", "VER": "verbe", "ADJ": "adjectif", "ADV": "adverbe"}[group["lexique_cgram"]]
                self.assertIn("{{S|" + expected + "|fr", pos)
                if name == "ipa":
                    self.assertIn(field["text"].strip("/"), span["raw_source_excerpt"])
                    exact = fields[key]
                    column = exact["ipa_variant_column_1_based"] - 1
                    self.assertEqual(span["raw_source_excerpt"][column:column + len(exact["ipa_variant_excerpt"])], field["text"].strip("/"))
        for binding in read("source-excerpts.json")["supporting_evidence"]:
            line = raw[binding["source_id"]][binding["line_1_based"]-1]
            start = binding["column_1_based"]-1
            self.assertEqual(binding["reviewed_excerpt"], line[start:start+len(binding["reviewed_excerpt"])])
        self.assertEqual((EVIDENCE / "COPYING").read_text(encoding="utf-8").rstrip(), (ROOT / "resources/core-pilot-79/COPYING").read_text(encoding="utf-8").rstrip())
        for authority in read("rights.json")["authority_captures"]:
            self.assertTrue(retained_path(authority["path"]).is_file())
            if "pages" in authority:
                data = json.loads(retained_path(authority["path"]).read_text(encoding="utf-8"))
                for capture in authority["pages"]:
                    page = next(p for p in data["query"]["pages"] if p["pageid"] == capture["pageid"])
                    revision = next(r for r in page["revisions"] if r["revid"] == capture["revid"])
                    self.assertEqual(retained_path(capture["raw_path"]).read_text(encoding="utf-8"), revision["slots"]["main"]["content"])
                    self.assertIn("oldid=" + str(capture["revid"]), capture["fixed_url"])
        for key in ("preparation_script", "selection_file"):
            self.assertTrue(Path(captures[key]).is_file())
        for source in sources.values():
            self.assertEqual(source["selected_text_review"], "passed_for_selected_spans")
            self.assertEqual(source["selected_rights_review"]["selected_text_rights_status"], "passed_at_selected_text_boundary")
        for record in read("rights.json")["construction_sources"]:
            self.assertEqual(record["independent_review"], "passed_at_selected_text_boundary")
            self.assertTrue(record["selected_text_rights_reason"])

    def test_contributor_pagination_is_complete_and_matches_article_credits(self):
        captures = read("source-captures.json")
        names, anonymous, continuation = {}, {}, {}
        parts = [retained_path(p) for p in captures["contributor_registry"]["parts"]]
        for prefix in ("contributors-", "contributors-support-"):
            continuation = {}
            selected = sorted((p for p in parts if p.name[len(prefix):].split(".")[0].isdigit() and p.name.startswith(prefix)),
                              key=lambda p: int(p.name[len(prefix):].split(".")[0]))
            self.assertTrue(selected)
            for path in selected:
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
        self.assertEqual(set(names), {p["title"] for p in captures["sources"]})
        contexts = {s["title"]:s for s in read("source-context.json")["current_content"]}
        for title in names:
            self.assertEqual(contexts[title]["current_contributors"], sorted(names[title]))
            self.assertEqual(contexts[title]["anonymous_contributors_count"], anonymous.get(title,0))

    def test_selected_uses_preserve_construction_register_and_pronunciation(self):
        groups = {g["frozen_row"]: g for g in read("groups.json")}
        for row, fragment in ((2824, "s’évanouir"), (2822, "weather"), (2831, "h aspiré"),
                              (2836, "Slang"), (2847, "affectionate"), (2834, "card noun"),
                              (2827, "buckle"), (2815, "not restricted")):
            self.assertIn(fragment, groups[row]["senses"][0]["use"])
        for row, text in ((2824, "/e.va.nwiʁ/"), (2834, "/as/"), (2831, "/ɑt/"),
                          (2848, "/ba.tiʁ/"), (2827, "/bukl/")):
            self.assertEqual(groups[row]["senses"][0]["fields"]["ipa"]["text"], text)
        faint = groups[2824]["senses"][0]["fields"]
        self.assertIn("s’évanouir", faint["french_definition"]["text"])
        self.assertIn("pronominal", faint["english"]["text"])
        self.assertIn("代词式", faint["chinese"]["text"])
        self.assertTrue(all(r["alias_admission"] == "not_admitted" for r in read("source-context.json")["spelling_relations"]))
        uses = read("field-review.json")["usage"]
        self.assertEqual(len(uses), 50)
        proposed_uses = {u["sense_id"]: u for u in read("authored/field-review.json")["usage"]}
        senses = {g["senses"][0]["sense_id"]: g["senses"][0] for g in groups.values()}
        for usage in uses:
            self.assertEqual(usage["accepted_usage_quality_status"], "passed")
            self.assertEqual(usage["everyday_sense_status"], "passed")
            self.assertTrue(usage["reviewer"])
            self.assertTrue(usage["review_reason"])
            self.assertTrue(usage["everyday_sense_reason"])
            self.assertEqual(usage["proposed_usage"], proposed_uses[usage["sense_id"]]["proposed_usage"])
            self.assertEqual(usage["accepted_usage"], senses[usage["sense_id"]]["use"])
            self.assertEqual(usage["excluded_uses"], senses[usage["sense_id"]]["excluded_uses"])
        self.assertTrue(all(r["independent_review"] == "passed_for_source_binding_only" and r["reviewer"] and r["review_reason"]
                            for r in read("source-context.json")["spelling_relations"]))
        for record in read("rights.json")["fields"]:
            self.assertEqual(record["independent_review"], "passed_at_selected_text_boundary")
            self.assertTrue(record["reviewer"])
            for layer in record["original_historical_dictionary_layers"]:
                if "eighth edition" in layer["edition"]:
                    self.assertEqual(layer["conservative_expiry"], "2006-01-01, using final publication in 1935")
                    self.assertEqual(layer["independent_review"], "passed_at_original_work_boundary")


class Batch137Tests(unittest.TestCase):
    def test_frozen_groups_have_admitted_content(self):
        resource = json.loads((ROOT / "resources/core-pilot-79/curated-inputs.json").read_text(encoding="utf-8"))
        admitted = {g["frozen_row"]: g for g in resource["groups"] if g["frozen_row"] in ROWS}
        self.assertEqual(ROWS - set(admitted), set(), "#137 frozen groups lack coordinator canonical admission")
        for group in read("groups.json"):
            self.assertEqual(admitted[group["frozen_row"]], group)


if __name__ == "__main__":
    review_only = "--review-only" in sys.argv
    if review_only:
        sys.argv.remove("--review-only")
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(ReviewedShard137Tests)
    if not review_only:
        suite.addTests(unittest.defaultTestLoader.loadTestsFromTestCase(Batch137Tests))
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    sys.exit(0 if result.wasSuccessful() else 1)
