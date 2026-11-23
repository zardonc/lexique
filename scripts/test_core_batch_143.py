"""Verify #143 independent field, source and rights decisions at admission."""
import csv
import json
from pathlib import Path
import unittest
from urllib.parse import parse_qs, urlparse

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / 'docs/evidence/issue-143'
MEMBERSHIP = [
    ('septembre', 'NOM'), ('distraire', 'VER'), ('tabac', 'NOM'), ('exception', 'NOM'),
    ('fonder', 'VER'), ('rage', 'NOM'), ('gras', 'ADJ'), ('accusé', 'NOM'),
    ('chevalier', 'NOM'), ('impressionnant', 'ADJ'), ('cuisinier', 'NOM'), ('fragile', 'ADJ'),
    ('saucisse', 'NOM'), ('tremper', 'VER'), ('grève', 'NOM'), ('tendresse', 'NOM'),
    ('ravissant', 'ADJ'), ('abîmer', 'VER'), ('applaudir', 'VER'), ('photographe', 'NOM'),
    ('are', 'NOM'), ('bidon', 'NOM'), ('lecture', 'NOM'), ('mexicain', 'NOM'),
    ('offenser', 'VER'), ('quatrième', 'ADJ'), ('accélérer', 'VER'), ('pars', 'NOM'),
    ('congrès', 'NOM'), ('percer', 'VER'), ('étape', 'NOM'), ('électricité', 'NOM'),
    ('déborder', 'VER'), ('difficulté', 'NOM'), ('uns', 'PRO:ind'), ('vachement', 'ADV'),
    ('curé', 'NOM'), ('féliciter', 'VER'), ('panier', 'NOM'), ('caresser', 'VER'),
    ('galerie', 'NOM'), ('temple', 'NOM'), ('braquer', 'VER'), ('population', 'NOM'),
    ('température', 'NOM'), ('jusque', 'PRE'), ('dominer', 'VER'), ('familial', 'ADJ'),
    ('habituel', 'ADJ'), ('allumette', 'NOM'),
]
LANGUAGES = {'french_definition': 'fr', 'ipa': 'und', 'english': 'en', 'chinese': 'zh'}
RIGHTS_PENDING_ROWS = set()


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding='utf-8'))


