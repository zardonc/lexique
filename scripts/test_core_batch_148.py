"""Check #148 at the authorized frozen resource/admission boundary."""
import csv
import json
from pathlib import Path
from urllib.parse import parse_qs, urlparse
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / 'docs/evidence/issue-148'
MEMBERSHIP = [
    ('prétexte', 'NOM'), ('brute', 'NOM'), ('oxygène', 'NOM'),
    ('petit-déjeuner', 'NOM'), ('audience', 'NOM'), ('tonton', 'NOM'),
    ('trafic', 'NOM'), ('curiosité', 'NOM'), ('quai', 'NOM'), ('yeah', 'ONO'),
    ('suspendre', 'VER'), ('dragon', 'NOM'), ('écrouler', 'VER'), ('cran', 'NOM'),
    ('traquer', 'VER'), ('hélicoptère', 'NOM'), ('influence', 'NOM'), ('rein', 'NOM'),
    ('touche', 'NOM'), ('commentaire', 'NOM'), ('engueuler', 'VER'), ('brique', 'NOM'),
    ('football', 'NOM'), ('relax', 'ADJ'), ('conclusion', 'NOM'), ('etc', 'ADV'),
    ('acier', 'NOM'), ('pratiquer', 'VER'), ('guitare', 'NOM'), ('ambassadeur', 'NOM'),
    ('atmosphère', 'NOM'), ('interdit', 'ADJ'), ('pervers', 'NOM'), ('pognon', 'NOM'),
    ('cueillir', 'VER'), ('pétrole', 'NOM'), ('accueil', 'NOM'), ('clou', 'NOM'),
    ('tragique', 'ADJ'), ('magicien', 'NOM'), ('revanche', 'NOM'), ('bouleverser', 'VER'),
    ('conflit', 'NOM'), ('insecte', 'NOM'), ('mondial', 'ADJ'), ('confortable', 'ADJ'),
    ('regret', 'NOM'), ('aérien', 'ADJ'), ('commercial', 'ADJ'), ('charité', 'NOM'),
]


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding='utf-8'))


