"""Check #134 at the approved resource/admission seam."""
import csv
import json
from pathlib import Path
from urllib.parse import parse_qs, urlparse
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-134"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
# Exact word/subclass membership from ticket #134, rows 2651–2700.
MEMBERSHIP = [
    ("disposer", "VER"), ("publier", "VER"), ("récemment", "ADV"),
    ("grimper", "VER"), ("naturellement", "ADV"), ("complice", "NOM"),
    ("tache", "NOM"), ("réjouir", "VER"), ("draguer", "VER"),
    ("réclamer", "VER"), ("appétit", "NOM"), ("organisation", "NOM"),
    ("marin", "NOM"), ("comportement", "NOM"), ("donnée", "NOM"),
    ("scandale", "NOM"), ("orange", "NOM"), ("assis", "ADJ"),
    ("lion", "NOM"), ("lesquels", "PRO:int"), ("lesquels", "PRO:rel"),
    ("baron", "NOM"), ("tomate", "NOM"), ("congé", "NOM"),
    ("nègre", "NOM"), ("cardiaque", "ADJ"), ("musée", "NOM"),
    ("tester", "VER"), ("bijoux", "NOM"), ("occupé", "ADJ"),
    ("piper", "VER"), ("arrestation", "NOM"), ("maximum", "NOM"),
    ("reconnaissant", "ADJ"), ("tournée", "NOM"), ("diamant", "NOM"),
    ("tradition", "NOM"), ("génération", "NOM"), ("voile", "NOM"),
    ("interne", "ADJ"), ("plaque", "NOM"), ("éducation", "NOM"),
    ("commandement", "NOM"), ("renseigner", "VER"), ("vieillir", "VER"),
    ("rythme", "NOM"), ("ennuyeux", "ADJ"), ("texte", "NOM"),
    ("développer", "VER"), ("légende", "NOM"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch134Tests(unittest.TestCase):
    def test_ghostwriter_replacement_has_independent_quality_and_selected_rights(self):
        sense = next(g for g in read("groups.json") if g["frozen_row"] == 2675)["senses"][0]
        expected = {
            "french_definition": "Au figuré, écrivain payé pour écrire un livre qu’une autre personne signe de son nom.",
            "ipa": "/nɛɡʁ/",
            "english": "In figurative use, a paid ghostwriter who writes a book credited to another person as its author. Masculine noun.",
            "chinese": "转义：受雇替他人写书、由对方署名的写作者，即代笔者。阳性名词。",
        }
        self.assertEqual({k: f["text"] for k, f in sense["fields"].items()}, expected)
        proposal = read("authored/coordinator-ghostwriter-proposal.json")
        decisions = read("ghostwriter-review.json")
        self.assertNotEqual(decisions["reviewer"], decisions["alternative_author"])
        self.assertNotEqual(decisions["reviewer"], decisions["original_author"])
        self.assertEqual(decisions["quality_status"], "passed")
        self.assertEqual(decisions["rights_status"], "passed_at_selected_text_boundary")
        self.assertEqual(decisions["selected_lines"], [3, 13, 14, 15, 56])
        self.assertEqual(decisions["original_proposal_preservation"], "authored/original-rejected-2675.json")
        replacements = read("accepted-replacements.json")
        self.assertEqual(len(replacements), 4)
        originals = read("authored/original-rejected-2675.json")
        for replacement in replacements:
            name = replacement["field"]
            self.assertEqual(replacement["proposal_id"], decisions["proposal_id"])
            self.assertEqual(replacement["coordinator_proposed_text"], proposal["fields"][name])
            self.assertEqual(replacement["proposed_text"], originals["group"]["senses"][0]["fields"][name]["text"])
            self.assertEqual(replacement["accepted_text"], expected[name])
            self.assertEqual(sense["fields"][name]["source_span"]["line_1_based"], 15 if name == "ipa" else 56)
        self.assertEqual(decisions["source_audit"]["raw_api_equal"], True)
        self.assertEqual(decisions["source_audit"]["contributor_continuation_complete"], True)
        self.assertEqual(decisions["selected_rights_boundary"]["initial_daf8_revision"], 434860)
        self.assertEqual(decisions["selected_rights_boundary"]["writer_addition_revision"], 448299)
        self.assertEqual(decisions["selected_rights_boundary"]["writer_addition_contributor"], "Ericdec")
        self.assertFalse(decisions["selected_rights_boundary"]["tlfi_wording_selected"])
        self.assertFalse(decisions["selected_rights_boundary"]["robert_wording_selected"])

    def test_independent_acceptance_matches_exact_fields_sources_and_replacement_use(self):
        review = read("field-review.json")
        admission = read("admission.json")
        self.assertEqual(review["kind"], "separate_independent_field_review")
        self.assertTrue(review["independence"])
        self.assertEqual((len(review["fields"]), len(review["usage"]),
                          len(review["spelling_relations"])), (200, 50, 50))
        self.assertTrue(review["batch_complete"])
        self.assertEqual((admission["independently_accepted_new_groups"],
                          admission["independently_accepted_new_fields"]), (50, 200))
        self.assertEqual(admission["pending_independent_review_rows"], [])
        self.assertEqual(admission["annotation_exception_groups"], 0)
        self.assertEqual(admission["specialist_exception_groups"], 0)
        self.assertEqual(admission["newly_admitted_fields"], 0)
        self.assertEqual(admission["missing_admitted_fields"], 200)
        self.assertEqual((admission["pending_independent_review_fields"],
                          admission["pending_independent_rights_fields"]), (0, 0))
        originals = {(s["sense_id"], k): f for g in read("authored/groups.json")
                     for s in g["senses"] for k, f in s["fields"].items()}
        fields = {(s["sense_id"], k): f for g in read("groups.json")
                  for s in g["senses"] for k, f in s["fields"].items()}
        decisions = {(r["sense_id"], r["field"]): r for r in review["fields"]}
        rights = {(r["sense_id"], r["field"]): r for r in read("rights.json")["fields"]}
        self.assertEqual(set(fields), set(originals))
        self.assertEqual(set(fields), set(decisions))
        self.assertEqual(set(fields), set(rights))
        self.assertEqual(len(fields), 200)
        self.assertEqual(len(read("accepted-replacements.json")), 4)
        contexts = {c["id"]: c for c in read("source-context.json")["current_content"]}
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        accepted_source_ids = set()
        for key, field in fields.items():
            decision, legal = decisions[key], rights[key]
            self.assertEqual(decision["proposed_text"], originals[key]["text"])
            if decision["frozen_row"] != 2675:
                self.assertEqual(field["text"], originals[key]["text"])
            self.assertEqual(decision["accepted_text"], field["text"])
            self.assertEqual(legal["accepted_text"], field["text"])
            self.assertEqual(decision["exact_source"]["source_id"], field["source_id"])
            self.assertEqual(decision["exact_source"]["line_1_based"], field["source_span"]["line_1_based"])
            self.assertEqual(decision["exact_source"]["reviewed_excerpt"], field["source_span"]["raw_source_excerpt"])
            self.assertTrue(decision["review_reason"].strip())
            reviewer = admission["ghostwriter_reviewer"] if decision["frozen_row"] == 2675 else admission["independent_reviewer"]
            self.assertEqual(decision["reviewer"], reviewer)
            accepted_source_ids.add(field["source_id"])
            self.assertEqual(field["independent_review"], "passed")
            self.assertEqual(decision["accepted_text_quality_status"], "passed")
            self.assertEqual(legal["independent_review"], "passed_at_selected_text_boundary")
            self.assertTrue(legal["selected_text_rights_reason"].strip())
        self.assertEqual(len(accepted_source_ids), 51)
        for sid, context in contexts.items():
            expected = "passed_at_selected_text_boundary" if sid in accepted_source_ids else "pending_separate_selected_span_audit"
            self.assertEqual(context["selected_text_rights_status"], expected)
            self.assertEqual(sources[sid]["selected_text_review"], "passed_for_selected_spans" if sid in accepted_source_ids else expected)
            if sid not in accepted_source_ids:
                self.assertEqual(context["status"], "unresolved_selected_use_not_admitted")
        senses = {g["frozen_row"]: g["senses"][0] for g in read("groups.json")}
        for row, sense in senses.items():
            self.assertEqual(sense["independent_review"], "passed_after_exact_accepted_replacements")
        for use in review["usage"]:
            self.assertEqual(use["accepted_usage"], senses[use["frozen_row"]]["use"])
            self.assertEqual(use["accepted_usage"], use["coordinator_proposed_usage"] if use["frozen_row"] == 2675 else use["proposed_usage"])
            self.assertEqual(use["accepted_usage_quality_status"], "passed")
            self.assertEqual(use["everyday_sense_status"], "passed")
            self.assertTrue(use["review_reason"].strip())
        for relation in review["spelling_relations"]:
            self.assertEqual(relation["alias_admission"], "not_admitted")
            self.assertTrue(relation["review_reason"].strip())

    def test_exact_frozen_groups_have_complete_resource_records(self):
        if (EVIDENCE / "groups.json").exists():
            groups = read("groups.json")
        else:
            canonical = json.loads((ROOT / "resources/core-pilot-79/curated-inputs.json").read_text(encoding="utf-8"))
            groups = [g for g in canonical["groups"] if 2651 <= g["frozen_row"] <= 2700]
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP,
                         "The frozen #134 resource slice lacks complete batch content")
        self.assertEqual([g["frozen_row"] for g in groups], list(range(2651, 2701)))
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
                self.assertEqual(set(sense["fields"]), set(LANGUAGES))
                for name, field in sense["fields"].items():
                    self.assertEqual(field["language"], LANGUAGES[name])
                    self.assertTrue(field["text"].strip())
                    self.assertTrue(field["modification"].strip())

    def test_fields_and_constructions_bind_to_retained_french_sources(self):
        captures = read("source-captures.json")
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        contexts = {s["id"]: s for s in read("source-context.json")["current_content"]}
        raw = {}
        capture_root = Path(captures["capture_root"])
        self.assertEqual(str(capture_root), read("admission.json")["author_evidence_root"])
        self.assertEqual(capture_root.name, "issue-134")
        self.assertEqual(capture_root.parent.name, "author-evidence")

        def retained(name):
            return capture_root.joinpath(*Path(name).parts[2:])

        self.assertEqual(len(captures["sources"]), 51)
        for capture in captures["sources"]:
            data = json.loads(retained(capture["api_path"]).read_text(encoding="utf-8-sig"))
            page = next(p for p in data["query"]["pages"] if p["pageid"] == capture["pageid"])
            revision = next(r for r in page["revisions"] if r["revid"] == capture["revid"])
            body = revision["slots"]["main"]["content"]
            self.assertEqual(page["title"], capture["title"])
            self.assertEqual(revision["timestamp"], capture["timestamp"])
            self.assertEqual(body, retained(capture["raw_path"]).read_text(encoding="utf-8"))
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
            self.assertFalse(record["reviewed_excerpt"].lstrip().startswith("#*"))
        by_field = {(r["sense_id"], r["field"]): r for r in excerpts["fields"]}
        for group in read("groups.json"):
            sense = group["senses"][0]
            for name, field in sense["fields"].items():
                exact = by_field[sense["sense_id"], name]
                self.assertEqual(field["source_id"], exact["source_id"])
                self.assertEqual(field["source_span"]["raw_source_excerpt"], exact["reviewed_excerpt"])
                self.assertEqual(field["source_span"]["line_1_based"], exact["line_1_based"])
                self.assertEqual(field["content_license"], "CC-BY-SA-4.0")
                if name == "ipa":
                    self.assertEqual(field["text"].strip("/"), exact["ipa_variant_excerpt"])
        contributor_names = {}
        anonymous_counts = {}
        chains = {}
        for name in captures["contributor_registry"]["parts"]:
            path = retained(name)
            data = json.loads(path.read_text(encoding="utf-8"))
            prefix, part = path.name.removesuffix(".api.json").rsplit("-", 1)
            chains.setdefault(prefix, {})[int(part)] = (path, data)
            for page in data["query"]["pages"]:
                contributor_names.setdefault(page["title"], set()).update(p["name"] for p in page.get("contributors", []))
                if "anoncontributors" in page:
                    anonymous_counts[page["title"]] = page["anoncontributors"]
        self.assertEqual(set(chains), {"contributors", "contributors-support"})
        for parts in chains.values():
            self.assertEqual(sorted(parts), list(range(1, len(parts) + 1)))
            for part, (path, data) in parts.items():
                continuation = data.get("continue")
                if part == len(parts):
                    self.assertFalse(continuation)
                else:
                    query = parse_qs(urlparse(parts[part + 1][0].with_suffix(".request.txt").read_text(encoding="utf-8")).query)
                    for key, value in continuation.items():
                        self.assertEqual(query[key], [str(value)])
        for context in contexts.values():
            self.assertEqual(context["current_contributors"], sorted(contributor_names[context["title"]]))
            self.assertEqual(context["current_registry_names"], len(contributor_names[context["title"]]))
            self.assertEqual(context["anonymous_contributors_count"], anonymous_counts.get(context["title"], 0))
        self.assertFalse(captures["contributor_registry"]["selected_sentence_authorship_proved"])

    def test_original_proposals_keep_review_and_admission_pending(self):
        admission = read("authored/admission.json")
        review = read("authored/field-review.json")
        rights = read("authored/rights.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual((admission["independently_accepted_new_groups"], admission["independently_accepted_new_fields"]), (0, 0))
        self.assertEqual((admission["pending_independent_review_fields"], admission["pending_independent_rights_fields"]), (200, 200))
        self.assertEqual(admission["missing_admitted_fields"], 200)
        self.assertEqual((admission["reused_groups"], admission["reused_fields"]), (0, 0))
        self.assertEqual(admission["unresolved_field_blockers"], [])
        self.assertFalse(review["independent_acceptance"])
        self.assertIsNone(review["reviewer"])
        self.assertEqual(len(review["fields"]), 200)
        self.assertEqual(len(review["usage"]), 50)
        self.assertEqual(len(review["spelling_relations"]), 50)
        self.assertEqual(len(rights["fields"]), 200)
        for field in review["fields"]:
            self.assertEqual(field["accepted_text_quality_status"], "pending")
            self.assertIsNone(field["reviewer"])
        for field in rights["fields"]:
            self.assertEqual(field["independent_review"], "pending")
            self.assertIsNone(field["reviewer"])
        for relation in review["spelling_relations"]:
            self.assertEqual(relation["alias_admission"], "not_admitted")

    def test_plural_register_and_pronunciation_boundaries_are_explicit(self):
        groups = {g["frozen_row"]: g["senses"][0] for g in read("groups.json")}
        self.assertEqual(groups[2670]["source_headword"], "lequel")
        self.assertEqual(groups[2671]["source_headword"], "lequel")
        self.assertEqual(groups[2679]["source_headword"], "bijou")
        self.assertEqual(groups[2682]["fields"]["ipa"]["text"], "/a.ʁɛs.ta.sjɔ̃/")
        self.assertEqual(groups[2681]["fields"]["ipa"]["text"], "/pi.pe/")
        for language, restrictions in [("french_definition", ("Au figuré", "payé", "livre", "signe de son nom")),
                                       ("english", ("figurative", "paid ghostwriter", "credited", "author", "Masculine noun")),
                                       ("chinese", ("转义", "受雇", "写书", "署名", "阳性名词"))]:
            for restriction in restrictions:
                self.assertIn(restriction, groups[2675]["fields"][language]["text"])
        legend = next(s for s in read("rights.json")["sources"] if s["title"] == "légende")
        cfc = next(m for m in legend["reference_markers"] if "Import:CFC" in m["raw_reference"])
        self.assertEqual(cfc["author_rights_proposal"], "excluded_from_selected_content")
        self.assertFalse(any(r["line_1_based"] == 18 for r in legend["selected_locations"]))
        self.assertEqual((EVIDENCE / "COPYING").read_bytes().rstrip(), (ROOT / "resources/core-pilot-79/COPYING").read_bytes().rstrip())


if __name__ == "__main__":
    unittest.main()
