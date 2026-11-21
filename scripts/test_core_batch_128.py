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


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch128Tests(unittest.TestCase):
    def test_exact_ticket_groups_have_complete_four_field_candidate_senses(self):
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
        self.assertFalse({g["frozen_row"] for g in groups} & {g["frozen_row"] for g in prior["groups"]})
        reuse = read("reuse.json")
        self.assertEqual((reuse["reused_groups"], reuse["reused_fields"], reuse["groups"]), (0, 0, []))
        admission = read("admission.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual(admission["independently_accepted_new_fields"], 0)
        self.assertEqual(admission["pending_independent_review_rows"], list(range(2351, 2401)))
        self.assertIsNone(admission["independent_reviewer"])

    def test_fixed_source_rights_and_constructions_are_reviewable_without_self_acceptance(self):
        rights = read("rights.json")
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        captures = read("source-captures.json")
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
                self.assertEqual(layer["independent_review"], "pending")
                self.assertTrue(layer["legal_basis_url"])
                if layer["original_authors"] == "Académie française":
                    self.assertTrue(layer["conservative_expiry"].startswith("2006-01-01"))
            raw_path = ROOT / capture["raw_path"]
            # Source captures are retained in the author archive, outside tracked candidate data.
            if raw_path.exists():
                data = json.loads((ROOT / capture["api_path"]).read_text(encoding="utf-8-sig"))
                page = next(p for p in data["query"]["pages"] if p["pageid"] == capture["pageid"])
                revision = next(r for r in page["revisions"] if r["revid"] == capture["revid"])
                self.assertEqual(revision["timestamp"], source["timestamp"])
                self.assertEqual(raw_path.read_bytes(), revision["slots"]["main"]["content"].encode("utf-8"))
                raw[source["id"]] = raw_path.read_text(encoding="utf-8").splitlines()
        self.assertTrue(captures["contributor_registry"]["pagination_complete"])
        self.assertFalse(captures["contributor_registry"]["selected_sentence_authorship_proved"])
        parts = [ROOT / p for p in captures["contributor_registry"]["parts"]]
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
        self.assertFalse(reviews["independent_acceptance"])
        self.assertIsNone(reviews["reviewer"])
        self.assertEqual(rights["independent_rights_fields_accepted"], 0)
        self.assertEqual((EVIDENCE / "COPYING").read_text(encoding="utf-8").rstrip(),
                         (ROOT / rights["complete_legal_text"]).read_text(encoding="utf-8").rstrip())
        self.assertTrue(rights["authority_captures"])
        self.assertTrue(rights["redistribution_obligations"])
        for group in read("groups.json"):
            sense = group["senses"][0]
            self.assertEqual(sense["independent_review"], "pending")
            for name, field in sense["fields"].items():
                with self.subTest(row=group["frozen_row"], field=name):
                    key = sense["sense_id"], name
                    span, binding = field["source_span"], bindings[key]
                    self.assertEqual(binding["reviewed_excerpt"], span["raw_source_excerpt"])
                    self.assertEqual(binding["line_1_based"], span["line_1_based"])
                    self.assertEqual(binding["source_id"], field["source_id"])
                    self.assertEqual(binding["revid"], sources[field["source_id"]]["revid"])
                    self.assertEqual(fields[key]["proposed_text"], field["text"])
                    self.assertEqual(proposals[key]["exact_source_span"], span)
                    self.assertEqual(proposals[key]["source_id"], field["source_id"])
                    for required in ("author_rights_basis", "selected_boundary", "attribution", "modification"):
                        self.assertTrue(proposals[key][required])
                    self.assertEqual(proposals[key]["independent_review"], "pending")
                    self.assertIsNone(proposals[key]["reviewer"])
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
            self.assertEqual(item["accepted_usage_quality_status"], "pending")
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
        self.assertEqual(read("admission.json")["pending_independent_review_fields"], 200)
        self.assertEqual(read("admission.json")["newly_admitted_fields"], 0)


if __name__ == "__main__":
    unittest.main()
