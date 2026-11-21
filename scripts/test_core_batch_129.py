"""Check #129 at the approved resource/admission boundary."""
import csv
import json
from pathlib import Path
from urllib.parse import parse_qs, urlparse
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-129"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}
# Exact membership from the self-contained #129 ticket.
MEMBERSHIP = [
    ("document", "NOM"), ("nier", "VER"), ("précieux", "ADJ"),
    ("écarter", "VER"), ("kilomètre", "NOM"), ("nation", "NOM"),
    ("guider", "VER"), ("protection", "NOM"), ("manque", "NOM"),
    ("musical", "ADJ"), ("écoute", "NOM"), ("ordinaire", "ADJ"),
    ("religion", "NOM"), ("mouche", "NOM"), ("avantage", "NOM"),
    ("solide", "ADJ"), ("boucler", "VER"), ("détective", "NOM"),
    ("indien", "ADJ"), ("indien", "NOM"), ("comporter", "VER"),
    ("panne", "NOM"), ("songer", "VER"), ("ours", "NOM"),
    ("écrivain", "NOM"), ("commission", "NOM"), ("passager", "NOM"),
    ("boucher", "VER"), ("ha", "ONO"), ("remuer", "VER"),
    ("communiste", "ADJ"), ("communiste", "NOM"), ("poussière", "NOM"),
    ("mai", "NOM"), ("clef", "NOM"), ("échec", "NOM"), ("balader", "VER"),
    ("chrétien", "ADJ"), ("chrétien", "NOM"), ("témoigner", "VER"),
    ("gratuit", "ADJ"), ("minable", "ADJ"), ("soulever", "VER"),
    ("ticket", "NOM"), ("forcément", "ADV"), ("céder", "VER"),
    ("hélas", "ONO"), ("foot", "NOM"), ("barbe", "NOM"), ("incapable", "ADJ"),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch129Tests(unittest.TestCase):
    def test_exact_ticket_groups_have_complete_four_field_candidate_senses(self):
        groups = read("groups.json")
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        self.assertEqual([g["frozen_row"] for g in groups], list(range(2401, 2451)))
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
                encoding="utf-8-sig", newline="") as handle:
            frozen = list(csv.DictReader(handle))
        for group in groups:
            with self.subTest(row=group["frozen_row"]):
                row = frozen[group["frozen_row"] - 1]
                self.assertEqual((group["headword"], group["lexique_cgram"]),
                                 (row["word"], row["lexique_cgram"]))
                self.assertEqual(group["subtitle_lemma_per_million"], row["subtitle_lemma_per_million"])
                self.assertEqual(group["subtitle_pos_per_million"], row["subtitle_pos_per_million"])
                self.assertEqual(len(group["senses"]), 1)
                sense = group["senses"][0]
                self.assertTrue(sense["use"].strip())
                self.assertIsNone(sense["cefr"])
                self.assertEqual(set(sense["fields"]), set(LANGUAGES))
                for name, field in sense["fields"].items():
                    self.assertEqual(field["language"], LANGUAGES[name])
                    self.assertTrue(field["text"].strip())
                    self.assertTrue(field["modification"].strip())
        reuse = read("reuse.json")
        prior = json.loads((ROOT / "resources/core-pilot-79/curated-inputs.json").read_text(encoding="utf-8"))
        self.assertFalse({g["frozen_row"] for g in groups} & {g["frozen_row"] for g in prior["groups"]})
        self.assertEqual((reuse["reused_groups"], reuse["reused_fields"], reuse["groups"]), (0, 0, []))
        admission = read("admission.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual(admission["independently_accepted_new_fields"], 0)
        self.assertEqual(admission["pending_independent_review_rows"], list(range(2401, 2451)))
        self.assertIsNone(admission["independent_reviewer"])

    def test_fields_bind_fixed_source_spans_and_rights_without_claiming_independent_acceptance(self):
        rights = read("rights.json")
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        captures = read("source-captures.json")
        contexts = {s["id"]: s for s in read("source-context.json")["current_content"]}
        raw = {}
        self.assertEqual(len(captures["sources"]), 48)
        for capture in captures["sources"]:
            data = json.loads((ROOT / capture["api_path"]).read_text(encoding="utf-8-sig"))
            page = next(p for p in data["query"]["pages"] if p["pageid"] == capture["pageid"])
            revision = next(r for r in page["revisions"] if r["revid"] == capture["revid"])
            source = sources[capture["source_id"]]
            self.assertEqual((source["title"], source["revid"], source["timestamp"]),
                             (page["title"], revision["revid"], revision["timestamp"]))
            body = (ROOT / capture["raw_path"]).read_bytes().decode("utf-8")
            self.assertEqual(body, revision["slots"]["main"]["content"])
            self.assertIn("oldid=" + str(revision["revid"]), source["fixed_url"])
            self.assertTrue(source["history_url"])
            self.assertTrue(source["attribution"])
            self.assertTrue(contexts[source["id"]]["current_contributors"])
            raw[source["id"]] = body.splitlines()
        parts = [ROOT / p for p in captures["contributor_registry"]["parts"]]
        self.assertTrue(parts)
        contributors = {}
        continuation = {}
        for path in parts:
            query = parse_qs(urlparse(path.with_suffix(".request.txt").read_text(encoding="utf-8")).query)
            for name, value in continuation.items():
                self.assertEqual(query[name], [str(value)])
            response = json.loads(path.read_text(encoding="utf-8"))
            self.assertNotIn("error", response)
            for page in response["query"]["pages"]:
                contributors.setdefault(page["title"], set()).update(p["name"] for p in page.get("contributors", []))
            continuation = response.get("continue", {})
        self.assertFalse(continuation)
        self.assertEqual(set(contributors), {s["title"] for s in sources.values()})
        for source in sources.values():
            self.assertEqual(contexts[source["id"]]["current_contributors"], sorted(contributors[source["title"]]))
        proposals = {(f["sense_id"], f["field"]): f for f in rights["fields"]}
        review = read("field-review.json")
        self.assertEqual(len(proposals), 200)
        self.assertEqual(len(review["fields"]), 200)
        self.assertFalse(review["independent_acceptance"])
        self.assertIsNone(review["reviewer"])
        self.assertEqual(rights["independent_rights_fields_accepted"], 0)
        self.assertIn("1966-01-01", rights["original_daf8_basis_scope"])
        self.assertTrue(rights["authority_captures"])
        for group in read("groups.json"):
            sense = group["senses"][0]
            self.assertEqual(sense["independent_review"], "pending")
            for name, field in sense["fields"].items():
                with self.subTest(row=group["frozen_row"], field=name):
                    span = field["source_span"]
                    self.assertEqual(span["raw_source_excerpt"], raw[field["source_id"]][span["line_1_based"] - 1])
                    proposal = proposals[(sense["sense_id"], name)]
                    self.assertEqual(proposal["exact_source_span"], span)
                    self.assertEqual(proposal["source_id"], field["source_id"])
                    self.assertTrue(proposal["author_rights_basis"])
                    self.assertTrue(proposal["selected_boundary"])
                    self.assertEqual(proposal["independent_review"], "pending")
                    self.assertIsNone(proposal["reviewer"])
                    self.assertEqual(field["content_license"], "CC-BY-SA-4.0")
        for binding in read("source-excerpts.json")["supporting_evidence"]:
            line = raw[binding["source_id"]][binding["line_1_based"] - 1]
            start = binding["column_1_based"] - 1
            self.assertEqual(binding["reviewed_excerpt"], line[start:start + len(binding["reviewed_excerpt"])])
        relations = read("source-context.json")["spelling_relations"]
        self.assertEqual(len(relations), 50)
        self.assertTrue(all(r["alias_admission"] == "not_admitted" for r in relations))


if __name__ == "__main__":
    unittest.main()
