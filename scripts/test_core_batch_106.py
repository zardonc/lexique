"""Check #106 candidates and the exact-spelling blocker at the approved resource seam."""
import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-106"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch106Tests(unittest.TestCase):
    def test_frozen_groups_have_traceable_fields_or_an_explicit_source_blocker(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
                encoding="utf-8-sig", newline="") as handle:
            scope = list(csv.DictReader(handle))
        candidates = read("review-candidates.json")
        self.assertEqual(candidates["scope_denominator_groups"], 4087)
        self.assertEqual(candidates["status"], "pending_independent_review")
        groups = candidates["groups"]
        self.assertEqual(groups, read("groups.json"))
        self.assertEqual([g["frozen_row"] for g in groups], list(range(1251, 1301)))
        admission = read("admission.json")
        self.assertEqual(admission["accepted_groups"], 0)
        self.assertEqual(admission["accepted_fields"], 0)
        self.assertEqual(admission["source_blocked_rows"], [1300])
        self.assertEqual(admission["annotation_exceptions"], [])
        review = read("field-review.json")
        self.assertFalse(review["independence"])
        self.assertIsNone(review["reviewer"])
        self.assertEqual(len(review["unresolved_fields"]), 200)
        reviews = {(f["sense_id"], f["field"]): f for f in review["fields"]}
        excerpts = {(f["sense_id"], f["field"]): f
                    for f in read("source-excerpts.json")["fields"]}
        self.assertEqual(len(reviews), 196)
        self.assertEqual(len(excerpts), 196)
        sources = {s["id"]: s for s in read("sources.json")}
        contexts = {s["id"]: s for s in read("source-context.json")["current_content"]}
        context_fields = {(f["sense_id"], f["field"]): f for c in contexts.values()
                          for f in c["selected_locations"]}
        usages = {u["sense_id"]: u for u in review["usage"]}
        spellings = {s["sense_id"]: s for s in review["spelling_relations"]}
        self.assertEqual(len(context_fields), 196)
        self.assertEqual(len(usages), 49)
        self.assertEqual(len(spellings), 49)
        captures = read("source-captures.json")
        raw_lines = {}
        for capture in captures["sources"]:
            source = sources[capture["source_id"]]
            raw = Path(capture["raw_path"]).read_text(encoding="utf-8")
            api = json.loads(Path(capture["api_path"]).read_text(encoding="utf-8"))
            page = next(p for p in api["query"]["pages"] if p["title"] == source["title"])
            revision = next(r for r in page["revisions"] if r["revid"] == source["revid"])
            self.assertEqual(raw, revision["slots"]["main"]["content"])
            self.assertEqual(source["timestamp"], revision["timestamp"])
            self.assertIn("oldid=" + str(source["revid"]), source["fixed_url"])
            self.assertEqual(contexts[source["id"]]["current_content_revid"], source["revid"])
            self.assertTrue(source["original_authors_and_rights"])
            self.assertTrue(source["redistribution_obligations"])
            raw_lines[source["id"]] = raw.splitlines()
        parts = captures["contributor_registry"]["parts"]
        self.assertTrue(parts)
        self.assertNotIn("continue", json.loads(Path(parts[-1]).read_text(encoding="utf-8")))
        blocker = read("source-context.json")["blocked_groups"][0]
        self.assertEqual(blocker["frozen_row"], 1300)
        self.assertFalse(blocker["annotation_exception"])
        contrast = blocker["contrast_capture"]
        contrast_raw = Path(contrast["raw_path"]).read_text(encoding="utf-8")
        contrast_api = json.loads(Path(contrast["api_path"]).read_text(encoding="utf-8"))
        contrast_revision = contrast_api["query"]["pages"][0]["revisions"][0]
        self.assertEqual(contrast_revision["revid"], contrast["revid"])
        self.assertEqual(contrast_raw, contrast_revision["slots"]["main"]["content"])
        self.assertIn("'''d’'''", blocker["contrast_evidence"][2]["raw_source_excerpt"])
        for span in blocker["supporting_evidence"]:
            self.assertEqual(raw_lines[span["source_id"]][span["line_1_based"] - 1],
                             span["raw_source_excerpt"])
        for span in blocker["contrast_evidence"]:
            self.assertEqual(contrast_raw.splitlines()[span["line_1_based"] - 1],
                             span["raw_source_excerpt"])
        for group in groups:
            row = scope[group["frozen_row"] - 1]
            with self.subTest(row=group["frozen_row"]):
                self.assertEqual((group["headword"], group["lexique_cgram"]),
                                 (row["word"], row["lexique_cgram"]))
                for frequency in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                    self.assertEqual(group[frequency], row[frequency])
                if group["frozen_row"] == 1300:
                    self.assertEqual(group["senses"], [])
                    self.assertEqual(group["status"], "blocked_exact_spelling_pos")
                    self.assertEqual(set(group["missing_fields"]), set(LANGUAGES))
                    self.assertTrue(group["unresolved_reason"])
                    continue
                self.assertEqual(len(group["senses"]), 1)
                sense = group["senses"][0]
                usage = usages[sense["sense_id"]]
                spelling = spellings[sense["sense_id"]]
                self.assertEqual(usage["proposed_usage"], sense["use"])
                self.assertEqual(usage["accepted_usage_quality_status"], "pending")
                self.assertEqual((spelling["query_headword"], spelling["source_headword"]),
                                 (group["headword"], group["headword"]))
                self.assertEqual(spelling["new_runtime_aliases"], [])
                self.assertEqual(sense["source_headword"], group["headword"])
                self.assertTrue(sense["use"])
                self.assertEqual(sense["independent_review"], "pending")
                self.assertIsNone(sense["cefr"])
                support = sense["supporting_evidence"]
                self.assertEqual(support[0]["raw_source_excerpt"], "== {{langue|fr}} ==")
                section = {"NOM": "nom", "VER": "verbe", "ADJ": "adjectif",
                           "CON": "conjonction"}[group["lexique_cgram"]]
                self.assertIn("{{S|" + section + "|fr", support[1]["raw_source_excerpt"])
                self.assertNotIn("flexion", support[1]["raw_source_excerpt"])
                self.assertIn("'''" + group["headword"] + "'''", support[2]["raw_source_excerpt"])
                self.assertEqual(set(sense["fields"]), set(LANGUAGES))
                for name, field in sense["fields"].items():
                    self.assertEqual(field["language"], LANGUAGES[name])
                    self.assertTrue(field["text"].strip())
                    self.assertTrue(field["modification"])
                    self.assertEqual(field["independent_review"], "pending")
                    binding = excerpts[sense["sense_id"], name]
                    proposed = reviews[sense["sense_id"], name]
                    source = sources[field["source_id"]]
                    self.assertEqual(source["title"], group["headword"])
                    self.assertEqual(proposed["frozen_row"], group["frozen_row"])
                    self.assertEqual((proposed["headword"], proposed["pos"]),
                                     (group["headword"], group["lexique_cgram"]))
                    self.assertEqual(proposed["exact_source"]["source_id"], source["id"])
                    self.assertEqual(proposed["exact_source"]["url"], source["fixed_url"])
                    self.assertEqual(proposed["exact_source"]["reviewed_excerpt"],
                                     field["source_span"]["raw_source_excerpt"])
                    self.assertEqual(proposed["exact_source"]["line_1_based"],
                                     field["source_span"]["line_1_based"])
                    self.assertEqual(proposed["exact_source"]["column_1_based"],
                                     field["source_span"]["column_1_based"])
                    context_field = context_fields[sense["sense_id"], name]
                    self.assertEqual({k: v for k, v in context_field.items()
                                      if k not in ("sense_id", "field")}, field["source_span"])
                    self.assertEqual(proposed["proposed_text"], field["text"])
                    self.assertEqual(proposed["exact_source"]["revid"], source["revid"])
                    self.assertEqual(proposed["accepted_text_quality_status"], "pending")
                    self.assertEqual(proposed["rights_eligibility_status"], "pending_selected_span_audit")
                    self.assertEqual(binding["source_span"], field["source_span"])
                    self.assertEqual(binding["source_id"], field["source_id"])
                    for span in [field["source_span"], *support]:
                        line = raw_lines[span.get("source_id", field["source_id"])][span["line_1_based"] - 1]
                        column = span["column_1_based"] - 1
                        self.assertEqual(line[column:column + len(span["raw_source_excerpt"])],
                                         span["raw_source_excerpt"])
        conjunction = groups[44]["senses"][0]
        self.assertEqual(conjunction["fields"]["ipa"]["text"], "/swa/")
        self.assertIn("soit… soit", conjunction["fields"]["french_definition"]["text"])


if __name__ == "__main__":
    unittest.main()
