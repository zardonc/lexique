"""Resource/admission boundary for the frozen #140 author batch."""
import csv
import json
from pathlib import Path
import re
import subprocess
import unittest
from urllib.parse import parse_qs, urlparse

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / 'docs/evidence/issue-140'
LANGUAGES = {'french_definition': 'fr', 'ipa': 'und', 'english': 'en', 'chinese': 'zh'}
MEMBERSHIP = list(zip(
    'racheter fondre thème épisode capacité excitant bloc éveiller diplôme invisible mater sensation humour psy cendre compliment loyer entièrement plateau site élection douloureux cigare orage tire canal pêcher miens trouble réalisateur veille vodka noce rocher câble linge cité guère peintre matière intime porno économie atelier instinct major collection automne pencher personnalité'.split(),
    'VER VER NOM NOM NOM ADJ NOM VER NOM ADJ VER NOM NOM NOM NOM NOM NOM ADV NOM NOM NOM ADJ NOM NOM NOM NOM VER PRO:pos NOM NOM NOM NOM NOM NOM NOM NOM NOM ADV NOM NOM ADJ NOM NOM NOM NOM NOM NOM NOM VER NOM'.split(), strict=True))


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding='utf-8'))


class Batch140ResourceTests(unittest.TestCase):
    def test_frozen_groups_have_complete_proposed_resource_records(self):
        if (EVIDENCE / 'groups.json').exists():
            groups = read('groups.json')
        else:
            canonical = json.loads((ROOT / 'resources/core-pilot-79/curated-inputs.json').read_text(encoding='utf-8'))
            groups = [g for g in canonical['groups'] if 2951 <= g['frozen_row'] <= 3000]
        self.assertEqual([(g['headword'], g['lexique_cgram']) for g in groups], MEMBERSHIP,
                         'The frozen #140 resource slice lacks complete batch content')
        self.assertEqual([g['frozen_row'] for g in groups], list(range(2951, 3001)))
        with (ROOT / 'resources/core-pilot-79/scope-manifest.csv').open(encoding='utf-8-sig', newline='') as handle:
            frozen = list(csv.DictReader(handle))
        self.assertEqual(len(frozen), 4087)
        for group in groups:
            row = frozen[group['frozen_row'] - 1]
            self.assertEqual((group['headword'], group['lexique_cgram']), (row['word'], row['lexique_cgram']))
            for frequency in ('subtitle_lemma_per_million', 'subtitle_pos_per_million'):
                self.assertEqual(group[frequency], row[frequency])
            self.assertEqual(len(group['senses']), 1)
            sense = group['senses'][0]
            self.assertTrue(sense['use'].strip())
            self.assertIsNone(sense['cefr'])
            self.assertEqual(set(sense['fields']), set(LANGUAGES))
            for name, field in sense['fields'].items():
                self.assertEqual(field['language'], LANGUAGES[name])
                self.assertTrue(field['text'].strip())
                self.assertTrue(field['modification'].strip())

    def test_handoff_retains_pending_independent_quality_and_rights(self):
        admission = read('admission.json')
        review = read('field-review.json')
        rights = read('rights.json')
        self.assertEqual(admission['scope_denominator_groups'], 4087)
        self.assertEqual((admission['proposed_groups'], admission['proposed_fields']), (50, 200))
        self.assertEqual((admission['independently_accepted_new_groups'], admission['independently_accepted_new_fields']), (0, 0))
        self.assertEqual((admission['pending_independent_review_fields'], admission['pending_independent_rights_fields']), (200, 200))
        self.assertEqual(admission['missing_admitted_fields'], 200)
        self.assertEqual(admission['missing_authored_fields'], 0)
        self.assertEqual(admission['author_unresolved_content_rows'], [])
        self.assertEqual(admission['unresolved_field_blockers'], [])
        self.assertEqual(admission['newly_admitted_fields'], 0)
        self.assertFalse(review['independent_acceptance'])
        self.assertIsNone(review['reviewer'])
        fields = {(r['sense_id'], r['field']): r for r in review['fields']}
        legal = {(r['sense_id'], r['field']): r for r in rights['fields']}
        self.assertEqual(len(fields), 200)
        self.assertEqual(set(fields), set(legal))
        self.assertEqual((len(review['usage']), len(review['spelling_relations'])), (50, 50))
        for group in read('groups.json'):
            sense = group['senses'][0]
            self.assertEqual(sense['independent_review'], 'pending')
            for name, field in sense['fields'].items():
                key = sense['sense_id'], name
                self.assertEqual(fields[key]['proposed_text'], field['text'])
                self.assertEqual(fields[key]['accepted_text_quality_status'], 'pending')
                self.assertEqual(legal[key]['independent_review'], 'pending')
                self.assertIsNone(fields[key]['reviewer'])
                self.assertIsNone(legal[key]['reviewer'])
                self.assertEqual(legal[key]['exact_source_span'], field['source_span'])
                self.assertEqual(legal[key]['modification'], field['modification'])
                self.assertEqual(field['content_license'], 'CC-BY-SA-4.0')
        for use in review['usage']:
            self.assertEqual(use['accepted_usage_quality_status'], 'pending')
            self.assertIsNone(use['reviewer'])
        for relation in review['spelling_relations']:
            self.assertEqual(relation['independent_review'], 'pending')
            self.assertEqual(relation['alias_admission'], 'not_admitted')
        self.assertEqual(rights['independent_rights_fields_accepted'], 0)
        self.assertEqual((EVIDENCE / 'COPYING').read_bytes().rstrip(),
                         (ROOT / 'resources/core-pilot-79/COPYING').read_bytes().rstrip())

    def test_fields_bind_to_fixed_captures_and_complete_contributor_chains(self):
        captures = read('source-captures.json')
        root = Path(captures['capture_root'])
        self.assertEqual(root.name, 'issue-140')
        self.assertEqual(root.parent.name, 'author-evidence')
        self.assertEqual(str(root), read('admission.json')['author_evidence_root'])
        self.assertEqual(captures['capture_root_fallback'], str(root))

        def retained(name):
            return root.joinpath(*Path(name).parts[2:])

        sources = {s['id']: s for s in read('sources.json')['sources']}
        contexts = {s['id']: s for s in read('source-context.json')['current_content']}
        self.assertEqual(len(sources), 53)
        self.assertEqual(set(sources), set(contexts))
        raw = {}
        for capture in captures['sources']:
            data = json.loads(retained(capture['api_path']).read_text(encoding='utf-8-sig'))
            page = next(p for p in data['query']['pages'] if p['pageid'] == capture['pageid'])
            revision = next(r for r in page['revisions'] if r['revid'] == capture['revid'])
            body = revision['slots']['main']['content']
            self.assertEqual(page['title'], capture['title'])
            self.assertEqual(revision['timestamp'], capture['timestamp'])
            self.assertEqual(body.encode('utf-8'), retained(capture['raw_path']).read_bytes())
            source = sources[capture['source_id']]
            self.assertEqual(source['revid'], capture['revid'])
            self.assertEqual(parse_qs(urlparse(source['fixed_url']).query)['oldid'], [str(capture['revid'])])
            raw[source['id']] = body.splitlines()
        excerpts = read('source-excerpts.json')
        self.assertEqual(len(excerpts['fields']), 200)
        fields = {(r['sense_id'], r['field']): r for r in excerpts['fields']}
        self.assertEqual(len(fields), 200)
        for record in [*excerpts['fields'], *excerpts['supporting_evidence']]:
            line = raw[record['source_id']][record['line_1_based'] - 1]
            column = record['column_1_based'] - 1
            self.assertEqual(line[column:column + len(record['reviewed_excerpt'])], record['reviewed_excerpt'])
            self.assertFalse(record['reviewed_excerpt'].lstrip().startswith('#*'))
            self.assertNotRegex(record['reviewed_excerpt'], r'\[\[(?:File|Image|Fichier):')
        pos_frames = {'NOM': 'nom', 'VER': 'verbe', 'ADJ': 'adjectif', 'ADV': 'adverbe', 'PRO:pos': 'pronom possessif'}
        for group in read('groups.json'):
            sense = group['senses'][0]
            support = [s for s in excerpts['supporting_evidence'] if s['sense_id'] == sense['sense_id']]
            self.assertTrue(any(re.fullmatch(r'==\s*\{\{langue\|fr\}\}\s*==', s['reviewed_excerpt']) for s in support))
            self.assertTrue(any('{{S|' + pos_frames[group['lexique_cgram']] + '|fr' in s['reviewed_excerpt'] for s in support))
            for name, field in sense['fields'].items():
                exact = fields[sense['sense_id'], name]
                self.assertEqual(field['source_id'], exact['source_id'])
                self.assertEqual(field['source_span']['raw_source_excerpt'], exact['reviewed_excerpt'])
                self.assertEqual(field['source_span']['line_1_based'], exact['line_1_based'])
                if name == 'ipa':
                    self.assertEqual(field['text'].strip('/'), exact['ipa_variant_excerpt'])
                    line = exact['reviewed_excerpt']
                    col = exact['ipa_variant_column_1_based'] - 1
                    self.assertEqual(line[col:col + len(exact['ipa_variant_excerpt'])], exact['ipa_variant_excerpt'])
        names, anonymous, chains = {}, {}, {}
        for name in captures['contributor_registry']['parts']:
            path = retained(name)
            data = json.loads(path.read_text(encoding='utf-8'))
            prefix, part = path.name.removesuffix('.api.json').rsplit('-', 1)
            chains.setdefault(prefix, {})[int(part)] = (path, data)
            for page in data['query']['pages']:
                names.setdefault(page['title'], set()).update(c['name'] for c in page.get('contributors', []))
                if 'anoncontributors' in page:
                    anonymous[page['title']] = page['anoncontributors']
        self.assertEqual({k: len(v) for k, v in chains.items()},
                         {'contributors': 6, 'contributors-support': 1, 'contributors-pronouns': 1})
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
            self.assertEqual(context['current_registry_names'], len(names[context['title']]))
            self.assertEqual(context['anonymous_contributors_count'], anonymous.get(context['title'], 0))
        self.assertFalse(captures['contributor_registry']['selected_sentence_authorship_proved'])

    def test_selected_uses_retain_pos_construction_and_import_boundaries(self):
        senses = {g['frozen_row']: g['senses'][0] for g in read('groups.json')}
        self.assertEqual(senses[2978]['source_headword'], 'les miens')
        self.assertEqual(senses[2978]['fields']['ipa']['text'], '/mjɛ̃/')
        self.assertIn('les miens', senses[2978]['fields']['french_definition']['text'])
        self.assertIn('first-person singular', senses[2978]['fields']['english']['text'])
        self.assertIn('ne… guère', senses[2988]['fields']['english']['text'])
        self.assertIn('se pencher', senses[2999]['fields']['english']['text'])
        self.assertIn('slang', senses[2961]['fields']['english']['text'])
        self.assertIn('France', senses[2975]['fields']['english']['text'])
        self.assertIn('feminine', senses[2975]['fields']['english']['text'])
        self.assertIn('psychotherapist', senses[2964]['fields']['english']['text'])
        self.assertIn('top-ranked student', senses[2996]['fields']['english']['text'])
        self.assertIn('singular or plural', senses[2983]['fields']['english']['text'])
        self.assertIn('uncommon and literary', senses[2998]['fields']['english']['text'])
        self.assertNotIn('7 août', senses[2998]['fields']['french_definition']['source_span']['raw_source_excerpt'])
        theme = next(s for s in read('rights.json')['sources'] if s['title'] == 'thème')
        marker = next(m for m in theme['reference_markers'] if 'Import:CFC' in m['raw_reference'])
        self.assertEqual(marker['line_1_based'], 41)
        self.assertEqual(marker['author_rights_proposal'], 'excluded_from_selected_content')
        self.assertFalse(any(s['line_1_based'] == 41 for s in theme['selected_locations']))
        layers = [l for s in read('sources.json')['sources'] for l in s['original_historical_dictionary_layers']]
        self.assertEqual(len(layers), 34)
        for layer in layers:
            self.assertEqual(layer['original_authors'], 'Académie française')
            self.assertTrue(layer['conservative_expiry'].startswith('2006-01-01'))
            self.assertEqual(layer['independent_review'], 'passed_at_original_work_boundary')
            self.assertIn('new wording and spans remain pending', layer['review_reuse_scope'])

    def test_reuse_is_measured_at_the_unchanged_author_base(self):
        reuse = read('reuse.json')
        baseline = json.loads(subprocess.check_output(
            ['git', 'show', reuse['base_commit'] + ':' + reuse['comparison_input']], cwd=ROOT))
        self.assertEqual(baseline['content_version'], reuse['canonical_content_version'])
        self.assertEqual(len(baseline['groups']), reuse['existing_admitted_groups'])
        selected = set(MEMBERSHIP)
        self.assertEqual([g for g in baseline['groups'] if (g['headword'], g['lexique_cgram']) in selected], [])
        self.assertEqual((reuse['reused_groups'], reuse['reused_fields'], reuse['groups']), (0, 0, []))

    def test_original_proposals_are_preserved_for_a_separate_reviewer(self):
        snapshot = read('authored/snapshot.json')
        self.assertEqual(snapshot['role'], 'content_author')
        self.assertFalse(snapshot['independent_acceptance'])
        self.assertEqual(len(snapshot['files']), 9)
        for name in snapshot['files']:
            self.assertEqual((EVIDENCE / 'authored' / name).read_bytes(), (EVIDENCE / name).read_bytes())


if __name__ == '__main__':
    unittest.main()
