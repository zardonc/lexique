"""Resource-boundary check for the issue #101 author handoff."""

import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-101"
RESOURCE = ROOT / "resources/core-pilot-79"


class Batch101Tests(unittest.TestCase):
    def test_exact_frozen_groups_have_sourced_fields_or_explicit_gaps(self):
        with (RESOURCE / "scope-manifest.csv").open(encoding="utf-8-sig", newline="") as handle:
            scope = list(csv.DictReader(handle))
        candidates = json.loads((EVIDENCE / "review-candidates.json").read_text(encoding="utf-8"))
        groups = candidates["groups"]
        self.assertEqual([group["frozen_row"] for group in groups], list(range(1001, 1051)))
        self.assertEqual(candidates["scope_denominator_groups"], 4087)
        excerpts = json.loads((EVIDENCE / "source-excerpts.json").read_text(encoding="utf-8"))
        bindings = {(record["sense_id"], record["field"]): record for record in excerpts["fields"]}
        self.assertEqual(len(excerpts["fields"]), 196)
        sources = {source["id"]: source for source in json.loads(
            (EVIDENCE / "sources.json").read_text(encoding="utf-8"))}
        self.assertEqual(len(bindings), 196)
        self.assertEqual(groups, json.loads((EVIDENCE / "groups.json").read_text(encoding="utf-8")))
        review = json.loads((EVIDENCE / "field-review.json").read_text(encoding="utf-8"))
        reviewed = {(record["sense_id"], record["field"]): record for record in review["fields"]}
        context = json.loads((EVIDENCE / "source-context.json").read_text(encoding="utf-8"))
        locations = {(record["sense_id"], record["field"]): record for source in context["current_content"]
                     for record in source["selected_locations"]}
        self.assertEqual(set(bindings), set(reviewed))
        self.assertEqual(set(bindings), set(locations))
        for group in groups:
            row = group["frozen_row"]
            with self.subTest(row=row):
                self.assertEqual((group["headword"], group["lexique_cgram"]),
                                 (scope[row - 1]["word"], scope[row - 1]["lexique_cgram"]))
                self.assertEqual(group["subtitle_lemma_per_million"], scope[row - 1]["subtitle_lemma_per_million"])
                self.assertEqual(group["subtitle_pos_per_million"], scope[row - 1]["subtitle_pos_per_million"])
                if row == 1005:
                    self.assertEqual(group["senses"], [])
                    self.assertEqual(group["status"], "source_spelling_pos_unresolved")
                    self.assertEqual(set(group["pending_fields"]), {"french_definition", "ipa", "english", "chinese"})
                    self.assertEqual((group["admitted_fields"], group["missing_fields"]), (0, 4))
                    self.assertTrue(group["unresolved_reason"])
                    continue
                self.assertEqual(len(group["senses"]), 1)
                sense = group["senses"][0]
                self.assertEqual(sense["source_headword"], group["headword"])
                self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                for name, field in sense["fields"].items():
                    self.assertTrue(field["text"].strip())
                    binding = bindings[sense["sense_id"], name]
                    self.assertEqual(field["source_id"], binding["source_id"])
                    self.assertEqual(field["source_span"]["raw_source_excerpt"], binding["raw_source_excerpt"])
                    self.assertEqual(field["source_span"]["line_1_based"], binding["line_1_based"])
                    self.assertEqual(field["source_span"]["column_1_based"], binding["column_1_based"])
                    source = sources[field["source_id"]]
                    self.assertEqual(source["revid"], binding["revid"])
                    self.assertEqual(source["fixed_url"], binding["fixed_url"])
                    self.assertIn("oldid=" + str(binding["revid"]), binding["fixed_url"])
                    self.assertEqual(field["text"], reviewed[sense["sense_id"], name]["candidate_text"])
                    self.assertEqual(binding["raw_source_excerpt"], locations[sense["sense_id"], name]["raw_source_excerpt"])
                    self.assertEqual(binding["revid"], locations[sense["sense_id"], name]["revid"])
                    if name == "ipa":
                        self.assertTrue(field["text"].startswith("/") and field["text"].endswith("/"))
                        self.assertIn("|" + field["text"][1:-1] + "|", binding["raw_source_excerpt"])
                    if not review["independence"]:
                        self.assertEqual(field["independent_review"], "pending")
                        self.assertIsNone(field["reviewer"])

    def test_fields_match_retained_original_revision_text(self):
        captures = json.loads((EVIDENCE / "source-captures.json").read_text(encoding="utf-8"))["sources"]
        captures_by_id = {f"b101_fr_{capture['pageid']}": capture for capture in captures}
        excerpts = json.loads((EVIDENCE / "source-excerpts.json").read_text(encoding="utf-8"))
        pages = {}
        for capture in captures:
            response = json.loads(Path(capture["api_path"]).read_bytes())
            page = next(page for page in response["query"]["pages"] if page["pageid"] == capture["pageid"])
            revision = next(revision for revision in page["revisions"] if revision["revid"] == capture["revid"])
            body = revision["slots"]["main"]["content"]
            self.assertEqual(revision["timestamp"], capture["timestamp"])
            self.assertEqual(body, Path(capture["raw_path"]).read_text(encoding="utf-8"))
            pages[capture["title"]] = body.splitlines()
        for binding in excerpts["fields"] + excerpts["supporting_evidence"]:
            with self.subTest(sense=binding.get("sense_id"), field=binding.get("field")):
                capture = captures_by_id[binding["source_id"]]
                self.assertEqual(binding["revid"], capture["revid"])
                self.assertEqual(binding["fixed_url"], capture["fixed_url"])
                lines = pages[capture["title"]]
                start = binding["column_1_based"] - 1
                line = lines[binding["line_1_based"] - 1]
                self.assertEqual(line[start:start + len(binding["raw_source_excerpt"])], binding["raw_source_excerpt"])
                language = max(i for i, value in enumerate(lines[:binding["line_1_based"]]) if value.startswith("== "))
                self.assertEqual(lines[language], "== {{langue|fr}} ==")
                if binding.get("field") == "french_definition":
                    pos = max(i for i, value in enumerate(lines[:binding["line_1_based"]]) if value.startswith("=== "))
                    expected = {"NOM": ("{{S|nom|fr", "{{S|lettre|fr"),
                                "VER": ("{{S|verbe|fr",), "ADV": ("{{S|adverbe|fr",),
                                "ADJ": ("{{S|adjectif|fr",), "ADJ:num": ("{{S|adjectif numéral|fr",),
                                "PRO:per": ("{{S|pronom personnel|fr",), "ONO": ("{{S|interjection|fr",)}
                    self.assertTrue(any(marker in lines[pos] for marker in expected[binding["lexique_cgram"]]), lines[pos])


if __name__ == "__main__":
    unittest.main()
