"""Check #126 independently reviewed content at the approved admission boundary."""
import csv
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / 'docs/evidence/issue-126'
LANGUAGES = {'french_definition': 'fr', 'ipa': 'und', 'english': 'en', 'chinese': 'zh'}


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding='utf-8'))


def capture_path(relative):
    path = ROOT / relative
    if path.is_file():
        return path
    return Path(read('admission.json')['author_evidence_root']).parents[1] / relative


class Batch126Tests(unittest.TestCase):
    def test_collective_original_work_rights_use_the_actual_statute(self):
        for name in ('sources.json', 'source-context.json', 'rights.json'):
            self.assertFalse('L126-3' in json.dumps(read(name)), name)
        source = next(item for item in read('sources.json')['sources'] if item['title'] == 'exprès')
        layer = source['original_historical_dictionary_layers'][0]
        self.assertIn('L123-3', layer['legal_basis'])
        self.assertEqual(layer['conservative_expiry'], '2006-01-01')

    def test_exaggerer_chinese_explains_only_the_selected_importance_use(self):
        group = next(item for item in read('groups.json') if item['frozen_row'] == 2273)
        self.assertEqual(group['senses'][0]['fields']['chinese']['text'],
                         '夸大；赋予某事物超出实际的重要性')

    def test_exact_frozen_groups_have_four_independently_reviewed_fields(self):
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
                self.assertEqual(sense['independent_review'], 'passed_after_exact_accepted_replacements')
                self.assertEqual(set(sense['fields']), set(LANGUAGES))
                for name, field in sense['fields'].items():
                    self.assertTrue(field['text'].strip())
                    self.assertEqual(field['language'], LANGUAGES[name])
        admission = read('admission.json')
        self.assertEqual(admission['scope_denominator_groups'], 4087)
        self.assertEqual(admission['proposed_fields'], 200)
        self.assertEqual(admission['accepted_batch_groups'], 50)
        self.assertEqual(admission['accepted_batch_fields'], 200)
        self.assertEqual(admission['independently_accepted_new_fields'], 200)
        self.assertEqual(admission['pending_independent_review_fields'], 0)
        self.assertEqual(admission['pending_independent_rights_fields'], 0)
        self.assertEqual(admission['missing_fields'], 0)
        self.assertEqual(admission['unresolved_field_blockers'], [])
        expected_admission = ('passed_combined_conversion_rights_notices_and_native_gates'
                              if admission['status'] == 'admitted_core_129_v1_windows_x64_development_resource'
                              else 'pending_parent_combined_conversion_rights_notices_and_native_gates')
        self.assertEqual(admission['resource_admission'], expected_admission)

    def test_parent_can_overlap_identical_reviewed_canonical_records(self):
        canonical = json.loads((ROOT / 'resources/core-pilot-79/curated-inputs.json').read_text(encoding='utf-8'))
        existing = {(group['headword'], group['lexique_cgram']): group for group in canonical['groups']}
        for group in read('groups.json'):
            previous = existing.get((group['headword'], group['lexique_cgram']))
            if previous is None:
                continue
            with self.subTest(row=group['frozen_row']):
                for key in ('frozen_row', 'subtitle_lemma_per_million', 'subtitle_pos_per_million'):
                    self.assertEqual(previous[key], group[key])
                self.assertEqual(len(previous['senses']), len(group['senses']))
                for left, right in zip(previous['senses'], group['senses'], strict=True):
                    for key in ('sense_id', 'source_headword', 'use', 'use_language', 'excluded_uses', 'cefr'):
                        self.assertEqual(left[key], right[key])
                    self.assertEqual(set(left['fields']), set(right['fields']))
                    for name in LANGUAGES:
                        for key in ('text', 'language', 'source_id', 'source_span', 'content_license', 'modification'):
                            self.assertEqual(left['fields'][name][key], right['fields'][name][key])

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
        self.assertEqual(reviews['kind'], 'separate_independent_field_review')
        self.assertTrue(reviews['independence'])
        self.assertTrue(reviews['independent_acceptance'])
        self.assertTrue(reviews['reviewer'])
        self.assertEqual(rights['independent_rights_fields_accepted'], 200)
        self.assertEqual(rights['independent_review'], 'passed_at_selected_text_boundary')
        decisions = {(item['sense_id'], item['field']): item for item in reviews['fields']}
        self.assertEqual(set(decisions), set(boundaries))
        proposals = {(sense['sense_id'], name): field['text']
                     for group in read('groups-author-proposal.json') for sense in group['senses']
                     for name, field in sense['fields'].items()}
        self.assertEqual((EVIDENCE / 'COPYING').read_text(encoding='utf-8').rstrip(),
                         (ROOT / 'resources/core-pilot-79/COPYING').read_text(encoding='utf-8').rstrip())
        for authority in rights['authority_captures']:
            self.assertTrue(Path(authority['path']).is_file())
        names, anonymous = {}, {}
        registry = captures['contributor_registry']
        self.assertTrue(registry['pagination_complete'])
        for part in registry['parts']:
            response = json.loads(capture_path(part).read_text(encoding='utf-8'))
            for page in response['query']['pages']:
                names.setdefault(page['title'], set()).update(person['name'] for person in page.get('contributors', []))
                if 'anoncontributors' in page:
                    anonymous[page['title']] = page['anoncontributors']
        self.assertNotIn('continue', json.loads(capture_path(registry['parts'][-1]).read_text(encoding='utf-8')))
        fixed = {}
        for part in captures['fixed_revision_recapture']['parts']:
            response = json.loads(capture_path(part).read_text(encoding='utf-8'))
            for page in response['query']['pages']:
                for revision in page['revisions']:
                    fixed[revision['revid']] = revision['slots']['main']['content'].encode('utf-8')
        histories = {item['pageid']: item for item in captures['revision_histories']['sources']}
        self.assertEqual(len(histories), 49)
        raw_by_id = {}
        for capture in captures['sources']:
            source = sources[capture['source_id']]
            context = contexts[source['id']]
            raw = capture_path(capture['raw_path']).read_bytes()
            self.assertEqual(raw, fixed[source['revid']])
            response = json.loads(capture_path(capture['api_path']).read_text(encoding='utf-8'))
            page = next(item for item in response['query']['pages'] if item['pageid'] == source['pageid'])
            revision = next(item for item in page['revisions'] if item['revid'] == source['revid'])
            self.assertEqual((page['title'], revision['timestamp']), (source['title'], source['timestamp']))
            self.assertEqual(raw, revision['slots']['main']['content'].encode('utf-8'))
            self.assertEqual(context['current_contributors'], sorted(names[source['title']]))
            self.assertEqual(context['current_registry_names'], len(names[source['title']]))
            self.assertEqual(context['anonymous_contributors_count'], anonymous.get(source['title'], 0))
            history = histories[source['pageid']]
            revisions = []
            for part in history['parts']:
                response = json.loads(capture_path(part).read_text(encoding='utf-8'))
                page = next(item for item in response['query']['pages'] if item['pageid'] == source['pageid'])
                revisions.extend(page['revisions'])
            self.assertNotIn('continue', response)
            self.assertEqual(revisions[0]['revid'], source['revid'])
            self.assertEqual(len(revisions), history['revisions'])
            self.assertEqual(revisions[-1], history['earliest_revision'])
            self.assertTrue(history['pagination_complete'])
            self.assertEqual(source['selected_text_review'], 'passed_at_selected_text_boundary')
            self.assertEqual(context['selected_text_rights_status'], 'passed_at_selected_text_boundary')
            for layer in source['original_historical_dictionary_layers']:
                self.assertEqual(layer['independent_review'], 'passed_at_original_work_boundary')
            raw_by_id[source['id']] = raw.decode('utf-8').splitlines()
            for location in context['selected_locations']:
                self.assertEqual(location['raw_source_excerpt'],
                                 raw_by_id[source['id']][location['line_1_based'] - 1])
            for frame in context['selected_grammar_and_construction_evidence']:
                self.assertEqual(frame['reviewed_excerpt'],
                                 raw_by_id[source['id']][frame['line_1_based'] - 1])
                self.assertEqual(frame['rights_eligibility_status'], 'passed_at_selected_text_boundary')
            for marker in context['import_notices'] + context['external_reference_markers']:
                self.assertEqual(marker.get('raw_marker', marker.get('raw_reference')),
                                 raw_by_id[source['id']][marker['line_1_based'] - 1])
        for frame in read('source-excerpts.json')['supporting_evidence']:
            self.assertEqual(frame['reviewed_excerpt'],
                             raw_by_id[frame['source_id']][frame['line_1_based'] - 1])
            self.assertEqual(frame['rights_eligibility_status'], 'passed_at_selected_text_boundary')
            self.assertEqual(frame['reviewer'], reviews['reviewer'])
        for frame in rights['construction_sources']:
            self.assertEqual(frame['independent_review'], 'passed_at_selected_text_boundary')
            self.assertTrue(frame['independent_rights_reason'])
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
                    self.assertEqual(boundary['independent_review'], 'passed_at_selected_text_boundary')
                    self.assertEqual(boundary['reviewer'], reviews['reviewer'])
                    self.assertEqual(boundary['accepted_text'], field['text'])
                    self.assertTrue(boundary['independent_rights_reason'])
                    decision = decisions[key]
                    self.assertEqual(decision['proposed_text'], proposals[key])
                    self.assertEqual(decision['accepted_text'], field['text'])
                    self.assertEqual(decision['accepted_text_quality_status'], 'passed')
                    self.assertEqual(decision['rights_eligibility_status'], 'passed_at_selected_text_boundary')
                    self.assertEqual(decision['reviewer'], reviews['reviewer'])
                    self.assertTrue(decision['review_reason'])
                    self.assertTrue(decision['selected_text_rights_reason'])
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
        replacements = {key for key, decision in decisions.items()
                        if decision['proposed_text'] != decision['accepted_text']}
        self.assertEqual(replacements, {('b126-2273-exagérer-ver-selected', 'chinese')})
        uses = {item['sense_id']: item for item in reviews['usage']}
        self.assertEqual(len(uses), 50)
        for group in groups:
            sense = group['senses'][0]
            self.assertEqual(uses[sense['sense_id']]['accepted_usage'], sense['use'])
            self.assertEqual(uses[sense['sense_id']]['everyday_sense_status'], 'passed')
            self.assertTrue(uses[sense['sense_id']]['everyday_sense_reason'])
        self.assertTrue(all(item['alias_admission'] == 'not_admitted' for item in reviews['spelling_relations']))


if __name__ == '__main__':
    unittest.main()
