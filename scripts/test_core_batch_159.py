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
    def test_converter_rejects_author_proposals_without_independent_review(self):
        with tempfile.TemporaryDirectory(prefix="lexique-159-admission-") as temporary:
            stage = Path(temporary)
            for name in (*MATERIALS, "input-checksums.json"):
                shutil.copyfile(RESOURCE / name, stage / name)
            shutil.copyfile(EVIDENCE / "field-review.json", stage / "field-review.json")
            with self.assertRaisesRegex(ValueError, "A separate independent field review is required"):
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
                self.assertEqual(len(group["senses"]), 1)
                sense = group["senses"][0]
                self.assertTrue(sense["use"].strip())
                self.assertIsNone(sense["cefr"])
                self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                for field in sense["fields"].values():
                    self.assertTrue(field["text"].strip())
                    self.assertTrue(field["modification"].strip())

    def test_fixed_fields_have_rights_material_and_honest_pending_review(self):
        admission, review, rights = read("admission.json"), read("field-review.json"), read("rights.json")
        self.assertEqual(admission["status"], "authored_candidates_pending_separate_independent_review")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual((admission["independently_accepted_new_groups"], admission["independently_accepted_new_fields"]), (0, 0))
        self.assertEqual(admission["pending_independent_review_fields"], 200)
        self.assertEqual(admission["pending_independent_rights_fields"], 200)
        self.assertEqual(admission["missing_admitted_fields"], 200)
        self.assertEqual([b["frozen_row"] for b in admission["unresolved_field_blockers"]], [3922, 3944])
        self.assertEqual(admission["ordinary_everyday_candidate_groups"], 48)
        self.assertEqual(admission["blocked_research_candidate_groups"], 2)
        self.assertFalse(review["independent_acceptance"])
        self.assertIsNone(review["reviewer"])
        self.assertEqual(rights["independent_rights_fields_accepted"], 0)
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
        languages = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
        pos_frames = {"NOM": "nom", "ADJ": "adjectif", "VER": "verbe", "ADV": "adverbe", "ONO": "interjection"}
        for group in read("groups.json"):
            sense = group["senses"][0]
            self.assertEqual(sense["source_headword"], group["headword"])
            self.assertEqual(sense["independent_review"], "pending")
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
                    if group["frozen_row"] == 3944:
                        self.assertIn("{{S|particule|fr", pos)
                        self.assertEqual(sense["author_everyday_eligibility"], "unresolved")
                    else:
                        self.assertIn("{{S|" + pos_frames[group["lexique_cgram"]] + "|fr", pos)
        self.assertEqual({(f["sense_id"], f["field"]) for f in rights["fields"]}, set(expected))
        for field in rights["fields"]:
            proposed = expected[field["sense_id"], field["field"]]
            self.assertEqual(field["exact_source_span"], proposed["source_span"])
            self.assertEqual(field["license"], "CC-BY-SA-4.0")
            self.assertTrue(field["attribution"])
            self.assertTrue(field["modification"])
            self.assertEqual(field["independent_review"], "pending")
            self.assertIsNone(field["reviewer"])
        self.assertEqual({(f["sense_id"], f["field"]) for f in review["fields"]}, set(expected))
        for field in review["fields"]:
            self.assertEqual(field["proposed_text"], expected[field["sense_id"], field["field"]]["text"])
            self.assertEqual(field["accepted_text_quality_status"], "pending")
            self.assertIsNone(field["reviewer"])
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
        notices = (EVIDENCE / "NOTICES.md").read_text(encoding="utf-8")
        for source in sources.values():
            self.assertIn(source["fixed_url"], notices)
            self.assertIn(source["history_url"], notices)
        self.assertEqual((EVIDENCE / "COPYING").read_bytes().rstrip(),
                         (ROOT / "resources/core-pilot-79/COPYING").read_bytes().rstrip())


if __name__ == "__main__":
    unittest.main()
