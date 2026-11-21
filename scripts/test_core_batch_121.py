"""Check #121 author candidates at the approved resource/admission boundary."""
import csv
import json
from pathlib import Path
import unittest
from urllib.parse import parse_qs, urlparse

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-121"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch121Tests(unittest.TestCase):
    def test_exact_frozen_membership_has_four_fields_without_claiming_admission(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            scope = list(csv.DictReader(handle))
        groups = read("groups.json")
        self.assertEqual([g["frozen_row"] for g in groups], list(range(2001, 2051)))
        for group, frozen in zip(groups, scope[2000:2050], strict=True):
            with self.subTest(row=group["frozen_row"]):
                self.assertEqual((group["headword"], group["lexique_cgram"]),
                                 (frozen["word"], frozen["lexique_cgram"]))
                for frequency in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                    self.assertEqual(group[frequency], frozen[frequency])
                self.assertEqual(len(group["senses"]), 1)
                sense = group["senses"][0]
                self.assertIsNone(sense["cefr"])
                self.assertEqual(sense["independent_review"], "pending")
                self.assertTrue(sense["use"])
                self.assertEqual(set(sense["fields"]), set(LANGUAGES))
                for name, field in sense["fields"].items():
                    self.assertTrue(field["text"].strip())
                    self.assertEqual(field["language"], LANGUAGES[name])
        admission = read("admission.json")
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual((admission["reused_groups"], admission["reused_fields"]), (0, 0))
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual(admission["independently_accepted_new_fields"], 0)
        self.assertEqual(admission["pending_independent_review_fields"], 200)
        self.assertIsNone(admission["independent_reviewer"])
        self.assertEqual(read("reuse.json")["groups"], [])

    def test_fixed_sources_and_selected_rights_are_complete_for_separate_review(self):
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        contexts = {s["id"]: s for s in read("source-context.json")["current_content"]}
        bindings = {(f["sense_id"], f["field"]): f for f in read("source-excerpts.json")["fields"]}
        review = read("field-review.json")
        reviews = {(f["sense_id"], f["field"]): f for f in review["fields"]}
        rights = read("rights.json")
        boundaries = {(f["sense_id"], f["field"]): f for f in rights["fields"]}
        self.assertEqual(len(bindings), 200)
        self.assertEqual(set(bindings), set(reviews))
        self.assertEqual(set(bindings), set(boundaries))
        self.assertIs(review["independent_acceptance"], False)
        self.assertIsNone(review["reviewer"])
        self.assertEqual(rights["independent_rights_fields_accepted"], 0)
        self.assertEqual(rights["independent_review"], "pending")
        self.assertEqual(len(review["usage"]), 50)
        self.assertEqual(len(review["spelling_relations"]), 50)
        self.assertEqual((EVIDENCE / "COPYING").read_text(encoding="utf-8").rstrip(),
                         (ROOT / "resources/core-pilot-79/COPYING").read_text(encoding="utf-8").rstrip())
        self.assertTrue(rights["authority_captures"])
        self.assertTrue(rights["redistribution_obligations"])
        for document in (read("source-context.json"), read("source-captures.json"), rights):
            validation_ids = [v["id"] for v in document["external_read_only_validation"]]
            self.assertEqual(sorted(validation_ids),
                             ["larousse-college-promis", "service-public-cheque", "usito-promis"])
        for group in read("groups.json"):
            sense = group["senses"][0]
            for name, field in sense["fields"].items():
                with self.subTest(row=group["frozen_row"], field=name):
                    key = sense["sense_id"], name
                    binding, proposal, boundary = bindings[key], reviews[key], boundaries[key]
                    source = sources[field["source_id"]]
                    context = contexts[source["id"]]
                    self.assertEqual(proposal["proposed_text"], field["text"])
                    self.assertEqual(proposal["accepted_text_quality_status"], "pending")
                    self.assertIsNone(proposal["reviewer"])
                    self.assertEqual(proposal["exact_source"],
                                     {k: v for k, v in binding.items() if k not in ("sense_id", "field")})
                    self.assertEqual(boundary["exact_source_span"], field["source_span"])
                    self.assertEqual(boundary["source_id"], field["source_id"])
                    self.assertEqual(boundary["modification"], field["modification"])
                    self.assertTrue(boundary["author_rights_basis"])
                    self.assertTrue(boundary["attribution"])
                    self.assertEqual(boundary["independent_review"], "pending")
                    self.assertEqual(field["content_license"], "CC-BY-SA-4.0")
                    self.assertEqual(field["independent_review"], "pending")
                    self.assertEqual(binding["source_id"], source["id"])
                    self.assertEqual(binding["revid"], source["revid"])
                    self.assertIn("oldid=" + str(source["revid"]), binding["url"])
                    self.assertEqual(binding["line_1_based"], field["source_span"]["line_1_based"])
                    self.assertEqual(binding["reviewed_excerpt"], field["source_span"]["raw_source_excerpt"])
                    self.assertEqual(context["current_content_revid"], source["revid"])
                    self.assertEqual(context["current_registry_names"], len(context["current_contributors"]))
                    self.assertTrue(context["current_contributors"])
                    if name == "ipa":
                        self.assertEqual(field["text"], "/" + binding["ipa_variant_excerpt"] + "/")
                        column = binding["ipa_variant_column_1_based"] - 1
                        self.assertEqual(binding["reviewed_excerpt"][column:column + len(binding["ipa_variant_excerpt"])],
                                         binding["ipa_variant_excerpt"])
                    for layer in boundary["original_historical_dictionary_layers"]:
                        self.assertTrue(layer["conservative_expiry"].startswith("2006-01-01"))
        for relation in review["spelling_relations"]:
            self.assertEqual(relation["frozen_headword"], relation["source_headword"])
            self.assertEqual(relation["alias_admission"], "not_admitted")
            self.assertEqual(relation["independent_review"], "pending")
        usages = {u["frozen_row"]: u for u in review["usage"]}
        self.assertIn("de + noun", usages[2041]["proposed_usage"])
        self.assertIn("letter name", usages[2039]["proposed_usage"])
        self.assertIn("Regional", usages[2036]["proposed_usage"])
        self.assertTrue(usages[2036]["register_validation_evidence"])

    def test_retained_first_party_bytes_reproduce_every_selected_span_and_contributor(self):
        captures = read("source-captures.json")
        if not Path(captures["capture_root"]).exists():
            self.skipTest("Owning author archive is required for first-party byte validation")
        self.assertTrue(captures["contributor_registry"]["pagination_complete"])
        sources = {s["source_id"]: s for s in captures["sources"]}
        contexts = {s["id"]: s for s in read("source-context.json")["current_content"]}
        registry, anonymous = {}, {}
        parts = captures["contributor_registry"]["parts"]
        for number, part in enumerate(parts):
            path = ROOT / part
            response = json.loads(path.read_text(encoding="utf-8"))
            self.assertNotIn("error", response)
            for page in response["query"]["pages"]:
                registry.setdefault(page["title"], set()).update(p["name"] for p in page.get("contributors", []))
                if "anoncontributors" in page:
                    anonymous[page["title"]] = page["anoncontributors"]
            if number + 1 < len(parts):
                next_url = (ROOT / parts[number + 1]).with_suffix(".request.txt").read_text(encoding="utf-8")
                next_query = parse_qs(urlparse(next_url).query)
                for key, value in response["continue"].items():
                    self.assertEqual(next_query[key], [str(value)])
            else:
                self.assertNotIn("continue", response)
        for source in sources.values():
            raw = (ROOT / source["raw_path"]).read_text(encoding="utf-8")
            api = json.loads((ROOT / source["api_path"]).read_text(encoding="utf-8"))
            page = next(p for p in api["query"]["pages"] if p["pageid"] == source["pageid"])
            revision = next(r for r in page["revisions"] if r["revid"] == source["revid"])
            self.assertEqual(raw, revision["slots"]["main"]["content"])
            self.assertEqual(revision["timestamp"], source["timestamp"])
            context = contexts[source["source_id"]]
            self.assertEqual(set(context["current_contributors"]), registry[source["title"]])
            self.assertEqual(context["anonymous_contributors_count"], anonymous.get(source["title"], 0))
        for binding in read("source-excerpts.json")["fields"] + read("source-excerpts.json")["supporting_evidence"]:
            source = sources[binding["source_id"]]
            line = (ROOT / source["raw_path"]).read_text(encoding="utf-8").splitlines()[binding["line_1_based"] - 1]
            self.assertEqual(line, binding["reviewed_excerpt"])


if __name__ == "__main__":
    unittest.main()
