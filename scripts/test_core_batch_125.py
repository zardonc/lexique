"""Check #125 reviewed content at the approved resource/admission seam."""
import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-125"


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch125Tests(unittest.TestCase):
    def test_noyer_chinese_retains_immersion_in_liquid_without_limiting_to_water(self):
        sense = next(g for g in read('groups.json') if g['frozen_row'] == 2232)['senses'][0]
        self.assertEqual(sense['fields']['chinese']['text'], '使某人因浸入液体而窒息（及物用法）。')

    def test_gay_adjective_retains_exclusive_same_sex_or_gender_romantic_scope(self):
        sense = next(g for g in read('groups.json') if g['frozen_row'] == 2228)['senses'][0]
        expected = {
            'french_definition': 'Homosexuel : attiré sexuellement ou romantiquement uniquement par des personnes du même sexe ou genre ; se dit généralement d’un homme.',
            'english': 'Gay; homosexual: sexually or romantically attracted only to people of the same sex or gender, usually said of a man (adjective).',
            'chinese': '同性恋的：在性或浪漫关系上只受到同性别者吸引，通常用于男性（形容词）。',
        }
        for name, text in expected.items():
            with self.subTest(field=name):
                self.assertEqual(sense['fields'][name]['text'], text)

    def test_independent_acceptance_matches_every_selected_field_and_rights_boundary(self):
        review = read('field-review.json')
        self.assertEqual(review['kind'], 'separate_independent_field_review')
        self.assertIs(review['independence'], True)
        self.assertTrue(review['independent_acceptance'])
        reviews = {(f['sense_id'], f['field']): f for f in review['fields']}
        boundaries = {(f['sense_id'], f['field']): f for f in read('rights.json')['fields']}
        fields = {(s['sense_id'], name): f for g in read('groups.json')
                  for s in g['senses'] for name, f in s['fields'].items()}
        self.assertEqual(len(fields), 200)
        self.assertEqual(set(fields), set(reviews))
        self.assertEqual(set(fields), set(boundaries))
        for key, field in fields.items():
            with self.subTest(sense=key[0], field=key[1]):
                item, rights = reviews[key], boundaries[key]
                self.assertEqual(item['accepted_text'], field['text'])
                self.assertEqual(rights['accepted_text'], field['text'])
                self.assertEqual(item['accepted_text_quality_status'], 'passed')
                self.assertEqual(field['independent_review'], 'passed')
                self.assertEqual(field['rights_eligibility_status'], 'passed_at_selected_text_boundary')
                self.assertEqual(rights['independent_review'], 'passed_at_selected_text_boundary')
                self.assertTrue(item['review_reason'])
                self.assertTrue(item['selected_text_rights_reason'])
                self.assertTrue(rights['independent_selected_rights_reason'])
        admission = read('admission.json')
        self.assertEqual(admission['accepted_groups'], 50)
        self.assertEqual(admission['accepted_fields'], 200)
        self.assertEqual(admission['independently_accepted_new_groups'] + admission['reused_groups'], 50)
        self.assertEqual(admission['independently_accepted_new_fields'] + admission['reused_fields'], 200)
        for name in ('pending_independent_review_fields', 'pending_independent_rights_fields', 'missing_fields'):
            self.assertEqual(admission[name], 0)
        self.assertEqual(admission['pending_independent_review_rows'], [])
        self.assertEqual(admission['unresolved_field_blockers'], [])

    def test_exact_frozen_groups_have_four_bound_reviewed_fields_and_allow_identical_canonical_overlap(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(encoding="utf-8-sig") as handle:
            scope = list(csv.DictReader(handle))
        self.assertEqual(len(scope), 4087)
        groups = read("groups.json")
        self.assertEqual(len(groups), 50)
        for row, group in zip(range(2201, 2251), groups, strict=True):
            with self.subTest(row=row):
                frozen = scope[row - 1]
                self.assertEqual((group["frozen_row"], group["headword"], group["lexique_cgram"]),
                                 (row, frozen["word"], frozen["lexique_cgram"]))
                for key in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                    self.assertEqual(group[key], frozen[key])
                self.assertTrue(group["senses"])
                for sense in group["senses"]:
                    self.assertTrue(sense["use"])
                    self.assertIsNone(sense["cefr"])
                    self.assertEqual(set(sense["fields"]), {"french_definition", "ipa", "english", "chinese"})
                    for name, language in (("french_definition", "fr"), ("ipa", "und"),
                                           ("english", "en"), ("chinese", "zh")):
                        field = sense["fields"][name]
                        self.assertEqual(field["language"], language)
                        self.assertTrue(field["text"].strip())
                        self.assertTrue(field["source_id"])
                        self.assertTrue(field["source_span"]["raw_source_excerpt"])
        canonical = json.loads((ROOT / 'resources/core-pilot-79/curated-inputs.json').read_text(encoding='utf-8'))
        canonical = {g['frozen_row']: g for g in canonical['groups']}
        for group in groups:
            if group['frozen_row'] in canonical:
                with self.subTest(canonical_overlap=group['frozen_row']):
                    existing = canonical[group['frozen_row']]
                    for key in ('headword', 'lexique_cgram', 'subtitle_lemma_per_million',
                                'subtitle_pos_per_million'):
                        self.assertEqual(existing[key], group[key])
                    self.assertEqual(len(existing['senses']), len(group['senses']))
                    for before, reviewed in zip(existing['senses'], group['senses'], strict=True):
                        self.assertEqual(before['sense_id'], reviewed['sense_id'])
                        self.assertEqual(before['use'], reviewed['use'])
                        self.assertEqual(set(before['fields']), set(reviewed['fields']))
                        for name, field in reviewed['fields'].items():
                            for key in ('text', 'language', 'source_id', 'source_span', 'content_license'):
                                self.assertEqual(before['fields'][name][key], field[key])

    def test_reviewed_sources_rights_and_original_proposals_are_preserved(self):
        sources = {source["id"]: source for source in read("sources.json")["sources"]}
        contexts = {source["id"]: source for source in read("source-context.json")["current_content"]}
        captures = {source['source_id']: source for source in read('source-captures.json')['sources']}
        rights = read("rights.json")
        review = read("field-review.json")
        admission = read("admission.json")
        self.assertEqual(len(sources), 51)
        self.assertTrue(review["independent_acceptance"])
        self.assertEqual(rights["reviewer"], review["reviewer"])
        self.assertEqual(rights['independent_rights_fields_accepted'], 200)
        self.assertEqual(rights['independent_rights_fields_pending'], 0)
        authored = read('admission-author-proposal.json')
        self.assertEqual(authored['independently_accepted_new_fields'], 0)
        self.assertEqual(authored['pending_independent_review_fields'], 200)
        self.assertEqual(admission["proposed_fields"], 200)
        self.assertEqual(len(rights["fields"]), 200)
        self.assertEqual(len(review["fields"]), 200)
        self.assertEqual({(f["sense_id"], f["field"]) for f in rights["fields"]},
                         {(f["sense_id"], f["field"]) for f in review["fields"]})
        originals = {(s['sense_id'], name): field for group in read('review-candidates.json')
                     for s in group['senses'] for name, field in s['fields'].items()}
        reviews = {(f['sense_id'], f['field']): f for f in review['fields']}
        bindings = {(f['sense_id'], f['field']): f for f in read('source-excerpts.json')['fields']}
        self.assertEqual(set(originals), set(reviews))
        self.assertEqual(set(bindings), set(reviews))
        replacement_keys = {(2228, 'french_definition'), (2228, 'english'),
                            (2228, 'chinese'), (2232, 'chinese')}
        changed = set()
        for field in rights["fields"]:
            with self.subTest(sense=field["sense_id"], field=field["field"]):
                source = sources[field["source_id"]]
                capture = captures[field['source_id']]
                self.assertEqual(field['revid'], source['revid'])
                self.assertEqual(field['fixed_url'], source['fixed_url'])
                self.assertEqual(source['revid'], capture['revid'])
                self.assertEqual(source['fixed_url'], capture['fixed_url'])
                self.assertIn(str(source["revid"]), source["fixed_url"])
                self.assertTrue(source["history_url"])
                self.assertEqual(source["content_license"], "CC-BY-SA-4.0")
                context = contexts[source["id"]]
                self.assertEqual(context["current_registry_names"], len(context["current_contributors"]))
                self.assertTrue(context["contributor_capture_parts"])
                self.assertEqual(source['selected_text_review'], 'passed_at_selected_text_boundary')
                self.assertEqual(context['selected_text_rights_status'], 'passed_at_selected_text_boundary')
                self.assertEqual(context['selected_frame_independent_review'], 'passed_at_selected_text_boundary')
                self.assertTrue(field["selected_rights_boundary"])
                self.assertEqual(field["independent_review"], "passed_at_selected_text_boundary")
                self.assertEqual(field['reviewer'], review['reviewer'])
                key = field['sense_id'], field['field']
                original, decision, binding = originals[key], reviews[key], bindings[key]
                self.assertEqual(field['proposed_text'], original['text'])
                self.assertEqual(decision['proposed_text'], original['text'])
                self.assertEqual(field['exact_source_span'], original['source_span'])
                self.assertEqual(field['source_id'], original['source_id'])
                for name in ('source_id', 'revid', 'line_1_based', 'column_1_based', 'reviewed_excerpt'):
                    self.assertEqual(decision['exact_source'][name], binding[name])
                self.assertEqual(binding['independent_review'], 'passed_at_selected_text_boundary')
                if (field['frozen_row'], field['field']) in replacement_keys:
                    self.assertNotEqual(field['accepted_text'], original['text'])
                    changed.add((field['frozen_row'], field['field']))
                else:
                    self.assertEqual(field['accepted_text'], original['text'])
                if field['field'] == 'ipa':
                    exact = decision['exact_source']
                    value = field['accepted_text'].strip('/')
                    self.assertEqual(exact['ipa_variant_excerpt'], value)
                    offset = exact['ipa_variant_column_1_based'] - exact['column_1_based']
                    self.assertEqual(exact['reviewed_excerpt'][offset:offset + len(value)], value)
        self.assertEqual(changed, replacement_keys)
        self.assertEqual(admission['replaced_fields'], 4)
        self.assertEqual(admission['unchanged_fields'], 196)
        self.assertEqual(len(review['usage']), 50)
        self.assertEqual(len(review['spelling_relations']), 50)
        for use in review['usage']:
            self.assertEqual(use['accepted_usage_quality_status'], 'passed')
            self.assertEqual(use['source_frame_independent_review'], 'passed_at_selected_text_boundary')
            self.assertTrue(use['review_reason'])
        for relation in review['spelling_relations']:
            self.assertEqual(relation['independent_review'], 'passed')
            self.assertEqual(relation['alias_admission'], 'not_admitted')
            self.assertTrue(relation['review_reason'])
        for frame in read('source-excerpts.json')['supporting_evidence']:
            self.assertEqual(frame['independent_review'], 'passed_at_selected_text_boundary')
            self.assertEqual(frame['reviewer'], review['reviewer'])
        cents = next(g for g in read('groups.json') if g['frozen_row'] == 2240)['senses'][0]
        self.assertEqual(cents['fields']['ipa']['text'], '/sɑ̃/')
        self.assertEqual(cents['fields']['ipa']['source_id'], 'b125_fr_193659')
        cents_frame = next(u for u in review['usage'] if u['frozen_row'] == 2240)
        cents_notes = {(s['source_id'], s['line_1_based']): s['reviewed_excerpt']
                       for s in cents_frame['supporting_sources']}
        self.assertEqual(cents_notes[('b125_fr_193659', 6)], "# ''Pluriel de'' [[cent#fr-adj|cent]].")
        self.assertIn('valeur ordinale', cents_notes[('b125_fr_2308', 41)])
        self.assertIn("deux '''cents'''", cents_notes[('b125_fr_2308', 38)])
        self.assertIn("trois '''cent''' un", cents_notes[('b125_fr_2308', 39)])
        rang = next(source for source in sources.values() if source["title"] == "rang")
        self.assertIn("Émile Littré", {layer["original_authors"] for layer in rang["original_historical_dictionary_layers"]})
        for source in sources.values():
            for layer in source["original_historical_dictionary_layers"]:
                if layer["original_authors"] == "Académie française":
                    self.assertTrue(layer["conservative_expiry"].startswith("2006-01-01"))
                self.assertEqual(layer['independent_review'], 'passed_at_original_edition_boundary')
        self.assertEqual((EVIDENCE / "COPYING").read_text(encoding="utf-8").rstrip(),
                         (ROOT / 'resources/core-pilot-79/COPYING').read_text(encoding='utf-8').rstrip())


if __name__ == "__main__":
    unittest.main()
