"""Check #161 independently reviewed content at the resource-artifact boundary."""
import csv
import json
from pathlib import Path
import re
import subprocess
import unittest

from core_batch_test_helpers import (
    verify_archived_contributor_captures,
    verify_archived_source_captures,
)

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-161"
AUTHOR_SHA = "10a62d9c4d0ced4515ba45dfb1922e4ec1c065e8"


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch161Tests(unittest.TestCase):
    def test_vaurien_retains_the_milder_sense_without_an_insult_threshold(self):
        group = next(g for g in read("groups.json") if g["frozen_row"] == 4047)
        self.assertEqual(group["senses"][0]["fields"]["english"]["text"],
                         "(Informal, in a milder sense) A rascal or scamp.")

    def test_exact_frozen_membership_has_four_scoped_candidate_fields(self):
        groups = read("groups.json")
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
                encoding="utf-8", newline="") as handle:
            frozen = list(csv.DictReader(handle))[4000:4050]
        self.assertEqual([g["frozen_row"] for g in groups], list(range(4001, 4051)))
        for group, row in zip(groups, frozen, strict=True):
            self.assertEqual(group["headword"], row["word"])
            self.assertEqual(group["lexique_cgram"], row["lexique_cgram"])
            for frequency in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                self.assertEqual(group[frequency], row[frequency])
            self.assertEqual(len(group["senses"]), 1)
            sense = group["senses"][0]
            self.assertTrue(sense["use"])
            self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
            languages = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
            for name, field in sense["fields"].items():
                self.assertEqual(field["language"], languages[name])
                self.assertTrue(field["text"].strip())
                self.assertTrue(field["source_id"])
                self.assertTrue(field["source_span"]["raw_source_excerpt"])

    def test_fixed_source_bytes_and_pos_spans_support_every_field(self):
        captures = read("source-captures.json")
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        archive = Path(captures["capture_root"])

        def resolve(value):
            path = Path(value)
            self.assertTrue(path.is_absolute())
            self.assertTrue(path.is_relative_to(archive))
            return path

        raw = verify_archived_source_captures(self, captures["sources"], sources, resolve)
        names, anonymous, chains = verify_archived_contributor_captures(
            self, captures["contributor_registry"]["parts"], resolve)
        self.assertEqual(set(chains), {f"contributors-new-{n}" for n in range(1, 6)})
        self.assertEqual(set(names), {s["title"] for s in sources.values()})
        contexts = read("source-context.json")["current_content"]
        for context in contexts:
            self.assertEqual(context["current_contributors"], sorted(names[context["title"]]))
            self.assertEqual(context["anonymous_contributors_count"], anonymous.get(context["title"], 0))
        self.assertFalse(captures["contributor_registry"]["selected_sentence_authorship_proved"])
        excerpts = {(f["sense_id"], f["field"]): f for f in read("source-excerpts.json")["fields"]}
        self.assertEqual(len(excerpts), 200)
        pos = {"NOM": "nom", "VER": "verbe", "ADJ": "adjectif", "ADV": "adverbe"}
        for group in read("groups.json"):
            sense = group["senses"][0]
            self.assertEqual(sense["source_headword"], group["headword"])
            for name, field in sense["fields"].items():
                exact = excerpts[sense["sense_id"], name]
                self.assertEqual(exact["source_id"], field["source_id"])
                lines = raw[field["source_id"]]
                line = lines[exact["line_1_based"] - 1]
                self.assertEqual(line, field["source_span"]["raw_source_excerpt"])
                self.assertEqual(exact["reviewed_excerpt"], line)
                self.assertEqual(exact["revid"], sources[field["source_id"]]["revid"])
                preceding = lines[:exact["line_1_based"]]
                language = next(l for l in reversed(preceding) if re.match(r"^==\s*\{\{langue\|", l))
                self.assertRegex(language, r"^==\s*\{\{langue\|fr\}\}\s*==$")
                frame = next(l for l in reversed(preceding) if l.startswith("=== {{S|"))
                self.assertIn("{{S|" + pos[group["lexique_cgram"]] + "|fr", frame)
                if name == "ipa":
                    variant = exact["ipa_variant_excerpt"]
                    start = exact["ipa_variant_column_1_based"] - 1
                    self.assertEqual(line[start:start + len(variant)], variant)
                    self.assertEqual(field["text"], "/" + variant + "/")
                else:
                    self.assertTrue(line.startswith("#"))
                    self.assertFalse(line.startswith(("#*", "##*")))
                    self.assertNotIn("{{Import:CFC", line)

    def test_independent_decisions_preserve_proposals_and_separate_canonical_admission(self):
        admission = read("admission.json")
        self.assertEqual((admission["frozen_groups"], admission["proposed_fields"],
                          admission["scope_denominator_groups"]), (50, 200, 4087))
        self.assertEqual(admission["independently_accepted_new_groups"], 50)
        self.assertEqual(admission["independently_accepted_new_fields"], 200)
        self.assertEqual(admission["newly_admitted_fields"], 0)
        self.assertEqual(admission["missing_admitted_fields"], 200)
        self.assertEqual((admission["accepted_unchanged_fields"], admission["accepted_changed_fields"]), (199, 1))
        self.assertEqual(admission["pending_independent_review_fields"], 0)
        self.assertEqual(admission["annotation_exception_groups"], 0)
        self.assertEqual(admission["specialist_exception_groups"], 0)
        self.assertEqual(admission["unresolved_field_blockers"], [])
        review = read("field-review.json")
        rights = read("rights.json")
        self.assertTrue(review["independent_acceptance"])
        self.assertTrue(review["reviewer"])
        authored = json.loads((EVIDENCE / "authored/groups.json").read_text(encoding="utf-8"))
        proposals = {(s["sense_id"], name): field for g in authored
                     for s in g["senses"] for name, field in s["fields"].items()}
        expected = {(s["sense_id"], name): field for g in read("groups.json")
                    for s in g["senses"] for name, field in s["fields"].items()}
        self.assertEqual({(f["sense_id"], f["field"]) for f in rights["fields"]}, set(expected))
        self.assertEqual({(f["sense_id"], f["field"]) for f in review["fields"]}, set(expected))
        for record in rights["fields"]:
            field = expected[record["sense_id"], record["field"]]
            self.assertEqual(record["exact_source_span"], field["source_span"])
            self.assertEqual(record["license"], "CC-BY-SA-4.0")
            self.assertTrue(record["attribution"])
            self.assertTrue(record["author_rights_basis"])
            self.assertTrue(record["modification"])
            self.assertEqual(record["accepted_text"], field["text"])
            self.assertEqual(record["independent_review"], "passed")
            self.assertEqual(record["reviewer"], review["reviewer"])
            self.assertTrue(record["selected_text_rights_reason"])
        for record in review["fields"]:
            field = expected[record["sense_id"], record["field"]]
            self.assertEqual(record["proposed_text"], proposals[record["sense_id"], record["field"]]["text"])
            self.assertEqual(record["accepted_text"], field["text"])
            self.assertEqual(record["accepted_text_quality_status"], "passed")
            self.assertEqual(field["independent_review"], "passed")
            self.assertTrue(record["review_reason"])
            self.assertTrue(record["independence"])
        self.assertEqual(len(review["usage"]), 50)
        for usage in review["usage"]:
            self.assertEqual(usage["everyday_sense_status"], "passed")
            self.assertEqual(usage["accepted_usage"], usage["proposed_usage"])
            self.assertTrue(usage["review_reason"])
        self.assertEqual(len(review["spelling_relations"]), 50)
        self.assertTrue(all(r["independent_review"] == "passed" and r["review_reason"]
                            for r in review["spelling_relations"]))
        replacements = read("accepted-replacements.json")["fields"]
        self.assertEqual([(r["frozen_row"], r["field"]) for r in replacements], [(4047, "english")])
        self.assertEqual(replacements[0]["proposed_text"],
                         "(Informal, less severe than an insult) A rascal or scamp.")

    def test_restricted_uses_and_excluded_imports_are_explicit(self):
        groups = {g["frozen_row"]: g for g in read("groups.json")}
        sense = lambda row: groups[row]["senses"][0]
        self.assertIn("not used on its own", sense(4035)["fields"]["english"]["text"])
        self.assertIn("ci-joint", sense(4035)["fields"]["french_definition"]["text"])
        self.assertIn("ci-dessous", sense(4035)["fields"]["chinese"]["text"])
        self.assertEqual(sense(4015)["fields"]["ipa"]["source_span"]["line_1_based"], 8)
        self.assertIn("{{fr-accord-rég|ku.ʃe}}", sense(4015)["fields"]["ipa"]["source_span"]["raw_source_excerpt"])
        self.assertIn("Intransitive", sense(4021)["use"])
        self.assertIn("{{intransitif|fr}}", sense(4021)["fields"]["french_definition"]["source_span"]["raw_source_excerpt"])
        self.assertIn("{{h aspiré}}", sense(4037)["fields"]["ipa"]["source_span"]["raw_source_excerpt"])
        self.assertIn("{{h muet|fr}}", sense(4038)["fields"]["ipa"]["source_span"]["raw_source_excerpt"])
        for name in ("french_definition", "english", "chinese"):
            self.assertIn("OR-3", sense(4043)["fields"][name]["text"])
        self.assertIn("{{invar}} ou {{mf}}", sense(4045)["fields"]["ipa"]["source_span"]["raw_source_excerpt"])
        self.assertIn("above ground", sense(4046)["fields"]["english"]["text"])
        self.assertIn("milder", sense(4047)["use"])
        source = next(s for s in read("rights.json")["sources"] if s["title"] == "fidélité")
        marker = next(r for r in source["reference_markers"] if "{{Import:CFC" in r["raw_reference"])
        self.assertEqual(marker["line_1_based"], 40)
        self.assertEqual(marker["author_rights_proposal"], "excluded_from_selected_content")
        self.assertTrue(all(r["alias_admission"] == "not_admitted"
                            for r in read("source-context.json")["spelling_relations"]))

    def test_original_expression_credits_and_authored_snapshot_are_preserved(self):
        notices = (EVIDENCE / "NOTICES.md").read_text(encoding="utf-8")
        for source in read("sources.json")["sources"]:
            self.assertIn(source["fixed_url"], notices)
            self.assertIn(source["history_url"], notices)
        self.assertIn("Académie française", notices)
        self.assertIn("Émile Littré", notices)
        originals = read("original-expression-review.json")
        self.assertEqual(len(originals["sources"]), 50)
        self.assertEqual(originals["unresolved_rights"], [])
        for source in originals["sources"]:
            self.assertTrue(source["selected_expression_reason"])
            self.assertEqual(source["independent_review"], "passed_at_selected_text_boundary")
        sources = {s["title"]: s for s in read("sources.json")["sources"]}
        for title in ("infiniment", "épais", "quête"):
            self.assertTrue(any("Littré" in layer["original_authors"]
                                for layer in sources[title]["original_historical_dictionary_layers"]))
        self.assertTrue(sources["uni"]["original_historical_dictionary_layers"])
        self.assertEqual((EVIDENCE / "COPYING").read_bytes().rstrip(),
                         (ROOT / "resources/core-pilot-79/COPYING").read_bytes().rstrip())
        for path in (EVIDENCE / "authored").rglob("*"):
            if path.is_file():
                relative = path.relative_to(ROOT).as_posix()
                original = subprocess.check_output(["git", "show", f"{AUTHOR_SHA}:{relative}"], cwd=ROOT)
                self.assertEqual(path.read_bytes(), original)
        for name in ("reuse.json", "COPYING"):
            self.assertEqual((EVIDENCE / name).read_bytes(), (EVIDENCE / "authored" / name).read_bytes())
        self.assertEqual(read("content-blockers.json")["blockers"], [])
        self.assertEqual(read("content-blockers.json")["independent_quality_and_rights"], "passed")
        authored_review = json.loads((EVIDENCE / "authored/field-review.json").read_text(encoding="utf-8"))
        self.assertFalse(authored_review["independent_acceptance"])


if __name__ == "__main__":
    unittest.main()
