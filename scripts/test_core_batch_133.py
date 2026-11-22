"""Check the independently accepted #133 shard at the resource/admission seam."""
import csv
import json
from pathlib import Path
import re
from urllib.parse import parse_qs, urlparse
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-133"
MEMBERSHIP = [
    ("circonstance", "NOM"), ("salade", "NOM"), ("salopard", "NOM"),
    ("cercle", "NOM"), ("piéger", "VER"), ("chauffer", "VER"),
    ("gouverneur", "NOM"), ("garce", "NOM"), ("cuire", "VER"),
    ("anneau", "NOM"), ("cependant", "ADV"), ("paire", "NOM"),
    ("pré", "NOM"), ("fleuve", "NOM"), ("domaine", "NOM"),
    ("soulager", "VER"), ("immense", "ADJ"), ("incident", "NOM"),
    ("essentiel", "NOM"), ("mercredi", "NOM"), ("pourvoir", "VER"),
    ("placard", "NOM"), ("fermé", "ADJ"), ("fan", "NOM"),
    ("jugement", "NOM"), ("r", "ADV"), ("magazine", "NOM"),
    ("estimer", "VER"), ("pile", "NOM"), ("renfort", "NOM"),
    ("relâcher", "VER"), ("réputation", "NOM"), ("logique", "ADJ"),
    ("poubelle", "NOM"), ("for", "NOM"), ("degré", "NOM"),
    ("tension", "NOM"), ("règlement", "NOM"), ("assiette", "NOM"),
    ("craquer", "VER"), ("serment", "NOM"), ("chanteur", "NOM"),
    ("clinique", "NOM"), ("célibataire", "ADJ"), ("merveille", "NOM"),
    ("décoller", "VER"), ("ingénieur", "NOM"), ("réserve", "NOM"),
    ("autorisation", "NOM"), ("voyou", "NOM"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


def retained_path(name):
    parts = name.replace("\\", "/").split("/")
    local = ROOT.joinpath(*parts)
    if local.is_file():
        return local
    admission = read("admission.json")
    reviewed = Path(admission["reviewer_evidence_root"]).joinpath("author-cache", *parts[2:])
    if reviewed.is_file():
        return reviewed
    return Path(admission["author_evidence_root"]).joinpath(*parts[2:])


class Batch133Tests(unittest.TestCase):
    def test_only_r_adv_has_the_recorded_owner_annotation(self):
        annotations = [g for g in read("groups.json") if not g["senses"]]
        self.assertEqual(len(annotations), 1)
        group = annotations[0]
        self.assertEqual((group["frozen_row"], group["headword"], group["lexique_cgram"]),
                         (2626, "r", "ADV"))
        self.assertEqual(group["status"], "approved_annotation_exception")
        self.assertEqual((group["admitted_fields"], group["missing_fields"]), (0, 4))
        approval = group["owner_approval"]
        self.assertEqual((approval["by"], approval["date"], approval["issue"]),
                         ("project_owner", "2026-11-21", 133))
        self.assertEqual(approval["exact_decision"], "批准仅 r/ADV 为零字段注记")
        self.assertEqual(group["alias_admission"], "not_admitted")
        self.assertFalse(read("authored/unresolved.json")["groups"][0]["approved_exception"])

    def test_canonical_annotation_ledger_keeps_owner_approval_and_four_missing_fields(self):
        review = read("field-review.json")
        self.assertNotIn("annotations", review.keys())
        annotations = review["annotation_exceptions"]
        self.assertEqual(len(annotations), 1)
        annotation = annotations[0]
        self.assertEqual((annotation["frozen_row"], annotation["headword"], annotation["pos"],
                          annotation["admitted_fields"], annotation["missing_fields"]),
                         (2626, "r", "ADV", 0, 4))
        decision = annotation["decision"]
        self.assertEqual((decision["issue"], decision["approved_by"], decision["approved_on"],
                          decision["decision"]),
                         ("https://github.com/zardonc/lexique/issues/133", "project_owner",
                          "2026-11-21", "批准仅 r/ADV 为零字段注记"))
        group = next(g for g in read("groups.json") if g["frozen_row"] == 2626)
        self.assertEqual(annotation["owner_approval"], group["owner_approval"])
        missing = review["unresolved_fields"]
        self.assertEqual(len(missing), 4)
        self.assertEqual({(r["frozen_row"], r["headword"], r["pos"], r["field"], r["language"])
                          for r in missing},
                         {(2626, "r", "ADV", field, language) for field, language in
                          (("french_definition", "fr"), ("ipa", "und"),
                           ("english", "en"), ("chinese", "zh"))})
        original = read("authored/unresolved.json")["groups"][0]
        for record in missing:
            self.assertEqual(record["status"], "unresolved")
            self.assertEqual(record["reason"], original["reason"])
        self.assertFalse(any(r["frozen_row"] == 2626 for r in review["fields"]))

    def test_canonical_source_context_carries_actual_selected_rights_verdicts(self):
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        rights = {s["source_id"]: s for s in read("rights.json")["sources"]}
        contexts = read("source-context.json")["current_content"]
        admitted = {field["source_id"] for group in read("groups.json")
                    for sense in group["senses"] for field in sense["fields"].values()}
        self.assertEqual(len(admitted), 50)
        self.assertEqual(len(contexts), 51)
        self.assertEqual({s["id"] for s in contexts}, set(sources))
        self.assertEqual(set(rights), set(sources))
        for context in contexts:
            with self.subTest(source=context["id"]):
                source = sources[context["id"]]
                verdict = rights[context["id"]]
                expected = ("passed_at_selected_text_boundary" if context["id"] in admitted
                            else "inspected_no_admitted_content")
                self.assertEqual(verdict["independent_review"], expected)
                self.assertEqual(context["selected_text_rights_status"], verdict["independent_review"])
                self.assertEqual(context["independent_review"], verdict["independent_review"])
                self.assertEqual(context["reviewer"], verdict["reviewer"])
                self.assertEqual(context["selected_text_rights_reason"], verdict["review_reason"])
                self.assertTrue(verdict["review_reason"])
                if context["id"] in admitted:
                    self.assertEqual(source["selected_text_review"], "passed_for_selected_spans")
                    self.assertTrue(context["selected_locations"])
                else:
                    self.assertEqual(source["title"], "r")
                    self.assertEqual(source["selected_text_review"], "inspected_no_admitted_content")
                    self.assertEqual(context["selected_locations"], [])
                    self.assertEqual(context["selected_grammar_and_construction_evidence"], [])

    def test_exact_frozen_groups_have_accounted_resource_content(self):
        self.assertTrue((EVIDENCE / "groups.json").is_file(),
                        "No #133 resource admission/content records for frozen rows 2601–2650")
        groups = read("groups.json")
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        self.assertEqual([g["frozen_row"] for g in groups], list(range(2601, 2651)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
                encoding="utf-8-sig", newline="") as handle:
            frozen = list(csv.DictReader(handle))
        self.assertEqual(len(frozen), 4087)
        for group in groups:
            row = frozen[group["frozen_row"] - 1]
            self.assertEqual((group["headword"], group["lexique_cgram"]),
                             (row["word"], row["lexique_cgram"]))
            for key in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                self.assertEqual(group[key], row[key])
        admission = read("admission.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)

    def test_accepted_fields_keep_exact_proposals_and_separate_canonical_gates(self):
        groups = read("groups.json")
        authored = read("authored/groups.json")
        self.assertEqual(authored, read("author-proposals.json")["groups"])
        candidates = [g for g in groups if g["senses"]]
        self.assertEqual(len(candidates), 49)
        self.assertEqual([g["frozen_row"] for g in groups if not g["senses"]], [2626])
        unresolved = read("authored/unresolved.json")["groups"][0]
        self.assertEqual((unresolved["headword"], unresolved["lexique_cgram"]), ("r", "ADV"))
        self.assertFalse(unresolved["approved_exception"])
        self.assertEqual(set(unresolved["missing_fields"]),
                         {"french_definition", "ipa", "english", "chinese"})
        self.assertEqual(read("unresolved.json")["groups"], [])
        self.assertEqual(read("unresolved.json")["ordinary_unresolved_groups"], 0)
        admission = read("admission.json")
        for key, value in {
            "frozen_groups": 50, "proposed_groups": 49, "proposed_fields": 196,
            "missing_authored_fields": 4, "missing_admitted_fields": 4,
            "pending_independent_review_fields": 0, "pending_independent_rights_fields": 0,
            "independently_accepted_new_groups": 49, "independently_accepted_new_fields": 196,
            "annotation_exception_groups": 1, "specialist_exception_groups": 0,
            "annotation_missing_fields": 4, "ordinary_missing_fields": 0,
            "newly_admitted_fields": 0, "canonical_newly_admitted_fields": 0,
            "canonical_pending_fields": 196, "canonical_missing_fields": 200,
            "unchanged_accepted_fields": 196, "changed_accepted_fields": 0,
        }.items():
            self.assertEqual(admission[key], value, key)
        self.assertEqual(admission["approved_annotation_rows"], [2626])
        self.assertEqual(admission["unresolved_field_blockers"], [])
        self.assertTrue(admission["independent_reviewer"])
        self.assertEqual(admission["native_checks"], "integration_owned_not_run")
        self.assertEqual(admission["conversion_and_packaging"], "integration_owned_not_run")
        reviews = read("field-review.json")
        self.assertTrue(reviews["independent_acceptance"])
        self.assertTrue(reviews["independence"])
        self.assertEqual(reviews["reviewer"], admission["independent_reviewer"])
        self.assertEqual(reviews["kind"], "separate_independent_field_review")
        self.assertEqual(len(reviews["fields"]), 196)
        self.assertEqual(len(reviews["usage"]), 49)
        self.assertEqual(len(reviews["unresolved_fields"]), 4)
        self.assertEqual(read("accepted-replacements.json"), [])
        snapshot = read("authored/snapshot.json")
        self.assertEqual(snapshot["author_commit"], "afca8fa7a4a09121ed77fb99e72cbbc67b4ceae0")
        self.assertEqual(snapshot["review_base"], "fdd2f3b1f057a9a5b7af05641502651253beb439")
        fields = {(r["sense_id"], r["field"]): r for r in reviews["fields"]}
        usage = {r["sense_id"]: r for r in reviews["usage"]}
        self.assertEqual(len(fields), 196)
        self.assertEqual(len(usage), 49)
        expected = set()
        for group, proposal in zip(groups, authored, strict=True):
            if not group["senses"]:
                continue
            sense, original = group["senses"][0], proposal["senses"][0]
            self.assertEqual(sense["independent_review"], "passed_after_exact_accepted_replacements")
            self.assertEqual(sense["resource_admission"], "independently_accepted_pending_canonical_assembly")
            self.assertEqual((sense["sense_id"], sense["use"], sense["excluded_uses"]),
                             (original["sense_id"], original["use"], original["excluded_uses"]))
            u = usage[sense["sense_id"]]
            self.assertEqual((u["language"], u["source_headword"]),
                             (sense["use_language"], sense["source_headword"]))
            self.assertEqual((u["proposed_usage"], u["accepted_usage"]), (sense["use"], sense["use"]))
            self.assertEqual(u["accepted_usage_quality_status"], "passed")
            self.assertEqual(u["everyday_use_decision"], "passed")
            self.assertTrue(u["review_reason"])
            self.assertTrue(u["everyday_sense_reason"])
            self.assertEqual(u["reviewer"], reviews["reviewer"])
            for name, field in sense["fields"].items():
                key = (sense["sense_id"], name)
                expected.add(key)
                r = fields[key]
                self.assertEqual((r["frozen_row"], r["headword"], r["pos"], r["language"]),
                                 (group["frozen_row"], group["headword"], group["lexique_cgram"], field["language"]))
                self.assertEqual((r["proposed_text"], r["accepted_text"]),
                                 (original["fields"][name]["text"], field["text"]))
                before = {k: v for k, v in field.items() if k not in ("independent_review", "reviewer")}
                prior = {k: v for k, v in original["fields"][name].items() if k != "independent_review"}
                self.assertEqual(before, prior)
                self.assertEqual(r["accepted_text_quality_status"], "passed")
                self.assertEqual(r["rights_eligibility_status"], "passed_at_selected_text_boundary")
                self.assertEqual(r["reviewer"], reviews["reviewer"])
                self.assertTrue(r["independence"])
                self.assertTrue(r["review_reason"])
                self.assertTrue(r["selected_text_rights_reason"])
        self.assertEqual(set(fields), expected)
        self.assertEqual(len({r["review_reason"] for r in fields.values()}), 196)

    def test_fixed_sources_bind_four_fields_to_exact_french_pos_and_use(self):
        captures = read("source-captures.json")
        self.assertEqual(Path(captures["capture_root"]), Path(read("admission.json")["author_evidence_root"]))
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        contexts = {s["id"]: s for s in read("source-context.json")["current_content"]}
        raw = {}
        self.assertEqual(len(captures["sources"]), 51)
        for capture in captures["sources"]:
            api = json.loads(retained_path(capture["api_path"]).read_text(encoding="utf-8-sig"))
            page = next(p for p in api["query"]["pages"] if p["pageid"] == capture["pageid"])
            revision = next(r for r in page["revisions"] if r["revid"] == capture["revid"])
            body = retained_path(capture["raw_path"]).read_bytes().decode("utf-8")
            self.assertEqual(body, revision["slots"]["main"]["content"])
            source = sources[capture["source_id"]]
            self.assertEqual((source["title"], source["revid"], source["timestamp"]),
                             (page["title"], revision["revid"], revision["timestamp"]))
            self.assertIn("oldid=" + str(revision["revid"]), source["fixed_url"])
            self.assertTrue(source["attribution"])
            self.assertTrue(source["history_url"])
            self.assertTrue(contexts[source["id"]]["current_contributors"])
            raw[source["id"]] = body.splitlines()
        languages = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
        bindings = {(b["sense_id"], b["field"]): b for b in read("source-excerpts.json")["fields"]}
        rights = {(b["sense_id"], b["field"]): b for b in read("rights.json")["fields"]}
        reviews = {(b["sense_id"], b["field"]): b for b in read("field-review.json")["fields"]}
        self.assertEqual(len(bindings), 196)
        self.assertEqual(set(bindings), set(rights))
        for group in read("groups.json"):
            if not group["senses"]:
                continue
            self.assertEqual(len(group["senses"]), 1)
            sense = group["senses"][0]
            self.assertTrue(sense["use"].strip())
            self.assertTrue(sense["excluded_uses"].strip())
            self.assertIsNone(sense["cefr"])
            self.assertEqual(set(sense["fields"]), set(languages))
            for name, field in sense["fields"].items():
                with self.subTest(row=group["frozen_row"], field=name):
                    self.assertTrue(field["text"].strip())
                    self.assertEqual(field["language"], languages[name])
                    expected_title = "for intérieur" if group["headword"] == "for" and name != "ipa" else group["headword"]
                    self.assertEqual(sources[field["source_id"]]["title"], expected_title)
                    self.assertEqual(field["independent_review"], "passed")
                    self.assertTrue(field["modification"].strip())
                    b = bindings[(sense["sense_id"], name)]
                    span = field["source_span"]
                    self.assertEqual((field["source_id"], span["line_1_based"], span["column_1_based"]),
                                     (b["source_id"], b["line_1_based"], b["column_1_based"]))
                    line = raw[field["source_id"]][span["line_1_based"] - 1]
                    self.assertEqual(span["raw_source_excerpt"], line)
                    before = raw[field["source_id"]][:span["line_1_based"]]
                    language = next(l for l in reversed(before) if re.match(r"^==\s*[^=]", l))
                    self.assertRegex(language, r"^==\s*\{\{langue\|fr\}\}\s*==$")
                    pos = next(l for l in reversed(before) if l.startswith("=== {{S|"))
                    expected = {"NOM": "nom", "ADJ": "adjectif", "VER": "verbe", "ADV": "adverbe"}
                    self.assertIn("{{S|" + expected[group["lexique_cgram"]] + "|fr", pos)
                    if name == "ipa":
                        self.assertEqual(field["text"], "/" + b["ipa_variant_excerpt"] + "/")
                        self.assertIn(b["ipa_variant_excerpt"], line)
                    else:
                        self.assertTrue(line.startswith("#") and not line.startswith("#*"))
                    proposal = rights[(sense["sense_id"], name)]
                    self.assertEqual(proposal["proposed_text"], field["text"])
                    self.assertEqual(proposal["accepted_text"], field["text"])
                    self.assertEqual((b["proposed_text"], b["accepted_text"]), (field["text"], field["text"]))
                    r = reviews[(sense["sense_id"], name)]
                    self.assertEqual(proposal["exact_source"], r["exact_source"])
                    self.assertEqual((r["exact_source"]["source_id"], r["exact_source"]["line_1_based"],
                                      r["exact_source"]["column_1_based"], r["exact_source"]["reviewed_excerpt"]),
                                     (b["source_id"], b["line_1_based"], b["column_1_based"], b["reviewed_excerpt"]))
                    self.assertEqual(proposal["content_license"], "CC-BY-SA-4.0")
                    self.assertEqual(proposal["independent_review"], "passed_at_selected_text_boundary")
                    self.assertEqual(proposal["reviewer"], r["reviewer"])
                    self.assertEqual(proposal["review_reason"], r["selected_text_rights_reason"])
                    self.assertTrue(proposal["attribution"])
        supports = read("source-excerpts.json")["supporting_evidence"]
        supports += read("authored/unresolved.json")["groups"][0]["inspected_evidence"]
        for b in supports:
            line = raw[b["source_id"]][b["line_1_based"] - 1]
            start = b["column_1_based"] - 1
            self.assertEqual(b["reviewed_excerpt"], line[start:start + len(b["reviewed_excerpt"])])
        relations = read("source-context.json")["spelling_relations"]
        self.assertEqual(len(relations), 49)
        self.assertTrue(all(r["alias_admission"] == "not_admitted" for r in relations))
        self.assertTrue(all(r["independent_review"] == "passed" and r["review_reason"] for r in relations))
        self.assertEqual(relations, read("field-review.json")["spelling_relations"])

    def test_paginated_contributor_capture_and_actual_license_material(self):
        captures = read("source-captures.json")
        parts = [retained_path(p) for p in captures["contributor_registry"]["parts"]]
        names, anonymous = {}, {}
        for support in (False, True):
            continuation = {}
            chain = [p for p in parts if ("support" in p.name) == support]
            self.assertTrue(chain)
            for path in chain:
                query = parse_qs(urlparse(path.with_suffix(".request.txt").read_text(encoding="utf-8")).query)
                for key, value in continuation.items():
                    self.assertEqual(query[key], [str(value)])
                response = json.loads(path.read_text(encoding="utf-8-sig"))
                self.assertNotIn("error", response)
                for page in response["query"]["pages"]:
                    names.setdefault(page["title"], set()).update(c["name"] for c in page.get("contributors", []))
                    if "anoncontributors" in page:
                        anonymous[page["title"]] = page["anoncontributors"]
                continuation = response.get("continue", {})
            self.assertFalse(continuation)
        contexts = read("source-context.json")["current_content"]
        self.assertEqual(set(names), {s["title"] for s in contexts})
        for source in contexts:
            self.assertEqual(source["current_contributors"], sorted(names[source["title"]]))
            self.assertEqual(source["anonymous_contributors_count"], anonymous.get(source["title"], 0))
        self.assertEqual((EVIDENCE / "COPYING").read_text(encoding="utf-8").rstrip(),
                         (ROOT / "resources/core-pilot-79/COPYING").read_text(encoding="utf-8").rstrip())
        notices = (EVIDENCE / "NOTICES.md").read_text(encoding="utf-8")
        credits = (EVIDENCE / "SOURCE-NOTICES.md").read_text(encoding="utf-8")
        self.assertIn("CC BY-SA 4.0", notices)
        self.assertIn("Boris New", notices)
        self.assertIn("Christophe Pallier", notices)
        self.assertIn("Académie française", notices)
        self.assertEqual(read("rights.json")["independent_rights_fields_accepted"], 196)
        for source in read("sources.json")["sources"]:
            self.assertIn(source["fixed_url"], credits)
            self.assertIn(source["history_url"], credits)
            for original in source["original_historical_dictionary_layers"]:
                self.assertEqual(original["conservative_expiry"], "2006-01-01")
                self.assertEqual(original["original_authors"], "Académie française")
                self.assertEqual(original["independent_review"], "passed_at_original_work_boundary")
            status = "inspected_no_admitted_content" if source["title"] == "r" else "passed_for_selected_spans"
            self.assertEqual(source["selected_text_review"], status)
        exclusions = read("source-context.json")["selected_boundary_exclusions"]
        self.assertEqual({(b["line_1_based"], b["marker"], b["status"]) for b in exclusions},
                         {(88, "Import:CFC", "unselected_no_redistribution_claim"),
                          (49, "File:Unglazed plates.jpg", "media_and_caption_excluded")})
        policy = json.loads(retained_path("target/issue-133/rights/fr-policy.api.json").read_text(encoding="utf-8-sig"))
        for page in policy["query"]["pages"]:
            body = retained_path(f"target/issue-133/rights/policy-{page['pageid']}.wiki").read_text(encoding="utf-8")
            self.assertEqual(body, page["revisions"][0]["slots"]["main"]["content"].replace("\r\n", "\n"))


if __name__ == "__main__":
    unittest.main()
