"""Check #126 candidates at the approved resource/admission boundary."""
import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / 'docs/evidence/issue-126'
LANGUAGES = {'french_definition': 'fr', 'ipa': 'und', 'english': 'en', 'chinese': 'zh'}


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding='utf-8'))


class Batch126Tests(unittest.TestCase):
    def test_exact_frozen_groups_have_four_scoped_candidates_without_claiming_acceptance(self):
        with (ROOT / 'resources/core-pilot-79/scope-manifest.csv').open(encoding='utf-8-sig') as handle:
            frozen = list(csv.DictReader(handle))[2250:2300]
        groups = read('groups.json')
        self.assertEqual(len(groups), 50)
        for row, expected, group in zip(range(2251, 2301), frozen, groups, strict=True):
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

    def test_selected_fields_reproduce_fixed_bytes_and_retain_full_histories_and_rights(self):
        groups = read('groups.json')
        captures = read('source-captures.json')
        sources = {item['id']: item for item in read('sources.json')['sources']}
        contexts = {item['id']: item for item in read('source-context.json')['current_content']}
        rights = read('rights.json')
        boundaries = {(item['sense_id'], item['field']): item for item in rights['fields']}
        excerpts = {(item['sense_id'], item['field']): item for item in read('source-excerpts.json')['fields']}
        reviews = read('field-review.json')
        self.assertEqual(len(boundaries), 200)
        self.assertEqual(set(boundaries), set(excerpts))
        self.assertFalse(reviews['independent_acceptance'])
        self.assertIsNone(reviews['reviewer'])
        self.assertEqual(rights['independent_rights_fields_accepted'], 0)
        self.assertEqual(rights['independent_review'], 'pending')
        self.assertEqual((EVIDENCE / 'COPYING').read_text(encoding='utf-8').rstrip(),
                         (ROOT / 'resources/core-pilot-79/COPYING').read_text(encoding='utf-8').rstrip())
        for authority in rights['authority_captures']:
            self.assertTrue(Path(authority['path']).is_file())
        names, anonymous = {}, {}
        registry = captures['contributor_registry']
        self.assertTrue(registry['pagination_complete'])
        for part in registry['parts']:
            response = json.loads((ROOT / part).read_text(encoding='utf-8'))
            for page in response['query']['pages']:
                names.setdefault(page['title'], set()).update(person['name'] for person in page.get('contributors', []))
                if 'anoncontributors' in page:
                    anonymous[page['title']] = page['anoncontributors']
        self.assertNotIn('continue', json.loads((ROOT / registry['parts'][-1]).read_text(encoding='utf-8')))
        fixed = {}
        for part in captures['fixed_revision_recapture']['parts']:
            response = json.loads((ROOT / part).read_text(encoding='utf-8'))
            for page in response['query']['pages']:
                for revision in page['revisions']:
                    fixed[revision['revid']] = revision['slots']['main']['content'].encode('utf-8')
        histories = {item['pageid']: item for item in captures['revision_histories']['sources']}
        self.assertEqual(len(histories), 49)
        raw_by_id = {}
        for capture in captures['sources']:
            source = sources[capture['source_id']]
            context = contexts[source['id']]
            raw = (ROOT / capture['raw_path']).read_bytes()
            self.assertEqual(raw, fixed[source['revid']])
            response = json.loads((ROOT / capture['api_path']).read_text(encoding='utf-8'))
            page = next(item for item in response['query']['pages'] if item['pageid'] == source['pageid'])
            revision = next(item for item in page['revisions'] if item['revid'] == source['revid'])
            self.assertEqual((page['title'], revision['timestamp']), (source['title'], source['timestamp']))
            self.assertEqual(raw, revision['slots']['main']['content'].encode('utf-8'))
            self.assertEqual(context['current_contributors'], sorted(names[source['title']]))
            self.assertEqual(context['anonymous_contributors_count'], anonymous.get(source['title'], 0))
            history = histories[source['pageid']]
            revisions = []
            for part in history['parts']:
                response = json.loads((ROOT / part).read_text(encoding='utf-8'))
                page = next(item for item in response['query']['pages'] if item['pageid'] == source['pageid'])
                revisions.extend(page['revisions'])
            self.assertNotIn('continue', response)
            self.assertEqual(revisions[0]['revid'], source['revid'])
            self.assertEqual(len(revisions), history['revisions'])
            self.assertEqual(revisions[-1], history['earliest_revision'])
            self.assertTrue(history['pagination_complete'])
            raw_by_id[source['id']] = raw.decode('utf-8').splitlines()
        for group in groups:
            sense = group['senses'][0]
            for name, field in sense['fields'].items():
                with self.subTest(row=group['frozen_row'], field=name):
                    key = sense['sense_id'], name
                    boundary, excerpt = boundaries[key], excerpts[key]
                    raw_line = raw_by_id[field['source_id']][field['source_span']['line_1_based'] - 1]
                    self.assertEqual(raw_line, field['source_span']['raw_source_excerpt'])
                    self.assertEqual(raw_line, excerpt['reviewed_excerpt'])
                    self.assertEqual(boundary['exact_source_span'], field['source_span'])
                    self.assertEqual(boundary['independent_review'], 'pending')
                    self.assertIsNone(boundary['reviewer'])
                    self.assertTrue(boundary['selected_import_original_work_rights'])
                    self.assertTrue(boundary['author_rights_basis'])
                    self.assertEqual(field['content_license'], 'CC-BY-SA-4.0')
                    self.assertTrue(field['modification'])
                    if name == 'ipa':
                        self.assertIn(field['text'].strip('/'), raw_line)
                    else:
                        self.assertTrue(raw_line.startswith('# '))
        by_row = {group['frozen_row']: group for group in groups}
        self.assertEqual(by_row[2251]['senses'][0]['fields']['ipa']['text'], '/ɛk.spʁɛs/')
        self.assertEqual(by_row[2291]['headword'], 'f')
        self.assertEqual(by_row[2291]['senses'][0]['fields']['ipa']['text'], '/ɛf/')
        for name in ('french_definition', 'english', 'chinese'):
            self.assertIn('se méfier de', by_row[2287]['senses'][0]['fields'][name]['text'])
        self.assertTrue(all(item['alias_admission'] == 'not_admitted' for item in reviews['spelling_relations']))


if __name__ == '__main__':
    unittest.main()