class Batch143(unittest.TestCase):
    def test_six_selected_packets_retain_facts_and_reassessed_wording(self):
        expected = {3102: '[[divertir]]', 3110: '[[impressionner|impressionne]]',
                    3115: '[[cesser]] leur [[travail]]', 3123: 'lire seul et des [[yeux]]',
                    3129: 'Assemblée', 3141: '[[salle|Salle]]'}
        groups = {g['frozen_row']: g for g in read('groups.json')}
        first = {g['frozen_row']: g for g in read('first-review/groups.json')}
        for row in expected:
            developed = first[row]['senses'][0]['fields']['french_definition']['source_span']['raw_source_excerpt']
            for name in ('groups.json', 'source-excerpts.json', 'source-context.json',
                         'sources.json', 'rights.json', 'field-review.json'):
                self.assertNotIn(json.dumps(developed, ensure_ascii=False),
                                 json.dumps(read(name), ensure_ascii=False), name)
        for row, excerpt in expected.items():
            for name in ('french_definition', 'english', 'chinese'):
                field = groups[row]['senses'][0]['fields'][name]
                self.assertEqual(field['source_span']['raw_source_excerpt'], excerpt)
                self.assertEqual(field['rights_eligibility_status'], 'passed')
        self.assertEqual(groups[3141]['senses'][0]['fields']['french_definition']['text'],
                         'Salle d’exposition artistique dans un palais ou un musée, dont la longueur dépasse la largeur ; nom féminin.')
        self.assertEqual(groups[3123]['senses'][0]['fields']['french_definition']['text'],
                         'Le fait de lire silencieusement, pour apprendre ou par plaisir ; nom féminin.')
        decision = read('rights-fix-review.json')
        self.assertEqual(decision['resolved_textual_fields'], 18)
        self.assertEqual(decision['unresolved_selected_textual_fields'], 0)
        self.assertEqual(decision['wording_replacements'], 7)
        self.assertEqual({r['frozen_row'] for r in decision['selected_uses']}, set(expected))
        for use in decision['selected_uses']:
            self.assertFalse(use['whole_daf8_us_clearance'])
            self.assertTrue(use['meaning_reason'])
            for field in use['fields']:
                self.assertTrue(field['quality_reason'])
                self.assertEqual(field['quality_status'], 'passed')
                self.assertEqual(field['accepted_text'], groups[use['frozen_row']]['senses'][0]['fields'][field['field']]['text'])

    def test_exact_pos_constructions_and_unresolved_fields_are_preserved(self):
        admission = read('admission.json')
        self.assertEqual((admission['frozen_groups'], admission['scope_denominator_groups']), (50, 4087))
        self.assertEqual((admission['proposed_groups'], admission['proposed_fields']), (50, 200))
        self.assertEqual(admission['proposed_senses'], 50)
        self.assertEqual(admission['unresolved_field_blockers'], [])
        self.assertEqual(admission['pending_independent_review_rows'], [])
        self.assertEqual(admission['independent_quality'], 'passed_for_selected_content')
        self.assertEqual(admission['construction_and_spelling_review'], 'passed_for_selected_uses_and_source_binding_only')
        self.assertEqual((admission['independently_accepted_new_groups'], admission['independently_accepted_new_fields']), (50, 200))
        self.assertEqual((admission['missing_authored_fields'], admission['missing_admitted_fields']), (0, 0))
        self.assertEqual(admission['canonical_missing_fields'], 200)
        self.assertEqual((admission['pending_independent_review_fields'], admission['pending_independent_rights_fields']), (0, 0))
        self.assertEqual((admission['annotation_exception_groups'], admission['specialist_exception_groups']), (0, 1))
        review = read('field-review.json')
        self.assertTrue(review['independent_acceptance'])
        self.assertTrue(review['reviewer'])
        self.assertEqual(review['unresolved_fields'], [])
        groups = {g['frozen_row']: g for g in read('groups.json')}
        pars = groups[3128]
        self.assertEqual((pars['headword'], pars['lexique_cgram']), ('pars', 'NOM'))
        self.assertEqual(len(pars['senses']), 1)
        proposal = read('scope-recommendation.json')
        self.assertEqual(pars['senses'][0]['fields']['ipa']['text'], '/paʁ/')
        self.assertEqual(pars['owner_exception_approval'], proposal['owner_decision'])
        use = next(u for u in review['usage'] if u['frozen_row'] == 3128)
        self.assertEqual(use['everyday_sense_status'], 'approved_specialist_exception')
        self.assertEqual(use['specialist_exception_decision'], proposal['owner_decision'])
        self.assertEqual(use['specialist_exception_decision']['approved_on'], '2026-11-22')
        self.assertEqual(use['specialist_exception_decision']['approved_by'], 'project_owner')
        self.assertEqual(use['specialist_exception_decision']['issue'], 'https://github.com/zardonc/lexique/issues/143')
        uns = groups[3135]
        self.assertEqual((uns['headword'], uns['lexique_cgram']), ('uns', 'PRO:ind'))
        self.assertEqual(uns['senses'][0]['fields']['ipa']['text'], '/œ̃/')
        self.assertIn('les uns', uns['senses'][0]['use'])
        self.assertIn('quelques-uns', uns['senses'][0]['excluded_uses'])
        relations = {r['frozen_row']: r for r in review['spelling_relations']}
        self.assertEqual(relations[3135]['relation'], 'explicit_masculine_plural_pronoun_construction')
        for relation in relations.values():
            self.assertEqual(relation['alias_admission'], 'not_admitted')
            self.assertEqual(relation['independent_review'], 'passed')
            self.assertTrue(relation['review_reason'])
        support = read('source-excerpts.json')['supporting_evidence']
        frames = {'NOM': 'nom', 'VER': 'verbe', 'ADJ': 'adjectif', 'ADV': 'adverbe',
                  'PRE': 'préposition', 'PRO:ind': 'pronom indéfini'}
        for row, group in groups.items():
            if not group['senses']:
                continue
            with self.subTest(row=row):
                records = [r for r in support if r['frozen_row'] == row]
                self.assertTrue(any(r['reviewed_excerpt'] == '== {{langue|fr}} ==' for r in records))
                self.assertTrue(any('{{S|' + frames[group['lexique_cgram']] + '|fr' in r['reviewed_excerpt'] for r in records))
                for use in [u for u in review['usage'] if u['frozen_row'] == row]:
                    self.assertEqual(use['accepted_usage_quality_status'], 'passed')
                    self.assertEqual(use['reviewer'], review['reviewer'])
                    self.assertEqual(use['accepted_usage'], group['senses'][0]['use'])
                    self.assertTrue(use['review_reason'])
        self.assertEqual((EVIDENCE / 'COPYING').read_bytes().rstrip(),
                         (ROOT / 'resources/core-pilot-79/COPYING').read_bytes().rstrip())
        authored_groups = read('authored/groups.json')
        reviewed = {(r['frozen_row'], r['field']): r for r in review['fields']}
        for authored, accepted in zip(authored_groups, read('groups.json')):
            self.assertEqual(authored['frozen_row'], accepted['frozen_row'])
            for old, new in zip(authored['senses'], accepted['senses']):
                self.assertEqual(old['use'], new['use'])
                self.assertEqual(old['excluded_uses'], new['excluded_uses'])
                for field in LANGUAGES:
                    record = reviewed[accepted['frozen_row'], field]
                    self.assertEqual(old['fields'][field]['text'], record['proposed_text'])
                    self.assertEqual(new['fields'][field]['text'], record['accepted_text'])
        admitted = read('accepted-replacements.json')
        self.assertEqual(len(admitted), 11)
        pars_additions = [r for r in admitted if r['frozen_row'] == 3128]
        self.assertEqual(len(pars_additions), 4)
        for record in pars_additions:
            self.assertIsNone(record['authored_original_text'])
            self.assertEqual(record['proposed_text'], record['accepted_text'])

    def test_fields_have_fixed_spans_and_independent_decisions(self):
        captures = read('source-captures.json')
        root = Path(captures['capture_root'])
        sources = {s['id']: s for s in read('sources.json')['sources']}
        contexts = {s['id']: s for s in read('source-context.json')['current_content']}
        raw = {}

        def retained(name):
            return root.joinpath(*Path(name).parts[2:])

        for capture in captures['sources']:
            data = json.loads(retained(capture['api_path']).read_text(encoding='utf-8-sig'))
            page = next(p for p in data['query']['pages'] if p['pageid'] == capture['pageid'])
            revision = next(r for r in page['revisions'] if r['revid'] == capture['revid'])
            body = revision['slots']['main']['content']
            self.assertEqual(body.encode('utf-8'), retained(capture['raw_path']).read_bytes())
            self.assertEqual((page['title'], revision['timestamp']), (capture['title'], capture['timestamp']))
            sid = capture['source_id']
            self.assertEqual(sources[sid]['revid'], capture['revid'])
            self.assertEqual(parse_qs(urlparse(sources[sid]['fixed_url']).query)['oldid'], [str(capture['revid'])])
            raw[sid] = body.splitlines()
        excerpts = read('source-excerpts.json')
        self.assertEqual(len(excerpts['fields']), 200)
        by_field = {(s['sense_id'], s['field']): s for s in excerpts['fields']}
        for exact in [*excerpts['fields'], *excerpts['supporting_evidence']]:
            line = raw[exact['source_id']][exact['line_1_based'] - 1]
            column = exact['column_1_based'] - 1
            self.assertEqual(line[column:column + len(exact['reviewed_excerpt'])], exact['reviewed_excerpt'])
            self.assertFalse(exact['reviewed_excerpt'].lstrip().startswith(('#*', '##*')))
        reviews = {(r['sense_id'], r['field']): r for r in read('field-review.json')['fields']}
        rights = {(r['sense_id'], r['field']): r for r in read('rights.json')['fields']}
        self.assertEqual(set(reviews), set(by_field))
        self.assertEqual(set(rights), set(by_field))
        for group in read('groups.json'):
            for sense in group['senses']:
                self.assertEqual(sense['independent_review'], 'passed')
                joint_pass = group['frozen_row'] not in RIGHTS_PENDING_ROWS
                self.assertEqual(sense['resource_admission'], 'accepted' if joint_pass else
                                 'pending_original_work_territorial_review')
                for name, field in sense['fields'].items():
                    key = sense['sense_id'], name
                    exact = by_field[key]
                    self.assertEqual(field['source_id'], exact['source_id'])
                    self.assertEqual(field['source_span']['raw_source_excerpt'], exact['reviewed_excerpt'])
                    self.assertEqual(field['text'], reviews[key]['accepted_text'])
                    self.assertEqual(rights[key]['exact_source_span'], field['source_span'])
                    self.assertEqual(rights[key]['modification'], field['modification'])
                    self.assertEqual(field['content_license'], 'CC-BY-SA-4.0')
                    self.assertEqual(reviews[key]['accepted_text_quality_status'], 'passed')
                    self.assertEqual(reviews[key]['reviewer'], read('field-review.json')['reviewer'])
                    pending = group['frozen_row'] in RIGHTS_PENDING_ROWS and name != 'ipa'
                    expected = 'pending_original_work_territorial_review' if pending else 'passed'
                    self.assertEqual(rights[key]['independent_review'], expected)
                    self.assertEqual(field['rights_eligibility_status'], expected)
                    self.assertEqual(reviews[key]['rights_eligibility_status'], expected)
                    self.assertTrue(reviews[key]['review_reason'])
                    self.assertEqual(rights[key]['reviewer'], reviews[key]['reviewer'])
                    self.assertEqual(rights[key]['review_reason'], reviews[key]['selected_text_rights_reason'])
                    self.assertEqual(rights[key]['accepted_text'], field['text'])
                    if name == 'ipa':
                        self.assertEqual(field['text'].strip('/'), exact['ipa_variant_excerpt'])
                        line = raw[exact['source_id']][exact['line_1_based'] - 1]
                        column = exact['ipa_variant_column_1_based'] - 1
                        self.assertEqual(line[column:column + len(exact['ipa_variant_excerpt'])], exact['ipa_variant_excerpt'])
                        if group['frozen_row'] == 3135:
                            self.assertTrue(line[:column].endswith('pmp=le.z‿'))
                    before = raw[exact['source_id']][:exact['line_1_based']]
                    language = next(l for l in reversed(before) if l.startswith('== '))
                    self.assertEqual(language, '== {{langue|fr}} ==')
                    frame = next(l for l in reversed(before) if l.startswith('=== {{S|')
                                 and not any(t in l for t in ('étymologie', 'prononciation', 'références')))
                    pos = {'NOM': 'nom', 'VER': 'verbe', 'ADJ': 'adjectif', 'ADV': 'adverbe',
                           'PRE': 'préposition', 'PRO:ind': 'pronom indéfini'}[group['lexique_cgram']]
                    self.assertIn('{{S|' + pos + '|fr', frame)
        names, anonymous, chains = {}, {}, {}
        for name in captures['contributor_registry']['parts']:
            path = retained(name)
            data = json.loads(path.read_text(encoding='utf-8'))
            prefix, part = path.name.removesuffix('.api.json').rsplit('-', 1)
            chains.setdefault(prefix, {})[int(part)] = path, data
            for page in data['query']['pages']:
                names.setdefault(page['title'], set()).update(p['name'] for p in page.get('contributors', []))
                if 'anoncontributors' in page:
                    anonymous[page['title']] = page['anoncontributors']
        for parts in chains.values():
            self.assertEqual(sorted(parts), list(range(1, len(parts) + 1)))
            for part, (path, data) in parts.items():
                if part == len(parts):
                    self.assertFalse(data.get('continue'))
                else:
                    query = parse_qs(urlparse(parts[part + 1][0].with_suffix('.request.txt').read_text(encoding='utf-8')).query)
                    for key, value in data['continue'].items():
                        self.assertEqual(query[key], [str(value)])
        for context in contexts.values():
            self.assertEqual(context['current_contributors'], sorted(names[context['title']]))
            self.assertEqual(context['anonymous_contributors_count'], anonymous.get(context['title'], 0))
        self.assertFalse(captures['contributor_registry']['selected_sentence_authorship_proved'])
        unresolved = read('unresolved-evidence.json')
        for exact in unresolved['source_evidence']:
            self.assertEqual(raw[exact['source_id']][exact['line_1_based'] - 1], exact['raw_source_excerpt'])
        for title in ('pars', 'par'):
            self.assertTrue(any(s['title'] == title and s['raw_source_excerpt'] == '== {{langue|fr}} =='
                                for s in unresolved['source_evidence']))

    def test_actual_original_boundaries_and_owner_scope_are_distinct(self):
        boundaries = read('original-boundaries.json')
        self.assertEqual(len(boundaries['selected_uses']), 50)
        self.assertEqual({r['frozen_row'] for r in boundaries['selected_uses']
                          if r['textual_rights_status'] != 'passed'}, RIGHTS_PENDING_ROWS)
        for decision in boundaries['selected_uses']:
            self.assertTrue(decision['selected_expression_reason'])
            self.assertTrue(decision['support_expression_reason'])
            if decision['frozen_row'] in {3102, 3110, 3115, 3123, 3129, 3141}:
                self.assertIn('cnrtl.fr/definition/academie8/', decision['original_comparison']['url'])
                self.assertTrue(decision['original_comparison']['matched_expression'])
        scope = read('scope-recommendation.json')
        self.assertEqual((scope['frozen_row'], scope['headword'], scope['pos']), (3128, 'pars', 'NOM'))
        self.assertEqual(scope['owner_decision'], read('owner-approval.json')['decision'])
        self.assertEqual(scope['recommended_ipa'], '/paʁ/')
        self.assertEqual([s['revid'] for s in scope['sources']], [39518389, 39972270])
        pars = next(g for g in read('groups.json') if g['frozen_row'] == 3128)
        fields = pars['senses'][0]['fields']
        for name in ('french_definition', 'english', 'chinese'):
            self.assertEqual(fields[name]['text'], scope['proposed_' + name])
        ipa = fields['ipa']
        self.assertEqual(ipa['source_id'], 'b143_fr_2252')
        self.assertEqual(ipa['source_span']['line_1_based'], 294)
        support = [s for s in read('source-excerpts.json')['supporting_evidence']
                   if s['frozen_row'] == 3128]
        self.assertTrue(any(s['source_id'] == 'b143_fr_67055' and s['line_1_based'] == 21
                            and s['reviewed_excerpt'] == '{{fr-rég|s=par}}' for s in support))
        self.assertFalse(any(s['source_id'] == 'b143_fr_67055' and s['line_1_based'] == 6 for s in support))

    def test_exact_frozen_resource_slice_has_content_or_explicit_blockers(self):
        if (EVIDENCE / 'groups.json').exists():
            groups = read('groups.json')
        else:
            canonical = json.loads((ROOT / 'resources/core-pilot-79/curated-inputs.json').read_text(encoding='utf-8'))
            groups = [g for g in canonical['groups'] if 3101 <= g['frozen_row'] <= 3150]
        self.assertEqual([(g['headword'], g['lexique_cgram']) for g in groups], MEMBERSHIP,
                         'The exact #143 resource slice lacks admitted content or an explicit author handoff')
        self.assertEqual([g['frozen_row'] for g in groups], list(range(3101, 3151)))
        with (ROOT / 'resources/core-pilot-79/scope-manifest.csv').open(encoding='utf-8-sig', newline='') as handle:
            frozen = list(csv.DictReader(handle))
        self.assertEqual(len(frozen), 4087)
        for group in groups:
            row = frozen[group['frozen_row'] - 1]
            self.assertEqual((group['headword'], group['lexique_cgram']), (row['word'], row['lexique_cgram']))
            for frequency in ('subtitle_lemma_per_million', 'subtitle_pos_per_million'):
                self.assertEqual(group[frequency], row[frequency])
            if not group['senses']:
                self.assertTrue(group['unresolved_reason'])
                self.assertEqual(set(group['unresolved_fields']), set(LANGUAGES))
            for sense in group['senses']:
                self.assertTrue(sense['use'].strip())
                self.assertEqual(set(sense['fields']), set(LANGUAGES))
                for name, field in sense['fields'].items():
                    self.assertEqual(field['language'], LANGUAGES[name])
                    self.assertTrue(field['text'].strip())


if __name__ == '__main__':
    unittest.main()
