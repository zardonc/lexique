"""Verify the exact #143 author package at the resource/admission boundary."""
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


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding='utf-8'))


class Batch143(unittest.TestCase):
    def test_exact_pos_constructions_and_unresolved_fields_are_preserved(self):
        admission = read('admission.json')
        self.assertEqual((admission['frozen_groups'], admission['scope_denominator_groups']), (50, 4087))
        self.assertEqual((admission['proposed_groups'], admission['proposed_fields']), (49, 196))
        self.assertEqual((admission['independently_accepted_new_groups'], admission['independently_accepted_new_fields']), (0, 0))
        self.assertEqual((admission['missing_authored_fields'], admission['missing_admitted_fields']), (4, 200))
        self.assertEqual((admission['pending_independent_review_fields'], admission['pending_independent_rights_fields']), (196, 196))
        self.assertEqual((admission['annotation_exception_groups'], admission['specialist_exception_groups']), (0, 0))
        review = read('field-review.json')
        self.assertFalse(review['independent_acceptance'])
        self.assertIsNone(review['reviewer'])
        self.assertEqual({r['frozen_row'] for r in review['unresolved_fields']}, {3128})
        self.assertEqual({r['field'] for r in review['unresolved_fields']}, set(LANGUAGES))
        groups = {g['frozen_row']: g for g in read('groups.json')}
        pars = groups[3128]
        self.assertEqual((pars['headword'], pars['lexique_cgram'], pars['senses']), ('pars', 'NOM', []))
        self.assertIsNone(pars['owner_exception_approval'])
        uns = groups[3135]
        self.assertEqual((uns['headword'], uns['lexique_cgram']), ('uns', 'PRO:ind'))
        self.assertEqual(uns['senses'][0]['fields']['ipa']['text'], '/œ̃/')
        self.assertIn('les uns', uns['senses'][0]['use'])
        self.assertIn('quelques-uns', uns['senses'][0]['excluded_uses'])
        relations = {r['frozen_row']: r for r in review['spelling_relations']}
        self.assertEqual(relations[3135]['relation'], 'explicit_masculine_plural_pronoun_construction')
        for relation in relations.values():
            self.assertEqual(relation['alias_admission'], 'not_admitted')
            self.assertEqual(relation['independent_review'], 'pending')
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
                    self.assertEqual(use['accepted_usage_quality_status'], 'pending')
                    self.assertIsNone(use['reviewer'])
        self.assertEqual((EVIDENCE / 'COPYING').read_bytes().rstrip(),
                         (ROOT / 'resources/core-pilot-79/COPYING').read_bytes().rstrip())
        self.assertEqual(read('authored/groups.json'), read('groups.json'))
        self.assertEqual(read('authored/field-review.json'), review)

    def test_fields_have_fixed_spans_rights_and_pending_author_status(self):
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
            self.assertEqual(body, retained(capture['raw_path']).read_text(encoding='utf-8'))
            self.assertEqual((page['title'], revision['timestamp']), (capture['title'], capture['timestamp']))
            sid = capture['source_id']
            self.assertEqual(sources[sid]['revid'], capture['revid'])
            self.assertEqual(parse_qs(urlparse(sources[sid]['fixed_url']).query)['oldid'], [str(capture['revid'])])
            raw[sid] = body.splitlines()
        excerpts = read('source-excerpts.json')
        self.assertEqual(len(excerpts['fields']), 196)
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
                self.assertEqual(sense['independent_review'], 'pending')
                for name, field in sense['fields'].items():
                    key = sense['sense_id'], name
                    exact = by_field[key]
                    self.assertEqual(field['source_id'], exact['source_id'])
                    self.assertEqual(field['source_span']['raw_source_excerpt'], exact['reviewed_excerpt'])
                    self.assertEqual(field['text'], reviews[key]['proposed_text'])
                    self.assertEqual(rights[key]['exact_source_span'], field['source_span'])
                    self.assertEqual(rights[key]['modification'], field['modification'])
                    self.assertEqual(field['content_license'], 'CC-BY-SA-4.0')
                    self.assertEqual(reviews[key]['accepted_text_quality_status'], 'pending')
                    self.assertIsNone(reviews[key]['reviewer'])
                    self.assertEqual(rights[key]['independent_review'], 'pending')
                    self.assertIsNone(rights[key]['reviewer'])
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
