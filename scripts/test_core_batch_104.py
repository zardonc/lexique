"""Check original candidates and exact independently accepted content for #104."""

import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-104"


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch104Tests(unittest.TestCase):
    def test_exact_frozen_groups_have_four_traceable_candidate_fields(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
                encoding="utf-8-sig", newline="") as handle:
            scope = list(csv.DictReader(handle))
        candidates = read("review-candidates.json")
        self.assertEqual(candidates["scope_denominator_groups"], 4087)
        self.assertEqual(candidates["status"], "pending_independent_review")
        groups = candidates["groups"]
        self.assertEqual([g["frozen_row"] for g in groups], list(range(1151, 1201)))
        sources = {s["id"]: s for s in read("sources.json")}
        captures = {s["source_id"]: s for s in read("source-captures.json")["sources"]}
        excerpts = {(e["sense_id"], e["field"]): e
                    for e in read("source-excerpts.json")["fields"]}
        contexts = {c["id"]: c for c in read("source-context.json")["current_content"]}
        self.assertEqual(len(excerpts), 200)
        for group in groups:
            row = scope[group["frozen_row"] - 1]
            with self.subTest(row=group["frozen_row"]):
                self.assertEqual((group["headword"], group["lexique_cgram"]),
                                 (row["word"], row["lexique_cgram"]))
                for frequency in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                    self.assertEqual(group[frequency], row[frequency])
                self.assertEqual(len(group["senses"]), 1)
                sense = group["senses"][0]
                self.assertEqual(sense["independent_review"], "pending")
                self.assertEqual(sense["resource_admission"], "pending")
                self.assertTrue(sense["use"])
                source = sources[sense["fields"]["ipa"]["source_id"]]
                context = contexts[source["id"]]
                self.assertEqual(context["current_content_revid"], source["revid"])
                support = sense["supporting_evidence"]
                self.assertEqual(support[0]["raw_source_excerpt"], "== {{langue|fr}} ==")
                section = {"NOM": "nom", "VER": "verbe", "ADJ": "adjectif",
                           "ADV": "adverbe", "PRE": "préposition"}[group["lexique_cgram"]]
                self.assertIn("{{S|" + section + "|fr", support[1]["raw_source_excerpt"])
                self.assertNotIn("flexion", support[1]["raw_source_excerpt"])
                self.assertIn("'''" + group["headword"] + "'''", support[2]["raw_source_excerpt"])
                self.assertEqual(set(sense["fields"]),
                                 {"french_definition", "ipa", "english", "chinese"})
                for name, field in sense["fields"].items():
                    self.assertTrue(field["text"].strip())
                    self.assertTrue(field["modification"])
                    self.assertEqual(field["independent_review"], "pending")
                    source = sources[field["source_id"]]
                    self.assertTrue(source["attribution"])
                    self.assertTrue(source["original_authors_and_rights"])
                    self.assertTrue(source["redistribution_obligations"])
                    self.assertIn("oldid=" + str(source["revid"]), source["fixed_url"])
                    capture = captures[source["id"]]
                    raw = Path(capture["raw_path"]).read_text(encoding="utf-8")
                    api = json.loads(Path(capture["api_path"]).read_text(encoding="utf-8"))
                    page = next(p for p in api["query"]["pages"] if p["title"] == source["title"])
                    revision = next(r for r in page["revisions"] if r["revid"] == source["revid"])
                    self.assertEqual(raw, revision["slots"]["main"]["content"])
                    self.assertEqual(revision["timestamp"], capture["timestamp"])
                    self.assertEqual(page["pageid"], capture["pageid"])
                    binding = excerpts[sense["sense_id"], name]
                    self.assertEqual(binding["revid"], source["revid"])
                    self.assertEqual(field["source_span"], binding["source_span"])
                    self.assertEqual(field["source_id"], binding["source_id"])
                    for span in [field["source_span"], *sense["supporting_evidence"]]:
                        line = raw.splitlines()[span["line_1_based"] - 1]
                        column = span["column_1_based"] - 1
                        self.assertEqual(line[column:column + len(span["raw_source_excerpt"])],
                                         span["raw_source_excerpt"])

    def test_reviewed_content_matches_preserved_candidates(self):
        groups = read("groups.json")
        candidates = read("review-candidates.json")["groups"]
        review = read("field-review.json")
        admission = read("admission.json")
        self.assertTrue(review["independence"])
        self.assertTrue(review["reviewer"])
        self.assertEqual([group["frozen_row"] for group in groups], list(range(1151, 1201)))
        fields = {(record["sense_id"], record["field"]): record for record in review["fields"]}
        usage = {record["sense_id"]: record for record in review["usage"]}
        for group, candidate in zip(groups, candidates, strict=True):
            for key in ("frozen_row", "headword", "lexique_cgram", "subtitle_lemma_per_million",
                        "subtitle_pos_per_million"):
                self.assertEqual(group[key], candidate[key])
            self.assertEqual(len(group["senses"]), 1)
            sense, original = group["senses"][0], candidate["senses"][0]
            for key in ("sense_id", "use", "use_language", "source_headword", "excluded_uses", "cefr"):
                self.assertEqual(sense[key], original[key])
            self.assertEqual(sense["independent_review"], "passed_after_exact_accepted_replacements")
            self.assertEqual(usage[sense["sense_id"]]["accepted_usage"], sense["use"])
            for name, field in sense["fields"].items():
                for key in ("text", "language", "source_id", "source_span", "content_license", "modification"):
                    self.assertEqual(field[key], original["fields"][name][key])
                record = fields[sense["sense_id"], name]
                self.assertEqual(record["accepted_text"], field["text"])
                self.assertEqual(record["accepted_text_quality_status"], "passed")
                self.assertEqual(record["rights_eligibility_status"], "passed_at_selected_text_boundary")
                self.assertEqual(record["exact_source"]["reviewed_excerpt"], field["source_span"]["raw_source_excerpt"])
        self.assertEqual(len(fields), 200)
        self.assertEqual(len(usage), 50)
        self.assertEqual((admission["accepted_groups"], admission["accepted_fields"],
                          admission["missing_admitted_fields"]), (50, 200, 0))
        self.assertEqual(review["unresolved_fields"], [])
        self.assertEqual(admission["approved_annotation_exceptions"], [])
        self.assertEqual(admission["runtime_aliases_added"], [])


if __name__ == "__main__":
    unittest.main()
