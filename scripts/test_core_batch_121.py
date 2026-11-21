"""Check #121 independently reviewed content at the approved admission boundary."""
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
    def test_separate_review_accepts_exactly_200_fields_and_50_uses(self):
        review = read('field-review.json')
        self.assertEqual(review['kind'], 'separate_independent_field_review')
        self.assertIs(review['independence'], True)
        self.assertIs(review['independent_acceptance'], True)
        self.assertEqual(len(review['fields']), 200)
        self.assertEqual(len(review['usage']), 50)
        self.assertEqual(review['unresolved_fields'], [])
        admission = read('admission.json')
        self.assertEqual(admission['independently_accepted_new_fields'], 200)
        self.assertEqual(admission['independently_accepted_new_groups'], 50)
        self.assertEqual(admission['pending_independent_review_fields'], 0)
        self.assertEqual(admission['pending_independent_rights_fields'], 0)
        self.assertEqual(admission['pending_independent_review_rows'], [])
        self.assertEqual(admission['independent_reviewer'], review['reviewer'])

    def test_explanation_is_an_attempt_to_make_something_understood(self):
        group = next(g for g in read('groups.json') if g['frozen_row'] == 2044)
        fields = group['senses'][0]['fields']
        expected = {
            'french_definition': 'Discours qui cherche à faire comprendre un texte, une idée ou une œuvre en donnant des renseignements, des détails ou des motifs.',
            'english': 'An explanation; an account intended to make a text, idea or work understandable through information, details or reasons.',
            'chinese': '解释、说明；旨在通过补充信息、细节或理由，使人理解文本、思想或作品的说明。',
        }
        for name, text in expected.items():
            with self.subTest(field=name):
                self.assertEqual(fields[name]['text'], text)

    def test_exact_frozen_membership_and_content_allow_identical_canonical_overlap(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            scope = list(csv.DictReader(handle))
        groups = read("groups.json")
        originals = {g['frozen_row']: g for g in read('review-candidates.json')}
        canonical = json.loads((ROOT / 'resources/core-pilot-79/curated-inputs.json').read_text(encoding='utf-8'))
        existing = {g['frozen_row']: g for g in canonical['groups']}
        accepted = {(f['sense_id'], f['field']): f for f in read('field-review.json')['fields']}
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
                self.assertIn(sense["independent_review"], ("passed", "passed_after_exact_accepted_replacements"))
                self.assertTrue(sense["use"])
                self.assertEqual(set(sense["fields"]), set(LANGUAGES))
                for name, field in sense["fields"].items():
                    self.assertTrue(field["text"].strip())
                    self.assertEqual(field["language"], LANGUAGES[name])
                    verdict = accepted[sense['sense_id'], name]
                    self.assertEqual(field['text'], verdict['accepted_text'])
                    if group['frozen_row'] != 2044 or name == 'ipa':
                        self.assertEqual(field['text'], originals[group['frozen_row']]['senses'][0]['fields'][name]['text'])
                if group['frozen_row'] in existing:
                    prior = existing[group['frozen_row']]
                    self.assertEqual((prior['headword'], prior['lexique_cgram']),
                                     (group['headword'], group['lexique_cgram']))
                    for frequency in ('subtitle_lemma_per_million', 'subtitle_pos_per_million'):
                        self.assertEqual(prior[frequency], group[frequency])
                    self.assertEqual(len(prior['senses']), len(group['senses']))
                    for current, copied in zip(group['senses'], prior['senses'], strict=True):
                        for key in ('sense_id', 'source_headword', 'use', 'use_language', 'cefr'):
                            self.assertEqual(current[key], copied[key])
                        for name in LANGUAGES:
                            for key in ('text', 'language', 'source_id', 'source_span', 'content_license', 'modification'):
                                self.assertEqual(current['fields'][name][key], copied['fields'][name][key])
        admission = read("admission.json")
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual((admission["reused_groups"], admission["reused_fields"]), (0, 0))
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual(admission["independently_accepted_new_fields"], 200)
        self.assertEqual(admission["pending_independent_review_fields"], 0)
        self.assertTrue(admission["independent_reviewer"])
        self.assertEqual(read("reuse.json")["groups"], [])
        proposal = read('admission-author-proposal.json')
        self.assertEqual(proposal['pending_independent_review_fields'], 200)
        self.assertEqual(proposal['independently_accepted_new_fields'], 0)
        self.assertEqual(proposal, read('authored-snapshots/admission.json'))
        self.assertEqual(read('review-candidates.json'), read('authored-snapshots/groups.json'))
        self.assertEqual(admission['correction_fields'], 3)
        # This ledger reports independent review; later parent admission counts may advance.
        self.assertEqual(admission['missing_independently_accepted_fields'], 0)

    def test_selected_content_rights_and_source_frames_pass_independently(self):
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
        self.assertIs(review["independent_acceptance"], True)
        self.assertTrue(review["reviewer"])
        self.assertEqual(rights["independent_rights_fields_accepted"], 200)
        self.assertEqual(rights["independent_review"], "passed_at_selected_text_boundary")
        self.assertEqual(len(review["usage"]), 50)
        self.assertEqual(len(review["spelling_relations"]), 50)
        self.assertEqual((EVIDENCE / "COPYING").read_text(encoding="utf-8").rstrip(),
                         (ROOT / "resources/core-pilot-79/COPYING").read_text(encoding="utf-8").rstrip())
        self.assertTrue(rights["authority_captures"])
        self.assertTrue(rights["redistribution_obligations"])
        self.assertEqual(len(sources), 48)
        self.assertEqual(sum(bool(c['original_historical_dictionary_layers']) for c in contexts.values()), 39)
        author_fields = {(f['sense_id'], f['field']): f for f in read('authored-snapshots/field-review.json')['fields']}
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
                    self.assertEqual(proposal["proposed_text"], author_fields[key]['proposed_text'])
                    self.assertEqual(proposal['accepted_text'], field['text'])
                    self.assertEqual(proposal["accepted_text_quality_status"], "passed")
                    self.assertEqual(proposal["reviewer"], review['reviewer'])
                    self.assertIs(proposal['independence'], True)
                    self.assertTrue(proposal['review_reason'])
                    self.assertTrue(proposal['selected_text_rights_reason'])
                    self.assertEqual(proposal["exact_source"],
                                     {k: v for k, v in binding.items() if k not in ("sense_id", "field")})
                    self.assertEqual(boundary["exact_source_span"], field["source_span"])
                    self.assertEqual(boundary["source_id"], field["source_id"])
                    self.assertEqual(boundary["modification"], field["modification"])
                    self.assertTrue(boundary["author_rights_basis"])
                    self.assertTrue(boundary["attribution"])
                    self.assertEqual(boundary["independent_review"], "passed_at_selected_text_boundary")
                    self.assertEqual(boundary['accepted_text'], field['text'])
                    self.assertEqual(boundary['independent_rights_reason'], proposal['selected_text_rights_reason'])
                    self.assertEqual(field["content_license"], "CC-BY-SA-4.0")
                    self.assertEqual(field["independent_review"], "passed")
                    self.assertEqual(field['rights_eligibility_status'], 'passed_at_selected_text_boundary')
                    self.assertEqual(source['selected_text_review'], 'passed_at_selected_text_boundary')
                    self.assertEqual(context['selected_text_rights_status'], 'passed_at_selected_text_boundary')
                    self.assertEqual(context['independent_selected_span_review']['status'], 'passed_at_selected_text_boundary')
                    self.assertIn(field['source_span']['line_1_based'], context['independent_selected_span_review']['selected_lines'])
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
                        self.assertEqual(layer['independent_review'], 'passed_at_original_collective_work_boundary')
                        self.assertIn('L113-2', layer['review_reason'])
        for relation in review["spelling_relations"]:
            self.assertEqual(relation["frozen_headword"], relation["source_headword"])
            self.assertEqual(relation["alias_admission"], "not_admitted")
            self.assertEqual(relation["independent_review"], "passed_for_source_binding_only")
            self.assertEqual(relation['accepted_spelling_quality_status'], 'passed')
            self.assertTrue(relation['review_reason'])
        usages = {u["frozen_row"]: u for u in review["usage"]}
        for usage in usages.values():
            self.assertEqual(usage['accepted_usage_quality_status'], 'passed')
            self.assertEqual(usage['everyday_sense_status'], 'passed')
            self.assertEqual(usage['accepted_usage'], next(g['senses'][0]['use'] for g in read('groups.json') if g['frozen_row']==usage['frozen_row']))
            self.assertTrue(usage['review_reason'])
        self.assertIn("de + noun", usages[2041]["proposed_usage"])
        self.assertIn("letter name", usages[2039]["proposed_usage"])
        self.assertIn("Regional", usages[2036]["proposed_usage"])
        self.assertTrue(usages[2036]["register_validation_evidence"])

    def test_retained_first_party_bytes_reproduce_every_selected_span_and_contributor(self):
        captures = read("source-captures.json")
        if not Path(captures["capture_root"]).exists():
            self.skipTest("Owning author archive is required for first-party byte validation")
        archive = Path(captures['capture_root']).parents[1]
        def captured(path):
            local = ROOT / path
            return local if local.exists() else archive / path
        self.assertTrue(captures["contributor_registry"]["pagination_complete"])
        sources = {s["source_id"]: s for s in captures["sources"]}
        contexts = {s["id"]: s for s in read("source-context.json")["current_content"]}
        registry, anonymous = {}, {}
        parts = captures["contributor_registry"]["parts"]
        for number, part in enumerate(parts):
            path = captured(part)
            response = json.loads(path.read_text(encoding="utf-8"))
            self.assertNotIn("error", response)
            for page in response["query"]["pages"]:
                registry.setdefault(page["title"], set()).update(p["name"] for p in page.get("contributors", []))
                if "anoncontributors" in page:
                    anonymous[page["title"]] = page["anoncontributors"]
            if number + 1 < len(parts):
                next_url = captured(parts[number + 1]).with_suffix(".request.txt").read_text(encoding="utf-8")
                next_query = parse_qs(urlparse(next_url).query)
                for key, value in response["continue"].items():
                    self.assertEqual(next_query[key], [str(value)])
            else:
                self.assertNotIn("continue", response)
        for source in sources.values():
            raw = captured(source["raw_path"]).read_bytes()
            api = json.loads(captured(source["api_path"]).read_text(encoding="utf-8"))
            page = next(p for p in api["query"]["pages"] if p["pageid"] == source["pageid"])
            revision = next(r for r in page["revisions"] if r["revid"] == source["revid"])
            self.assertEqual(raw, revision["slots"]["main"]["content"].encode('utf-8'))
            self.assertEqual(revision["timestamp"], source["timestamp"])
            context = contexts[source["source_id"]]
            self.assertEqual(set(context["current_contributors"]), registry[source["title"]])
            self.assertEqual(context["anonymous_contributors_count"], anonymous.get(source["title"], 0))
        for binding in read("source-excerpts.json")["fields"] + read("source-excerpts.json")["supporting_evidence"]:
            source = sources[binding["source_id"]]
            lines = captured(source["raw_path"]).read_text(encoding="utf-8").splitlines()
            line = lines[binding["line_1_based"] - 1]
            self.assertEqual(line, binding["reviewed_excerpt"])
            language = next(line for line in reversed(lines[:binding['line_1_based']]) if line.startswith('== {{langue'))
            self.assertEqual(language, '== {{langue|fr}} ==')


if __name__ == "__main__":
    unittest.main()
