"""Check #157 at the approved resource/admission boundary."""
import csv
import json
from pathlib import Path
import shutil
import tempfile
import unittest

from core_batch_test_helpers import verify_archived_source_captures, verify_archived_contributor_captures
from core_pilot import load_inputs, MATERIALS

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-157"
MEMBERSHIP = [
    ("tennis", "NOM"), ("gré", "NOM"), ("flinguer", "VER"),
    ("sirène", "NOM"), ("slip", "NOM"), ("obligation", "NOM"),
    ("exclure", "VER"), ("gentiment", "ADV"), ("cinquième", "ADJ"),
    ("clocher", "VER"), ("chaos", "NOM"), ("crapaud", "NOM"),
    ("album", "NOM"), ("gonzesse", "NOM"), ("lointain", "ADJ"),
    ("déposition", "NOM"), ("engin", "NOM"), ("dernièrement", "ADV"),
    ("remarque", "NOM"), ("avril", "NOM"), ("humide", "ADJ"),
    ("élire", "VER"), ("taux", "NOM"), ("budget", "NOM"),
    ("défunt", "NOM"), ("périr", "VER"), ("limiter", "VER"),
    ("comptable", "NOM"), ("fermier", "NOM"), ("joint", "NOM"),
    ("reportage", "NOM"), ("ambition", "NOM"), ("aveu", "NOM"),
    ("bénédiction", "NOM"), ("concierge", "NOM"), ("création", "NOM"),
    ("raide", "ADJ"), ("voyageur", "NOM"), ("banane", "NOM"),
    ("latin", "NOM"), ("maquiller", "VER"), ("alibi", "NOM"),
    ("logement", "NOM"), ("chauve", "ADJ"), ("fixe", "ADJ"),
    ("harceler", "VER"), ("noeud", "NOM"), ("mûr", "ADJ"),
    ("nous-mêmes", "PRO:per"), ("fichier", "NOM"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch157Tests(unittest.TestCase):
    def test_crapaud_selected_expression_excludes_unverified_quillet_additions(self):
        group = next(g for g in read("groups.json") if g["frozen_row"] == 3812)
        fields = group["senses"][0]["fields"]
        for name in ("french_definition", "english", "chinese"):
            span = fields[name]["source_span"]["raw_source_excerpt"]
            self.assertNotIn("[[patte]]", span)
            self.assertNotIn("[[venimeuse]]", span)
            self.assertIn("[[grenouille]]", span)
        self.assertEqual(fields["french_definition"]["text"], "Amphibien ressemblant à une grenouille.")
        self.assertEqual(fields["english"]["text"], "A toad: an amphibian resembling a frog.")
        self.assertEqual(fields["chinese"]["text"], "蟾蜍：与青蛙相似的两栖动物。")

    def test_author_proposals_cannot_pass_the_real_independent_admission_gate(self):
        temporary_root = ROOT / "target/issue-157-review"
        temporary_root.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="admission-157-", dir=temporary_root) as folder:
            staged = Path(folder)
            for name in (*MATERIALS, "input-checksums.json"):
                shutil.copy2(ROOT / "resources/core-pilot-79" / name, staged / name)
            shutil.copy2(EVIDENCE / "authored/field-review.json", staged / "field-review.json")
            with self.assertRaisesRegex(ValueError, "A separate independent field review is required"):
                load_inputs(staged)

    def test_fixed_source_bytes_spans_frames_and_complete_contributors(self):
        captures = read("source-captures.json")
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        archive = Path(captures["capture_root"])

        def resolve(value):
            path = Path(value)
            self.assertTrue(path.is_relative_to(archive), value)
            self.assertTrue(path.is_file(), value)
            return path

        raw = verify_archived_source_captures(self, captures["sources"], sources, resolve)
        excerpts = read("source-excerpts.json")
        selected = {(r["sense_id"], r["field"]): r for r in excerpts["fields"]}
        self.assertEqual(len(selected), 200)
        for record in [*excerpts["fields"], *excerpts["supporting_evidence"]]:
            line = raw[record["source_id"]][record["line_1_based"] - 1]
            column = record["column_1_based"] - 1
            self.assertEqual(line[column:column + len(record["reviewed_excerpt"])],
                             record["reviewed_excerpt"])
            self.assertEqual(record["revid"], sources[record["source_id"]]["revid"])
            self.assertNotRegex(record["reviewed_excerpt"].lstrip(), r"^#+\*")
        pos_frames = {"NOM": "nom", "VER": "verbe", "ADJ": "adjectif",
                      "ADV": "adverbe", "PRO:per": "pronom personnel"}
        languages = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
        for group in read("groups.json"):
            sense = group["senses"][0]
            for name, field in sense["fields"].items():
                exact = selected[sense["sense_id"], name]
                self.assertEqual(field["source_id"], exact["source_id"])
                self.assertEqual(field["language"], languages[name])
                self.assertEqual(field["source_span"]["raw_source_excerpt"], exact["reviewed_excerpt"])
                before = raw[exact["source_id"]][:exact["line_1_based"]]
                language = next(line for line in reversed(before) if line.startswith("== "))
                self.assertRegex(language, r"^==\s*\{\{langue\|fr\}\}\s*==$")
                if name == "ipa":
                    self.assertEqual(field["text"].strip("/"), exact["ipa_variant_excerpt"])
                    column = exact["ipa_variant_column_1_based"] - 1
                    self.assertEqual(exact["reviewed_excerpt"][column:column + len(exact["ipa_variant_excerpt"])],
                                     exact["ipa_variant_excerpt"])
                else:
                    pos = next(line for line in reversed(before) if line.startswith("=== {{S|"))
                    self.assertIn("{{S|" + pos_frames[group["lexique_cgram"]] + "|fr", pos)
        names, anonymous, chains = verify_archived_contributor_captures(
            self, captures["contributor_registry"]["parts"], resolve)
        for context in read("source-context.json")["current_content"]:
            self.assertEqual(context["current_contributors"], sorted(names[context["title"]]))
            self.assertEqual(context["current_registry_names"], len(names[context["title"]]))
            self.assertEqual(context["anonymous_contributors_count"], anonymous.get(context["title"], 0))
        self.assertFalse(captures["contributor_registry"]["selected_sentence_authorship_proved"])

    def test_source_spelling_and_grammatical_subclasses_remain_distinct(self):
        groups = {g["frozen_row"]: g for g in read("groups.json")}
        knot = groups[3847]
        self.assertEqual((knot["headword"], knot["lexique_cgram"]), ("noeud", "NOM"))
        self.assertEqual(knot["senses"][0]["source_headword"], "nœud")
        relation = next(r for r in read("source-context.json")["spelling_relations"] if r["frozen_row"] == 3847)
        self.assertEqual(relation["alias_admission"], "not_admitted")
        self.assertTrue(any("Orthographe par contrainte typographique" in r["reviewed_excerpt"]
                            for r in relation["proposed_evidence"]))
        pronoun = groups[3849]
        self.assertEqual(pronoun["lexique_cgram"], "PRO:per")
        self.assertEqual(pronoun["senses"][0]["fields"]["ipa"]["text"], "/nu.mɛm/")
        verb = groups[3810]
        self.assertEqual(verb["lexique_cgram"], "VER")
        self.assertEqual(verb["senses"][0]["fields"]["french_definition"]["source_span"]["line_1_based"], 144)

    def test_accepted_fields_match_independent_quality_rights_and_authored_proposals(self):
        rights, admission, review = read("rights.json"), read("admission.json"), read("field-review.json")
        self.assertEqual(rights["independent_review"], "200_passed")
        self.assertEqual(rights["reviewer"], review["reviewer"])
        self.assertEqual(rights["independent_rights_fields_accepted"], 200)
        self.assertEqual(admission["independently_accepted_new_groups"], 50)
        self.assertEqual(admission["independently_accepted_new_fields"], 200)
        self.assertEqual(admission["missing_admitted_fields"], 200)
        self.assertEqual(admission["newly_admitted_fields"], 0)
        self.assertEqual(admission["pending_independent_review_fields"], 0)
        self.assertEqual(admission["pending_independent_rights_fields"], 0)
        self.assertEqual((admission["accepted_unchanged_fields"], admission["accepted_changed_fields"]), (197, 3))
        self.assertEqual(admission["unresolved_field_blockers"], [])
        self.assertEqual(review["unresolved_fields"], [])
        self.assertTrue(review["independent_acceptance"])
        self.assertTrue(review["independence"])
        self.assertEqual(review["kind"], "separate_independent_field_review")
        self.assertEqual(review["reviewer"], admission["independent_reviewer"])
        expected = {(g["senses"][0]["sense_id"], name): field
                    for g in read("groups.json") for name, field in g["senses"][0]["fields"].items()}
        authored = {(g["senses"][0]["sense_id"], name): field
                    for g in read("authored/groups.json") for name, field in g["senses"][0]["fields"].items()}
        decisions = {(r["sense_id"], r["field"]): r for r in review["fields"]}
        self.assertEqual(len(decisions), 200)
        self.assertEqual(set(decisions), set(expected))
        for key, field in expected.items():
            decision = decisions[key]
            self.assertEqual(decision["proposed_text"], authored[key]["text"])
            self.assertEqual(decision["accepted_text"], field["text"])
            self.assertEqual(decision["accepted_source"], decision["exact_source"])
            self.assertEqual(decision["accepted_source"]["reviewed_excerpt"], field["source_span"]["raw_source_excerpt"])
            self.assertEqual(decision["accepted_text_quality_status"], "passed")
            self.assertEqual(decision["rights_eligibility_status"], "passed_at_selected_text_boundary")
            self.assertEqual(field["independent_review"], "passed")
            self.assertEqual(decision["reviewer"], review["reviewer"])
            self.assertTrue(decision["review_reason"].strip())
            self.assertTrue(decision["rights_review_reason"].strip())
        self.assertEqual({(r["sense_id"], r["field"]) for r in rights["fields"]}, set(expected))
        for record in rights["fields"]:
            field = expected[record["sense_id"], record["field"]]
            self.assertEqual(record["exact_source_span"], field["source_span"])
            self.assertEqual(record["source_id"], field["source_id"])
            self.assertEqual(record["modification"], field["modification"])
            self.assertEqual(record["license"], "CC-BY-SA-4.0")
            self.assertTrue(record["attribution"])
            self.assertEqual(record["accepted_text"], field["text"])
            self.assertEqual(record["independent_review"], "passed")
            self.assertEqual(record["reviewer"], review["reviewer"])
            self.assertEqual(record["review_reason"], decisions[record["sense_id"], record["field"]]["rights_review_reason"])
        replacements = read("accepted-replacements.json")
        self.assertEqual({(r["frozen_row"], r["field"]) for r in replacements},
                         {(3812, k) for k in ("french_definition", "english", "chinese")})
        for r in replacements:
            key = r["sense_id"], r["field"]
            self.assertEqual(r["proposed_text"], authored[key]["text"])
            self.assertEqual(r["accepted_text"], expected[key]["text"])
            self.assertEqual(r["proposed_source"], decisions[key]["proposed_source"])
            self.assertEqual(r["accepted_source"], decisions[key]["exact_source"])
        self.assertEqual((EVIDENCE / "COPYING").read_bytes(),
                         (ROOT / "resources/core-pilot-79/COPYING").read_bytes().rstrip() + b"\n")

    def test_all_selected_uses_spellings_and_original_boundaries_have_actual_decisions(self):
        review = read("field-review.json")
        uses = {r["sense_id"]: r for r in review["usage"]}
        self.assertEqual(len(uses), 50)
        originals = read("original-expression-review.json")
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        self.assertEqual({r["source_id"] for r in originals["sources"]}, set(sources))
        self.assertEqual(originals["unresolved_sources"], [])
        self.assertEqual(len(originals["excluded_unresolved_original_material"]), 1)
        for record in originals["sources"]:
            self.assertEqual(record["fixed_revision"], sources[record["source_id"]]["revid"])
            self.assertEqual(record["selected_original_expression_decision"], "passed")
            original = record["original_comparison"]
            if original:
                text = Path(original["plain_text_path"]).read_text(encoding="utf-8")
                column = original["plain_text_column_1_based"] - 1
                phrase = original["original_expression"]
                self.assertEqual(text[column:column + len(phrase)], phrase)
                self.assertIn("8e édition (1935)", text)
        for group in read("groups.json"):
            sense = group["senses"][0]
            decision = uses[sense["sense_id"]]
            self.assertEqual(decision["accepted_usage"], sense["use"])
            self.assertEqual(decision["accepted_usage_quality_status"], "passed")
            self.assertEqual(decision["everyday_use_decision"], "passed")
            self.assertEqual(decision["reviewer"], review["reviewer"])
            self.assertTrue(decision["everyday_sense_reason"].strip())
        for name in ("field-review.json", "source-context.json"):
            relations = read(name)["spelling_relations"]
            self.assertEqual(len(relations), 50)
            for record in relations:
                self.assertEqual(record["independent_review"], "passed")
                self.assertEqual(record["alias_admission"], "not_admitted")
                self.assertEqual(record["reviewer"], review["reviewer"])
                self.assertTrue(record["review_reason"].strip())

    def test_exact_frozen_groups_have_four_scoped_accepted_fields(self):
        groups = read("groups.json")
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        self.assertEqual([g["frozen_row"] for g in groups], list(range(3801, 3851)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            frozen = list(csv.DictReader(handle))[3800:3850]
        for group, row in zip(groups, frozen, strict=True):
            for key in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                self.assertEqual(group[key], row[key])
            self.assertTrue(group["senses"])
            for sense in group["senses"]:
                self.assertTrue(sense["use"])
                self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                for field in sense["fields"].values():
                    self.assertTrue(field["text"].strip())
                    self.assertTrue(field["source_id"])
                    self.assertEqual(field["independent_review"], "passed")


if __name__ == "__main__":
    unittest.main()
