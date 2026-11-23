"""Check #146 at the approved resource/admission boundary."""
import json
import csv
from pathlib import Path
import re
import sys
from urllib.parse import parse_qs, urlparse
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-146"
ROWS = set(range(3251, 3301))
ADMITTED = "--admitted" in sys.argv
if ADMITTED:
    sys.argv.remove("--admitted")
MEMBERSHIP = [
    ("faucher", "VER"), ("foirer", "VER"), ("laboratoire", "NOM"),
    ("nuire", "VER"), ("la plupart", "PRO:ind"), ("faiblesse", "NOM"),
    ("écart", "NOM"), ("imiter", "VER"), ("effrayant", "ADJ"),
    ("extrêmement", "ADV"), ("orchestre", "NOM"), ("munition", "NOM"),
    ("pratiquement", "ADV"), ("ceux-là", "PRO:dem"), ("tiroir", "NOM"),
    ("agiter", "VER"), ("dignité", "NOM"), ("larguer", "VER"),
    ("préserver", "VER"), ("rebelle", "NOM"), ("machin", "NOM"),
    ("attentat", "NOM"), ("complexe", "ADJ"), ("accent", "NOM"),
    ("cervelle", "NOM"), ("combinaison", "NOM"), ("divers", "ADJ"),
    ("divers", "ADJ:ind"), ("crainte", "NOM"), ("efficace", "ADJ"),
    ("coca", "NOM"), ("exercer", "VER"), ("saute", "NOM"),
    ("déterminer", "VER"), ("bander", "VER"), ("aura", "NOM"),
    ("mêmes", "PRO:ind"), ("patate", "NOM"), ("bouclier", "NOM"),
    ("change", "NOM"), ("délivrer", "VER"), ("patrouille", "NOM"),
    ("usage", "NOM"), ("auberge", "NOM"), ("profil", "NOM"),
    ("consoler", "VER"), ("figurer", "VER"), ("réseau", "NOM"),
    ("chariot", "NOM"), ("formation", "NOM"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


def proposal(name):
    original = EVIDENCE / "authored/author-current" / name
    return json.loads(original.read_text(encoding="utf-8")) if original.exists() else read(name)


def retained(name):
    local = ROOT / name
    if local.exists():
        return local
    archive = Path(read("source-captures.json")["capture_root_fallback"])
    return archive.joinpath(*Path(name).parts[2:])


class Batch146AdmissionTests(unittest.TestCase):
    @unittest.skipUnless(ADMITTED, "Canonical admission is owned by integration; use --admitted")
    def test_all_frozen_groups_have_admitted_four_field_content(self):
        resource = json.loads((ROOT / "resources/core-pilot-79/curated-inputs.json").read_text(encoding="utf-8"))
        groups = [group for group in resource["groups"] if group["frozen_row"] in ROWS]
        self.assertEqual({group["frozen_row"] for group in groups}, ROWS,
                         "Ticket #146 lacks admitted content at the resource boundary")
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        for group in groups:
            self.assertTrue(group["senses"])
            for sense in group["senses"]:
                self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                for field in sense["fields"].values():
                    self.assertTrue(field["text"].strip())
                    self.assertEqual(field["independent_review"], "passed")

    def test_author_handoff_has_all_fifty_exact_groups(self):
        groups = proposal("groups.json")
        self.assertEqual([group["frozen_row"] for group in groups], list(range(3251, 3301)))
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(encoding="utf-8-sig") as handle:
            scope = list(csv.DictReader(handle))
        self.assertEqual(len(scope), 4087)
        for group in groups:
            row = scope[group["frozen_row"] - 1]
            self.assertEqual((group["headword"], group["lexique_cgram"],
                              group["subtitle_lemma_per_million"], group["subtitle_pos_per_million"]), tuple(row.values()))
            self.assertEqual(len(group["senses"]), 1)
            sense = group["senses"][0]
            self.assertEqual(sense["independent_review"], "pending")
            self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
            for field in sense["fields"].values():
                self.assertTrue(field["text"].strip())
                self.assertEqual(field["independent_review"], "pending")

    def test_rights_and_admission_remain_complete_author_proposals(self):
        rights = proposal("rights.json")
        review = proposal("field-review.json")
        self.assertEqual(len(rights["fields"]), 200)
        self.assertEqual(len(review["fields"]), 200)
        self.assertFalse(review["independent_acceptance"])
        self.assertIsNone(review["reviewer"])
        fields = {(s["sense_id"], name): field for g in proposal("groups.json")
                  for s in g["senses"] for name, field in s["fields"].items()}
        self.assertEqual({(r["sense_id"], r["field"]) for r in rights["fields"]}, set(fields))
        for record in rights["fields"]:
            field = fields[record["sense_id"], record["field"]]
            self.assertEqual(record["source_id"], field["source_id"])
            self.assertEqual(record["exact_source_span"], field["source_span"])
            self.assertTrue(record["attribution"])
            self.assertTrue(record["modification"])
            self.assertEqual(record["independent_review"], "pending")
            self.assertIsNone(record["reviewer"])
        admission = proposal("admission.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual(admission["independently_accepted_new_fields"], 0)
        self.assertEqual(admission["missing_authored_fields"], 0)
        self.assertEqual(admission["missing_admitted_fields"], 200)
        self.assertEqual(admission["pending_independent_review_fields"], 200)
        self.assertEqual(admission["pending_independent_rights_fields"], 200)
        self.assertEqual(admission["annotation_exception_groups"], 0)
        self.assertEqual(admission["specialist_exception_groups"], 0)
        self.assertEqual(admission["author_unresolved_content_rows"], [])
        self.assertEqual((EVIDENCE / "COPYING").read_bytes().rstrip(),
                         (ROOT / "resources/core-pilot-79/COPYING").read_bytes().rstrip())

    def test_fixed_api_bytes_and_all_selected_language_pos_ipa_spans(self):
        captures = read("source-captures.json")
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        raw_lines = {}
        self.assertEqual(len(captures["sources"]), 54)
        for capture in captures["sources"]:
            api = json.loads(retained(capture["api_path"]).read_text(encoding="utf-8"))
            page = next(p for p in api["query"]["pages"] if p["pageid"] == capture["pageid"])
            rev = next(r for r in page["revisions"] if r["revid"] == capture["revid"])
            raw = retained(capture["raw_path"]).read_bytes()
            self.assertEqual(raw, rev["slots"]["main"]["content"].encode("utf-8"))
            self.assertEqual(rev["timestamp"], capture["timestamp"])
            self.assertEqual(page["title"], capture["title"])
            raw_lines[capture["source_id"]] = raw.decode("utf-8").splitlines()
            self.assertEqual(sources[capture["source_id"]]["revid"], rev["revid"])
        excerpts = read("source-excerpts.json")
        selected = {(r["sense_id"], r["field"]): r for r in excerpts["fields"]}
        self.assertEqual(len(selected), 200)
        for record in excerpts["fields"] + excerpts["supporting_evidence"]:
            line = raw_lines[record["source_id"]][record["line_1_based"] - 1]
            column = record["column_1_based"] - 1
            self.assertEqual(line[column:column + len(record["reviewed_excerpt"])], record["reviewed_excerpt"])
            self.assertEqual(record["revid"], sources[record["source_id"]]["revid"])
            self.assertFalse(record["reviewed_excerpt"].lstrip().startswith("#*"))
        pos_tags = {"NOM": "nom", "VER": "verbe", "ADJ": "adjectif", "ADJ:ind": "adjectif indéfini",
                    "PRO:ind": "pronom indéfini", "PRO:dem": "pronom démonstratif", "ADV": "adverbe"}
        for group in read("groups.json"):
            sense = group["senses"][0]
            for name, field in sense["fields"].items():
                span = selected[sense["sense_id"], name]
                self.assertEqual(field["source_id"], span["source_id"])
                self.assertEqual(field["source_span"]["raw_source_excerpt"], span["reviewed_excerpt"])
                before = raw_lines[field["source_id"]][:span["line_1_based"]]
                language = next(line for line in reversed(before) if line.startswith("== "))
                self.assertEqual(language.replace(" ", ""), "=={{langue|fr}}==")
                frame = next(line for line in reversed(before) if line.startswith("=== {{S|")
                             and not any(tag in line for tag in ("étymologie", "prononciation", "références")))
                expected = "nom" if group["frozen_row"] == 3255 else pos_tags[group["lexique_cgram"]]
                self.assertIn("{{S|" + expected + "|fr", frame)
                if name == "ipa":
                    if group["frozen_row"] == 3255:
                        self.assertEqual(field["text"], "/la ply.paʁ/")
                        self.assertEqual(span["composed_ipa"], field["text"])
                        self.assertEqual([sources[r["source_id"]]["title"] for r in span["ipa_component_sources"]],
                                         ["la", "plupart"])
                        self.assertIn("{{pron|la|fr}}", span["ipa_component_sources"][0]["reviewed_excerpt"])
                        self.assertIn("{{pron|ply.paʁ|fr}}", span["ipa_component_sources"][1]["reviewed_excerpt"])
                    else:
                        self.assertEqual(field["text"].strip("/"), span["ipa_variant_excerpt"])
                    self.assertIn(span["ipa_variant_excerpt"], span["reviewed_excerpt"])
                self.assertEqual(field["content_license"], "CC-BY-SA-4.0")

    def test_complete_contributor_requests_and_coordinator_reuse(self):
        captures = read("source-captures.json")
        parts = captures["contributor_registry"]["parts"]
        names, anonymous = {}, {}
        for prefix in ("contributors-", "contributors-extra-", "contributors-support-"):
            chain = sorted((p for p in parts if re.fullmatch(re.escape(prefix) + r"\d+\.api\.json", Path(p).name)),
                           key=lambda p: int(Path(p).name.removeprefix(prefix).split(".")[0]))
            self.assertTrue(chain)
            for index, path in enumerate(chain):
                data = json.loads(retained(path).read_text(encoding="utf-8"))
                for page in data["query"]["pages"]:
                    names.setdefault(page["title"], set()).update(p["name"] for p in page.get("contributors", []))
                    if "anoncontributors" in page:
                        anonymous[page["title"]] = page["anoncontributors"]
                if index == len(chain) - 1:
                    self.assertFalse(data.get("continue"))
                else:
                    query = parse_qs(urlparse(retained(chain[index + 1]).with_suffix(".request.txt").read_text(encoding="utf-8")).query)
                    for key, value in data["continue"].items():
                        self.assertEqual(query[key], [str(value)])
        self.assertEqual(set(names), {s["title"] for s in captures["sources"]})
        self.assertFalse(captures["contributor_registry"]["selected_sentence_authorship_proved"])
        for context in read("source-context.json")["current_content"]:
            self.assertEqual(context["current_contributors"], sorted(names[context["title"]]))
            self.assertEqual(context["anonymous_contributors_count"], anonymous.get(context["title"], 0))
        archive = Path(captures["capture_root_fallback"])
        cache = Path(captures["capture_root"])
        original_path = (cache if cache.exists() else archive) / "source-index.original.json"
        original = json.loads(original_path.read_text(encoding="utf-8"))
        self.assertEqual(len(original), 49)
        copied = {s["title"]: s for s in captures["sources"]}
        for item in original:
            record = copied[item["title"]]
            self.assertEqual((record["revid"], record["timestamp"]), (item["revid"], item["timestamp"]))
            self.assertEqual(record["original_api_path"], item["api_path"])
            self.assertEqual(record["original_raw_path"], item["raw_path"])
            self.assertTrue(retained(record["api_path"]).with_suffix(".request.txt").read_text(encoding="utf-8").startswith("https://fr.wiktionary.org/w/api.php?"))

    def test_exact_subclasses_constructions_and_excluded_external_bodies(self):
        senses = {g["frozen_row"]: g["senses"][0] for g in proposal("groups.json")}
        self.assertNotEqual(senses[3277]["fields"]["french_definition"]["text"], senses[3278]["fields"]["french_definition"]["text"])
        self.assertEqual(senses[3277]["fields"]["french_definition"]["source_span"]["line_1_based"], 10)
        self.assertEqual(senses[3278]["fields"]["french_definition"]["source_span"]["line_1_based"], 82)
        self.assertIn("devant un nom pluriel", senses[3278]["fields"]["french_definition"]["text"])
        self.assertIn("sans complément", senses[3255]["fields"]["french_definition"]["text"])
        self.assertIn("plural", senses[3255]["fields"]["english"]["text"])
        self.assertIn("les", senses[3287]["fields"]["french_definition"]["text"])
        self.assertEqual(senses[3287]["source_headword"], "le même")
        self.assertIn("vulgaire", senses[3252]["fields"]["french_definition"]["text"])
        self.assertIn("généralement au pluriel", senses[3262]["fields"]["french_definition"]["text"])
        self.assertIn("expressions", senses[3283]["fields"]["french_definition"]["text"])
        rights = proposal("rights.json")
        exclusions = {(r["title"], r["declared_source"]): r for r in rights["excluded_source_boundaries"]}
        self.assertIn(("même", "Reverso"), exclusions)
        self.assertIn(("même", "Alexandria"), exclusions)
        excluded = set(exclusions["même", "Reverso"]["lines_1_based"] + exclusions["même", "Alexandria"]["lines_1_based"])
        source_ids = {s["id"]: s["title"] for s in read("sources.json")["sources"]}
        for record in read("source-excerpts.json")["fields"] + read("source-excerpts.json")["supporting_evidence"]:
            if source_ids[record["source_id"]] == "même":
                self.assertNotIn(record["line_1_based"], excluded)
            if source_ids[record["source_id"]] == "écart":
                self.assertNotEqual(record["line_1_based"], 43)
        self.assertEqual(len([s for s in rights["sources"] if s["original_historical_dictionary_layers"]]), 32)
        for source in rights["sources"]:
            for layer in source["original_historical_dictionary_layers"]:
                self.assertIn("2006-01-01", layer["conservative_expiry"])
        self.assertEqual(rights["modern_grammar_research"]["edition"],
                         "Dictionnaire de l’Académie française, ninth edition, article A9P2969")

    def test_current_original_text_proposals_are_source_specific(self):
        rights = proposal("rights.json")
        self.assertFalse(rights["original_daf8_blanket_territorial_claim"])
        selected = [s for s in rights["sources"] if s["original_historical_dictionary_layers"]]
        self.assertEqual(len(selected), 32)
        for source in selected:
            assessment = source["author_selected_original_text_assessment"]
            self.assertEqual(assessment["source_revision"], source["revid"])
            self.assertEqual(assessment["selected_lines_1_based"], sorted({r["line_1_based"] for r in source["selected_locations"]}))
            self.assertTrue(assessment["author_assessment"])
            self.assertIn("No worldwide public-domain inference", assessment["territorial_basis"])
            self.assertEqual(assessment["independent_review"], "pending")
        self.assertEqual(len(proposal("groups.json")), 50)

    def test_plupart_selected_meaning_supports_people_and_things(self):
        group = next(g for g in read("groups.json") if g["frozen_row"] == 3255)
        for name in ("french_definition", "english", "chinese"):
            span = group["senses"][0]["fields"][name]["source_span"]
            self.assertEqual(span["line_1_based"], 9,
                             "The absolute people-only line 23 does not bind the selected antecedent use")
            self.assertEqual(span["raw_source_excerpt"],
                             "La plus grande [[partie]], le plus grand nombre.")

    def test_selected_patrol_support_excludes_historical_purpose_prose(self):
        records = read("source-excerpts.json")["supporting_evidence"]
        patrol = [r for r in records if r["frozen_row"] == 3292 and r["line_1_based"] == 15]
        self.assertTrue(patrol)
        for record in patrol:
            self.assertNotIn("[[soit]]", record["reviewed_excerpt"],
                             "The personnel fact does not require historical enemy/crime purpose prose")
            self.assertTrue(record["reviewed_excerpt"].endswith("d’[[agent]]s de [[police]]"))

    def test_independent_review_matches_actual_accepted_resource_shard(self):
        groups, review, rights = read("groups.json"), read("field-review.json"), read("rights.json")
        self.assertTrue(review["independent_acceptance"])
        self.assertEqual(review["kind"], "separate_independent_field_review")
        self.assertTrue(review["reviewer"])
        self.assertEqual([g["frozen_row"] for g in groups], list(range(3251, 3301)))
        self.assertEqual([(g["headword"],g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        self.assertEqual([(g["subtitle_lemma_per_million"],g["subtitle_pos_per_million"]) for g in groups],
                         [(g["subtitle_lemma_per_million"],g["subtitle_pos_per_million"]) for g in proposal("groups.json")])
        fields = {(s["sense_id"], name): f for g in groups for s in g["senses"]
                  for name, f in s["fields"].items()}
        proposals = {(s["sense_id"], name): f for g in proposal("groups.json")
                     for s in g["senses"] for name, f in s["fields"].items()}
        reviews = {(r["sense_id"], r["field"]): r for r in review["fields"]}
        decisions = {(r["sense_id"], r["field"]): r for r in rights["fields"]}
        self.assertEqual(set(fields), set(reviews))
        self.assertEqual(set(fields), set(decisions))
        self.assertEqual(len(fields), 200)
        for key, f in fields.items():
            r, legal = reviews[key], decisions[key]
            self.assertEqual(f["independent_review"], "passed")
            self.assertEqual(f["rights_eligibility_status"], "passed_at_selected_text_boundary")
            self.assertEqual(r["accepted_text"], f["text"])
            self.assertEqual(r["proposed_text"], proposals[key]["text"])
            self.assertEqual(r["accepted_text_quality_status"], "passed")
            self.assertEqual(r["rights_eligibility_status"], "passed_at_selected_text_boundary")
            self.assertEqual(r["reviewer"], review["reviewer"])
            group = next(g for g in groups if g["senses"][0]["sense_id"] == key[0])
            self.assertEqual((r["frozen_row"],r["headword"],r["pos"]),
                             (group["frozen_row"],group["headword"],group["lexique_cgram"]))
            self.assertTrue(r["quality_reason"])
            self.assertTrue(r["rights_reason"])
            self.assertEqual(r["exact_source"]["source_id"], f["source_id"])
            self.assertEqual(r["exact_source"]["reviewed_excerpt"], f["source_span"]["raw_source_excerpt"])
            self.assertEqual(legal["exact_source_span"], f["source_span"])
            self.assertEqual(legal["independent_review"], "passed_at_selected_text_boundary")
            self.assertEqual(legal["reviewer"], review["reviewer"])
        self.assertEqual(len(review["usage"]), 50)
        for use in review["usage"]:
            self.assertEqual(use["accepted_usage_quality_status"], "passed")
            self.assertEqual(use["everyday_sense_status"], "passed")
            self.assertTrue(use["quality_reason"])
            sense = next(g["senses"][0] for g in groups if g["frozen_row"] == use["frozen_row"])
            self.assertEqual(sense["use"], use["accepted_usage"])
            self.assertEqual(sense["independent_review"], "passed_after_exact_accepted_replacements")
        self.assertEqual(len(review["spelling_relations"]), 50)
        for relation in review["spelling_relations"]:
            self.assertEqual(relation["independent_review"], "passed")
            self.assertEqual(relation["alias_admission"], "not_admitted")
            self.assertTrue(relation["review_reason"])
        source_records = {r["id"]: r for r in read("sources.json")["sources"]}
        context_records = {r["id"]: r for r in read("source-context.json")["current_content"]}
        for legal in rights["sources"]:
            source = source_records[legal["source_id"]]
            verdict = legal["independent_selected_original_text_assessment"]
            self.assertEqual(verdict["source_revision"], source["revid"])
            self.assertEqual(verdict["independent_review"], "passed_at_selected_text_boundary")
            self.assertTrue(verdict["selected_span_assessment"])
            self.assertEqual(legal["selected_locations"], source["selected_locations"])
            self.assertEqual(legal["selected_locations"], context_records[source["id"]]["selected_locations"])
        self.assertEqual(len(source_records), 54)
        self.assertFalse(rights["original_daf8_blanket_territorial_claim"])
        admission = read("admission.json")
        self.assertEqual(admission["independently_accepted_new_fields"], 200)
        self.assertEqual(admission["independently_accepted_new_groups"], 50)
        self.assertEqual(admission["unresolved_field_blockers"], [])
        self.assertEqual(admission["annotation_exception_groups"], 0)
        self.assertEqual(admission["specialist_exception_groups"], 0)
        self.assertEqual(admission["newly_admitted_fields"], 0)


if __name__ == "__main__":
    unittest.main()
