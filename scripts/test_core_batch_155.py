"""Check #155 at the approved resource/admission boundary."""
import csv
import json
from pathlib import Path
from urllib.parse import parse_qs, urlparse
import unittest

from core_batch_test_helpers import verify_archived_source_captures

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-155"
MEMBERSHIP = [
    ("colonne", "NOM"), ("clope", "NOM"), ("hériter", "VER"),
    ("nan", "NOM"), ("coopérer", "VER"), ("onde", "NOM"),
    ("chiant", "ADJ"), ("intact", "ADJ"), ("obligé", "ADJ"),
    ("précipiter", "VER"), ("retarder", "VER"), ("réciter", "VER"),
    ("implorer", "VER"), ("phare", "NOM"), ("terriblement", "ADV"),
    ("quelquefois", "ADV"), ("char", "NOM"), ("femelle", "NOM"),
    ("berger", "NOM"), ("chirurgie", "NOM"), ("opportunité", "NOM"),
    ("foire", "NOM"), ("essuyer", "VER"), ("malentendu", "NOM"),
    ("moine", "NOM"), ("cachette", "NOM"), ("instituteur", "NOM"),
    ("miser", "VER"), ("perturber", "VER"), ("octobre", "NOM"),
    ("après-demain", "ADV"), ("horaire", "NOM"), ("portugais", "NOM"),
    ("vieillard", "NOM"), ("familier", "ADJ"), ("sublime", "ADJ"),
    ("actif", "ADJ"), ("potentiel", "NOM"), ("talon", "NOM"),
    ("extra", "ADJ"), ("sueur", "NOM"), ("domicile", "NOM"),
    ("fatal", "ADJ"), ("ronger", "VER"), ("cérébral", "ADJ"),
    ("grenade", "NOM"), ("interrogatoire", "NOM"), ("juré", "NOM"),
    ("menotte", "NOM"), ("audition", "NOM"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch155Tests(unittest.TestCase):
    def test_frozen_groups_have_complete_scoped_candidates(self):
        groups = read("groups.json")
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        self.assertEqual([g["frozen_row"] for g in groups], list(range(3701, 3751)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            frozen = list(csv.DictReader(handle))
        self.assertEqual(len(frozen), 4087)
        for group, row in zip(groups, frozen[3700:3750], strict=True):
            self.assertEqual((group["headword"], group["lexique_cgram"]),
                             (row["word"], row["lexique_cgram"]))
            self.assertEqual(group["subtitle_lemma_per_million"], row["subtitle_lemma_per_million"])
            self.assertEqual(group["subtitle_pos_per_million"], row["subtitle_pos_per_million"])
            self.assertTrue(group["senses"])
            for sense in group["senses"]:
                self.assertTrue(sense["use"].strip())
                self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                for field in sense["fields"].values():
                    self.assertTrue(field["text"].strip())
                    self.assertTrue(field["source_span"]["raw_source_excerpt"].strip())

    def test_noun_spelling_and_selected_pos_frames_keep_their_actual_use(self):
        groups = {g["frozen_row"]: g for g in read("groups.json")}
        nan = groups[3704]["senses"][0]
        self.assertEqual(nan["source_headword"], "naan")
        self.assertEqual(nan["fields"]["ipa"]["text"], "/nan/")
        relations = read("source-context.json")["spelling_relations"]
        relation = next(r for r in relations if r["frozen_row"] == 3704)
        self.assertEqual(relation["relation"], "attested_french_noun_spelling_variant")
        self.assertEqual(relation["alias_admission"], "not_admitted")
        self.assertTrue(any(r["reviewed_excerpt"] == "* [[nan]]" for r in relation["proposed_evidence"]))
        pos_names = {"NOM": "nom", "VER": "verbe", "ADJ": "adjectif", "ADV": "adverbe"}
        for usage in read("field-review.json")["usage"]:
            group = groups[usage["frozen_row"]]
            supporting = [s["reviewed_excerpt"] for s in usage["supporting_sources"]]
            self.assertIn("=={{langue|fr}}==", {"".join(s.split()) for s in supporting})
            self.assertTrue(any(s.startswith("=== {{S|" + pos_names[group["lexique_cgram"]] + "|fr")
                                for s in supporting))
        for field in read("source-excerpts.json")["fields"]:
            self.assertNotIn("Import:CFC", field["reviewed_excerpt"])
        cfc = next(r for s in read("rights.json")["sources"] if s["title"] == "talon"
                   for r in s["reference_markers"] if "Import:CFC" in r["raw_reference"])
        self.assertEqual(cfc["author_rights_proposal"], "excluded_from_selected_content")

    def test_contributor_histories_are_complete_and_match_source_credits(self):
        captures = read("source-captures.json")
        chains, names, anonymous = {}, {}, {}
        for name in captures["contributor_registry"]["parts"]:
            path = Path(name)
            data = json.loads(path.read_text(encoding="utf-8"))
            prefix, part = path.name.removesuffix(".api.json").rsplit("-", 1)
            chains.setdefault(prefix, {})[int(part)] = path, data
            for page in data["query"]["pages"]:
                names.setdefault(page["title"], set()).update(p["name"] for p in page.get("contributors", []))
                if "anoncontributors" in page:
                    anonymous[page["title"]] = page["anoncontributors"]
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
        for context in read("source-context.json")["current_content"]:
            self.assertEqual(set(context["current_contributors"]), names[context["title"]])
            self.assertEqual(context["anonymous_contributors_count"], anonymous.get(context["title"], 0))
            self.assertTrue(context["history_url"].endswith("&action=history"))

    def test_author_proposals_keep_all_quality_and_rights_acceptance_pending(self):
        admission, review, rights = read("admission.json"), read("field-review.json"), read("rights.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual(admission["independently_accepted_new_groups"], 0)
        self.assertEqual(admission["independently_accepted_new_fields"], 0)
        self.assertEqual(admission["pending_independent_review_fields"], 200)
        self.assertEqual(admission["pending_independent_rights_fields"], 200)
        self.assertEqual(admission["missing_authored_fields"], 0)
        self.assertIsNone(review["reviewer"])
        self.assertFalse(review["independent_acceptance"])
        proposed = {(g["frozen_row"], name): field for g in read("groups.json")
                    for name, field in g["senses"][0]["fields"].items()}
        self.assertEqual(len(rights["fields"]), 200)
        for record in rights["fields"]:
            field = proposed[record["frozen_row"], record["field"]]
            self.assertEqual(record["exact_source_span"], field["source_span"])
            self.assertEqual(record["license"], "CC-BY-SA-4.0")
            self.assertTrue(record["attribution"].strip())
            self.assertEqual(record["independent_review"], "pending")
            self.assertIsNone(record["reviewer"])
        for field in review["fields"]:
            self.assertEqual(field["proposed_text"], proposed[field["frozen_row"], field["field"]]["text"])
            self.assertEqual(field["accepted_text_quality_status"], "pending")
            self.assertIsNone(field["reviewer"])

    def test_fixed_source_bytes_and_selected_spans_match_recorded_revisions(self):
        captures = read("source-captures.json")
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        lines = verify_archived_source_captures(self, captures["sources"], sources, Path)
        excerpts = {(f["sense_id"], f["field"]): f for f in read("source-excerpts.json")["fields"]}
        for group in read("groups.json"):
            for sense in group["senses"]:
                for name, field in sense["fields"].items():
                    exact = excerpts[sense["sense_id"], name]
                    raw = lines[field["source_id"]][exact["line_1_based"] - 1]
                    self.assertEqual(raw, field["source_span"]["raw_source_excerpt"])
                    self.assertEqual(raw, exact["reviewed_excerpt"])
                    self.assertEqual(exact["revid"], sources[field["source_id"]]["revid"])
                    if name == "ipa":
                        self.assertEqual(field["text"].strip("/"), exact["ipa_variant_excerpt"])
                        start = exact["ipa_variant_column_1_based"] - 1
                        self.assertEqual(raw[start:start + len(exact["ipa_variant_excerpt"])], exact["ipa_variant_excerpt"])
                    else:
                        self.assertTrue(raw.startswith("#"))
                        self.assertFalse(raw.startswith("#*"))
        for record in read("source-excerpts.json")["supporting_evidence"]:
            raw = lines[record["source_id"]][record["line_1_based"] - 1]
            start = record["column_1_based"] - 1
            self.assertEqual(raw[start:start + len(record["reviewed_excerpt"])], record["reviewed_excerpt"])


if __name__ == "__main__":
    unittest.main()
