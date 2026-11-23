"""Check #159 at the approved resource/admission boundary."""
import csv
import json
from pathlib import Path
import shutil
import tempfile
from urllib.parse import parse_qs, urlparse
import unittest

from core_batch_test_helpers import verify_archived_source_captures
from core_pilot import load_inputs, MATERIALS, RESOURCE

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-159"
MEMBERSHIP = [
    ("défiler", "VER"), ("hello", "ONO"), ("héritier", "NOM"),
    ("taureau", "NOM"), ("épouvantable", "ADJ"), ("représentant", "NOM"),
    ("intimité", "NOM"), ("judiciaire", "ADJ"), ("drôlement", "ADV"),
    ("étroit", "ADJ"), ("circulation", "NOM"), ("camionnette", "NOM"),
    ("phénomène", "NOM"), ("banc", "NOM"), ("poker", "NOM"),
    ("rencard", "NOM"), ("steak", "NOM"), ("immédiat", "ADJ"),
    ("exemplaire", "NOM"), ("expédition", "NOM"), ("vase", "NOM"),
    ("haver", "VER"), ("karma", "NOM"), ("vanter", "VER"),
    ("payé", "ADJ"), ("élégant", "ADJ"), ("traité", "NOM"),
    ("débat", "NOM"), ("brûlé", "ADJ"), ("crochet", "NOM"),
    ("remords", "NOM"), ("thèse", "NOM"), ("maths", "NOM"),
    ("équilibre", "NOM"), ("obstacle", "NOM"), ("récolter", "VER"),
    ("briquet", "NOM"), ("modeste", "ADJ"), ("innocence", "NOM"),
    ("désespoir", "NOM"), ("infection", "NOM"), ("déprimer", "VER"),
    ("proie", "NOM"), ("da", "ONO"), ("rançon", "NOM"),
    ("évoluer", "VER"), ("constater", "VER"), ("disperser", "VER"),
    ("allure", "NOM"), ("grossir", "VER"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch159Tests(unittest.TestCase):
    def test_only_approved_rows_are_zero_field_annotations(self):
        groups = read("groups.json")
        annotations = [g for g in groups if not g["senses"]]
        self.assertEqual([(g["frozen_row"], g["headword"], g["lexique_cgram"])
                          for g in annotations], [(3922, "haver", "VER"), (3944, "da", "ONO")])
        approvals = {a["frozen_row"]: a for a in read("owner-approval.json")["approved_annotations"]}
        review, admission = read("field-review.json"), read("admission.json")
        self.assertEqual(set(approvals), {3922, 3944})
        for group in annotations:
            approval = approvals[group["frozen_row"]]
            decision = group["annotation_exception_decision"]
            self.assertEqual(group["status"], "approved_annotation_exception")
            self.assertEqual(group["annotation_kind"], "owner_approved_zero_field")
            self.assertEqual((group["admitted_fields"], group["missing_fields"]), (0, 4))
            self.assertEqual(group["owner_approval"], decision)
            self.assertEqual(decision["approved_by"], "project_owner")
            self.assertEqual(decision["approved_on"], "2026-11-23")
            self.assertEqual(decision["timezone"], "America/Vancouver")
            self.assertEqual(decision["exact_reply"], "Annotate both groups with zero fields")
            self.assertEqual(decision["decision"], approval["annotation_exception_decision"])
            for frequency in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                self.assertEqual(group[frequency], approval[frequency])
        self.assertEqual((len(review["fields"]), len(review["usage"])), (192, 48))
        self.assertEqual(len(review["annotation_exceptions"]), 2)
        self.assertEqual({(f["frozen_row"], f["field"]) for f in review["unresolved_fields"]},
                         {(row, name) for row in (3922, 3944)
                          for name in ("french_definition", "ipa", "english", "chinese")})
        for field in review["unresolved_fields"]:
            group = next(g for g in annotations if g["frozen_row"] == field["frozen_row"])
            self.assertEqual(field["status"], "unresolved")
            self.assertEqual(field["annotation_exception_decision"], group["owner_approval"])
        for annotation, group in zip(review["annotation_exceptions"], annotations, strict=True):
            self.assertEqual(annotation["frozen_row"], group["frozen_row"])
            self.assertEqual(annotation["decision"], group["owner_approval"])
        self.assertEqual(admission["approved_annotations"], annotations)
        self.assertEqual(admission["approved_annotation_rows"], [3922, 3944])
        self.assertEqual((admission["annotation_exception_groups"], admission["specialist_exception_groups"]), (2, 0))
        self.assertEqual((admission["accepted_groups"], admission["accepted_fields"]), (48, 192))
        self.assertEqual((admission["missing_annotation_fields"], admission["ordinary_missing_fields"]), (8, 0))

    def test_selected_live_prey_and_uncooked_steak_restrictions(self):
        groups = {g["frozen_row"]: g["senses"][0] for g in read("groups.json") if g["senses"]}
        expected = {
            (3917, "chinese"): "用于煎制的肉排、鱼排或植物食材制成的排；不只指牛排。",
            (3943, "french_definition"): "Animal vivant qu’un animal carnassier capture pour le manger.",
            (3943, "english"): "Prey: a living animal caught by a carnivorous animal to be eaten.",
            (3943, "chinese"): "猎物；被食肉动物捕获以供食用的活体动物。",
        }
        for (row, name), text in expected.items():
            with self.subTest(row=row, field=name):
                self.assertEqual(groups[row]["fields"][name]["text"], text)

    def test_converter_rejects_author_proposals_without_independent_review(self):
        with tempfile.TemporaryDirectory(prefix="lexique-159-admission-") as temporary:
            stage = Path(temporary)
            for name in (*MATERIALS, "input-checksums.json"):
                shutil.copyfile(RESOURCE / name, stage / name)
            shutil.copyfile(EVIDENCE / "authored/field-review.json", stage / "field-review.json")
            with self.assertRaisesRegex(ValueError, "A separate independent field review is required"):
                load_inputs(stage)

    def test_converter_rejects_scope_unresolved_field_review(self):
        with tempfile.TemporaryDirectory(prefix="lexique-159-scope-") as temporary:
            stage = Path(temporary)
            for name in (*MATERIALS, "input-checksums.json"):
                shutil.copyfile(RESOURCE / name, stage / name)
            review = json.loads((stage / "field-review.json").read_text(encoding="utf-8"))
            review["fields"][0]["accepted_text_quality_status"] = "unresolved"
            (stage / "field-review.json").write_text(json.dumps(review), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "A selected field is not independently reviewed or rights-eligible"):
                load_inputs(stage)

    def test_exact_frozen_groups_have_complete_resource_records(self):
        if (EVIDENCE / "groups.json").exists():
            groups = read("groups.json")
        else:
            canonical = json.loads((ROOT / "resources/core-pilot-79/curated-inputs.json").read_text(encoding="utf-8"))
            groups = [g for g in canonical["groups"] if 3901 <= g["frozen_row"] <= 3950]
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP,
                         "The frozen #159 resource slice lacks complete batch content")
        self.assertEqual([g["frozen_row"] for g in groups], list(range(3901, 3951)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(encoding="utf-8-sig", newline="") as handle:
            frozen = list(csv.DictReader(handle))
        self.assertEqual(len(frozen), 4087)
        for group in groups:
            with self.subTest(row=group["frozen_row"]):
                row = frozen[group["frozen_row"] - 1]
                self.assertEqual((group["headword"], group["lexique_cgram"]), (row["word"], row["lexique_cgram"]))
                for frequency in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                    self.assertEqual(group[frequency], row[frequency])
                if group["frozen_row"] in {3922, 3944}:
                    self.assertEqual(group["senses"], [])
                    continue
                self.assertEqual(len(group["senses"]), 1)
                sense = group["senses"][0]
                self.assertTrue(sense["use"].strip())
                self.assertIsNone(sense["cefr"])
                self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                for field in sense["fields"].values():
                    self.assertTrue(field["text"].strip())
                    self.assertTrue(field["modification"].strip())

    def test_independent_field_use_source_and_rights_decisions(self):
        admission, review, rights = read("admission.json"), read("field-review.json"), read("rights.json")
        self.assertEqual(admission["status"], "independently_reviewed_content_and_scoped_exceptions_parent_admission_pending")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (48, 192))
        self.assertEqual((admission["independently_accepted_new_groups"], admission["independently_accepted_new_fields"]), (48, 192))
        self.assertEqual(admission["pending_independent_review_fields"], 0)
        self.assertEqual(admission["pending_independent_rights_fields"], 0)
        self.assertEqual(admission["missing_admitted_fields"], 8)
        self.assertEqual(admission["newly_admitted_fields"], 0)
        self.assertFalse(admission["ticket_complete"])
        self.assertEqual(admission["annotation_exception_groups"], 2)
        self.assertEqual(admission["specialist_exception_groups"], 0)
        self.assertEqual(admission["unresolved_field_blockers"], [])
        self.assertEqual(admission["ordinary_everyday_candidate_groups"], 48)
        self.assertEqual(admission["blocked_research_candidate_groups"], 0)
        self.assertTrue(review["independent_acceptance"])
        self.assertEqual(review["kind"], "separate_independent_field_review")
        self.assertTrue(review["reviewer"])
        self.assertEqual(rights["independent_rights_fields_accepted"], 200)
        self.assertEqual(rights["admissible_rights_fields"], 192)
        self.assertEqual(rights["unadmitted_research_fields"], 8)
        self.assertEqual(rights["independent_review"], "passed_at_selected_text_boundary")
        captures = read("source-captures.json")
        archive = Path(captures["capture_root"])
        self.assertEqual((archive.name, archive.parent.name), ("issue-159", "author-evidence"))
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        raw = verify_archived_source_captures(self, captures["sources"], sources, Path)
        self.assertEqual(len(raw), 50)
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
        authored_groups = read("authored/groups.json")
        authored = {(s["sense_id"], name): field for g in authored_groups
                    for s in g["senses"] for name, field in s["fields"].items()}
        usage = {u["sense_id"]: u for u in review["usage"]}
        self.assertEqual(len(usage), 48)
        languages = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
        pos_frames = {"NOM": "nom", "ADJ": "adjectif", "VER": "verbe", "ADV": "adverbe", "ONO": "interjection"}
        for group in read("groups.json"):
            if not group["senses"]:
                continue
            sense = group["senses"][0]
            self.assertEqual(sense["source_headword"], group["headword"])
            self.assertEqual(sense["independent_review"], "passed_after_exact_accepted_replacements")
            use = usage[sense["sense_id"]]
            self.assertEqual(use["accepted_usage"], sense["use"])
            self.assertEqual(use["accepted_usage_quality_status"], "passed")
            self.assertEqual(use["everyday_sense_status"], "passed")
            self.assertTrue(use["review_reason"])
            self.assertEqual(use["reviewer"], review["reviewer"])
            for name, field in sense["fields"].items():
                exact = by_field[sense["sense_id"], name]
                expected[sense["sense_id"], name] = field
                self.assertEqual(field["independent_review"], "passed")
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
        rejected = {(s["sense_id"], name): field for g in read("rejected-proposals.json")["groups"]
                    for s in g["senses"] for name, field in s["fields"].items()}
        audited = {**expected, **rejected}
        self.assertEqual({(f["sense_id"], f["field"]) for f in rights["fields"]}, set(audited))
        for field in rights["fields"]:
            key = field["sense_id"], field["field"]
            proposed = audited[key]
            self.assertEqual(field["admission"], "eligible_for_coordinator_admission" if key in expected
                             else "not_admitted_owner_approved_zero_field_annotation")
            self.assertEqual(field["exact_source_span"], proposed["source_span"])
            self.assertEqual(field["license"], "CC-BY-SA-4.0")
            self.assertTrue(field["attribution"])
            self.assertTrue(field["modification"])
            self.assertEqual(field["accepted_text"], proposed["text"])
            self.assertEqual(field["independent_review"], "passed_at_selected_text_boundary")
            self.assertTrue(field["selected_text_rights_reason"])
            self.assertEqual(field["reviewer"], review["reviewer"])
        self.assertEqual({(f["sense_id"], f["field"]) for f in review["fields"]}, set(expected))
        for field in review["fields"]:
            key = field["sense_id"], field["field"]
            self.assertEqual(field["proposed_text"], authored[key]["text"])
            self.assertEqual(field["accepted_text"], expected[key]["text"])
            self.assertEqual(field["accepted_text_quality_status"], "passed")
            self.assertEqual(field["rights_eligibility_status"], "passed_at_selected_text_boundary")
            self.assertEqual(field["source_use_semantic_quality_status"], "passed")
            self.assertTrue(field["review_reason"])
            self.assertTrue(field["selected_text_rights_reason"])
            self.assertEqual(field["reviewer"], review["reviewer"])
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
        self.assertEqual(set(chains), {f"contributors-new-{n}" for n in range(1, 6)})
        for parts in chains.values():
            self.assertEqual(sorted(parts), list(range(1, len(parts) + 1)))
            for part, (_, data) in parts.items():
                continuation = data.get("continue")
                if part == len(parts):
                    self.assertFalse(continuation)
                else:
                    self.assertTrue(continuation)
                    request = parts[part + 1][0].with_suffix(".request.txt").read_text(encoding="utf-8")
                    query = parse_qs(urlparse(request).query)
                    for key, value in continuation.items():
                        self.assertEqual(query[key], [str(value)])
        self.assertEqual(set(names), {s["title"] for s in sources.values()})
        for context in read("source-context.json")["current_content"]:
            self.assertEqual(context["current_contributors"], sorted(names[context["title"]]))
            self.assertEqual(context["current_registry_names"], len(names[context["title"]]))
            self.assertEqual(context["anonymous_contributors_count"], anonymous.get(context["title"], 0))
        self.assertFalse(captures["contributor_registry"]["selected_sentence_authorship_proved"])
        for source in read("sources.json")["sources"]:
            self.assertEqual(source["selected_text_review"], "passed_at_selected_text_boundary")
        for context in read("source-context.json")["current_content"]:
            self.assertEqual(context["selected_text_rights_status"], "passed_at_selected_text_boundary")
            self.assertTrue(context["selected_text_rights_reason"])
        spelling = review["spelling_relations"]
        self.assertEqual(len(spelling), 48)
        self.assertEqual({r["sense_id"] for r in spelling}, set(usage))
        for relation in spelling:
            self.assertEqual(relation["frozen_headword"], relation["source_headword"])
            self.assertEqual(relation["independent_review"], "passed")
            self.assertEqual(relation["alias_admission"], "not_admitted")
        notices = (EVIDENCE / "NOTICES.md").read_text(encoding="utf-8")
        for source in sources.values():
            self.assertIn(source["fixed_url"], notices)
            self.assertIn(source["history_url"], notices)
        self.assertEqual((EVIDENCE / "COPYING").read_bytes().rstrip(),
                         (ROOT / "resources/core-pilot-79/COPYING").read_bytes().rstrip())

    def test_exact_replacements_and_preserved_authored_package(self):
        captures = read("source-captures.json")
        package = Path(captures["capture_root"]) / "package"
        for snapshot in (EVIDENCE / "authored").iterdir():
            if snapshot.name != ".gitattributes":
                self.assertEqual(snapshot.read_bytes(), (package / snapshot.name).read_bytes(), snapshot.name)
        replacements = read("accepted-replacements.json")
        self.assertEqual({(r["frozen_row"], r["field"]) for r in replacements},
                         {(3917, "chinese"), (3943, "french_definition"), (3943, "english"), (3943, "chinese")})
        old = {(s["sense_id"], name): field["text"] for g in read("authored/groups.json")
               for s in g["senses"] for name, field in s["fields"].items()}
        new = {(s["sense_id"], name): field["text"] for g in read("groups.json")
               for s in g["senses"] for name, field in s["fields"].items()}
        changed = {key for key in new if old[key] != new[key]}
        rejected = {(s["sense_id"], name) for g in read("rejected-proposals.json")["groups"]
                    for s in g["senses"] for name in s["fields"]}
        self.assertEqual(set(old) - set(new), rejected)
        self.assertEqual(len(rejected), 8)
        self.assertEqual(changed, {(r["sense_id"], r["field"]) for r in replacements})
        for r in replacements:
            key = r["sense_id"], r["field"]
            self.assertEqual((r["proposed_text"], r["accepted_text"]), (old[key], new[key]))
            self.assertTrue(r["reason"])
        self.assertEqual(len(read("field-review.json")["unresolved_fields"]), 8)

    def test_rejected_proposals_keep_independent_scope_decisions(self):
        rejected = read("rejected-proposals.json")
        groups = {g["frozen_row"]: g for g in rejected["groups"]}
        blockers = rejected["blockers"]
        self.assertEqual(read("content-blockers.json")["blockers"], [])
        self.assertEqual([b["frozen_row"] for b in blockers], [3922, 3944])
        for b in blockers:
            self.assertIsNone(b["owner_approval"])
            self.assertEqual(b["admission"], "blocked")
            self.assertTrue(b["owner_decision_proposal"])
            self.assertEqual(len(groups[b["frozen_row"]]["senses"][0]["fields"]), 4)
        self.assertEqual((len(rejected["fields"]), len(rejected["usage"])), (8, 2))
        for field in rejected["fields"]:
            self.assertEqual(field["accepted_text_quality_status"], "unresolved")
            self.assertEqual(field["source_use_semantic_quality_status"], "passed")
            self.assertEqual(field["rights_eligibility_status"], "passed_at_selected_text_boundary")
        for use in rejected["usage"]:
            self.assertEqual(use["everyday_sense_status"], "unresolved")
        haver = groups[3922]["senses"][0]
        self.assertIn("Transitive mining", haver["use"])
        self.assertIn("h aspiré", haver["use"])
        self.assertIn("{{h aspiré}}", haver["fields"]["ipa"]["source_span"]["raw_source_excerpt"])
        da = groups[3944]["senses"][0]
        self.assertEqual(groups[3944]["lexique_cgram"], "ONO")
        self.assertIn("{{vieilli|fr}}", da["fields"]["french_definition"]["source_span"]["raw_source_excerpt"])
        self.assertIn("adverbe", da["fields"]["french_definition"]["source_span"]["raw_source_excerpt"])
        self.assertEqual((groups[3922]["headword"], groups[3944]["headword"]), ("haver", "da"))


if __name__ == "__main__":
    unittest.main()
