"""Resource-boundary check of #113 authored candidates and fixed captures."""

import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-113"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
REPLACEMENTS = {
    (1615, "chinese"): "鼠；吻部尖、腿短、尾巴长，会啃食谷粒、秸秆等的啮齿动物。",
    (1627, "french_definition"): "Qui représente la moitié d’une chose divisée ou divisible en deux parties égales. Devant le nom, reste invariable et se lie au nom par un trait d’union ; immédiatement après le nom, s’accorde généralement en genre, sans accord en nombre.",
    (1627, "english"): "Half: representing one of two equal parts of a thing. Before the noun it is invariable and joined to the noun by a hyphen; immediately after the noun it generally agrees in gender, but not in number.",
    (1627, "chinese"): "一半的；表示事物分成或可分成的两个相等部分中的一个。在名词前不变形，用连字符与名词连接；紧接名词后通常按性配合，但不按数配合。",
    (1644, "chinese"): "咬；用牙齿夹紧某物，咬入或咬破其表面。",
}


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


def content(value):
    if isinstance(value, list):
        return [content(item) for item in value]
    if isinstance(value, dict):
        return {key: content(item) for key, item in value.items()
                if key not in {"independent_review", "resource_admission", "rights_eligibility_status"}}
    return value


class Batch113Tests(unittest.TestCase):
    def test_exact_frozen_groups_have_four_bound_candidate_fields(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            scope = list(csv.DictReader(handle))
        groups = read("groups.json")
        self.assertEqual([group["frozen_row"] for group in groups], list(range(1601, 1651)))
        admission = read("admission.json")
        reviewed = admission["independent_reviewer"] is not None
        status = "passed" if reviewed else "pending"
        if reviewed:
            expected = read("review-candidates.json")
            for group in expected:
                for name, field in group["senses"][0]["fields"].items():
                    if (group["frozen_row"], name) in REPLACEMENTS:
                        field["text"] = REPLACEMENTS[group["frozen_row"], name]
            self.assertEqual(content(groups), content(expected))
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual(admission["independently_accepted_new_fields"], 200 if reviewed else 0)
        self.assertEqual(admission["resource_admission"],
                         "passed_independent_selected_content_and_rights_only" if reviewed
                         else "pending_separate_independent_review")
        sources = {source["id"]: source for source in read("sources.json")["sources"]}
        contexts = {item["id"]: item for item in read("source-context.json")["current_content"]}
        bindings = {(item["sense_id"], item["field"]): item
                    for item in read("source-excerpts.json")["fields"]}
        reviews = read("field-review.json")
        self.assertEqual(reviews["independent_acceptance"], reviewed)
        self.assertEqual(reviews["reviewer"], admission["independent_reviewer"])
        self.assertEqual(len(reviews["fields"]), 200)
        self.assertEqual(len(bindings), 200)
        proposed = {(item["sense_id"], item["field"]): item for item in reviews["fields"]}
        self.assertEqual(set(proposed), set(bindings))
        self.assertEqual(len(reviews["usage"]), 50)
        self.assertEqual(len(reviews["spelling_relations"]), 50)
        captures = {item["title"]: item for item in read("source-captures.json")["sources"]}
        for group in groups:
            row = group["frozen_row"]
            with self.subTest(row=row):
                frozen = scope[row - 1]
                self.assertEqual((group["headword"], group["lexique_cgram"]),
                                 (frozen["word"], frozen["lexique_cgram"]))
                for frequency in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                    self.assertEqual(group[frequency], frozen[frequency])
                self.assertEqual(len(group["senses"]), 1)
                sense = group["senses"][0]
                self.assertEqual(sense["independent_review"],
                                 "passed_after_exact_accepted_replacements" if reviewed else "pending")
                self.assertIsNone(sense["cefr"])
                self.assertTrue(sense["use"].strip())
                self.assertEqual(set(sense["fields"]), set(LANGUAGES))
                for name, field in sense["fields"].items():
                    binding = bindings[sense["sense_id"], name]
                    review = proposed[sense["sense_id"], name]
                    source = sources[field["source_id"]]
                    context = contexts[source["id"]]
                    self.assertTrue(field["text"].strip())
                    self.assertEqual(review["accepted_text" if reviewed else "proposed_text"], field["text"])
                    self.assertEqual((review["frozen_row"], review["headword"], review["pos"]),
                                     (row, group["headword"], group["lexique_cgram"]))
                    self.assertEqual(review["exact_source"],
                                     {key: value for key, value in binding.items()
                                      if key not in ("sense_id", "field")})
                    self.assertEqual(review["accepted_text_quality_status"], status)
                    self.assertEqual(review["reviewer"], admission["independent_reviewer"])
                    self.assertEqual(field["language"], LANGUAGES[name])
                    self.assertTrue(field["modification"])
                    self.assertEqual(field["independent_review"], status)
                    self.assertEqual(field["content_license"], "CC-BY-SA-4.0")
                    self.assertEqual(field["source_id"], binding["source_id"])
                    self.assertEqual(field["source_span"]["line_1_based"], binding["line_1_based"])
                    self.assertEqual(field["source_span"]["raw_source_excerpt"], binding["reviewed_excerpt"])
                    self.assertEqual(source["revid"], binding["revid"])
                    self.assertEqual(source["fixed_url"], binding["url"])
                    self.assertIn("oldid=" + str(binding["revid"]), binding["url"])
                    self.assertEqual(context["current_content_revid"], source["revid"])
                    self.assertTrue(context["contributor_capture_parts"])
                    self.assertTrue(context["selected_locations"])
                    capture = captures[source["title"]]
                    raw_path = ROOT / capture["raw_path"]
                    # Raw captures are author evidence; detached/CI checkouts retain bindings.
                    if raw_path.exists():
                        lines = raw_path.read_text(encoding="utf-8").splitlines()
                        self.assertEqual(lines[binding["line_1_based"] - 1], binding["reviewed_excerpt"])
        self.assertEqual(read("reuse.json")["reused_groups"], [])

    def test_selected_rights_and_construction_evidence_remains_separate_from_final_gates(self):
        rights = read("rights.json")
        admission = read("admission.json")
        reviewed = admission["independent_reviewer"] is not None
        self.assertEqual(rights["status"], "passed_at_selected_text_boundary" if reviewed
                         else "author_proposal_pending_independent_selected_span_review")
        self.assertEqual(rights["independent_acceptance"], reviewed)
        self.assertEqual(rights["reviewer"], admission["independent_reviewer"])
        self.assertEqual(admission["native_checks"], "integration_owned_not_run")
        self.assertEqual(admission["conversion_and_packaging"], "integration_owned_not_run")
        self.assertEqual(rights["adaptation_license"], "CC-BY-SA-4.0")
        self.assertTrue((EVIDENCE / rights["complete_legal_text"]).resolve().is_file())
        self.assertTrue(rights["redistribution_obligations"])
        self.assertTrue(rights["authority_captures"])
        self.assertEqual({item["title"] for item in rights["unselected_import_boundaries"]},
                         {"objet", "valeur"})
        self.assertTrue(all(item["status"] == "excluded_from_selected_fields"
                            for item in rights["unselected_import_boundaries"]))
        groups = {group["frozen_row"]: group for group in read("groups.json")}
        self.assertEqual(groups[1610]["headword"], "mlle")
        title = groups[1610]["senses"][0]
        self.assertEqual(title["source_headword"], "Mlle")
        self.assertEqual(title["fields"]["ipa"]["text"], "/mad.mwa.zɛl/")
        for language, words in {"french_definition": ("Mlle", "Mlles", "Québec", "adresse"),
                                "english": ("Mlle", "Mlles", "Quebec", "address"),
                                "chinese": ("Mlle", "Mlles", "魁北克", "称呼")}.items():
            for word in words:
                self.assertIn(word, title["fields"][language]["text"])
        self.assertEqual(groups[1633]["senses"][0]["fields"]["ipa"]["text"], "/mɛ/")
        relation = next(item for item in read("field-review.json")["spelling_relations"]
                        if item["frozen_row"] == 1610)
        self.assertEqual(relation["alias_admission"], "not_admitted")
        self.assertEqual(len(relation["corroboration"]), 2)

    def test_exact_independent_acceptance_preserves_candidates_and_corrections(self):
        groups = {g["frozen_row"]: g for g in read("groups.json")}
        for (row, name), expected_text in REPLACEMENTS.items():
            self.assertEqual(groups[row]["senses"][0]["fields"][name]["text"], expected_text)
        review = read("field-review.json")
        self.assertEqual(review["kind"], "separate_independent_field_review")
        self.assertIs(review["independence"], True)
        self.assertEqual(review["reviewed_author_commit"],
                         "e26d8b4c67ffcc53b4dc3ed62dbe9f7189022451")
        candidates = {(g["frozen_row"], name): f["text"]
                      for g in read("review-candidates.json")
                      for name, f in g["senses"][0]["fields"].items()}
        changed = set()
        for field in review["fields"]:
            key = field["frozen_row"], field["field"]
            self.assertEqual(field["proposed_text"], candidates[key])
            self.assertEqual(field["accepted_text"], REPLACEMENTS.get(key, candidates[key]))
            self.assertTrue(field["review_reason"])
            self.assertTrue(field["selected_text_rights_reason"])
            if field["accepted_text"] != field["proposed_text"]:
                changed.add(key)
                self.assertTrue(field["replacement_reason"])
        self.assertEqual(changed, set(REPLACEMENTS))
        self.assertEqual(review["unresolved_fields"], [])
        for relation in review["spelling_relations"]:
            self.assertEqual(relation["independent_review"], "passed_for_source_binding")
            self.assertEqual(relation["alias_admission"], "not_admitted")
        demi = [f for f in review["fields"] if f["frozen_row"] == 1627]
        for field in demi:
            if field["field"] != "ipa":
                self.assertTrue({102, 103}.issubset(
                    {b["line_1_based"] for b in field["supporting_sources"]}))


if __name__ == "__main__":
    unittest.main()
