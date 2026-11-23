"""Check #150 at the approved resource/admission boundary."""
import csv
import json
from pathlib import Path
import tempfile
from urllib.parse import parse_qs, urlparse
import unittest

from core_batch_test_helpers import verify_archived_source_captures, verify_archived_contributor_captures

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-150"
MEMBERSHIP = [
    ("pull", "NOM"), ("cavalier", "NOM"), ("satellite", "NOM"),
    ("administration", "NOM"), ("barrage", "NOM"), ("dessert", "NOM"),
    ("juin", "NOM"), ("agresser", "VER"), ("reporter", "VER"),
    ("tristesse", "NOM"), ("planche", "NOM"), ("capitale", "NOM"),
    ("capital", "ADJ"), ("demeure", "NOM"), ("record", "NOM"),
    ("intervention", "NOM"), ("conviction", "NOM"), ("inconscient", "ADJ"),
    ("pointe", "NOM"), ("frite", "NOM"), ("historique", "ADJ"),
    ("conte", "NOM"), ("fiancer", "VER"), ("profondeur", "NOM"),
    ("publicité", "NOM"), ("quotidien", "ADJ"), ("angle", "NOM"),
    ("lot", "NOM"), ("tailler", "VER"), ("entretenir", "VER"),
    ("lance", "NOM"), ("fillette", "NOM"), ("tranquillement", "ADV"),
    ("effondrer", "VER"), ("mark", "NOM"), ("carnet", "NOM"),
    ("enveloppe", "NOM"), ("impatient", "ADJ"), ("résistance", "NOM"),
    ("ère", "NOM"), ("phase", "NOM"), ("remarquable", "ADJ"),
    ("revenu", "NOM"), ("richesse", "NOM"), ("baby", "NOM"),
    ("délire", "NOM"), ("dépôt", "NOM"), ("disponible", "ADJ"),
    ("paysage", "NOM"), ("anéantir", "VER"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch150Tests(unittest.TestCase):
    def test_publicite_selects_the_daf8_advertisement_boundary(self):
        group = next(g for g in read("groups.json") if g["headword"] == "publicité")
        sense = group["senses"][0]
        self.assertEqual(sense["use"], "Feminine count noun for a commercial advertisement, in any medium.")
        accepted = {
            "french_definition": "Annonce commerciale diffusée dans un journal, sur une affiche ou par un autre moyen de réclame.",
            "english": "A commercial advertisement carried in a newspaper, on a poster or through another advertising medium.",
            "chinese": "通过报纸、海报或其他宣传媒介发布的商业广告。",
        }
        for name, text in accepted.items():
            with self.subTest(field=name):
                field = sense["fields"][name]
                self.assertEqual(field["text"], text)
                self.assertEqual(field["source_span"]["line_1_based"], 26)
                self.assertIn("[[annonce|Annonce]]", field["source_span"]["raw_source_excerpt"])
        selected = [r for r in read("source-excerpts.json")["fields"]
                    if r["sense_id"] == sense["sense_id"] and r["field"] != "ipa"]
        self.assertEqual([r["line_1_based"] for r in selected], [26, 26, 26])

    def test_exact_frozen_groups_have_complete_resource_records(self):
        if (EVIDENCE / "groups.json").exists():
            groups = read("groups.json")
        else:
            canonical = json.loads((ROOT / "resources/core-pilot-79/curated-inputs.json").read_text(encoding="utf-8"))
            groups = [g for g in canonical["groups"] if 3451 <= g["frozen_row"] <= 3500]
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP,
                         "The frozen #150 resource slice lacks complete batch content")
        self.assertEqual([g["frozen_row"] for g in groups], list(range(3451, 3501)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(encoding="utf-8-sig", newline="") as handle:
            frozen = list(csv.DictReader(handle))
        self.assertEqual(len(frozen), 4087)
        for group in groups:
            with self.subTest(row=group["frozen_row"]):
                row = frozen[group["frozen_row"] - 1]
                self.assertEqual((group["headword"], group["lexique_cgram"]), (row["word"], row["lexique_cgram"]))
                for frequency in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                    self.assertEqual(group[frequency], row[frequency])
                self.assertEqual(len(group["senses"]), 1)
                sense = group["senses"][0]
                self.assertTrue(sense["use"].strip())
                self.assertIsNone(sense["cefr"])
                self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                for field in sense["fields"].values():
                    self.assertTrue(field["text"].strip())
                    self.assertTrue(field["modification"].strip())

    def test_fixed_fields_have_independent_review_and_selected_rights(self):
        admission, review, rights = read("admission.json"), read("field-review.json"), read("rights.json")
        self.assertEqual(admission["status"], "independently_reviewed_selected_content_pending_canonical_assembly")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual((admission["independently_accepted_new_groups"], admission["independently_accepted_new_fields"]), (50, 200))
        self.assertEqual(admission["pending_independent_review_fields"], 0)
        self.assertEqual(admission["pending_independent_rights_fields"], 0)
        self.assertEqual(admission["newly_admitted_fields"], 0)
        self.assertEqual(admission["missing_admitted_fields"], 200)
        self.assertEqual(admission["unresolved_field_blockers"], [])
        self.assertTrue(review["independent_acceptance"])
        self.assertEqual(review["kind"], "separate_independent_field_review")
        self.assertTrue(review["independence"])
        self.assertEqual(review["reviewer"], admission["independent_reviewer"])
        self.assertEqual(rights["independent_rights_fields_accepted"], 200)
        captures = read("source-captures.json")
        archive = Path(captures["capture_root"])
        self.assertEqual((archive.name, archive.parent.name), ("issue-150", "author-evidence"))
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        raw = verify_archived_source_captures(self, captures["sources"], sources, Path)
        self.assertEqual(len(raw), 54)
        for capture in captures["sources"]:
            for name in ("api_path", "raw_path"):
                self.assertTrue(Path(capture[name]).is_relative_to(archive))
            self.assertEqual(parse_qs(urlparse(capture["fixed_url"]).query)["oldid"], [str(capture["revid"])])
        excerpts = read("source-excerpts.json")
        by_field = {(r["sense_id"], r["field"]): r for r in excerpts["fields"]}
        self.assertEqual(len(by_field), 200)
        for record in [*excerpts["fields"], *excerpts["supporting_evidence"]]:
            line = raw[record["source_id"]][record["line_1_based"] - 1]
            column = record["column_1_based"] - 1
            self.assertEqual(line[column:column + len(record["reviewed_excerpt"])], record["reviewed_excerpt"])
            self.assertNotRegex(record["reviewed_excerpt"].lstrip(), r"^#+\*")
            self.assertEqual(record["revid"], sources[record["source_id"]]["revid"])
        expected = {}
        languages = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
        pos_frames = {"NOM": "nom", "ADJ": "adjectif", "VER": "verbe", "ADV": "adverbe"}
        for group in read("groups.json"):
            sense = group["senses"][0]
            self.assertEqual(sense["source_headword"], group["headword"])
            self.assertIn(sense["independent_review"], ("passed", "passed_after_exact_accepted_replacements"))
            for name, field in sense["fields"].items():
                exact = by_field[sense["sense_id"], name]
                expected[sense["sense_id"], name] = field
                self.assertEqual(field["language"], languages[name])
                self.assertEqual(field["source_id"], exact["source_id"])
                self.assertEqual(field["source_span"]["raw_source_excerpt"], exact["reviewed_excerpt"])
                self.assertEqual(field["source_span"]["line_1_based"], exact["line_1_based"])
                preceding = raw[exact["source_id"]][:exact["line_1_based"]]
                language = next(line for line in reversed(preceding) if line.startswith("== "))
                self.assertRegex(language, r"^==\s*\{\{langue\|fr\}\}\s*==$")
                if name == "ipa":
                    self.assertEqual(field["text"].strip("/"), exact["ipa_variant_excerpt"])
                    column = exact["ipa_variant_column_1_based"] - 1
                    self.assertEqual(exact["reviewed_excerpt"][column:column + len(exact["ipa_variant_excerpt"])],
                                     exact["ipa_variant_excerpt"])
                else:
                    pos = next(line for line in reversed(preceding) if line.startswith("=== {{S|"))
                    self.assertIn("{{S|" + pos_frames[group["lexique_cgram"]] + "|fr", pos)
        self.assertEqual({(f["sense_id"], f["field"]) for f in rights["fields"]}, set(expected))
        for field in rights["fields"]:
            proposed = expected[field["sense_id"], field["field"]]
            self.assertEqual(field["exact_source_span"], proposed["source_span"])
            self.assertEqual(field["license"], "CC-BY-SA-4.0")
            self.assertTrue(field["attribution"])
            self.assertTrue(field["modification"])
            self.assertEqual(field["independent_review"], "passed_at_selected_text_boundary")
            self.assertEqual(field["reviewer"], review["reviewer"])
            self.assertEqual(field["accepted_text"], proposed["text"])
            self.assertTrue(field["selected_text_rights_reason"].strip())
        self.assertEqual({(f["sense_id"], f["field"]) for f in review["fields"]}, set(expected))
        for field in review["fields"]:
            accepted = expected[field["sense_id"], field["field"]]
            self.assertEqual(field["accepted_text"], accepted["text"])
            self.assertEqual(field["accepted_text_quality_status"], accepted["independent_review"])
            self.assertEqual(field["rights_eligibility_status"], accepted["rights_eligibility_status"])
            self.assertEqual(field["reviewer"], review["reviewer"])
            self.assertTrue(field["independence"])
            self.assertTrue(field["review_reason"].strip())
            self.assertTrue(field["selected_text_rights_reason"].strip())
            exact = by_field[field["sense_id"], field["field"]]
            for key in ("source_id", "revid", "line_1_based", "column_1_based", "reviewed_excerpt"):
                self.assertEqual(field["exact_source"][key], exact[key])
        def resolve(value):
            path = Path(value)
            self.assertTrue(path.is_relative_to(archive))
            return path
        names, anonymous, chains = verify_archived_contributor_captures(
            self, captures["contributor_registry"]["parts"], resolve)
        self.assertEqual(set(chains), {*(f"contributors-new-{n}" for n in range(1, 6)), "contributors-support"})
        self.assertEqual(set(names), {s["title"] for s in sources.values()})
        for context in read("source-context.json")["current_content"]:
            self.assertEqual(context["current_contributors"], sorted(names[context["title"]]))
            self.assertEqual(context["current_registry_names"], len(names[context["title"]]))
            self.assertEqual(context["anonymous_contributors_count"], anonymous.get(context["title"], 0))
        self.assertFalse(captures["contributor_registry"]["selected_sentence_authorship_proved"])
        notices = (EVIDENCE / "NOTICES.md").read_text(encoding="utf-8")
        for source in sources.values():
            self.assertIn(source["fixed_url"], notices)
            self.assertIn(source["history_url"], notices)
        self.assertEqual((EVIDENCE / "COPYING").read_bytes().rstrip(),
                         (ROOT / "resources/core-pilot-79/COPYING").read_bytes().rstrip())

    def test_proposals_replacements_uses_and_original_credits_align(self):
        authored = {s["sense_id"]: s for g in read("authored/groups.json") for s in g["senses"]}
        accepted = {s["sense_id"]: s for g in read("groups.json") for s in g["senses"]}
        self.assertEqual(set(authored), set(accepted))
        replacements = {(r["sense_id"], r["field"]): r for r in read("accepted-replacements.json")}
        changes = set()
        for decision in read("field-review.json")["fields"]:
            key = decision["sense_id"], decision["field"]
            proposal = authored[key[0]]["fields"][key[1]]
            field = accepted[key[0]]["fields"][key[1]]
            self.assertEqual(decision["proposed_text"], proposal["text"])
            self.assertEqual(decision["proposed_source"]["line_1_based"], proposal["source_span"]["line_1_based"])
            self.assertEqual(decision["proposed_source"]["reviewed_excerpt"], proposal["source_span"]["raw_source_excerpt"])
            if field["text"] != proposal["text"]:
                changes.add(key)
                replacement = replacements[key]
                self.assertEqual(replacement["proposed_text"], proposal["text"])
                self.assertEqual(replacement["accepted_text"], field["text"])
                self.assertEqual(replacement["proposed_source_span"], proposal["source_span"])
                self.assertEqual(replacement["accepted_source_span"], field["source_span"])
            else:
                self.assertEqual(field["source_span"], proposal["source_span"])
        self.assertEqual(changes, set(replacements))
        self.assertEqual(len(changes), 3)
        review = read("field-review.json")
        self.assertEqual(len(review["usage"]), 50)
        self.assertEqual({u["sense_id"] for u in review["usage"]}, set(accepted))
        changed_uses = 0
        for use in review["usage"]:
            sense = accepted[use["sense_id"]]
            self.assertEqual(use["proposed_usage"], authored[use["sense_id"]]["use"])
            self.assertEqual(use["accepted_usage"], sense["use"])
            self.assertEqual(use["excluded_uses"], sense["excluded_uses"])
            self.assertEqual(use["accepted_usage_quality_status"], "passed")
            self.assertEqual(use["everyday_sense_status"], "passed")
            self.assertEqual(use["reviewer"], review["reviewer"])
            self.assertTrue(use["review_reason"].strip())
            changed_uses += use["accepted_usage"] != use["proposed_usage"]
        self.assertEqual(changed_uses, 1)
        originals = read("original-expression-review.json")["sources"]
        self.assertEqual(len(originals), 54)
        sources = read("sources.json")["sources"]
        self.assertEqual({s["source_id"] for s in originals}, {s["id"] for s in sources})
        for source in sources:
            self.assertEqual(source["selected_text_review"], "passed_for_selected_spans")
            for layer in source["original_historical_dictionary_layers"]:
                self.assertEqual(layer["independent_review"], "passed_at_original_work_boundary")
                if layer["original_authors"] == "Académie française":
                    self.assertTrue(layer["conservative_expiry"].startswith("2006-01-01"))
                else:
                    self.assertEqual(layer["original_authors"], "Émile Littré")
                    self.assertEqual(layer["conservative_expiry"], "1952-01-01")
        tailler = next(s for s in sources if s["title"] == "tailler")
        self.assertEqual(tailler["original_historical_dictionary_layers"][0]["original_authors"], "Académie française")
        calm = next(s for s in sources if s["title"] == "tranquillement")
        self.assertEqual({l["original_authors"] for l in calm["original_historical_dictionary_layers"]},
                         {"Académie française", "Émile Littré"})


class ContributorCaptureTests(unittest.TestCase):
    def setUp(self):
        scratch = ROOT / "target/final-review-fixes"
        scratch.mkdir(parents=True, exist_ok=True)
        folder = tempfile.TemporaryDirectory(dir=scratch)
        self.addCleanup(folder.cleanup)
        self.archive = Path(folder.name)
        self.parts = [self.archive / f"contributors-new-1-{part}.api.json" for part in (1, 2)]
        self.responses = [
            {"query": {"pages": [{"title": "pull", "contributors": [{"name": "Alice"}], "anoncontributors": 2}]},
             "continue": {"continue": "||", "pccontinue": "10|20"}},
            {"query": {"pages": [{"title": "pull", "contributors": [{"name": "Alice"}, {"name": "Bob"}]},
                                  {"title": "cavalier", "contributors": []}]}},
        ]
        self.request = "https://fr.wiktionary.org/w/api.php?continue=%7C%7C&pccontinue=10%7C20"

    def verify(self):
        for path, response in zip(self.parts, self.responses):
            path.write_text(json.dumps(response), encoding="utf-8")
        self.parts[1].with_suffix(".request.txt").write_text(self.request, encoding="utf-8")
        return verify_archived_contributor_captures(self, list(reversed(self.parts)), Path)

    def test_complete_pagination_aggregates_unique_names_and_anonymous_counts(self):
        names, anonymous, chains = self.verify()
        self.assertEqual(names, {"pull": {"Alice", "Bob"}, "cavalier": set()})
        self.assertEqual(anonymous, {"pull": 2})
        self.assertEqual(set(chains), {"contributors-new-1"})

    def test_incomplete_pagination_and_mismatched_requests_are_rejected(self):
        with self.subTest("missing continuation"):
            continuation = self.responses[0].pop("continue")
            with self.assertRaises(AssertionError):
                self.verify()
            self.responses[0]["continue"] = continuation
        with self.subTest("empty continuation"):
            self.responses[0]["continue"] = {}
            with self.assertRaises(AssertionError):
                self.verify()
            self.responses[0]["continue"] = continuation
        with self.subTest("missing intermediate part"):
            terminal = self.parts[1]
            self.parts[1] = self.archive / "contributors-new-1-3.api.json"
            with self.assertRaises(AssertionError):
                self.verify()
            self.parts[1] = terminal
        with self.subTest("unfinished final part"):
            self.responses[1]["continue"] = continuation
            with self.assertRaises(AssertionError):
                self.verify()
            del self.responses[1]["continue"]
        with self.subTest("wrong next request token"):
            self.request = self.request.replace("10%7C20", "10%7C21")
            with self.assertRaises(AssertionError):
                self.verify()
        with self.subTest("missing next request token"):
            self.request = "https://fr.wiktionary.org/w/api.php?continue=%7C%7C"
            with self.assertRaises(KeyError):
                self.verify()


if __name__ == "__main__":
    unittest.main()
