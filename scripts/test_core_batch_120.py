"""Check #120 at the approved resource/admission boundary."""
import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-120"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
# Exact membership from the self-contained #120 ticket.
MEMBERSHIP = [
    ("passion", "NOM"), ("adulte", "ADJ"), ("adulte", "NOM"),
    ("puissance", "NOM"), ("complet", "ADJ"), ("fortune", "NOM"),
    ("disque", "NOM"), ("hum", "ONO"), ("participer", "VER"),
    ("cible", "NOM"), ("aile", "NOM"), ("brillant", "ADJ"),
    ("lunette", "NOM"), ("rare", "ADJ"), ("canon", "ADJ"),
    ("canon", "NOM"), ("essence", "NOM"), ("décevoir", "VER"),
    ("foutu", "ADJ"), ("paradis", "NOM"), ("réagir", "VER"),
    ("hurler", "VER"), ("aéroport", "NOM"), ("mise", "NOM"),
    ("piège", "NOM"), ("pédé", "NOM"), ("haine", "NOM"),
    ("débile", "ADJ"), ("débile", "NOM"), ("enregistrer", "VER"),
    ("idéal", "ADJ"), ("idéal", "NOM"), ("coupe", "NOM"),
    ("enfance", "NOM"), ("science", "NOM"), ("autorité", "NOM"),
    ("choc", "NOM"), ("ballon", "NOM"), ("chocolat", "NOM"),
    ("joueur", "NOM"), ("pis", "ADV"), ("charge", "NOM"),
    ("parc", "NOM"), ("zéro", "NOM"), ("épée", "NOM"),
    ("limite", "NOM"), ("échange", "NOM"), ("fixer", "VER"),
    ("prière", "NOM"), ("rivière", "NOM"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch120Tests(unittest.TestCase):
    def test_independent_selected_weapon_scope_and_original_rights_are_exact(self):
        group = next(g for g in read("groups.json") if g["frozen_row"] == 1995)
        expected = {
            "french_definition": "Dans ce sens d’arme, épée à longue lame affilée de section triangulaire, munie d’une poignée, utilisée pour attaquer ou se défendre et portée dans un fourreau.",
            "english": "In this selected weapon sense, a sword with a long sharpened blade of triangular cross-section and a handle, used to attack or defend and carried in a scabbard.",
            "chinese": "在所选武器义中：剑；带柄、长而锋利且横截面呈三角形的刀刃武器，用于进攻或防御，佩带时置于剑鞘中。",
        }
        for name, text in expected.items():
            self.assertEqual(group["senses"][0]["fields"][name]["text"], text)
        review = read("field-review.json")
        self.assertEqual(review["kind"], "separate_independent_field_review")
        self.assertTrue(review["independent_acceptance"])
        self.assertEqual(len(review["fields"]), 200)
        self.assertEqual(read("admission.json")["missing_admitted_fields"], 0)
        for source in read("sources.json")["sources"]:
            self.assertEqual(source["selected_text_review"], "passed_at_selected_text_boundary")
            for layer in source["original_historical_dictionary_layers"]:
                if layer["original_authors"] == "Académie française":
                    self.assertEqual(layer["conservative_expiry"], "2006-01-01, using final publication in 1935")

    def test_exact_ticket_groups_have_complete_four_field_candidate_senses(self):
        groups = read("groups.json")
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        self.assertEqual([g["frozen_row"] for g in groups], list(range(1951, 2001)))
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
        # Parent assembly may subsequently include these exact reviewed records.
        for group in groups:
            if group["frozen_row"] in canonical:
                self.assertEqual(canonical[group["frozen_row"]], group)
        self.assertEqual((reuse["reused_groups"], reuse["reused_fields"], reuse["groups"]), (0, 0, []))
        admission = read("admission.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual(admission["independently_accepted_new_fields"], 200)
        self.assertEqual(admission["independently_accepted_new_groups"], 50)
        self.assertEqual(admission["pending_independent_review_rows"], [])
        self.assertEqual(admission["pending_independent_review_fields"], 0)
        self.assertEqual(admission["pending_independent_rights_fields"], 0)
        self.assertEqual(admission["unresolved_field_blockers"], [])
        self.assertTrue(admission["independent_reviewer"])
        authored = read("authored-admission.json")
        self.assertEqual(authored["independently_accepted_new_fields"], 0)
        self.assertEqual(authored["pending_independent_review_fields"], 200)
        self.assertIsNone(authored["independent_reviewer"])

    def test_all_reviewed_fields_bind_exact_proposals_sources_rights_and_use(self):
        rights = read("rights.json")
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        contexts = {s["id"]: s for s in read("source-context.json")["current_content"]}
        bindings = {(f["sense_id"], f["field"]): f for f in read("source-excerpts.json")["fields"]}
        boundaries = {(f["sense_id"], f["field"]): f for f in rights["fields"]}
        proposals = {(g["frozen_row"], name): field
                     for g in read("review-candidates.json")
                     for name, field in g["senses"][0]["fields"].items()}
        review = read("field-review.json")
        reviewed = {(f["sense_id"], f["field"]): f for f in review["fields"]}
        self.assertEqual(len(proposals), 200)
        self.assertEqual(len(reviewed), 200)
        self.assertEqual(len(sources), 46)
        self.assertEqual(set(reviewed), set(bindings))
        self.assertEqual(set(reviewed), set(boundaries))
        self.assertTrue(review["independent_acceptance"])
        self.assertTrue(review["independence"])
        self.assertEqual(rights["independent_rights_fields_accepted"], 200)
        self.assertEqual(rights["independent_review"], "passed_at_selected_text_boundary")
        self.assertEqual(rights["reviewer"], review["reviewer"])
        self.assertEqual(review["reviewer"], read("admission.json")["independent_reviewer"])
        self.assertTrue(rights["authority_captures"])
        changes = []
        for group in read("groups.json"):
            sense = group["senses"][0]
            self.assertEqual(sense["independent_review"], "passed_after_exact_accepted_replacements")
            for name, field in sense["fields"].items():
                with self.subTest(row=group["frozen_row"], field=name):
                    key = (sense["sense_id"], name)
                    row, binding, boundary = reviewed[key], bindings[key], boundaries[key]
                    original = proposals[(group["frozen_row"], name)]
                    source = sources[field["source_id"]]
                    context = contexts[field["source_id"]]
                    span = field["source_span"]
                    self.assertEqual(row["accepted_text"], field["text"])
                    self.assertEqual(row["proposed_text"], original["text"])
                    self.assertEqual(original["independent_review"], "pending")
                    self.assertEqual(row["accepted_text_quality_status"], "passed")
                    self.assertEqual(row["rights_eligibility_status"], "passed_at_selected_text_boundary")
                    self.assertEqual(row["reviewer"], review["reviewer"])
                    self.assertTrue(row["independence"])
                    self.assertTrue(row["review_reason"])
                    self.assertTrue(row["selected_text_rights_reason"])
                    self.assertEqual((row["frozen_row"], row["headword"], row["pos"]),
                                     (group["frozen_row"], group["headword"], group["lexique_cgram"]))
                    self.assertEqual(row["exact_source"], {k:v for k,v in binding.items() if k not in ("sense_id", "field")})
                    self.assertEqual(binding["revid"], source["revid"])
                    self.assertEqual(binding["url"], source["fixed_url"])
                    self.assertEqual(binding["history_url"], source["history_url"])
                    self.assertEqual(binding["source_id"], field["source_id"])
                    self.assertEqual(binding["reviewed_excerpt"], span["raw_source_excerpt"])
                    self.assertEqual(binding["line_1_based"], span["line_1_based"])
                    self.assertEqual(boundary["exact_source_span"], span)
                    self.assertEqual(boundary["source_id"], field["source_id"])
                    self.assertEqual(boundary["modification"], field["modification"])
                    self.assertEqual(boundary["independent_review"], "passed_at_selected_text_boundary")
                    self.assertEqual(boundary["reviewer"], review["reviewer"])
                    self.assertTrue(boundary["attribution"])
                    self.assertTrue(boundary["author_rights_basis"])
                    self.assertTrue(boundary["selected_boundary"])
                    self.assertEqual(source["selected_text_review"], "passed_at_selected_text_boundary")
                    self.assertEqual(source["reviewer"], review["reviewer"])
                    self.assertEqual(context["selected_text_rights_status"], "passed_at_selected_text_boundary")
                    self.assertTrue(context["current_contributors"])
                    self.assertEqual(context["current_registry_names"], len(context["current_contributors"]))
                    self.assertEqual(context["current_content_revid"], source["revid"])
                    self.assertEqual(field["content_license"], "CC-BY-SA-4.0")
                    self.assertEqual(field["independent_review"], "passed")
                    if field["text"] != original["text"]:
                        changes.append((group["frozen_row"], name))
                    if name == "ipa":
                        self.assertEqual(field["text"], "/" + binding["ipa_variant_excerpt"] + "/")
                        column = binding["ipa_variant_column_1_based"] - 1
                        self.assertEqual(binding["reviewed_excerpt"][column:column+len(binding["ipa_variant_excerpt"])], binding["ipa_variant_excerpt"])
        self.assertEqual(changes, [(1995, "french_definition"), (1995, "english"), (1995, "chinese")])
        self.assertEqual(len(review["usage"]), 50)
        senses = {g["senses"][0]["sense_id"]: g["senses"][0] for g in read("groups.json")}
        self.assertEqual({u["sense_id"] for u in review["usage"]}, set(senses))
        for usage in review["usage"]:
            self.assertEqual(usage["accepted_usage"], senses[usage["sense_id"]]["use"])
            self.assertEqual(usage["accepted_usage_quality_status"], "passed")
            self.assertEqual(usage["everyday_sense_status"], "passed")
            self.assertTrue(usage["construction_and_flexion_evidence"])
            self.assertTrue(usage["review_reason"])
            self.assertEqual(usage["reviewer"], review["reviewer"])
        relations = review["spelling_relations"]
        self.assertEqual(read("source-context.json")["spelling_relations"], relations)
        self.assertEqual(len(relations), 50)
        self.assertTrue(all(r["alias_admission"] == "not_admitted" for r in relations))
        self.assertTrue(all(r["independent_review"] == "passed" for r in relations))
        self.assertEqual(read("authored-rights.json")["independent_rights_fields_accepted"], 0)
        self.assertFalse(read("authored-field-review.json")["independent_acceptance"])
        authored_review = {(r["sense_id"], r["field"]): r for r in read("authored-field-review.json")["fields"]}
        for key, field in reviewed.items():
            self.assertEqual(field["author_rights_proposal"], authored_review[key]["author_rights_proposal"])
        for frame in rights["construction_sources"]:
            self.assertEqual(frame["independent_review"], "passed_at_selected_text_boundary")
            self.assertEqual(frame["reviewer"], review["reviewer"])
        for source in rights["sources"]:
            self.assertEqual(source["independent_review"], "passed_at_selected_text_boundary")
        for source in contexts.values():
            self.assertEqual(source["selected_frame_review"], "passed_exact_french_language_pos_headword_and_pronunciation_spans")
            for layer in source["original_historical_dictionary_layers"]:
                self.assertEqual(layer["independent_review"], "passed_at_original_work_boundary")
                if layer["original_authors"] == "Académie française":
                    self.assertEqual(layer["conservative_expiry"], "2006-01-01, using final publication in 1935")


if __name__ == "__main__":
    unittest.main()
