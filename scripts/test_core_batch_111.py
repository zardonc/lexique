"""Resource-boundary checks of #111 independently accepted content."""

from contextlib import contextmanager
import copy
import csv
import json
from pathlib import Path
import re
import shutil
import tempfile
import unittest
from unittest.mock import patch

import core_pilot

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-111"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


@contextmanager
def isolated_inputs():
    """Exercise the parent-approved validator seam without canonical admission."""
    with tempfile.TemporaryDirectory() as temporary:
        destination = Path(temporary)
        for name in (*core_pilot.MATERIALS, "input-checksums.json"):
            shutil.copyfile(core_pilot.RESOURCE / name, destination / name)
        curated = core_pilot.read_json(destination / "curated-inputs.json")
        canonical_relations = copy.deepcopy(curated["spelling_relations"])
        curated["groups"] = read("groups.json")
        curated["sources"] = read("sources.json")["sources"]
        review = read("field-review.json")
        # Batch relations are bindings; only the existing canonical aliases enter this seam.
        review["spelling_relations"] = canonical_relations
        review["annotation_exceptions"] = []
        (destination / "curated-inputs.json").write_text(
            json.dumps(curated, ensure_ascii=False), encoding="utf-8")
        (destination / "field-review.json").write_text(
            json.dumps(review, ensure_ascii=False), encoding="utf-8")
        with patch.object(core_pilot, "ROWS", tuple(range(1501, 1551))), \
             patch.object(core_pilot, "ANNOTATIONS", ()), \
             patch.object(core_pilot, "ANNOTATION_ROWS", ()):
            yield destination


def validate(destination):
    (destination / "input-checksums.json").write_text(
        json.dumps({name: core_pilot.digest(destination / name)
                    for name in core_pilot.MATERIALS}), encoding="utf-8")
    return core_pilot.load_inputs(destination)


