"""Check #144 through the spec-approved resource/admission boundary."""
import csv
import json
from pathlib import Path
from urllib.parse import parse_qs, urlparse
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-144"
MEMBERSHIP = [
    ("afin de", "PRE"), ("pouls", "NOM"), ("business", "NOM"),
    ("luxe", "NOM"), ("affirmer", "VER"), ("baser", "VER"),
    ("discret", "ADJ"), ("mentionner", "VER"), ("muscle", "NOM"),
    ("sienne", "PRO:pos"), ("sous-marin", "NOM"), ("tragédie", "NOM"),
    ("luire", "VER"), ("chirurgien", "NOM"), ("enchanter", "VER"),
    ("maîtriser", "VER"), ("vexer", "VER"), ("justifier", "VER"),
    ("botter", "VER"), ("brun", "ADJ"), ("département", "NOM"),
    ("soupçon", "NOM"), ("doubler", "VER"), ("parcourir", "VER"),
    ("filet", "NOM"), ("four", "NOM"), ("boiter", "VER"),
    ("enculer", "VER"), ("bonhomme", "NOM"), ("casque", "NOM"),
    ("traduire", "VER"), ("analyser", "VER"), ("dentiste", "NOM"),
    ("approuver", "VER"), ("individu", "NOM"), ("cabane", "NOM"),
    ("pipi", "NOM"), ("éléphant", "NOM"), ("fabuleux", "ADJ"),
    ("promenade", "NOM"), ("viol", "NOM"), ("ciao", "ONO"),
    ("fée", "NOM"), ("go", "NOM"), ("gage", "NOM"),
    ("plume", "NOM"), ("souper", "VER"), ("reconnaissance", "NOM"),
    ("suprême", "ADJ"), ("encourager", "VER"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch144Tests(unittest.TestCase):
    def test_luxe_evidence_selects_lexical_facts_instead_of_extended_exposition(self):
        group = next(g for g in read("groups.json") if g["headword"] == "luxe")
        sense = group["senses"][0]
        authored = next(g for g in read("authored/groups.json") if g["headword"] == "luxe")["senses"][0]
        for name in ("french_definition", "english", "chinese"):
            self.assertEqual(sense["fields"][name]["source_span"]["raw_source_excerpt"],
                             "[[dépense]]s [[somptuaire]]s et [[superflu]]es")
            self.assertEqual(sense["fields"][name]["text"], authored["fields"][name]["text"])
        support = [s["reviewed_excerpt"] for s in read("source-excerpts.json")["supporting_evidence"]
                   if s["sense_id"] == sense["sense_id"]]
        for fact in ("s’[[entourer]] d’un [[raffinement]] [[fastueux]]",
                     "pur [[goût]] de l’[[ostentation]]", "la [[strict]]e [[nécessité]]"):
            self.assertIn(fact, support)

    def test_ipa_modification_notices_describe_the_stored_notation(self):
        for group in read("groups.json"):
            field = group["senses"][0]["fields"]["ipa"]
            self.assertNotIn("slashes added", field["modification"])
            self.assertFalse(field["text"].startswith("/"))

    def test_fee_evidence_keeps_the_short_type_and_power_facts(self):
        sense = next(g for g in read("groups.json") if g["headword"] == "fée")["senses"][0]
        for name in ("french_definition", "english", "chinese"):
            self.assertEqual(sense["fields"][name]["source_span"]["raw_source_excerpt"], "Être [[imaginaire]]")
        support = [s["reviewed_excerpt"] for s in read("source-excerpts.json")["supporting_evidence"]
                   if s["sense_id"] == sense["sense_id"]]
        for fact in ("une [[puissance]] [[surnaturel]]le", "la [[tradition]] [[populaire]]",
                     "l’[[imagination]] des [[conteur]]s"):
            self.assertIn(fact, support)

    def test_reviewed_resource_has_joint_field_use_source_and_rights_acceptance(self):
        review = read("field-review.json")
        self.assertEqual(review["kind"], "separate_independent_field_review")
        self.assertTrue(review["independent_acceptance"])
        self.assertEqual(len(review["fields"]), 200)
        self.assertEqual(len(review["usage"]), 50)
        fields = {(f["sense_id"], f["field"]): f for f in review["fields"]}
        originals = {(f["sense_id"], f["field"]): f for f in read("authored/field-review.json")["fields"]}
        rights = {(f["sense_id"], f["field"]): f for f in read("rights.json")["fields"]}
        uses = {u["sense_id"]: u for u in review["usage"]}
        excerpts = {(f["sense_id"], f["field"]): f for f in read("source-excerpts.json")["fields"]}
        self.assertEqual(len(fields), 200)
        self.assertEqual(set(fields), set(rights))
        self.assertEqual(set(fields), set(excerpts))
        reviewer = review["reviewer"]
        self.assertIn("separate from author", reviewer)
        for group in read("groups.json"):
            sense = group["senses"][0]
            use = uses[sense["sense_id"]]
            self.assertEqual(use["accepted_usage"], sense["use"])
            self.assertEqual(use["accepted_usage_quality_status"], "passed")
            self.assertEqual(use["everyday_sense_status"], "passed")
            self.assertTrue(use["review_reason"].strip())
            self.assertEqual(sense["independent_review"], "passed_after_exact_accepted_replacements")
            for name, field in sense["fields"].items():
                key = sense["sense_id"], name
                entry, right = fields[key], rights[key]
                self.assertEqual(entry["proposed_text"], originals[key]["proposed_text"])
                self.assertEqual(entry["proposed_exact_source"], originals[key]["exact_source"])
                self.assertEqual(entry["accepted_text"], field["text"])
                self.assertEqual(entry["accepted_text_quality_status"], "passed")
                self.assertEqual(entry["rights_eligibility_status"], "passed_at_selected_text_boundary")
                self.assertEqual(right["independent_review"], "passed_at_selected_text_boundary")
                self.assertEqual(entry["reviewer"], reviewer)
                self.assertEqual(right["reviewer"], reviewer)
                self.assertTrue(entry["review_reason"].strip())
                self.assertTrue(entry["selected_text_rights_reason"].strip())
                exact = entry["exact_source"]
                for column in ("source_id", "revid", "line_1_based", "column_1_based", "reviewed_excerpt"):
                    self.assertEqual(exact[column], excerpts[key][column])
                self.assertEqual(field["source_span"]["column_1_based"], exact["column_1_based"])
                self.assertEqual(field["source_span"]["raw_source_excerpt"], exact["reviewed_excerpt"])
                self.assertEqual(right["exact_source_span"]["raw_source_excerpt"], exact["reviewed_excerpt"])
        for source in read("sources.json")["sources"]:
            self.assertEqual(source["selected_text_review"], "passed_for_selected_spans")
            for layer in source["original_historical_dictionary_layers"]:
                self.assertFalse(layer["worldwide_expression_clearance"])
                self.assertEqual(layer["territorial_basis"], "France")
        admission = read("admission.json")
        self.assertEqual((admission["independently_accepted_new_groups"], admission["independently_accepted_new_fields"]), (50, 200))
        self.assertEqual((admission["changed_text_fields"], admission["changed_source_span_fields"], admission["changed_modification_notice_fields"]), (0, 6, 50))
        self.assertEqual((admission["missing_admitted_fields"], admission["pending_canonical_fields"], admission["newly_admitted_fields"]), (0, 200, 0))
        self.assertEqual((admission["annotation_exception_groups"], admission["specialist_exception_groups"]), (0, 0))
        self.assertEqual(len(read("accepted-replacements.json")), 56)

    def test_exact_frozen_groups_have_complete_resource_records(self):
        path = EVIDENCE / "groups.json"
        if path.exists():
            groups = json.loads(path.read_text(encoding="utf-8"))
        else:
            canonical = json.loads((ROOT / "resources/core-pilot-79/curated-inputs.json").read_text(encoding="utf-8"))
            groups = [g for g in canonical["groups"] if 3151 <= g["frozen_row"] <= 3200]
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP,
                         "The frozen #144 resource slice lacks complete batch content")
        self.assertEqual([g["frozen_row"] for g in groups], list(range(3151, 3201)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(encoding="utf-8-sig", newline="") as handle:
            frozen = list(csv.DictReader(handle))
        self.assertEqual(len(frozen), 4087)
        for group in groups:
            for frequency in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                self.assertEqual(group[frequency], frozen[group["frozen_row"] - 1][frequency])
            self.assertTrue(group["senses"])
            for sense in group["senses"]:
                self.assertTrue(sense["use"].strip())
                self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                for field in sense["fields"].values():
                    self.assertTrue(field["text"].strip())

    def test_fixed_field_and_grammar_spans_have_complete_contributor_chains(self):
        captures = read("source-captures.json")
        archive = Path(captures["capture_root"])
        self.assertEqual(archive.name, "issue-144")
        self.assertEqual(archive.parent.name, "author-evidence")
        self.assertEqual(len(captures["sources"]), 52)

        def retained(name):
            return archive.joinpath(*Path(name).parts[2:])

        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        raw = {}
        for capture in captures["sources"]:
            data = json.loads(retained(capture["api_path"]).read_text(encoding="utf-8"))
            page = next(p for p in data["query"]["pages"] if p["pageid"] == capture["pageid"])
            revision = next(r for r in page["revisions"] if r["revid"] == capture["revid"])
            body = revision["slots"]["main"]["content"]
            self.assertEqual(body.encode("utf-8"), retained(capture["raw_path"]).read_bytes())
            self.assertEqual(revision["timestamp"], capture["timestamp"])
            source = next(s for s in sources.values() if s["pageid"] == page["pageid"])
            self.assertEqual(source["revid"], revision["revid"])
            self.assertEqual(parse_qs(urlparse(source["fixed_url"]).query)["oldid"], [str(revision["revid"])])
            raw[source["id"]] = body.splitlines()
        excerpts = read("source-excerpts.json")
        self.assertEqual(len(excerpts["fields"]), 200)
        for span in [*excerpts["fields"], *excerpts["supporting_evidence"]]:
            line = raw[span["source_id"]][span["line_1_based"] - 1]
            start = span["column_1_based"] - 1
            self.assertEqual(line[start:start + len(span["reviewed_excerpt"])], span["reviewed_excerpt"])
            self.assertFalse(span["reviewed_excerpt"].lstrip().startswith("#*"))
            self.assertNotIn("[[Image:", span["reviewed_excerpt"])
            self.assertNotIn("[[Fichier:", span["reviewed_excerpt"])
        fields = {(s["sense_id"], s["field"]): s for s in excerpts["fields"]}
        classes = {"PRE": "préposition", "NOM": "nom", "VER": "verbe", "ADJ": "adjectif",
                   "PRO:pos": "pronom possessif", "ONO": "interjection"}
        languages = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
        for group in read("groups.json"):
            sense = group["senses"][0]
            support = [s for s in excerpts["supporting_evidence"] if s["sense_id"] == sense["sense_id"]]
            self.assertTrue(any(s["reviewed_excerpt"].replace(" ", "") == "=={{langue|fr}}==" for s in support))
            self.assertTrue(any("{{S|" + classes[group["lexique_cgram"]] + "|fr" in s["reviewed_excerpt"] for s in support))
            for name, field in sense["fields"].items():
                exact = fields[sense["sense_id"], name]
                self.assertEqual(field["source_id"], exact["source_id"])
                self.assertEqual(field["source_span"]["raw_source_excerpt"], exact["reviewed_excerpt"])
                self.assertEqual(field["language"], languages[name])
                self.assertEqual(field["content_license"], "CC-BY-SA-4.0")
                if name == "ipa":
                    self.assertEqual(field["text"].strip("/"), exact["ipa_variant_excerpt"])
                    start = exact["ipa_variant_column_1_based"] - 1
                    self.assertEqual(exact["reviewed_excerpt"][start:start + len(exact["ipa_variant_excerpt"])], exact["ipa_variant_excerpt"])
        chains, names, anonymous = {}, {}, {}
        for name in captures["contributor_registry"]["parts"]:
            path = retained(name)
            data = json.loads(path.read_text(encoding="utf-8"))
            prefix, part = path.name.removesuffix(".api.json").rsplit("-", 1)
            chains.setdefault(prefix, {})[int(part)] = (path, data)
            for page in data["query"]["pages"]:
                names.setdefault(page["title"], set()).update(p["name"] for p in page.get("contributors", []))
                if "anoncontributors" in page:
                    anonymous[page["title"]] = page["anoncontributors"]
        self.assertEqual(set(chains), {"contributors", "contributors-support"})
        for parts in chains.values():
            self.assertEqual(sorted(parts), list(range(1, len(parts) + 1)))
            for part, (_, data) in parts.items():
                continuation = data.get("continue")
                if part == len(parts):
                    self.assertFalse(continuation)
                else:
                    query = parse_qs(urlparse(parts[part + 1][0].with_suffix(".request.txt").read_text(encoding="utf-8")).query)
                    for key, value in continuation.items():
                        self.assertEqual(query[key], [str(value)])
        for context in read("source-context.json")["current_content"]:
            self.assertEqual(context["current_contributors"], sorted(names[context["title"]]))
            self.assertEqual(context["anonymous_contributors_count"], anonymous.get(context["title"], 0))
        self.assertFalse(captures["contributor_registry"]["selected_sentence_authorship_proved"])

    def test_original_author_proposals_keep_quality_rights_and_admission_pending(self):
        admission = read("authored/admission.json")
        review = read("authored/field-review.json")
        rights = read("authored/rights.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual((admission["independently_accepted_new_groups"], admission["independently_accepted_new_fields"]), (0, 0))
        self.assertEqual((admission["pending_independent_review_fields"], admission["pending_independent_rights_fields"]), (200, 200))
        self.assertEqual(admission["missing_admitted_fields"], 200)
        self.assertFalse(review["independent_acceptance"])
        self.assertIsNone(review["reviewer"])
        self.assertEqual(len(rights["fields"]), 200)
        for field in review["fields"]:
            self.assertEqual(field["accepted_text_quality_status"], "pending")
            self.assertIsNone(field["reviewer"])
        for field in rights["fields"]:
            self.assertEqual(field["independent_review"], "pending")
            self.assertIsNone(field["reviewer"])

    def test_unselected_external_definitions_are_not_redistributed_as_reference_text(self):
        rights = read("rights.json")
        for source in rights["sources"]:
            for marker in source["reference_markers"]:
                self.assertFalse(marker["raw_reference"].startswith("#"),
                                 "Unselected definition prose must stay in the external raw archive")
        net = next(s for s in rights["sources"] if s["title"] == "filet")
        cfc = next(m for m in net["reference_markers"] if "Import:CFC" in m["raw_reference"])
        self.assertEqual(cfc["raw_reference"], "{{Import:CFC}}")
        self.assertEqual(cfc["line_1_based"], 93)
        self.assertEqual(cfc["author_rights_proposal"], "excluded_from_selected_content")
        self.assertFalse(any(s["line_1_based"] == 93 for s in net["selected_locations"]))


if __name__ == "__main__":
    unittest.main()
