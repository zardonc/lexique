"""Check #122 candidates at the spec-approved resource/admission seam."""
import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-122"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch122Tests(unittest.TestCase):
    def test_exact_ticket_membership_has_complete_candidates_without_claiming_admission(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            scope = list(csv.DictReader(handle))
        groups = read("groups.json")
        self.assertEqual([group["frozen_row"] for group in groups], list(range(2051, 2101)))
        for group in groups:
            frozen = scope[group["frozen_row"] - 1]
            self.assertEqual((group["headword"], group["lexique_cgram"]),
                             (frozen["word"], frozen["lexique_cgram"]))
            for frequency in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                self.assertEqual(group[frequency], frozen[frequency])
            self.assertEqual(len(group["senses"]), 1)
            sense = group["senses"][0]
            self.assertIsNone(sense["cefr"])
            self.assertTrue(sense["use"].strip())
            self.assertEqual(set(sense["fields"]), set(LANGUAGES))
            for name, field in sense["fields"].items():
                self.assertTrue(field["text"].strip())
                self.assertEqual(field["language"], LANGUAGES[name])
                self.assertEqual(field["independent_review"], "pending")
        admission = read("admission.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual(admission["proposed_fields"], 200)
        self.assertEqual(admission["newly_admitted_fields"], 0)
        self.assertEqual(admission["pending_independent_review_fields"], 200)
        self.assertEqual(admission["annotation_exception_groups"], 0)
        self.assertEqual(read("reuse.json")["reused_fields"], 0)

    def test_every_field_has_a_reproducible_fixed_span_and_selected_rights_proposal(self):
        rights = read("rights.json")
        review = read("field-review.json")
        sources = {source["id"]: source for source in read("sources.json")["sources"]}
        contexts = {item["id"]: item for item in read("source-context.json")["current_content"]}
        bindings = {(item["sense_id"], item["field"]): item
                    for item in read("source-excerpts.json")["fields"]}
        proposals = {(item["sense_id"], item["field"]): item for item in rights["fields"]}
        self.assertEqual(len(bindings), 200)
        self.assertEqual(set(proposals), set(bindings))
        self.assertFalse(review["independent_acceptance"])
        self.assertIsNone(review["reviewer"])
        captures = read("source-captures.json")
        capture_root = Path(captures["capture_root"])
        capture_by_title = {item["title"]: item for item in captures["sources"]}
        contributors, anonymous = {}, {}
        for path in (capture_root / "sources").glob("contributors*.api.json"):
            data = json.loads(path.read_text(encoding="utf-8"))
            for page in data["query"]["pages"]:
                contributors.setdefault(page["title"], set()).update(
                    person["name"] for person in page.get("contributors", [])
                )
                if "anoncontributors" in page:
                    anonymous[page["title"]] = page["anoncontributors"]
        for context in contexts.values():
            self.assertEqual(context["current_contributors"], sorted(contributors[context["title"]]))
            self.assertEqual(context["current_registry_names"], len(contributors[context["title"]]))
            self.assertEqual(context["anonymous_contributors_count"], anonymous.get(context["title"], 0))
        for group in read("groups.json"):
            sense = group["senses"][0]
            for name, field in sense["fields"].items():
                key = sense["sense_id"], name
                binding, proposal = bindings[key], proposals[key]
                source = sources[field["source_id"]]
                context = contexts[source["id"]]
                capture = capture_by_title[source["title"]]
                api = json.loads((capture_root / "sources" / Path(capture["api_path"]).name)
                                 .read_text(encoding="utf-8-sig"))
                page = next(page for page in api["query"]["pages"] if page["title"] == source["title"])
                revision = page["revisions"][0]
                raw = revision["slots"]["main"]["content"]
                self.assertEqual(raw, (capture_root / "sources" / Path(capture["raw_path"]).name)
                                 .read_text(encoding="utf-8"))
                self.assertEqual((source["revid"], source["timestamp"]),
                                 (revision["revid"], revision["timestamp"]))
                self.assertEqual(binding["reviewed_excerpt"], raw.splitlines()[binding["line_1_based"] - 1])
                self.assertEqual(field["source_span"]["raw_source_excerpt"], binding["reviewed_excerpt"])
                self.assertEqual(proposal["exact_source_span"], field["source_span"])
                self.assertEqual(proposal["independent_review"], "pending")
                self.assertTrue(proposal["author_rights_basis"])
                self.assertTrue(field["modification"])
                self.assertEqual(field["content_license"], "CC-BY-SA-4.0")
                self.assertTrue(context["current_contributors"])
                self.assertIn("oldid=", source["fixed_url"])
                self.assertIn("action=history", source["history_url"])
                if name == "ipa":
                    self.assertEqual(binding["ipa_variant_excerpt"], field["text"].strip("/"))
                    self.assertIn(binding["ipa_variant_excerpt"], binding["reviewed_excerpt"])
        for prefix in ("contributors-", "contributors-support-"):
            parts = sorted((capture_root / "sources").glob(prefix + "*.api.json"))
            if prefix == "contributors-":
                parts = [path for path in parts if "support" not in path.name]
            self.assertTrue(parts)
            self.assertNotIn("continue", json.loads(parts[-1].read_text(encoding="utf-8")))
        for binding in read("source-excerpts.json")["supporting_evidence"]:
            source = sources[binding["source_id"]]
            raw_lines = (capture_root / "sources" / (source["title"] + ".wiki"))
            self.assertEqual(binding["reviewed_excerpt"],
                             raw_lines.read_text(encoding="utf-8").splitlines()[binding["line_1_based"] - 1])
        for relation in read("source-context.json")["spelling_relations"]:
            self.assertEqual(relation["alias_admission"], "not_admitted")
        layers = [layer for source in sources.values()
                  for layer in source["original_historical_dictionary_layers"]]
        self.assertTrue(any("Littré" in layer["original_authors"] for layer in layers))
        self.assertTrue(all(layer["conservative_expiry"].startswith("2006-01-01")
                            for layer in layers if "Académie" in layer["original_authors"]))


if __name__ == "__main__":
    unittest.main()
