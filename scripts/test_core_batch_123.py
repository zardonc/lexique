"""Check #123 candidates at the approved resource/admission boundary."""
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
    def test_exact_frozen_groups_have_four_scoped_candidates_without_claiming_acceptance(self):
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
                self.assertEqual(sense['independent_review'], 'pending')
                self.assertEqual(set(sense['fields']), set(LANGUAGES))
                for name, field in sense['fields'].items():
                    self.assertTrue(field['text'].strip())
                    self.assertEqual(field['language'], LANGUAGES[name])
        admission = read('admission.json')
        self.assertEqual(admission['scope_denominator_groups'], 4087)
        self.assertEqual(admission['proposed_fields'], 200)
        self.assertEqual(admission['independently_accepted_new_fields'], 0)
        self.assertEqual(admission['resource_admission'], 'pending_separate_independent_review')

    def test_all_fields_retain_exact_sources_contributors_and_pending_rights_review(self):
        groups = read('groups.json')
        sources = {item['id']: item for item in read('sources.json')['sources']}
        contexts = {item['id']: item for item in read('source-context.json')['current_content']}
        captures = {item['source_id']: item for item in read('source-captures.json')['sources']}
        excerpts = {(item['sense_id'], item['field']): item for item in read('source-excerpts.json')['fields']}
        review = read('field-review.json')
        reviews = {(item['sense_id'], item['field']): item for item in review['fields']}
        rights = read('rights.json')
        boundaries = {(item['sense_id'], item['field']): item for item in rights['fields']}
        self.assertEqual(len(excerpts), 200)
        self.assertEqual(set(excerpts), set(reviews))
        self.assertEqual(set(excerpts), set(boundaries))
        self.assertFalse(review['independent_acceptance'])
        self.assertIsNone(review['reviewer'])
        self.assertEqual(rights['independent_rights_fields_accepted'], 0)
        self.assertEqual(rights['independent_review'], 'pending')
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
            raw = json.loads((ROOT / capture['api_path']).read_text(encoding='utf-8-sig'))
            page = next(item for item in raw['query']['pages'] if item['pageid'] == source['pageid'])
            revision = next(item for item in page['revisions'] if item['revid'] == source['revid'])
            self.assertEqual((page['title'], revision['timestamp']), (source['title'], source['timestamp']))
            self.assertEqual(revision['slots']['main']['content'], (ROOT / capture['raw_path']).read_text(encoding='utf-8'))
            self.assertEqual(context['current_contributors'], sorted(names[source['title']]))
            self.assertEqual(context['anonymous_contributors_count'], anonymous.get(source['title'], 0))
            for layer in source['original_historical_dictionary_layers']:
                self.assertEqual(layer['conservative_expiry'], '2006-01-01')
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
                    self.assertEqual(proposed['proposed_text'], field['text'])
                    self.assertEqual(boundary['exact_source_span'], field['source_span'])
                    self.assertEqual(boundary['independent_review'], 'pending')
                    self.assertIsNone(boundary['reviewer'])
                    self.assertTrue(boundary['author_rights_basis'])
                    self.assertTrue(source['attribution'])
                    self.assertTrue(field['modification'])
                    self.assertEqual(field['content_license'], 'CC-BY-SA-4.0')
                    if name == 'ipa':
                        self.assertIn(field['text'].strip('/'), binding['reviewed_excerpt'])
                    else:
                        self.assertTrue(binding['reviewed_excerpt'].startswith('# '))
        special = {item['frozen_row']: item for item in review['spelling_relations']}
        for row in (2115, 2120, 2121, 2131, 2143):
            self.assertEqual(special[row]['alias_admission'], 'not_admitted')
            self.assertTrue(special[row]['construction_and_flexion_interpretation'])


if __name__ == '__main__':
    unittest.main()