class Batch148Tests(unittest.TestCase):
    def test_tonton_chinese_retains_uncle_relationship(self):
        group = next(g for g in read('groups.json') if g['frozen_row'] == 3356)
        self.assertEqual(group['senses'][0]['fields']['chinese']['text'],
                         '叔叔、伯伯、舅舅、姑父或姨父的儿童亲昵称呼（阳性名词）。')

    def test_exact_frozen_groups_have_complete_resource_records(self):
        if (EVIDENCE / 'groups.json').exists():
            groups = read('groups.json')
        else:
            canonical = json.loads((ROOT / 'resources/core-pilot-79/curated-inputs.json').read_text(encoding='utf-8'))
            groups = [g for g in canonical['groups'] if 3351 <= g['frozen_row'] <= 3400]
        self.assertEqual([(g['headword'], g['lexique_cgram']) for g in groups], MEMBERSHIP,
                         'The frozen #148 resource slice lacks batch content')
        self.assertEqual([g['frozen_row'] for g in groups], list(range(3351, 3401)))
        with (ROOT / 'resources/core-pilot-79/scope-manifest.csv').open(encoding='utf-8-sig', newline='') as handle:
            frozen = list(csv.DictReader(handle))
        self.assertEqual(len(frozen), 4087)
        for group in groups:
            row = frozen[group['frozen_row'] - 1]
            self.assertEqual((group['headword'], group['lexique_cgram']), (row['word'], row['lexique_cgram']))
            for frequency in ('subtitle_lemma_per_million', 'subtitle_pos_per_million'):
                self.assertEqual(group[frequency], row[frequency])
            self.assertTrue(group['senses'])
            for sense in group['senses']:
                self.assertTrue(sense['use'].strip())
                self.assertIsNone(sense['cefr'])
                self.assertEqual(set(sense['fields']), {'french_definition', 'ipa', 'english', 'chinese'})
                for name, field in sense['fields'].items():
                    self.assertEqual(field['language'], {'french_definition': 'fr', 'ipa': 'und', 'english': 'en', 'chinese': 'zh'}[name])
                    self.assertTrue(field['text'].strip())
                    self.assertTrue(field['modification'].strip())

    def test_authored_snapshot_preserves_pending_proposals(self):
        rights, admission, review = (read('authored/' + name) for name in
                                    ('rights.json', 'admission.json', 'field-review.json'))
        self.assertEqual((admission['proposed_groups'], admission['proposed_fields']), (50, 200))
        self.assertEqual(admission['scope_denominator_groups'], 4087)
        self.assertEqual(admission['missing_admitted_fields'], 200)
        self.assertEqual(admission['unresolved_field_blockers'], [])
        self.assertEqual(rights['independent_rights_fields_accepted'], 0)
        self.assertFalse(review['independent_acceptance'])
        self.assertIsNone(review['reviewer'])
        self.assertEqual((len(review['fields']), len(review['usage'])), (200, 50))
        expected = {(g['senses'][0]['sense_id'], name): field for g in read('authored/groups.json')
                    for name, field in g['senses'][0]['fields'].items()}
        self.assertEqual({(f['sense_id'], f['field']) for f in rights['fields']}, set(expected))
        for field in rights['fields']:
            proposed = expected[field['sense_id'], field['field']]
            self.assertEqual(field['exact_source_span'], proposed['source_span'])
            self.assertEqual(field['license'], 'CC-BY-SA-4.0')
            self.assertEqual(field['independent_review'], 'pending')
            self.assertIsNone(field['reviewer'])
            self.assertTrue(field['attribution'])
            self.assertTrue(field['modification'])

    def test_reviewed_shard_has_exact_field_use_source_rights_alignment(self):
        groups, review, rights = read('groups.json'), read('field-review.json'), read('rights.json')
        admission = read('admission.json')
        self.assertEqual(review['kind'], 'separate_independent_field_review')
        self.assertTrue(review['independence'])
        self.assertTrue(review['independent_acceptance'])
        self.assertTrue(review['reviewer'])
        self.assertEqual((admission['independently_accepted_new_groups'],
                          admission['independently_accepted_new_fields']), (50, 200))
        self.assertEqual(admission['newly_admitted_fields'], 0)
        self.assertEqual(admission['missing_admitted_fields'], 200)
        self.assertEqual(admission['pending_independent_review_fields'], 0)
        self.assertEqual(admission['pending_independent_rights_fields'], 0)
        self.assertEqual(admission['unresolved_field_blockers'], [])
        self.assertEqual((rights['independent_rights_fields_accepted'], len(review['fields']),
                          len(review['usage'])), (200, 200, 50))
        authored = {(g['senses'][0]['sense_id'], name): f for g in read('authored/groups.json')
                    for name, f in g['senses'][0]['fields'].items()}
        fields = {(f['sense_id'], f['field']): f for f in review['fields']}
        legal = {(f['sense_id'], f['field']): f for f in rights['fields']}
        self.assertEqual(len(fields), 200)
        self.assertEqual(set(fields), set(authored))
        self.assertEqual(set(legal), set(fields))
        usage = {u['sense_id']: u for u in review['usage']}
        self.assertEqual(len(usage), 50)
        sources = {s['id']: s for s in read('sources.json')['sources']}
        changed = []
        for group in groups:
            self.assertEqual(len(group['senses']), 1)
            sense = group['senses'][0]
            u = usage[sense['sense_id']]
            self.assertEqual(sense['independent_review'], 'passed_after_exact_accepted_replacements')
            self.assertEqual((sense['use'], sense['use_language'], sense['source_headword']),
                             (u['accepted_usage'], u['language'], u['source_headword']))
            self.assertEqual(u['accepted_usage_quality_status'], 'passed')
            self.assertEqual(u['everyday_sense_status'], 'passed')
            self.assertEqual(u['reviewer'], review['reviewer'])
            self.assertTrue(u['review_reason'])
            self.assertTrue(u['everyday_sense_reason'])
            for name, f in sense['fields'].items():
                key = sense['sense_id'], name
                r, l = fields[key], legal[key]
                self.assertEqual(r['proposed_text'], authored[key]['text'])
                self.assertEqual(r['accepted_text'], f['text'])
                self.assertEqual(l['accepted_text'], f['text'])
                self.assertEqual(f['source_span'], l['exact_source_span'])
                self.assertEqual((f['source_id'], r['exact_source']['source_id'], l['source_id']),
                                 (f['source_id'], f['source_id'], f['source_id']))
                self.assertEqual(r['exact_source']['revid'], sources[f['source_id']]['revid'])
                self.assertEqual(f['independent_review'], 'passed')
                self.assertEqual(r['accepted_text_quality_status'], 'passed')
                self.assertEqual(r['rights_eligibility_status'], 'passed_at_selected_text_boundary')
                self.assertEqual(l['independent_review'], 'passed_at_selected_text_boundary')
                self.assertEqual(r['reviewer'], review['reviewer'])
                self.assertEqual(l['reviewer'], review['reviewer'])
                self.assertTrue(r['review_reason'])
                self.assertTrue(r['selected_text_rights_reason'])
                self.assertEqual(l['review_reason'], r['selected_text_rights_reason'])
                self.assertEqual(sources[f['source_id']]['selected_text_review'], 'passed_for_selected_spans')
                if f['text'] != authored[key]['text']:
                    changed.append((group['frozen_row'], name))
        self.assertEqual(changed, [(3356, 'chinese')])
        replacements = read('accepted-replacements.json')['replacements']
        self.assertEqual(len(replacements), 1)
        replacement = replacements[0]
        self.assertEqual((replacement['frozen_row'], replacement['field']), (3356, 'chinese'))
        key = replacement['sense_id'], replacement['field']
        self.assertEqual((replacement['proposed_text'], replacement['accepted_text']),
                         (fields[key]['proposed_text'], fields[key]['accepted_text']))
        self.assertEqual(replacement['exact_source'], fields[key]['exact_source'])
        for relation in review['spelling_relations']:
            self.assertEqual(relation['alias_admission'], 'not_admitted')
            self.assertTrue(relation['review_reason'])

    def test_fixed_source_bytes_spans_and_complete_contributor_chains(self):
        captures = read('source-captures.json')
        root = Path(captures['capture_root'])
        self.assertEqual((root.parent.name, root.name), ('author-evidence', 'issue-148'))
        self.assertEqual(len(captures['sources']), 53)
        sources = {s['id']: s for s in read('sources.json')['sources']}
        raw = {}

        def retained(name):
            path = Path(name)
            self.assertEqual(path.parts[:2], ('target', 'issue-148'))
            return root.joinpath(*path.parts[2:])

        for capture in captures['sources']:
            data = json.loads(retained(capture['api_path']).read_text(encoding='utf-8-sig'))
            page = next(p for p in data['query']['pages'] if p['pageid'] == capture['pageid'])
            revision = next(r for r in page['revisions'] if r['revid'] == capture['revid'])
            self.assertEqual((page['title'], revision['timestamp']), (capture['title'], capture['timestamp']))
            body = revision['slots']['main']['content']
            self.assertEqual(body.encode('utf-8'), retained(capture['raw_path']).read_bytes())
            source = next(s for s in sources.values() if s['pageid'] == capture['pageid'])
            self.assertEqual(source['revid'], capture['revid'])
            self.assertEqual(parse_qs(urlparse(source['fixed_url']).query)['oldid'], [str(capture['revid'])])
            self.assertTrue(retained(capture['api_path']).with_suffix('.request.txt').is_file())
            raw[source['id']] = body.splitlines()
        excerpts = read('source-excerpts.json')
        self.assertEqual(len(excerpts['fields']), 200)
        for record in [*excerpts['fields'], *excerpts['supporting_evidence']]:
            line = raw[record['source_id']][record['line_1_based'] - 1]
            column = record['column_1_based'] - 1
            self.assertEqual(line[column:column+len(record['reviewed_excerpt'])], record['reviewed_excerpt'])
            self.assertEqual(record['revid'], sources[record['source_id']]['revid'])
            self.assertFalse(record['reviewed_excerpt'].lstrip().startswith(('#*', '##*')))
        by_field = {(r['sense_id'], r['field']): r for r in excerpts['fields']}
        self.assertEqual(len(by_field), 200)
        pos_frames = {'NOM': 'nom', 'VER': 'verbe', 'ADJ': 'adjectif', 'ADV': 'adverbe', 'ONO': 'interjection'}
        for group in read('groups.json'):
            sense = group['senses'][0]
            for name, field in sense['fields'].items():
                record = by_field[sense['sense_id'], name]
                self.assertEqual(field['source_id'], record['source_id'])
                self.assertEqual(field['source_span']['line_1_based'], record['line_1_based'])
                self.assertEqual(field['source_span']['raw_source_excerpt'], record['reviewed_excerpt'])
                preceding = raw[record['source_id']][:record['line_1_based']]
                language = next(line for line in reversed(preceding) if line.startswith('== '))
                self.assertRegex(language, r'^==\s*\{\{langue\|fr\}\}\s*==$')
                if name == 'ipa':
                    self.assertEqual(field['text'].strip('/'), record['ipa_variant_excerpt'])
                    column = record['ipa_variant_column_1_based'] - 1
                    self.assertEqual(record['reviewed_excerpt'][column:column+len(record['ipa_variant_excerpt'])], record['ipa_variant_excerpt'])
                else:
                    pos = next(line for line in reversed(preceding) if line.startswith('=== {{S|'))
                    self.assertIn('{{S|' + pos_frames[group['lexique_cgram']] + '|fr', pos)
        names, anonymous, chains = {}, {}, {}
        for name in captures['contributor_registry']['parts']:
            path = retained(name)
            data = json.loads(path.read_text(encoding='utf-8-sig'))
            prefix, number = path.name.removesuffix('.api.json').rsplit('-', 1)
            chains.setdefault(prefix, {})[int(number)] = (path, data)
            for page in data['query']['pages']:
                names.setdefault(page['title'], set()).update(p['name'] for p in page.get('contributors', []))
                if 'anoncontributors' in page:
                    anonymous[page['title']] = page['anoncontributors']
        self.assertEqual(set(chains), {f'contributors-new-{n}' for n in range(1, 12)} | {'contributors-support'})
        for parts in chains.values():
            self.assertEqual(sorted(parts), list(range(1, len(parts)+1)))
            for number, (path, data) in parts.items():
                continuation = data.get('continue')
                if number == len(parts):
                    self.assertFalse(continuation, str(path))
                else:
                    self.assertTrue(continuation)
                    query = parse_qs(urlparse(parts[number+1][0].with_suffix('.request.txt').read_text(encoding='utf-8')).query)
                    for key, value in continuation.items():
                        self.assertEqual(query[key], [str(value)])
        self.assertEqual(set(names), {s['title'] for s in sources.values()})
        for context in read('source-context.json')['current_content']:
            self.assertEqual(context['current_contributors'], sorted(names[context['title']]))
            self.assertEqual(context['current_registry_names'], len(names[context['title']]))
            self.assertEqual(context['anonymous_contributors_count'], anonymous.get(context['title'], 0))
        self.assertFalse(captures['contributor_registry']['selected_sentence_authorship_proved'])

    def test_spelling_and_pronominal_bindings_preserve_frozen_groups(self):
        groups = {g['frozen_row']: g for g in read('groups.json')}
        support = read('source-excerpts.json')['supporting_evidence']
        for row, source, token in ((3354, 'petit déjeuner', 'Variante orthographique'),
                                   (3376, 'etc.', 'Variante orthographique')):
            sense = groups[row]['senses'][0]
            self.assertEqual(sense['source_headword'], source)
            self.assertTrue(any(token in r['reviewed_excerpt'] for r in support if r['sense_id'] == sense['sense_id']))
        sense = groups[3363]['senses'][0]
        self.assertEqual((groups[3363]['headword'], groups[3363]['lexique_cgram']), ('écrouler', 'VER'))
        for field in ('french_definition', 'english', 'chinese'):
            self.assertIn('s’écrouler', sense['fields'][field]['text'].lower())
        self.assertTrue(any('exclusivement=oui' in r['reviewed_excerpt'] for r in support if r['sense_id'] == sense['sense_id']))
        self.assertEqual(groups[3360]['senses'][0]['fields']['ipa']['text'], '/jɛ/')
        self.assertEqual(groups[3352]['senses'][0]['fields']['ipa']['text'], '/bʁyt/')
        self.assertEqual(groups[3353]['senses'][0]['fields']['french_definition']['source_span']['line_1_based'], 47)
        self.assertEqual(groups[3383]['senses'][0]['fields']['french_definition']['source_span']['line_1_based'], 64)
        for relation in read('source-context.json')['spelling_relations']:
            self.assertEqual(relation['alias_admission'], 'not_admitted')

    def test_original_proposals_and_notices_are_available_for_review(self):
        for name in ('groups.json', 'field-review.json', 'rights.json', 'sources.json', 'source-context.json',
                     'source-excerpts.json', 'admission.json', 'source-captures.json', 'reuse.json'):
            self.assertTrue((EVIDENCE / 'authored' / name).is_file(), name)
        notices = (EVIDENCE / 'NOTICES.md').read_text(encoding='utf-8')
        self.assertEqual((EVIDENCE / 'COPYING').read_bytes().rstrip(), (ROOT / 'resources/core-pilot-79/COPYING').read_bytes().rstrip())
        for source in read('sources.json')['sources']:
            self.assertIn(source['fixed_url'], notices)
            self.assertIn(source['history_url'], notices)
        rights = (EVIDENCE / 'selected-rights.md').read_text(encoding='utf-8')
        self.assertIn('2006-01-01', rights)
        self.assertIn('CC BY-SA 4.0', rights)


if __name__ == '__main__':
    unittest.main()
