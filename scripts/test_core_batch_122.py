"""Check #122 accepted content at the spec-approved resource/admission seam."""
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
    def test_independent_acceptance_binds_exact_fields_uses_and_rights(self):
        admission, review, rights = read("admission.json"), read("field-review.json"), read("rights.json")
        self.assertTrue(review["independent_acceptance"])
        self.assertTrue(review["independence"])
        self.assertEqual((admission["accepted_batch_groups"], admission["accepted_batch_fields"]), (50, 200))
        self.assertEqual((admission["pending_independent_review_fields"],
                          admission["pending_independent_rights_fields"],
                          admission["missing_admitted_fields"]), (0, 0, 0))
        self.assertEqual(admission["unresolved_fields"], [])
        decisions = {(item["sense_id"], item["field"]): item for item in review["fields"]}
        proofs = {(item["sense_id"], item["field"]): item for item in rights["fields"]}
        uses = {item["sense_id"]: item for item in review["usage"]}
        original = {group["frozen_row"]: group for group in read("review-candidates.json")["groups"]}
        canonical = json.loads((ROOT / "resources/core-pilot-79/curated-inputs.json").read_text(encoding="utf-8"))
        assembled = {group["frozen_row"]: group for group in canonical["groups"]}
        changed = 0
        for group in read("groups.json"):
            sense = group["senses"][0]
            usage = uses[sense["sense_id"]]
            self.assertEqual(usage["accepted_usage"], sense["use"])
            self.assertEqual(usage["proposed_usage"], original[group["frozen_row"]]["senses"][0]["use"])
            self.assertEqual(usage["everyday_sense_status"], "passed")
            self.assertTrue(usage["review_reason"])
            for name, field in sense["fields"].items():
                key = sense["sense_id"], name
                decision, proof = decisions[key], proofs[key]
                self.assertEqual(field["text"], decision["accepted_text"])
                self.assertEqual(proof["accepted_text"], field["text"])
                self.assertEqual(decision["proposed_text"], original[group["frozen_row"]]["senses"][0]["fields"][name]["text"])
                self.assertEqual(decision["accepted_text_quality_status"], "passed")
                self.assertEqual(proof["independent_review"], "passed_at_selected_text_boundary")
                self.assertTrue(decision["review_reason"])
                self.assertTrue(proof["independent_rights_reason"])
                self.assertEqual(proof["exact_source_span"], field["source_span"])
                changed += decision["proposed_text"] != decision["accepted_text"]
            # The parent may assemble this shard later; an existing row is reviewed once.
            if group["frozen_row"] in assembled:
                canonical_sense = assembled[group["frozen_row"]]["senses"][0]
                self.assertEqual(canonical_sense["use"], sense["use"])
                for name, field in sense["fields"].items():
                    for key in ("text", "source_id", "source_span", "content_license"):
                        self.assertEqual(canonical_sense["fields"][name][key], field[key])
        self.assertEqual(changed, 7)
        self.assertEqual(read("author-admission.json")["pending_independent_review_fields"], 200)

    def test_money_receipt_retains_private_signature_restriction(self):
        group = next(group for group in read("groups.json") if group["frozen_row"] == 2100)
        fields = group["senses"][0]["fields"]
        for name, restriction in (("french_definition", "sous seing privé"),
                                  ("english", "privately signed"),
                                  ("chinese", "私人签署")):
            with self.subTest(field=name):
                self.assertIn(restriction, fields[name]["text"])

    def test_breed_meanings_retain_economic_performance_restriction(self):
        group = next(group for group in read("groups.json") if group["frozen_row"] == 2096)
        fields = group["senses"][0]["fields"]
        for name, restriction in (("french_definition", "performances économiques"),
                                  ("english", "economic performance"),
                                  ("chinese", "经济性能")):
            with self.subTest(field=name):
                self.assertIn(restriction, fields[name]["text"])

    def test_selected_euphemistic_exclamation_has_grammatical_english(self):
        group = next(group for group in read("groups.json") if group["frozen_row"] == 2080)
        self.assertEqual(group["senses"][0]["fields"]["english"]["text"],
                         "Darn! A euphemistic exclamation used instead of merde! (shit!).")

    def test_exact_ticket_membership_has_complete_independently_accepted_fields(self):
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
                self.assertEqual(field["independent_review"], "passed")
        admission = read("admission.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual(admission["proposed_fields"], 200)
        self.assertEqual(admission["newly_admitted_fields"], 200)
        self.assertEqual(admission["pending_independent_review_fields"], 0)
        self.assertEqual(admission["annotation_exception_groups"], 0)
        self.assertEqual(read("reuse.json")["reused_fields"], 0)

    def test_every_field_has_a_reproducible_fixed_span_and_accepted_selected_rights(self):
        rights = read("rights.json")
        review = read("field-review.json")
        sources = {source["id"]: source for source in read("sources.json")["sources"]}
        contexts = {item["id"]: item for item in read("source-context.json")["current_content"]}
        bindings = {(item["sense_id"], item["field"]): item
                    for item in read("source-excerpts.json")["fields"]}
        proposals = {(item["sense_id"], item["field"]): item for item in rights["fields"]}
        self.assertEqual(len(bindings), 200)
        self.assertEqual(set(proposals), set(bindings))
        self.assertTrue(review["independent_acceptance"])
        self.assertTrue(review["reviewer"])
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
        if capture_root.is_dir():
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
                # Owning captures verify raw bytes; detached/CI checkouts retain fixed bindings.
                if capture_root.is_dir():
                    api = json.loads((capture_root / "sources" / Path(capture["api_path"]).name)
                                     .read_text(encoding="utf-8-sig"))
                    page = next(page for page in api["query"]["pages"] if page["title"] == source["title"])
                    revision = page["revisions"][0]
                    raw = revision["slots"]["main"]["content"]
                    self.assertEqual(raw.encode("utf-8"),
                                     (capture_root / "sources" / Path(capture["raw_path"]).name).read_bytes())
                    self.assertEqual((source["revid"], source["timestamp"]),
                                     (revision["revid"], revision["timestamp"]))
                    self.assertEqual(binding["reviewed_excerpt"], raw.splitlines()[binding["line_1_based"] - 1])
                self.assertEqual(field["source_span"]["raw_source_excerpt"], binding["reviewed_excerpt"])
                self.assertEqual(proposal["exact_source_span"], field["source_span"])
                self.assertEqual(proposal["independent_review"], "passed_at_selected_text_boundary")
                self.assertTrue(proposal["author_rights_basis"])
                self.assertTrue(field["modification"])
                self.assertEqual(field["content_license"], "CC-BY-SA-4.0")
                self.assertTrue(context["current_contributors"])
                self.assertIn("oldid=", source["fixed_url"])
                self.assertIn("action=history", source["history_url"])
                if name == "ipa":
                    self.assertEqual(binding["ipa_variant_excerpt"], field["text"].strip("/"))
                    self.assertIn(binding["ipa_variant_excerpt"], binding["reviewed_excerpt"])
        for prefix in (("contributors-", "contributors-support-") if capture_root.is_dir() else ()):
            parts = sorted((capture_root / "sources").glob(prefix + "*.api.json"))
            if prefix == "contributors-":
                parts = [path for path in parts if "support" not in path.name]
            self.assertTrue(parts)
            self.assertNotIn("continue", json.loads(parts[-1].read_text(encoding="utf-8")))
        for binding in read("source-excerpts.json")["supporting_evidence"]:
            source = sources[binding["source_id"]]
            raw_lines = (capture_root / "sources" / (source["title"] + ".wiki"))
            if capture_root.is_dir():
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
