"""Check #129 at the approved resource/admission boundary."""
import csv
import json
from pathlib import Path
from urllib.parse import parse_qs, urlparse
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-129"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
# Exact membership from the self-contained #129 ticket.
MEMBERSHIP = [
    ("document", "NOM"), ("nier", "VER"), ("précieux", "ADJ"),
    ("écarter", "VER"), ("kilomètre", "NOM"), ("nation", "NOM"),
    ("guider", "VER"), ("protection", "NOM"), ("manque", "NOM"),
    ("musical", "ADJ"), ("écoute", "NOM"), ("ordinaire", "ADJ"),
    ("religion", "NOM"), ("mouche", "NOM"), ("avantage", "NOM"),
    ("solide", "ADJ"), ("boucler", "VER"), ("détective", "NOM"),
    ("indien", "ADJ"), ("indien", "NOM"), ("comporter", "VER"),
    ("panne", "NOM"), ("songer", "VER"), ("ours", "NOM"),
    ("écrivain", "NOM"), ("commission", "NOM"), ("passager", "NOM"),
    ("boucher", "VER"), ("ha", "ONO"), ("remuer", "VER"),
    ("communiste", "ADJ"), ("communiste", "NOM"), ("poussière", "NOM"),
    ("mai", "NOM"), ("clef", "NOM"), ("échec", "NOM"), ("balader", "VER"),
    ("chrétien", "ADJ"), ("chrétien", "NOM"), ("témoigner", "VER"),
    ("gratuit", "ADJ"), ("minable", "ADJ"), ("soulever", "VER"),
    ("ticket", "NOM"), ("forcément", "ADV"), ("céder", "VER"),
    ("hélas", "ONO"), ("foot", "NOM"), ("barbe", "NOM"), ("incapable", "ADJ"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


def retained_path(name):
    parts = name.replace("\\", "/").split("/")
    local = ROOT.joinpath(*parts)
    if local.is_file():
        return local
    return Path(read("authored/admission.json")["author_evidence_root"]).joinpath(*parts[2:])


class Batch129Tests(unittest.TestCase):
    def test_boucler_chinese_requires_a_buckle_in_the_selected_transitive_use(self):
        sense = read("groups.json")[16]["senses"][0]
        self.assertEqual(sense["fields"]["chinese"]["text"], "用搭扣扣住、扣紧；及物动词。")
        proposal = read("authored/groups.json")[16]["senses"][0]
        self.assertEqual(proposal["fields"]["chinese"]["text"], "用扣子扣住、扣紧；及物动词。")

    def test_selected_original_work_rights_keep_the_reviewed_expiry_and_author_proposal(self):
        rights = read("rights.json")
        self.assertIn("2006-01-01", rights["original_daf8_basis_scope"])
        self.assertIn("1966-01-01", read("authored/rights.json")["original_daf8_basis_scope"])
        for filename, key in (("sources.json", "sources"),
                              ("source-context.json", "current_content"),
                              ("rights.json", "sources"), ("rights.json", "fields")):
            for record in read(filename)[key]:
                for layer in record["original_historical_dictionary_layers"]:
                    self.assertEqual(layer["independent_review"], "passed_at_original_work_boundary")
                    if layer["original_authors"] == "Académie française":
                        self.assertIn("2006-01-01", layer["conservative_expiry"])
                        self.assertIn("1966-01-01", layer["author_proposed_conservative_expiry"])
                    else:
                        self.assertEqual(layer["conservative_expiry"], "1952-01-01")

    def test_exact_ticket_groups_have_complete_independently_reviewed_senses(self):
        groups = read("groups.json")
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        self.assertEqual([g["frozen_row"] for g in groups], list(range(2401, 2451)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
                encoding="utf-8-sig", newline="") as handle:
            frozen = list(csv.DictReader(handle))
        for group in groups:
            with self.subTest(row=group["frozen_row"]):
                row = frozen[group["frozen_row"] - 1]
                self.assertEqual((group["headword"], group["lexique_cgram"]),
                                 (row["word"], row["lexique_cgram"]))
                self.assertEqual(group["subtitle_lemma_per_million"], row["subtitle_lemma_per_million"])
                self.assertEqual(group["subtitle_pos_per_million"], row["subtitle_pos_per_million"])
                self.assertEqual(len(group["senses"]), 1)
                sense = group["senses"][0]
                self.assertTrue(sense["use"].strip())
                self.assertIsNone(sense["cefr"])
                self.assertEqual(set(sense["fields"]), set(LANGUAGES))
                for name, field in sense["fields"].items():
                    self.assertEqual(field["language"], LANGUAGES[name])
                    self.assertTrue(field["text"].strip())
                    self.assertTrue(field["modification"].strip())
        reuse = read("reuse.json")
        prior = json.loads((ROOT / "resources/core-pilot-79/curated-inputs.json").read_text(encoding="utf-8"))
        canonical = {g["frozen_row"]: g for g in prior["groups"]}
        for group in groups:
            if group["frozen_row"] in canonical:
                self.assertEqual(group, canonical[group["frozen_row"]])
        self.assertEqual((reuse["reused_groups"], reuse["reused_fields"], reuse["groups"]), (0, 0, []))
        admission = read("admission.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual((admission["independently_accepted_new_groups"],
                          admission["independently_accepted_new_fields"]), (50, 200))
        self.assertEqual(admission["pending_independent_review_rows"], [])
        self.assertEqual(admission["pending_independent_review_fields"], 0)
        self.assertEqual(admission["pending_independent_rights_fields"], 0)
        self.assertEqual(admission["missing_fields"], 0)
        self.assertEqual(admission["unresolved_field_blockers"], [])
        self.assertTrue(admission["independent_reviewer"])
        authored = read("authored/admission.json")
        self.assertEqual(authored["independently_accepted_new_fields"], 0)
        self.assertEqual(authored["pending_independent_review_fields"], 200)

    def test_exact_accepted_fields_have_independent_quality_rights_and_fixed_source_frames(self):
        rights = read("rights.json")
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        captures = read("source-captures.json")
        contexts = {s["id"]: s for s in read("source-context.json")["current_content"]}
        raw = {}
        self.assertEqual(len(captures["sources"]), 48)
        for capture in captures["sources"]:
            data = json.loads(retained_path(capture["api_path"]).read_text(encoding="utf-8-sig"))
            page = next(p for p in data["query"]["pages"] if p["pageid"] == capture["pageid"])
            revision = next(r for r in page["revisions"] if r["revid"] == capture["revid"])
            source = sources[capture["source_id"]]
            self.assertEqual((source["title"], source["revid"], source["timestamp"]),
                             (page["title"], revision["revid"], revision["timestamp"]))
            body = retained_path(capture["raw_path"]).read_bytes().decode("utf-8")
            self.assertEqual(body, revision["slots"]["main"]["content"])
            self.assertIn("oldid=" + str(revision["revid"]), source["fixed_url"])
            self.assertTrue(source["history_url"])
            self.assertTrue(source["attribution"])
            self.assertTrue(contexts[source["id"]]["current_contributors"])
            self.assertEqual(source["selected_text_review"], "passed_at_selected_text_boundary")
            self.assertEqual(contexts[source["id"]]["status"], "passed_independent_selected_source_frame_review")
            raw[source["id"]] = body.splitlines()
        parts = [retained_path(p) for p in captures["contributor_registry"]["parts"]]
        self.assertTrue(parts)
        contributors = {}
        anonymous = {}
        continuation = {}
        for path in parts:
            query = parse_qs(urlparse(path.with_suffix(".request.txt").read_text(encoding="utf-8")).query)
            for name, value in continuation.items():
                self.assertEqual(query[name], [str(value)])
            response = json.loads(path.read_text(encoding="utf-8"))
            self.assertNotIn("error", response)
            for page in response["query"]["pages"]:
                contributors.setdefault(page["title"], set()).update(p["name"] for p in page.get("contributors", []))
                if "anoncontributors" in page:
                    anonymous[page["title"]] = page["anoncontributors"]
            continuation = response.get("continue", {})
        self.assertFalse(continuation)
        self.assertEqual(set(contributors), {s["title"] for s in sources.values()})
        for source in sources.values():
            self.assertEqual(contexts[source["id"]]["current_contributors"], sorted(contributors[source["title"]]))
            self.assertEqual(contexts[source["id"]]["anonymous_contributors_count"], anonymous.get(source["title"], 0))
        proposals = {(f["sense_id"], f["field"]): f for f in rights["fields"]}
        review = read("field-review.json")
        self.assertEqual(len(proposals), 200)
        self.assertEqual(len(review["fields"]), 200)
        self.assertEqual(review["kind"], "separate_independent_field_review")
        self.assertTrue(review["independent_acceptance"])
        self.assertTrue(review["independence"])
        self.assertEqual(review["reviewer"], read("admission.json")["independent_reviewer"])
        self.assertEqual(rights["independent_rights_fields_accepted"], 200)
        self.assertEqual(rights["independent_review"], "passed_at_selected_text_boundary")
        self.assertTrue(rights["authority_captures"])
        self.assertEqual((EVIDENCE / "COPYING").read_text(encoding="utf-8").rstrip(),
                         (ROOT / "resources/core-pilot-79/COPYING").read_text(encoding="utf-8").rstrip())
        reviews = {(f["sense_id"], f["field"]): f for f in review["fields"]}
        originals = {(s["sense_id"], name): f for g in read("authored/groups.json")
                     for s in g["senses"] for name, f in s["fields"].items()}
        bindings = {(f["sense_id"], f["field"]): f for f in read("source-excerpts.json")["fields"]}
        self.assertEqual(set(reviews), set(proposals))
        self.assertEqual(set(reviews), set(originals))
        self.assertEqual(set(reviews), set(bindings))
        for group in read("groups.json"):
            sense = group["senses"][0]
            self.assertEqual(sense["independent_review"], "passed_after_exact_accepted_replacements")
            for name, field in sense["fields"].items():
                with self.subTest(row=group["frozen_row"], field=name):
                    span = field["source_span"]
                    self.assertEqual(span["raw_source_excerpt"], raw[field["source_id"]][span["line_1_based"] - 1])
                    lines = raw[field["source_id"]][:span["line_1_based"]]
                    language = next(line for line in reversed(lines) if line.startswith("== "))
                    self.assertEqual(language, "== {{langue|fr}} ==")
                    source_pos = next(line for line in reversed(lines) if line.startswith("=== "))
                    pos = {"NOM": "nom", "ADJ": "adjectif", "VER": "verbe", "ADV": "adverbe"}
                    allowed = ("onomatopée", "interjection") if group["lexique_cgram"] == "ONO" else (pos[group["lexique_cgram"]],)
                    self.assertTrue(any("{{S|" + p + "|fr" in source_pos for p in allowed))
                    proposal = proposals[(sense["sense_id"], name)]
                    self.assertEqual(proposal["exact_source_span"], span)
                    self.assertEqual(proposal["source_id"], field["source_id"])
                    self.assertTrue(proposal["author_rights_basis"])
                    self.assertTrue(proposal["selected_boundary"])
                    self.assertEqual(proposal["independent_review"], "passed_at_selected_text_boundary")
                    self.assertEqual(proposal["reviewer"], review["reviewer"])
                    self.assertEqual(proposal["accepted_text"], field["text"])
                    self.assertTrue(proposal["selected_text_rights_reason"])
                    self.assertEqual(field["content_license"], "CC-BY-SA-4.0")
                    r = reviews[(sense["sense_id"], name)]
                    self.assertEqual((r["exact_source"]["source_id"], r["exact_source"]["revid"],
                                      r["exact_source"]["line_1_based"], r["exact_source"]["column_1_based"],
                                      r["exact_source"]["reviewed_excerpt"]),
                                     (field["source_id"], sources[field["source_id"]]["revid"],
                                      span["line_1_based"], span["column_1_based"], span["raw_source_excerpt"]))
                    self.assertEqual(r["proposed_text"], originals[(sense["sense_id"], name)]["text"])
                    self.assertEqual(r["accepted_text"], field["text"])
                    self.assertEqual(bindings[(sense["sense_id"], name)]["accepted_text"], field["text"])
                    self.assertEqual(r["accepted_text_quality_status"], "passed")
                    self.assertEqual(r["rights_eligibility_status"], "passed_at_selected_text_boundary")
                    self.assertEqual(r["reviewer"], review["reviewer"])
                    self.assertTrue(r["review_reason"])
                    self.assertTrue(r["selected_text_rights_reason"])
                    self.assertEqual(field["independent_review"], "passed")
                    self.assertEqual(field["rights_eligibility_status"], "passed_at_selected_text_boundary")
        self.assertEqual(len(review["usage"]), 50)
        for usage in review["usage"]:
            self.assertEqual(usage["accepted_usage_quality_status"], "passed")
            self.assertEqual(usage["everyday_sense_status"], "passed")
            self.assertTrue(usage["review_reason"])
            self.assertEqual(usage["accepted_usage"], next(g["senses"][0]["use"] for g in read("groups.json")
                                                        if g["frozen_row"] == usage["frozen_row"]))
        for record in review["fields"] + review["usage"] + review["spelling_relations"]:
            spans = record.get("supporting_sources", record.get("proposed_evidence", []))
            for binding in spans:
                line = raw[binding["source_id"]][binding["line_1_based"] - 1]
                start = binding["column_1_based"] - 1
                self.assertEqual(binding["reviewed_excerpt"], line[start:start + len(binding["reviewed_excerpt"])])
        for binding in read("source-excerpts.json")["supporting_evidence"]:
            line = raw[binding["source_id"]][binding["line_1_based"] - 1]
            start = binding["column_1_based"] - 1
            self.assertEqual(binding["reviewed_excerpt"], line[start:start + len(binding["reviewed_excerpt"])])
        relations = read("source-context.json")["spelling_relations"]
        self.assertEqual(len(relations), 50)
        self.assertTrue(all(r["alias_admission"] == "not_admitted" for r in relations))
        self.assertTrue(all(r["independent_review"] == "passed" for r in relations))


if __name__ == "__main__":
    unittest.main()
