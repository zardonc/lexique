"""Check #119 at the parent-approved resource/admission boundary."""
import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-119"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch119Tests(unittest.TestCase):
    def test_exact_frozen_slice_preserves_accepted_cote_and_reviews_196_new_fields(self):
        admissions = []
        final_keys = {
            "status", "resource_admission", "final_output_validation", "native_checks",
            "conversion_and_packaging", "packaging_obligations_status",
            "canonical_newly_admitted_groups", "canonical_newly_admitted_fields",
            "missing_admitted_fields", "count_boundary", "newly_admitted_fields",
            "admitted_groups", "admitted_fields", "canonical_pending_fields",
            "pending_independent_rights_fields", "author_rights_assessment", "pending_parent_gates",
        }
        for issue in range(110, 120):
            with self.subTest(issue=issue):
                evidence = ROOT / f"docs/evidence/issue-{issue}"
                admission = json.loads((evidence / "admission.json").read_text(encoding="utf-8"))
                admissions.append(admission)
                new_groups, new_fields = (49, 196) if issue == 119 else (50, 200)
                self.assertEqual(admission["status"], "admitted_core_119_v1_windows_x64_development_resource")
                self.assertEqual(admission["resource_admission"],
                                 "passed_combined_conversion_rights_notices_and_native_gates")
                self.assertEqual(admission["conversion_and_packaging"],
                                 "passed_core_119_v1_and_verified_portable_package")
                self.assertEqual(admission["native_checks"],
                                 "passed_windows_11_x64_exact_batch_lookup_trusted_save_reopen_and_upgrade")
                self.assertEqual(admission["final_output_validation"],
                                 "passed_reproducible_conversion_fixed_denominator_and_prior_payload_preservation")
                self.assertEqual(admission["packaging_obligations_status"],
                                 "passed_actual_combined_package_fixed_sources_credits_modifications_and_licences")
                self.assertEqual((admission["canonical_newly_admitted_groups"],
                                  admission["canonical_newly_admitted_fields"]), (new_groups, new_fields))
                self.assertEqual((admission["completed_frozen_batch_groups"],
                                  admission["completed_frozen_batch_fields"]), (50, 200))
                for key in ("missing_admitted_fields", "canonical_pending_fields",
                            "pending_independent_rights_fields"):
                    if key in admission:
                        self.assertEqual(admission[key], 0)
                for key, expected in (("newly_admitted_fields", new_fields),
                                      ("admitted_groups", new_groups), ("admitted_fields", new_fields)):
                    if key in admission:
                        self.assertEqual(admission[key], expected)
                self.assertNotIn("pending", admission.get("count_boundary", ""))
                if "author_rights_assessment" in admission:
                    self.assertEqual(admission["author_rights_assessment"],
                                     "completed_proposal_independently_reviewed_at_selected_span_boundary")
                self.assertEqual(admission["pending_parent_gates"], [
                    "Whole-machine offline execution", "Other-platform native execution",
                    "Learner acceptance", "Formal release readiness",
                ])
                review = json.loads((evidence / "field-review.json").read_text(encoding="utf-8"))
                self.assertIs(review["independent_acceptance"], True)
                self.assertTrue(review["independence"])
                self.assertTrue(admission["independent_reviewer"])
                self.assertEqual(review["reviewer"], admission["independent_reviewer"])
                self.assertEqual(len(review["fields"]), new_fields)
                execution = admission["parent_execution_evidence"]
                self.assertEqual(execution["build_source_sha"], "00ea06228926e1b9a4e8644a267bce46316f1a39")
                snapshot = Path(execution["root"]) / execution["original_admission_snapshot"]
                # Owning worktrees retain original snapshots; detached/CI checkouts retain review records.
                if snapshot.exists():
                    history = json.loads(snapshot.read_text(encoding="utf-8"))
                    original_status = "passed_independent_selected_content_and_rights_only"
                    if issue in (114, 116, 117, 119):
                        original_status = "pending_parent_conversion_and_packaged_notices"
                    elif issue == 118:
                        original_status = "pending_parent_admission_conversion"
                    self.assertEqual(history["resource_admission"], original_status)
                    self.assertEqual(history["independent_quality"], "passed")
                    self.assertEqual(history["selected_span_rights"], "passed_at_selected_text_boundary")
                    for key, value in history.items():
                        if key not in final_keys:
                            self.assertEqual(admission[key], value, key)
        self.assertEqual(sum(item["canonical_newly_admitted_groups"] for item in admissions), 499)
        self.assertEqual(sum(item["canonical_newly_admitted_fields"] for item in admissions), 1996)
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            scope = list(csv.DictReader(handle))
        groups = read("groups.json")
        self.assertEqual([group["frozen_row"] for group in groups], list(range(1901, 1951)))
        for group in groups:
            frozen = scope[group["frozen_row"] - 1]
            self.assertEqual((group["headword"], group["lexique_cgram"]),
                             (frozen["word"], frozen["lexique_cgram"]))
            for frequency in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                self.assertEqual(group[frequency], frozen[frequency])
            self.assertEqual(len(group["senses"]), 1)
            self.assertEqual(set(group["senses"][0]["fields"]), set(LANGUAGES))
        curated = json.loads((ROOT / "resources/core-pilot-79/curated-inputs.json").read_text(encoding="utf-8"))
        accepted = next(group for group in curated["groups"] if group["frozen_row"] == 1911)
        self.assertEqual(groups[10], accepted)
        reuse = read("reuse.json")
        self.assertEqual([group["frozen_row"] for group in reuse["groups"]], [1911])
        self.assertEqual(reuse["groups"][0]["sense_id"], "cote-noun-rib")
        self.assertEqual(reuse["groups"][0]["field_identifiers"],
                         {name: {"sense_id": "cote-noun-rib", "field": name, "source_id": "fr_cote"}
                          for name in LANGUAGES})
        admission = read("admission.json")
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (49, 196))
        self.assertEqual((admission["reused_groups"], admission["reused_fields"]), (1, 4))
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual(admission["pending_independent_review_rows"], [])
        self.assertEqual(admission["independently_accepted_new_fields"], 196)
        self.assertEqual(admission["independently_accepted_new_groups"], 49)
        self.assertTrue(admission["independent_reviewer"])
        prior = json.loads((ROOT / "resources/core-pilot-79/field-review.json").read_text(encoding="utf-8"))
        for name, field in accepted["senses"][0]["fields"].items():
            previous = next(item for item in prior["fields"]
                            if item["sense_id"] == "cote-noun-rib" and item["field"] == name)
            self.assertEqual(previous["accepted_text"], field["text"])
            self.assertEqual(previous["exact_source"]["source_id"], "fr_cote")
            self.assertEqual(previous["accepted_text_quality_status"], "passed")
            self.assertEqual(previous["rights_eligibility_status"], "passed_at_selected_text_boundary")

    def test_new_field_sources_rights_and_quality_handoff_are_complete_without_reused_copies(self):
        groups = [group for group in read("groups.json") if group["frozen_row"] != 1911]
        sources = {source["id"]: source for source in read("sources.json")["sources"]}
        contexts = {source["id"]: source for source in read("source-context.json")["current_content"]}
        captures = {source["source_id"]: source for source in read("source-captures.json")["sources"]}
        bindings = {(item["sense_id"], item["field"]): item for item in read("source-excerpts.json")["fields"]}
        review = read("field-review.json")
        reviews = {(item["sense_id"], item["field"]): item for item in review["fields"]}
        rights = read("rights.json")
        boundaries = {(item["sense_id"], item["field"]): item for item in rights["fields"]}
        self.assertEqual(len(bindings), 196)
        self.assertEqual(set(bindings), set(reviews))
        self.assertEqual(set(bindings), set(boundaries))
        self.assertTrue(review["independent_acceptance"])
        self.assertTrue(review["reviewer"])
        self.assertEqual(rights["independent_rights_fields_accepted"], 196)
        self.assertEqual(rights["reviewer"], review["reviewer"])
        self.assertEqual(rights["independent_review"], "passed_at_selected_text_boundary")
        self.assertEqual({item["source_id"] for item in rights["sources"]}, set(sources))
        self.assertEqual((EVIDENCE / "COPYING").read_text(encoding="utf-8").rstrip(),
                         (ROOT / rights["complete_legal_text"]).read_text(encoding="utf-8").rstrip())
        self.assertTrue(rights["authority_captures"])
        self.assertTrue(rights["redistribution_obligations"])
        self.assertTrue(read("source-captures.json")["contributor_registry"]["pagination_complete"])
        self.assertEqual(len(review["usage"]), 49)
        self.assertEqual(len(review["spelling_relations"]), 49)
        for group in groups:
            sense = group["senses"][0]
            with self.subTest(row=group["frozen_row"]):
                self.assertEqual(sense["independent_review"], "passed")
                self.assertEqual(sense["resource_admission"], "pending_parent_conversion_and_packaged_notices")
                self.assertIsNone(sense["cefr"])
                self.assertTrue(sense["use"].strip())
                for name, field in sense["fields"].items():
                    key = sense["sense_id"], name
                    binding, proposed, boundary = bindings[key], reviews[key], boundaries[key]
                    source = sources[field["source_id"]]
                    context = contexts[source["id"]]
                    self.assertTrue(field["text"].strip())
                    self.assertEqual(field["language"], LANGUAGES[name])
                    self.assertEqual(field["content_license"], "CC-BY-SA-4.0")
                    self.assertTrue(field["modification"])
                    self.assertEqual(field["independent_review"], "passed")
                    self.assertEqual(proposed["accepted_text"], field["text"])
                    self.assertTrue(proposed["proposed_text"])
                    self.assertTrue(proposed["review_reason"])
                    self.assertTrue(proposed["selected_text_rights_reason"])
                    self.assertEqual((proposed["frozen_row"], proposed["headword"], proposed["pos"]),
                                     (group["frozen_row"], group["headword"], group["lexique_cgram"]))
                    self.assertEqual(proposed["exact_source"],
                                     {key: value for key, value in binding.items() if key not in ("sense_id", "field")})
                    self.assertEqual(proposed["accepted_text_quality_status"], "passed")
                    self.assertEqual(proposed["reviewer"], review["reviewer"])
                    self.assertEqual(binding["source_id"], field["source_id"])
                    self.assertEqual(binding["line_1_based"], field["source_span"]["line_1_based"])
                    self.assertEqual(binding["reviewed_excerpt"], field["source_span"]["raw_source_excerpt"])
                    self.assertEqual(binding["revid"], source["revid"])
                    self.assertEqual(binding["url"], source["fixed_url"])
                    self.assertIn("oldid=" + str(source["revid"]), source["fixed_url"])
                    self.assertEqual(context["current_content_revid"], source["revid"])
                    self.assertTrue(context["current_contributors"])
                    self.assertEqual(context["current_registry_names"], len(context["current_contributors"]))
                    self.assertTrue(context["contributor_capture_parts"])
                    self.assertEqual(boundary["exact_source_span"], field["source_span"])
                    self.assertEqual(boundary["source_id"], field["source_id"])
                    self.assertEqual(boundary["modification"], field["modification"])
                    self.assertTrue(boundary["attribution"])
                    self.assertTrue(boundary["author_rights_basis"])
                    self.assertEqual(boundary["independent_review"], "passed_at_selected_text_boundary")
                    self.assertEqual(boundary["reviewer"], review["reviewer"])
                    if name == "ipa":
                        self.assertEqual(field["text"], "/" + binding["ipa_variant_excerpt"] + "/")
                        column = binding["ipa_variant_column_1_based"] - 1
                        self.assertEqual(binding["reviewed_excerpt"][column:column + len(binding["ipa_variant_excerpt"])],
                                         binding["ipa_variant_excerpt"])
                    raw_path = ROOT / captures[source["id"]]["raw_path"]
                    if raw_path.exists():
                        self.assertEqual(raw_path.read_text(encoding="utf-8").splitlines()[binding["line_1_based"] - 1],
                                         binding["reviewed_excerpt"])
        for item in review["fields"] + review["usage"] + review["spelling_relations"]:
            self.assertNotEqual(item["frozen_row"], 1911)
            self.assertNotEqual(item["sense_id"], "cote-noun-rib")
        for relation in review["spelling_relations"]:
            self.assertEqual(relation["relation_role"], "source_binding_only")
            self.assertEqual(relation["alias_admission"], "not_admitted")
            self.assertTrue(relation["proposed_evidence"])
        for usage in review["usage"]:
            self.assertTrue(usage["construction_and_flexion_evidence"])
            self.assertTrue(usage["author_construction_and_flexion_interpretation"])
            self.assertEqual(usage["reviewer"], review["reviewer"])
            self.assertEqual(usage["accepted_usage_quality_status"], "passed")
            self.assertTrue(usage["review_reason"])
        for relation in review["spelling_relations"]:
            self.assertEqual(relation["reviewer"], review["reviewer"])
            self.assertEqual(relation["accepted_spelling_quality_status"], "passed")
            self.assertTrue(relation["review_reason"])
        relations = {item["frozen_row"]: item for item in review["spelling_relations"]}
        self.assertEqual(relations[1908]["source_headword"], "continu")
        self.assertEqual(relations[1910]["source_headword"], "euh")
        self.assertEqual(relations[1930]["source_headword"], "celui")
        self.assertEqual(relations[1934]["source_headword"], "vieux")
        self.assertEqual(relations[1936]["source_headword"], "le vôtre")
        spelling = rights["heu_spelling_boundary"]
        self.assertEqual(spelling["frozen_row"], 1910)
        self.assertEqual(spelling["exact_source_span"]["raw_source_excerpt"], "# {{alternative spelling of|fr|euh}}")
        self.assertTrue(spelling["fixed_url"].startswith("https://en.wiktionary.org/"))
        self.assertIn(spelling["source_id"], sources)
        self.assertEqual(read("admission.json")["pending_independent_review_fields"], 0)
        self.assertEqual(read("admission.json")["newly_admitted_fields"], 0)

    def test_retained_proposals_and_corrected_constructions_and_original_rights(self):
        originals = {g["frozen_row"]: g for g in read("review-candidates.json")}
        groups = {g["frozen_row"]: g for g in read("groups.json")}
        review = read("field-review.json")
        for item in review["fields"]:
            original = originals[item["frozen_row"]]["senses"][0]["fields"][item["field"]]
            self.assertEqual(item["proposed_text"], original["text"])
            self.assertEqual(groups[item["frozen_row"]]["senses"][0]["fields"][item["field"]]["source_span"],
                             original["source_span"])
        replacements = [item for item in review["fields"] if item["accepted_text"] != item["proposed_text"]]
        self.assertEqual({(item["frozen_row"], item["field"]) for item in replacements},
                         {(1921, "chinese"), (1933, "french_definition"), (1933, "english"),
                          (1933, "chinese"), (1948, "french_definition")})
        for name in ("french_definition", "english", "chinese"):
            self.assertIn("dans le civil", groups[1933]["senses"][0]["fields"][name]["text"].casefold())
        self.assertTrue(groups[1948]["senses"][0]["fields"]["french_definition"]["text"].startswith("Familier"))
        self.assertIn("虔敬", groups[1921]["senses"][0]["fields"]["chinese"]["text"])
        self.assertNotIn("古迹", groups[1921]["senses"][0]["fields"]["chinese"]["text"])
        contexts = read("source-context.json")["current_content"]
        self.assertEqual(read("source-context.json")["spelling_relations"], review["spelling_relations"])
        for context in contexts:
            for layer in context["original_historical_dictionary_layers"]:
                self.assertTrue(layer["conservative_expiry"].startswith("2006-01-01"))
                self.assertTrue(layer["author_proposed_conservative_expiry"].startswith("1966-01-01"))
                self.assertEqual(layer["independent_review"], "passed_at_original_work_boundary")


if __name__ == "__main__":
    unittest.main()
