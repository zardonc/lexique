"""Check #123 accepted evidence at the approved resource/admission boundary."""
import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / 'docs/evidence/issue-123'
LANGUAGES = {'french_definition': 'fr', 'ipa': 'und', 'english': 'en', 'chinese': 'zh'}


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding='utf-8'))


class Batch123Tests(unittest.TestCase):
    def test_whisky_chinese_explains_malted_barley_without_widening_to_other_grains(self):
        sense = next(group for group in read('groups.json') if group['frozen_row'] == 2114)['senses'][0]
        self.assertEqual(sense['fields']['chinese']['text'], '麦芽威士忌；以发芽大麦制成的烈酒')

    def test_letter_noun_does_not_teach_the_sources_incorrect_consonant_ordinal(self):
        sense = next(group for group in read('groups.json') if group['frozen_row'] == 2120)['senses'][0]
        expected = {
            'french_definition': 'Lettre minuscule v, vingt-deuxième lettre de l’alphabet français.',
            'english': 'lowercase v; the twenty-second letter of the French alphabet',
            'chinese': '小写字母 v；法语字母表中的第二十二个字母',
            'ipa': '/ve/',
        }
        for name, text in expected.items():
            with self.subTest(field=name):
                self.assertEqual(sense['fields'][name]['text'], text)

    def test_exact_frozen_groups_have_four_reviewed_fields_before_parent_admission(self):
        with (ROOT / 'resources/core-pilot-79/scope-manifest.csv').open(encoding='utf-8-sig') as handle:
            frozen = list(csv.DictReader(handle))[2100:2150]
        groups = read('groups.json')
        self.assertEqual(len(groups), 50)
        for row, expected, group in zip(range(2101, 2151), frozen, groups, strict=True):
            with self.subTest(row=row):
                self.assertEqual((group['frozen_row'], group['headword'], group['lexique_cgram']),
                                 (row, expected['word'], expected['lexique_cgram']))
                self.assertEqual(group['subtitle_lemma_per_million'], expected['subtitle_lemma_per_million'])
                self.assertEqual(group['subtitle_pos_per_million'], expected['subtitle_pos_per_million'])
                self.assertEqual(len(group['senses']), 1)
                sense = group['senses'][0]
                self.assertTrue(sense['use'].strip())
                self.assertEqual(sense['independent_review'], 'passed_after_exact_accepted_replacements')
                self.assertEqual(set(sense['fields']), set(LANGUAGES))
                for name, field in sense['fields'].items():
                    self.assertTrue(field['text'].strip())
                    self.assertEqual(field['language'], LANGUAGES[name])
                    self.assertEqual(field['independent_review'], 'passed')
                    self.assertEqual(field['rights_eligibility_status'], 'passed_at_selected_text_boundary')
        admission = read('admission.json')
        self.assertEqual(admission['scope_denominator_groups'], 4087)
        self.assertEqual(admission['accepted_fields'], 200)
        self.assertEqual(admission['accepted_groups'], 50)
        self.assertEqual(admission['independently_accepted_new_fields'] + admission['reused_fields'], 200)
        self.assertEqual(admission['independently_accepted_new_groups'] + admission['reused_groups'], 50)
        self.assertEqual(admission['pending_independent_review_rows'], [])
        self.assertEqual(admission['unresolved_field_blockers'], [])
        self.assertEqual(admission['pending_independent_review_fields'], 0)
        self.assertEqual(admission['pending_independent_rights_fields'], 0)
        self.assertEqual(admission['missing_fields'], 0)
        authored = read('admission-author-proposal.json')
        self.assertEqual(authored['proposed_fields'], 200)
        self.assertEqual(authored['independently_accepted_new_fields'], 0)

    def test_all_accepted_fields_retain_proposals_sources_contributors_and_selected_rights(self):
        groups = read('groups.json')
        sources = {item['id']: item for item in read('sources.json')['sources']}
        contexts = {item['id']: item for item in read('source-context.json')['current_content']}
        captures = {item['source_id']: item for item in read('source-captures.json')['sources']}
        excerpts = {(item['sense_id'], item['field']): item for item in read('source-excerpts.json')['fields']}
        review = read('field-review.json')
        reviews = {(item['sense_id'], item['field']): item for item in review['fields']}
        rights = read('rights.json')
        boundaries = {(item['sense_id'], item['field']): item for item in rights['fields']}
        proposals = {(sense['sense_id'], name): field['text']
                     for group in read('review-candidates.json')
                     for sense in group['senses'] for name, field in sense['fields'].items()}
        self.assertEqual(len(excerpts), 200)
        self.assertEqual(set(excerpts), set(reviews))
        self.assertEqual(set(excerpts), set(boundaries))
        self.assertTrue(review['independent_acceptance'])
        self.assertTrue(review['reviewer'])
        self.assertEqual(review['unresolved_fields'], [])
        self.assertEqual(rights['independent_rights_fields_accepted'], 200)
        self.assertEqual(rights['independent_review'], 'passed_at_selected_text_boundary')
        self.assertTrue(rights['redistribution_obligations'])
        for authority in rights['authority_captures']:
            self.assertTrue(Path(authority['path']).is_file())
        self.assertEqual((EVIDENCE / 'COPYING').read_text(encoding='utf-8').rstrip(),
                         (ROOT / 'resources/core-pilot-79/COPYING').read_text(encoding='utf-8').rstrip())
        registry = read('source-captures.json')['contributor_registry']
        self.assertTrue(registry['pagination_complete'])
        names, anonymous = {}, {}
        for relative in registry['parts']:
            response = json.loads((ROOT / relative).read_text(encoding='utf-8'))
            for page in response['query']['pages']:
                names.setdefault(page['title'], set()).update(person['name'] for person in page.get('contributors', []))
                if 'anoncontributors' in page:
                    anonymous[page['title']] = page['anoncontributors']
        for prefix in ('contributors-', 'contributors-support-'):
            parts = [path for path in registry['parts'] if Path(path).name.removeprefix(prefix).split('.')[0].isdigit()]
            self.assertTrue(parts)
            self.assertNotIn('continue', json.loads((ROOT / parts[-1]).read_text(encoding='utf-8')))
        for source_id, source in sources.items():
            capture, context = captures[source_id], contexts[source_id]
            self.assertEqual(source['selected_text_review'], 'passed_at_selected_text_boundary')
            self.assertEqual(context['selected_text_rights_status'], 'passed_at_selected_text_boundary')
            raw = json.loads((ROOT / capture['api_path']).read_text(encoding='utf-8-sig'))
            page = next(item for item in raw['query']['pages'] if item['pageid'] == source['pageid'])
            revision = next(item for item in page['revisions'] if item['revid'] == source['revid'])
            self.assertEqual((page['title'], revision['timestamp']), (source['title'], source['timestamp']))
            self.assertEqual(revision['slots']['main']['content'], (ROOT / capture['raw_path']).read_text(encoding='utf-8'))
            self.assertEqual(context['current_contributors'], sorted(names[source['title']]))
            self.assertEqual(context['anonymous_contributors_count'], anonymous.get(source['title'], 0))
            for layer in source['original_historical_dictionary_layers']:
                self.assertEqual(layer['conservative_expiry'], '2006-01-01')
                self.assertEqual(layer['independent_review'], 'passed_at_original_work_boundary')
        for group in groups:
            sense = group['senses'][0]
            for name, field in sense['fields'].items():
                with self.subTest(row=group['frozen_row'], field=name):
                    key = sense['sense_id'], name
                    binding, proposed, boundary = excerpts[key], reviews[key], boundaries[key]
                    source = sources[field['source_id']]
                    lines = (ROOT / captures[source['id']]['raw_path']).read_text(encoding='utf-8').splitlines()
                    self.assertEqual(binding['reviewed_excerpt'], lines[binding['line_1_based'] - 1])
                    self.assertEqual(field['source_span']['raw_source_excerpt'], binding['reviewed_excerpt'])
                    self.assertEqual(proposed['proposed_text'], proposals[key])
                    self.assertEqual(proposed['accepted_text'], field['text'])
                    self.assertEqual(proposed['accepted_text_quality_status'], 'passed')
                    self.assertTrue(proposed['independence'])
                    self.assertTrue(proposed['review_reason'])
                    self.assertTrue(proposed['selected_text_rights_reason'])
                    self.assertEqual(boundary['exact_source_span'], field['source_span'])
                    self.assertEqual(boundary['independent_review'], 'passed_at_selected_text_boundary')
                    self.assertEqual(boundary['reviewer'], review['reviewer'])
                    self.assertEqual(boundary['accepted_text'], field['text'])
                    self.assertTrue(boundary['independent_rights_reason'])
                    self.assertTrue(boundary['author_rights_basis'])
                    self.assertTrue(source['attribution'])
                    self.assertTrue(field['modification'])
                    self.assertEqual(field['content_license'], 'CC-BY-SA-4.0')
                    if name == 'ipa':
                        self.assertIn(field['text'].strip('/'), binding['reviewed_excerpt'])
                    else:
                        self.assertTrue(binding['reviewed_excerpt'].startswith('# '))
        special = {item['frozen_row']: item for item in review['spelling_relations']}
        self.assertEqual(len(special), 50)
        uses = {item['sense_id']: item for item in review['usage']}
        self.assertEqual(len(uses), 50)
        for group in groups:
            sense = group['senses'][0]
            use = uses[sense['sense_id']]
            self.assertEqual(use['accepted_usage'], sense['use'])
            self.assertEqual(use['everyday_sense_status'], 'passed')
            self.assertTrue(use['everyday_sense_reason'])
            self.assertEqual(special[group['frozen_row']]['accepted_spelling_quality_status'], 'passed')
            self.assertEqual(special[group['frozen_row']]['alias_admission'], 'not_admitted')
        replacements = {key for key, proposed in reviews.items()
                        if proposed['proposed_text'] != proposed['accepted_text']}
        self.assertEqual(replacements, {
            ('b123-2120-v-nom-selected', 'french_definition'),
            ('b123-2120-v-nom-selected', 'english'),
            ('b123-2120-v-nom-selected', 'chinese'),
            ('b123-2114-whisky-nom-selected', 'chinese'),
        })
        for row in (2115, 2120, 2121, 2131, 2143):
            self.assertEqual(special[row]['alias_admission'], 'not_admitted')
            self.assertTrue(special[row]['construction_and_flexion_interpretation'])


if __name__ == '__main__':
    unittest.main()
