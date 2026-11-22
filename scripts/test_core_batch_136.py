"""Check #136 candidates at the authorized resource/admission boundary."""
import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-136"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
MEMBERSHIP = [
    ("nôtre", "PRO:pos"), ("sénateur", "NOM"), ("paniquer", "VER"),
    ("poison", "NOM"), ("déclaration", "NOM"), ("garantir", "VER"),
    ("veine", "NOM"), ("tempête", "NOM"), ("accusation", "NOM"),
    ("meuble", "NOM"), ("bandit", "NOM"), ("dévorer", "VER"),
    ("trente", "ADJ:num"), ("drapeau", "NOM"), ("détacher", "VER"),
    ("bourse", "NOM"), ("récompense", "NOM"), ("enquêter", "VER"),
    ("mesurer", "VER"), ("solitude", "NOM"), ("empoisonner", "VER"),
    ("mystérieux", "ADJ"), ("opéra", "NOM"), ("commerce", "NOM"),
    ("puits", "NOM"), ("sincère", "ADJ"), ("statue", "NOM"),
    ("cap", "NOM"), ("sécher", "VER"), ("empire", "NOM"),
    ("employer", "VER"), ("pro", "NOM"), ("hôte", "NOM"),
    ("régime", "NOM"), ("jumeau", "NOM"), ("volontiers", "ADV"),
    ("mental", "ADJ"), ("mouiller", "VER"), ("top", "NOM"),
    ("misérable", "ADJ"), ("populaire", "ADJ"), ("man", "NOM"),
    ("alarme", "NOM"), ("frigo", "NOM"), ("conséquence", "NOM"),
    ("demi-tour", "NOM"), ("fauteuil", "NOM"), ("bagarre", "NOM"),
    ("cinglé", "NOM"), ("correct", "ADJ"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


def retained_path(name):
    parts = name.replace("\\", "/").split("/")
    local = ROOT.joinpath(*parts)
    if local.is_file():
        return local
    return Path(read("admission.json")["author_evidence_root"]).joinpath(*parts[2:])


class Batch136Tests(unittest.TestCase):
    def test_frozen_groups_have_four_complete_candidates_without_claiming_admission(self):
        self.assertTrue((EVIDENCE / "groups.json").is_file(),
                        "The 50 frozen #136 groups lack a sourced four-field candidate package")
        groups = read("groups.json")
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        self.assertEqual([g["frozen_row"] for g in groups], list(range(2751, 2801)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
                encoding="utf-8-sig", newline="") as handle:
            frozen = list(csv.DictReader(handle))
        field_count = 0
        for group in groups:
            row = frozen[group["frozen_row"] - 1]
            self.assertEqual((group["headword"], group["lexique_cgram"]),
                             (row["word"], row["lexique_cgram"]))
            for name in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                self.assertEqual(group[name], row[name])
            self.assertEqual(len(group["senses"]), 1)
            sense = group["senses"][0]
            self.assertTrue(sense["use"].strip())
            self.assertEqual(sense["independent_review"], "pending")
            self.assertEqual(sense["resource_admission"], "pending_separate_independent_review")
            self.assertEqual(set(sense["fields"]), set(LANGUAGES))
            for name, field in sense["fields"].items():
                self.assertTrue(field["text"].strip())
                self.assertEqual(field["language"], LANGUAGES[name])
                self.assertTrue(field["source_id"])
                self.assertTrue(field["source_span"])
                field_count += 1
        self.assertEqual(field_count, 200)
        admission = read("admission.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual(admission["proposed_fields"], 200)
        self.assertEqual(admission["independently_accepted_new_fields"], 0)
        self.assertIsNone(admission["independent_reviewer"])

    def test_every_candidate_keeps_its_exact_source_and_pending_quality_and_rights(self):
        rights = read("rights.json")
        sources = {source["id"]: source for source in read("sources.json")["sources"]}
        captures = {record["source_id"]: record for record in read("source-captures.json")["sources"]}
        self.assertEqual(len(sources), 55)
        raw_lines = {}
        for source_id, capture in captures.items():
            raw = retained_path(capture["raw_path"]).read_text(encoding="utf-8")
            raw_lines[source_id] = raw.splitlines()
            if capture["api_path"]:
                api = json.loads(retained_path(capture["api_path"]).read_text(encoding="utf-8-sig"))
                page = next(page for page in api["query"]["pages"] if page["title"] == capture["title"])
                revision = page["revisions"][0]
                self.assertEqual(revision["slots"]["main"]["content"], raw)
                self.assertEqual(revision["revid"], sources[source_id]["revid"])
                self.assertEqual(revision["timestamp"], sources[source_id]["timestamp"])
            else:
                self.assertEqual(sources[source_id]["content_license"], "not_established_research_only")
        fields = read("field-review.json")["fields"]
        self.assertEqual(len(fields), 200)
        self.assertEqual(len(rights["fields"]), 200)
        by_identity = {(group["senses"][0]["sense_id"], name): field
                       for group in read("groups.json") for name, field in group["senses"][0]["fields"].items()}
        for field in fields:
            value = by_identity[(field["sense_id"], field["field"])]
            self.assertEqual(field["proposed_text"], value["text"])
            self.assertEqual(field["accepted_text_quality_status"], "pending")
            self.assertEqual(field["rights_eligibility_status"], "pending_separate_selected_span_audit")
            self.assertIsNone(field["reviewer"])
            self.assertNotIn("accepted_text", field)
            for span in [field["exact_source"], *field["supporting_sources"]]:
                line = raw_lines[span["source_id"]][span["line_1_based"] - 1]
                column = span["column_1_based"] - 1
                self.assertEqual(line[column:column + len(span["reviewed_excerpt"])], span["reviewed_excerpt"])
                self.assertEqual(span["revid"], sources[span["source_id"]]["revid"])
        for field in rights["fields"]:
            self.assertEqual(field["independent_review"], "pending")
            self.assertIsNone(field["reviewer"])
        self.assertEqual(rights["independent_rights_fields_accepted"], 0)
        admission = read("admission.json")
        self.assertEqual(admission["missing_admitted_fields"], 200)
        self.assertEqual(admission["pending_independent_review_fields"], 200)
        self.assertEqual(admission["pending_independent_rights_fields"], 200)
        self.assertEqual(admission["author_unresolved_content_rows"], [2792])
        self.assertEqual(admission["native_checks"], "integration_owned_not_run")
        self.assertEqual(admission["conversion_and_packaging"], "integration_owned_not_run")
        self.assertEqual(admission["annotation_exception_groups"], 0)
        self.assertEqual(admission["specialist_exception_groups"], 0)
        self.assertEqual(read("reuse.json")["reused_fields"], 0)

    def test_stable_fallback_retains_source_material_and_original_proposals(self):
        archive = Path(read("admission.json")["author_evidence_root"])
        self.assertTrue(archive.is_dir(), "Stable author-evidence fallback has not been retained")
        for capture in read("source-captures.json")["sources"]:
            for name in ("raw_path", "api_path"):
                if capture[name]:
                    parts = capture[name].replace("\\", "/").split("/")
                    path = archive.joinpath(*parts[2:])
                    self.assertTrue(path.is_file(), str(path))
                    self.assertEqual(path.read_bytes(), retained_path(capture[name]).read_bytes())
        for authority in read("rights.json")["authority_captures"]:
            self.assertTrue(Path(authority["path"]).is_file(), authority["path"])
            for page in authority.get("pages", []):
                self.assertTrue(Path(page["raw_path"]).is_file(), page["raw_path"])
        for name in ("groups.json", "field-review.json", "rights.json", "admission.json"):
            self.assertEqual(read(name), read("authored/" + name))
        for name in ("selection.py", "prepare_candidates.py", "finish_author_package.py", "reproduce.py"):
            self.assertTrue((archive / name).is_file(), name)


if __name__ == "__main__":
    unittest.main()
