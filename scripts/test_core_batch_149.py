"""Check #149 at the approved resource/admission boundary."""
import csv
import json
from pathlib import Path
from urllib.parse import parse_qs, urlparse
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-149"
MEMBERSHIP = [
    ("pénible", "ADJ"), ("délirer", "VER"), ("massacre", "NOM"),
    ("emménager", "VER"), ("onze", "ADJ:num"), ("loge", "NOM"),
    ("organe", "NOM"), ("zut", "ONO"), ("livraison", "NOM"),
    ("sien", "PRO:pos"), ("associer", "VER"), ("gendarme", "NOM"),
    ("jalousie", "NOM"), ("provenir", "VER"), ("violon", "NOM"),
    ("écriture", "NOM"), ("désordre", "NOM"), ("éclair", "NOM"),
    ("condamné", "NOM"), ("thérapie", "NOM"), ("casse", "NOM"),
    ("lesbien", "ADJ"), ("poignée", "NOM"), ("fierté", "NOM"),
    ("sagesse", "NOM"), ("sincèrement", "ADV"), ("user", "VER"),
    ("effectivement", "ADV"), ("choquer", "VER"), ("déesse", "NOM"),
    ("mairie", "NOM"), ("vedette", "NOM"), ("silencieux", "ADJ"),
    ("surpris", "ADJ"), ("vallée", "NOM"), ("cadre", "NOM"),
    ("aussitôt", "ADV"), ("délit", "NOM"), ("psychiatre", "NOM"),
    ("sous-sol", "NOM"), ("bouquin", "NOM"), ("défier", "VER"),
    ("menu", "NOM"), ("lady", "NOM"), ("boss", "NOM"),
    ("bracelet", "NOM"), ("supprimer", "VER"), ("adapter", "VER"),
    ("pneu", "NOM"), ("endormi", "ADJ"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch149Tests(unittest.TestCase):
    def test_menu_and_silencieux_have_actual_daf8_original_expression_evidence(self):
        originals = read("original-expression-review.json")
        by_title = {r["title"]: r for r in originals["sources"]}
        for title, article, expression in (
            ("silencieux", "A8S0941", "Il se dit aussi des Choses qui font peu de bruit."),
            ("menu", "A8M0869", "Il se dit aussi de la Liste détaillée des mets qui seront servis au cours du repas."),
        ):
            with self.subTest(title=title):
                record = by_title[title]
                self.assertEqual(record["selected_original_expression_decision"], "passed")
                original = record["original_comparison"]
                self.assertEqual(original["url"], "https://www.dictionnaire-academie.fr/article/" + article)
                self.assertEqual(original["original_expression"], expression)
                self.assertEqual(original["dictionary"], "academie8")
                self.assertTrue(Path(original["raw_path"]).is_file())
        self.assertEqual(originals["unresolved_sources"], [])

    def test_actual_independent_decisions_align_with_accepted_content_and_rights(self):
        review, rights, admission = read("field-review.json"), read("rights.json"), read("admission.json")
        self.assertEqual(review["kind"], "separate_independent_field_review")
        self.assertTrue(review["independence"])
        self.assertEqual(admission["independently_accepted_new_fields"], 200)
        self.assertEqual(admission["independently_accepted_new_groups"], 50)
        self.assertEqual(admission["pending_independent_rights_fields"], 0)
        self.assertEqual(admission["pending_owner_choices"], [])
        current = {(g["frozen_row"], k): f for g in read("groups.json")
                   for k, f in g["senses"][0]["fields"].items()}
        authored = {(g["frozen_row"], k): f for g in read("authored/groups.json")
                    for k, f in g["senses"][0]["fields"].items()}
        right_fields = {(f["frozen_row"], f["field"]): f for f in rights["fields"]}
        expected_pending = set()
        self.assertEqual({(f["frozen_row"], f["field"]) for f in review["unresolved_fields"]}, expected_pending)
        self.assertEqual(len(review["fields"]), 200)
        replacements = read("accepted-replacements.json")
        self.assertEqual({(r["frozen_row"], r["field"]) for r in replacements},
                         {(3407, "chinese"), (3429, "english"), (3429, "chinese"),
                          (3444, "french_definition"), (3444, "english"), (3444, "chinese")})
        for record in review["fields"]:
            key = record["frozen_row"], record["field"]
            with self.subTest(key=key):
                self.assertEqual(record["proposed_text"], authored[key]["text"])
                self.assertEqual(record["accepted_text"], current[key]["text"])
                self.assertEqual(record["accepted_text_quality_status"], "passed")
                self.assertEqual(right_fields[key]["accepted_text"], current[key]["text"])
                self.assertEqual(right_fields[key]["exact_source_span"], current[key]["source_span"])
                self.assertTrue(record["review_reason"].strip())
                self.assertTrue(record["rights_review_reason"].strip())
                self.assertEqual(record["reviewer"], review["reviewer"])
                self.assertEqual(record["rights_eligibility_status"], "passed_at_selected_text_boundary")
        uses = {u["sense_id"]: u for u in review["usage"]}
        self.assertEqual(len(uses), 50)
        for g in read("groups.json"):
            s = g["senses"][0]
            self.assertEqual(uses[s["sense_id"]]["accepted_usage"], s["use"])
            self.assertEqual(uses[s["sense_id"]]["accepted_usage_quality_status"], "passed")
            self.assertEqual(uses[s["sense_id"]]["everyday_sense_status"], "passed")
        self.assertEqual(len(review["spelling_relations"]), 50)
        self.assertTrue(all(r["independent_review"] == "passed" and r["alias_admission"] == "not_admitted"
                            for r in review["spelling_relations"]))

    def test_choquer_selected_offence_does_not_include_generic_surprise(self):
        g = next(g for g in read("groups.json") if g["frozen_row"] == 3429)
        fields = g["senses"][0]["fields"]
        self.assertEqual(fields["english"]["text"], "To offend someone (figurative transitive use).")
        self.assertEqual(fields["chinese"]["text"], "冒犯某人，使其感到不快（比喻义；及物动词）。")

    def test_lady_courtesy_title_has_the_correct_daughter_ranks(self):
        g = next(g for g in read("groups.json") if g["frozen_row"] == 3444)
        fields = g["senses"][0]["fields"]
        self.assertIn("ducs, marquis ou comtes", fields["french_definition"]["text"])
        self.assertIn("dukes, marquesses or earls", fields["english"]["text"])
        self.assertIn("公爵、侯爵或伯爵", fields["chinese"]["text"])

    def test_organ_meaning_does_not_require_an_internal_location(self):
        group = next(g for g in read("groups.json") if g["frozen_row"] == 3407)
        self.assertEqual(group["senses"][0]["fields"]["chinese"]["text"],
                         "器官；生物体的一个组成部分，构成一个整体并承担必要或有益的生命功能（阳性名词）。")

    def test_rights_and_independent_review_gates_are_explicit(self):
        rights = read("authored/rights.json")
        admission = read("authored/admission.json")
        review = read("authored/field-review.json")
        self.assertEqual(len(rights["fields"]), 200)
        self.assertEqual(rights["independent_rights_fields_accepted"], 0)
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual((admission["independently_accepted_new_groups"], admission["independently_accepted_new_fields"]), (0, 0))
        self.assertEqual(admission["pending_independent_review_fields"], 200)
        self.assertEqual(admission["pending_independent_rights_fields"], 200)
        self.assertEqual(admission["missing_admitted_fields"], 200)
        self.assertEqual(admission["unresolved_field_blockers"], [])
        self.assertFalse(review["independent_acceptance"])
        self.assertIsNone(review["reviewer"])
        expected = {(g["senses"][0]["sense_id"], name): field for g in read("authored/groups.json")
                    for name, field in g["senses"][0]["fields"].items()}
        self.assertEqual({(f["sense_id"], f["field"]) for f in rights["fields"]}, set(expected))
        for field in rights["fields"]:
            proposed = expected[field["sense_id"], field["field"]]
            self.assertEqual(field["exact_source_span"], proposed["source_span"])
            self.assertEqual(field["license"], "CC-BY-SA-4.0")
            self.assertTrue(field["attribution"])
            self.assertTrue(field["modification"])
            self.assertEqual(field["independent_review"], "pending")
            self.assertIsNone(field["reviewer"])
        for field in review["fields"]:
            self.assertEqual(field["accepted_text_quality_status"], "pending")
            self.assertIsNone(field["reviewer"])

    def test_exact_frozen_groups_have_complete_resource_records(self):
        if (EVIDENCE / "groups.json").exists():
            groups = read("groups.json")
        else:
            canonical = json.loads((ROOT / "resources/core-pilot-79/curated-inputs.json").read_text(encoding="utf-8"))
            groups = [g for g in canonical["groups"] if 3401 <= g["frozen_row"] <= 3450]
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP,
                         "The frozen #149 resource slice lacks complete batch content")
        self.assertEqual([g["frozen_row"] for g in groups], list(range(3401, 3451)))
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

    def test_original_proposals_and_actual_notices_accompany_content(self):
        for name in ("groups.json", "field-review.json", "rights.json", "sources.json",
                     "source-context.json", "source-excerpts.json", "admission.json", "reuse.json", "source-captures.json"):
            self.assertTrue((EVIDENCE / "authored" / name).is_file(), name)
        original = read("authored/groups.json")
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in original], MEMBERSHIP)
        notices = (EVIDENCE / "NOTICES.md").read_text(encoding="utf-8")
        self.assertEqual((EVIDENCE / "COPYING").read_bytes().rstrip(),
                         (ROOT / "resources/core-pilot-79/COPYING").read_bytes().rstrip())
        for source in read("sources.json")["sources"]:
            self.assertIn(source["fixed_url"], notices)
            self.assertIn(source["history_url"], notices)
        rights_text = (EVIDENCE / "selected-rights.md").read_text(encoding="utf-8")
        self.assertIn("2006-01-01", rights_text)
        self.assertIn("CC BY-SA 4.0", rights_text)
        self.assertIn("CFC", rights_text)

    def test_selected_fields_and_contributors_match_actual_fixed_captures(self):
        captures = read("source-captures.json")
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        contexts = read("source-context.json")["current_content"]
        archive = Path(captures["capture_root"])
        self.assertEqual(str(archive), read("admission.json")["author_evidence_root"])
        self.assertEqual((archive.name, archive.parent.name), ("issue-149", "author-evidence"))
        self.assertEqual(captures["original_capture_root"], str(archive))
        self.assertEqual(len(captures["sources"]), 53)
        raw = {}
        for capture in captures["sources"]:
            api_path, raw_path = Path(capture["api_path"]), Path(capture["raw_path"])
            self.assertTrue(api_path.is_relative_to(archive))
            self.assertTrue(raw_path.is_relative_to(archive))
            data = json.loads(api_path.read_bytes())
            page = next(p for p in data["query"]["pages"] if p["pageid"] == capture["pageid"])
            revision = next(r for r in page["revisions"] if r["revid"] == capture["revid"])
            body = revision["slots"]["main"]["content"]
            self.assertEqual(body.encode("utf-8"), raw_path.read_bytes())
            self.assertEqual((page["title"], revision["timestamp"]), (capture["title"], capture["timestamp"]))
            source = next(s for s in sources.values() if s["pageid"] == capture["pageid"])
            self.assertEqual(source["revid"], capture["revid"])
            self.assertEqual(parse_qs(urlparse(source["fixed_url"]).query)["oldid"], [str(capture["revid"])])
            raw[source["id"]] = body.splitlines()
        excerpts = read("source-excerpts.json")
        self.assertEqual(len(excerpts["fields"]), 200)
        for record in [*excerpts["fields"], *excerpts["supporting_evidence"]]:
            line = raw[record["source_id"]][record["line_1_based"] - 1]
            column = record["column_1_based"] - 1
            self.assertEqual(line[column:column + len(record["reviewed_excerpt"])], record["reviewed_excerpt"])
            self.assertNotRegex(record["reviewed_excerpt"].lstrip(), r"^#+\*")
            self.assertEqual(record["revid"], sources[record["source_id"]]["revid"])
        by_field = {(r["sense_id"], r["field"]): r for r in excerpts["fields"]}
        self.assertEqual(len(by_field), 200)
        pos_frames = {"NOM": "nom", "VER": "verbe", "ADJ": "adjectif", "ADV": "adverbe",
                      "ADJ:num": "adjectif numéral", "PRO:pos": "pronom possessif", "ONO": "interjection"}
        languages = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
        for group in read("groups.json"):
            sense = group["senses"][0]
            for name, field in sense["fields"].items():
                exact = by_field[sense["sense_id"], name]
                self.assertEqual(field["language"], languages[name])
                self.assertEqual(field["source_id"], exact["source_id"])
                self.assertEqual(field["source_span"]["raw_source_excerpt"], exact["reviewed_excerpt"])
                self.assertEqual(field["source_span"]["line_1_based"], exact["line_1_based"])
                preceding = raw[exact["source_id"]][:exact["line_1_based"]]
                language = next(line for line in reversed(preceding) if line.startswith("== "))
                self.assertRegex(language, r"^==\s*\{\{langue\|fr\}\}\s*==$")
                if name != "ipa":
                    pos = next(line for line in reversed(preceding) if line.startswith("=== {{S|"))
                    self.assertIn("{{S|" + pos_frames[group["lexique_cgram"]] + "|fr", pos)
                else:
                    self.assertEqual(field["text"].strip("/"), exact["ipa_variant_excerpt"])
                    column = exact["ipa_variant_column_1_based"] - 1
                    self.assertEqual(exact["reviewed_excerpt"][column:column+len(exact["ipa_variant_excerpt"])],
                                     exact["ipa_variant_excerpt"])
        names, anonymous, chains = {}, {}, {}
        for name in captures["contributor_registry"]["parts"]:
            path = Path(name)
            self.assertTrue(path.is_relative_to(archive))
            data = json.loads(path.read_bytes())
            prefix, part = path.name.removesuffix(".api.json").rsplit("-", 1)
            chains.setdefault(prefix, {})[int(part)] = (path, data)
            for page in data["query"]["pages"]:
                names.setdefault(page["title"], set()).update(p["name"] for p in page.get("contributors", []))
                if "anoncontributors" in page:
                    anonymous[page["title"]] = page["anoncontributors"]
        self.assertEqual(set(chains), {*(f"contributors-new-{n}" for n in range(1, 7)), "contributors-support"})
        for parts in chains.values():
            self.assertEqual(sorted(parts), list(range(1, len(parts) + 1)))
            for part, (path, data) in parts.items():
                continuation = data.get("continue")
                if part == len(parts):
                    self.assertFalse(continuation)
                else:
                    self.assertTrue(continuation)
                    request = parts[part+1][0].with_suffix(".request.txt").read_text(encoding="utf-8")
                    query = parse_qs(urlparse(request).query)
                    for key, value in continuation.items():
                        self.assertEqual(query[key], [str(value)])
        self.assertEqual(set(names), {s["title"] for s in sources.values()})
        for context in contexts:
            self.assertEqual(context["current_contributors"], sorted(names[context["title"]]))
            self.assertEqual(context["current_registry_names"], len(names[context["title"]]))
            self.assertEqual(context["anonymous_contributors_count"], anonymous.get(context["title"], 0))
        self.assertFalse(captures["contributor_registry"]["selected_sentence_authorship_proved"])

    def test_pronoun_numeral_and_participial_adjectives_keep_exact_use(self):
        groups = {g["frozen_row"]: g for g in read("groups.json")}
        support = read("source-excerpts.json")["supporting_evidence"]
        sien = groups[3410]["senses"][0]
        self.assertEqual((groups[3410]["headword"], groups[3410]["lexique_cgram"]), ("sien", "PRO:pos"))
        self.assertEqual(sien["source_headword"], "le sien")
        self.assertEqual(sien["fields"]["ipa"]["text"], "/sjɛ̃/")
        self.assertIn("le sien", sien["fields"]["french_definition"]["text"])
        self.assertIn("masculin singulier", sien["fields"]["french_definition"]["text"])
        self.assertIn("lui ou à elle", sien["fields"]["french_definition"]["text"])
        self.assertTrue(any(r["sense_id"] == sien["sense_id"] and "|ms=le sien" in r["reviewed_excerpt"] for r in support))
        onze = groups[3405]["senses"][0]
        self.assertIn("avant un nom", onze["fields"]["french_definition"]["text"])
        self.assertTrue(any(r["sense_id"] == onze["sense_id"] and "[[liaison]]" in r["reviewed_excerpt"]
                            and "après ''de'' et ''que''" in r["reviewed_excerpt"] for r in support))
        for row, headword in ((3434, "surpris"), (3450, "endormi")):
            self.assertEqual((groups[row]["headword"], groups[row]["lexique_cgram"]), (headword, "ADJ"))
            self.assertTrue(any(r["sense_id"] == groups[row]["senses"][0]["sense_id"]
                                and r["reviewed_excerpt"] == "=== {{S|adjectif|fr}} ===" for r in support))
        cadre_source = groups[3436]["senses"][0]["fields"]["french_definition"]["source_id"]
        selected = [*read("source-excerpts.json")["fields"], *support]
        self.assertFalse(any(r["source_id"] == cadre_source and r["line_1_based"] == 103 for r in selected))


if __name__ == "__main__":
    unittest.main()