class Batch111Tests(unittest.TestCase):
    def test_selected_rights_keep_attribution_and_parent_packaging_pending(self):
        rights = read("rights.json")
        self.assertEqual(rights["independent_review"], "passed_at_selected_text_boundary_for_200_fields")
        self.assertTrue(rights["independent_acceptance"])
        self.assertEqual(len(rights["fields"]), 200)
        self.assertEqual({(item["sense_id"], item["field"]) for item in rights["fields"]},
                         {(item["sense_id"], item["field"]) for item in read("source-excerpts.json")["fields"]})
        self.assertEqual(rights["actual_combined_packaging"], "parent_owned_not_run")
        self.assertIn("Attribution-ShareAlike 4.0 International",
                      (EVIDENCE / "COPYING").read_text(encoding="utf-8"))
        self.assertTrue((EVIDENCE / "NOTICES.md").is_file())
        sources = {source["id"]: source for source in read("sources.json")["sources"]}
        for field in rights["fields"]:
            with self.subTest(sense=field["sense_id"], field=field["field"]):
                self.assertEqual(field["rights_eligibility_status"], "passed_at_selected_text_boundary")
                self.assertEqual(field["content_license"], "CC-BY-SA-4.0")
                self.assertEqual(field["attribution"], sources[field["source_id"]]["attribution"])
                self.assertTrue(field["contributor_history_url"])
                self.assertTrue(field["modification"])
                self.assertEqual(field["independent_reviewer"], read("admission.json")["independent_reviewer"])
                self.assertTrue(field["selected_text_rights_reason"])
                self.assertTrue(field["independence"])

    def test_exact_frozen_groups_have_four_bound_accepted_fields_and_original_proposals(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            scope = list(csv.DictReader(handle))
        groups = read("groups.json")
        self.assertEqual([group["frozen_row"] for group in groups], list(range(1501, 1551)))
        admission = read("admission.json")
        self.assertIsNotNone(admission["independent_reviewer"])
        status = "passed"
        candidates = {g["frozen_row"]: g for g in read("review-candidates.json")}
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual(admission["independently_accepted_new_fields"], 200)
        self.assertEqual(admission["resource_admission"],
                         "passed_combined_conversion_rights_notices_and_native_gates")
        self.assertEqual(admission["missing_admitted_fields"], 0)
        self.assertEqual(admission["runtime_aliases_added"], [])
        sources = {source["id"]: source for source in read("sources.json")["sources"]}
        contexts = {item["id"]: item for item in read("source-context.json")["current_content"]}
        bindings = {(item["sense_id"], item["field"]): item
                    for item in read("source-excerpts.json")["fields"]}
        reviews = read("field-review.json")
        self.assertTrue(reviews["independent_acceptance"])
        self.assertEqual(reviews["kind"], "separate_independent_field_review")
        self.assertTrue(reviews["independence"])
        self.assertEqual(reviews["unresolved_fields"], [])
        self.assertEqual([item["frozen_row"] for item in reviews["reviewed_groups"]], list(range(1501, 1551)))
        self.assertEqual(reviews["reviewer"], admission["independent_reviewer"])
        self.assertEqual(len(reviews["fields"]), 200)
        self.assertEqual(len(bindings), 200)
        proposed = {(item["sense_id"], item["field"]): item for item in reviews["fields"]}
        self.assertEqual(set(proposed), set(bindings))
        self.assertEqual(len(reviews["usage"]), 50)
        self.assertEqual(len(reviews["spelling_relations"]), 50)
        for relation in reviews["spelling_relations"]:
            self.assertEqual(relation["alias_admission"], "not_admitted")
            self.assertEqual(relation["independent_review"], "passed")
            self.assertTrue(relation["review_reason"])
        usage = {item["sense_id"]: item for item in reviews["usage"]}
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
                                 "passed_after_exact_accepted_replacements")
                self.assertEqual(usage[sense["sense_id"]]["accepted_usage"], sense["use"])
                self.assertEqual(usage[sense["sense_id"]]["everyday_sense_status"], "passed")
                self.assertTrue(usage[sense["sense_id"]]["review_reason"])
                self.assertIsNone(sense["cefr"])
                self.assertTrue(sense["use"].strip())
                self.assertEqual(set(sense["fields"]), set(LANGUAGES))
                for name, field in sense["fields"].items():
                    binding = bindings[sense["sense_id"], name]
                    review = proposed[sense["sense_id"], name]
                    source = sources[field["source_id"]]
                    context = contexts[source["id"]]
                    self.assertTrue(field["text"].strip())
                    self.assertEqual(review["accepted_text"], field["text"])
                    self.assertEqual(review["proposed_text"], candidates[row]["senses"][0]["fields"][name]["text"])
                    self.assertEqual(candidates[row]["senses"][0]["fields"][name]["independent_review"], "pending")
                    self.assertTrue(review["review_reason"])
                    self.assertTrue(review["selected_text_rights_reason"])
                    if name == "ipa":
                        self.assertIn(field["text"][1:-1],
                                      re.findall(r"\{\{pron\|([^|]+)\|fr\}\}", binding["reviewed_excerpt"]))
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
                    author_root = Path(admission["author_evidence_root"]).parents[1]
                    raw_path = author_root / capture["raw_path"]
                    # Raw captures are author evidence; detached/CI checkouts retain bindings.
                    if raw_path.exists():
                        lines = raw_path.read_text(encoding="utf-8").splitlines()
                        self.assertEqual(lines[binding["line_1_based"] - 1], binding["reviewed_excerpt"])
        self.assertEqual(read("reuse.json")["reused_groups"], [])

    def test_existing_validator_accepts_selected_content_and_preserves_canonical_aliases(self):
        canonical = core_pilot.read_json(core_pilot.RESOURCE / "curated-inputs.json")
        expected_aliases = {r["source_headword"]: r["frozen_headword"]
                            for r in canonical["spelling_relations"]}
        with isolated_inputs() as destination:
            _, scope, selected, aliases, gaps = validate(destination)
        self.assertEqual(len(scope), 4087)
        self.assertEqual([item[0] for item in selected], list(range(1501, 1551)))
        self.assertEqual(aliases, expected_aliases)
        self.assertEqual(gaps, {})

    def test_existing_validator_rejects_changed_text_source_usage_rights_and_aliases(self):
        mutations = [
            ("text", "Selected field differs", lambda c, r: c["groups"][0]["senses"][0]["fields"]["english"].update(text="fairness")),
            ("source", "source span differs", lambda c, r: c["groups"][0]["senses"][0]["fields"]["ipa"]["source_span"].update(line_1_based=11)),
            ("usage", "Selected usage differs", lambda c, r: c["groups"][0]["senses"][0].update(use="A moral ideal")),
            ("rights", "not independently reviewed or rights-eligible", lambda c, r: r["fields"][0].update(rights_eligibility_status="pending")),
            ("aliases", "Unsupported spelling relation", lambda c, r: [relation.update(source_headword="tel") for relation in c["spelling_relations"] if relation["source_headword"] == "cœur"]),
        ]
        for name, message, mutate in mutations:
            with self.subTest(mutation=name), isolated_inputs() as destination:
                curated = core_pilot.read_json(destination / "curated-inputs.json")
                review = core_pilot.read_json(destination / "field-review.json")
                mutate(curated, review)
                if name == "aliases":
                    review["spelling_relations"] = copy.deepcopy(curated["spelling_relations"])
                for filename, data in [("curated-inputs.json", curated), ("field-review.json", review)]:
                    (destination / filename).write_text(json.dumps(data, ensure_ascii=False), encoding="utf-8")
                with self.assertRaisesRegex(ValueError, message):
                    validate(destination)


if __name__ == "__main__":
    unittest.main()
