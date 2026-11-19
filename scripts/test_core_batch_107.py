"""Check issue #107 candidates at the approved resource admission boundary."""

import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-107"


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch107Tests(unittest.TestCase):
    def test_exact_frozen_groups_have_four_traceable_candidate_fields(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
                encoding="utf-8-sig", newline="") as handle:
            scope = list(csv.DictReader(handle))
        candidates = read("review-candidates.json")
        self.assertEqual(candidates["scope_denominator_groups"], 4087)
        admission = read("admission.json")
        pending = admission["status"] == "pending_independent_review"
        self.assertEqual(admission["status"], "pending_independent_review" if pending else "passed")
        self.assertEqual(candidates["status"], "pending_independent_review" if pending else "independently_accepted")
        groups = candidates["groups"]
        self.assertEqual(groups, read("groups.json"))
        self.assertEqual([g["frozen_row"] for g in groups], list(range(1301, 1351)))
        sources = {s["id"]: s for s in read("sources.json")}
        captures = {s["source_id"]: s for s in read("source-captures.json")["sources"]}
        contexts = {s["id"]: s for s in read("source-context.json")["current_content"]}
        excerpts = {(s["sense_id"], s["field"]): s
                    for s in read("source-excerpts.json")["fields"]}
        review = read("field-review.json")
        self.assertEqual(review["independence"], not pending)
        if pending:
            self.assertIsNone(review["reviewer"])
        else:
            self.assertTrue(review["reviewer"])
            self.assertTrue(admission["independence"])
            self.assertTrue(admission["reviewer"])
        reviews = {(s["sense_id"], s["field"]): s for s in review["fields"]}
        self.assertEqual(len(excerpts), 200)
        self.assertEqual(len(reviews), 200)
        self.assertEqual(len(review["unresolved_fields"]), 200 if pending else 0)
        self.assertEqual(len(review["usage"]), 50)
        self.assertEqual(len(review["spelling_relations"]), 50)
        raw_sources = {}
        for source_id, source in sources.items():
            capture = captures[source_id]
            raw = Path(capture["raw_path"]).read_text(encoding="utf-8")
            api = json.loads(Path(capture["api_path"]).read_text(encoding="utf-8"))
            page = next(p for p in api["query"]["pages"] if p["title"] == source["title"])
            revision = next(r for r in page["revisions"] if r["revid"] == source["revid"])
            self.assertEqual(raw, revision["slots"]["main"]["content"])
            self.assertEqual(source["timestamp"], revision["timestamp"])
            self.assertEqual(contexts[source_id]["current_content_revid"], source["revid"])
            self.assertTrue(source["attribution"])
            self.assertTrue(source["original_authors_and_rights"])
            self.assertTrue(source["redistribution_obligations"])
            self.assertIn("oldid=" + str(source["revid"]), source["fixed_url"])
            raw_sources[source_id] = raw.splitlines()
        for group in groups:
            with self.subTest(row=group["frozen_row"]):
                row = scope[group["frozen_row"] - 1]
                self.assertEqual((group["headword"], group["lexique_cgram"]),
                                 (row["word"], row["lexique_cgram"]))
                for frequency in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                    self.assertEqual(group[frequency], row[frequency])
                self.assertEqual(len(group["senses"]), 1)
                sense = group["senses"][0]
                self.assertEqual(sense["independent_review"], "pending" if pending else "passed_after_exact_accepted_replacements")
                self.assertEqual(sense["resource_admission"], "pending" if pending else "passed_independent_selected_content_and_rights_only")
                self.assertIsNone(sense["cefr"])
                self.assertTrue(sense["use"])
                support = sense["supporting_evidence"]
                self.assertEqual(support[0]["raw_source_excerpt"], "== {{langue|fr}} ==")
                section = "lettre" if group["frozen_row"] == 1339 else {"NOM": "nom", "VER": "verbe", "ADJ": "adjectif",
                           "ADV": "adverbe"}[group["lexique_cgram"]]
                self.assertIn("{{S|" + section + "|fr", support[1]["raw_source_excerpt"])
                self.assertNotIn("flexion", support[1]["raw_source_excerpt"])
                self.assertIn("'''" + group["headword"] + "'''", support[2]["raw_source_excerpt"])
                self.assertEqual(set(sense["fields"]),
                                 {"french_definition", "ipa", "english", "chinese"})
                for name, field in sense["fields"].items():
                    self.assertTrue(field["text"].strip())
                    self.assertTrue(field["modification"])
                    self.assertEqual(field["independent_review"], "pending" if pending else "passed")
                    binding = excerpts[sense["sense_id"], name]
                    field_review = reviews[sense["sense_id"], name]
                    source = sources[field["source_id"]]
                    self.assertEqual(field_review["proposed_text" if pending else "accepted_text"], field["text"])
                    self.assertEqual(field_review["accepted_text_quality_status"], "pending" if pending else "passed")
                    self.assertEqual(field_review["exact_source"]["revid"], source["revid"])
                    self.assertEqual(binding["revid"], source["revid"])
                    self.assertEqual(field_review["exact_source"]["reviewed_excerpt"],
                                     field["source_span"]["raw_source_excerpt"])
                    self.assertEqual(field["source_span"], binding["source_span"])
                    self.assertEqual(field["source_id"], binding["source_id"])
                    for span in [field["source_span"], *support]:
                        line = raw_sources[field["source_id"]][span["line_1_based"] - 1]
                        column = span["column_1_based"] - 1
                        self.assertEqual(line[column:column + len(span["raw_source_excerpt"])],
                                         span["raw_source_excerpt"])
        self.assertEqual(admission["accepted_groups"], 0 if pending else 50)
        self.assertEqual(admission["accepted_fields"], 0 if pending else 200)
        self.assertEqual(admission["approved_annotations"], [])
        grammar = {s["frozen_row"]: s for s in read("source-context.json")["grammar_corroboration"]}
        e_lines = Path(grammar[1339]["text_path"]).read_text(encoding="utf-8").splitlines()
        self.assertEqual(e_lines[42], "nom masculin invariable")
        self.assertEqual(e_lines[45], "Cinquième lettre, deuxième voyelle de l'alphabet.")
        self.assertFalse(grammar[1339]["selected_text_copied"])
        tel = next(g for g in groups if g["frozen_row"] == 1343)["senses"][0]
        self.assertEqual(tel["fields"]["ipa"]["source_span"]["raw_source_excerpt"],
                         "{{pron|te.le.fɔ.ne|fr}}")
        self.assertEqual(tel["supporting_evidence"][2]["line_1_based"], 7)
        for relation in review["spelling_relations"]:
            self.assertEqual(relation["alias_admission"], "not_admitted")
            self.assertEqual(relation["new_runtime_aliases"], [])


if __name__ == "__main__":
    unittest.main()
