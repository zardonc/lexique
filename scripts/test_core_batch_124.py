"""Check #124 at the parent-approved resource/admission boundary."""
import csv
import json
from pathlib import Path
import re
import unittest
from urllib.parse import parse_qs, urlparse

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-124"
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


def capture_path(relative):
    path = ROOT / relative
    if path.is_file():
        return path
    return Path('C:/Users/inthe/IdeaProjects/lexique-batch-124') / relative


class Batch124Tests(unittest.TestCase):
    def test_selected_original_work_layers_are_resolved_in_live_rights(self):
        rights = read('rights.json')
        for record in rights['fields'] + rights['construction_sources'] + rights['sources']:
            for layer in record['original_historical_dictionary_layers']:
                self.assertEqual(layer['independent_review'], 'passed_at_original_work_boundary')
                self.assertEqual(layer['conservative_expiry'], '2006-01-01')

    def test_all_200_exact_fields_and_source_frames_have_independent_acceptance(self):
        review = read('field-review.json')
        self.assertEqual(review['kind'], 'separate_independent_field_review')
        self.assertTrue(review['independence'])
        self.assertTrue(review['independent_acceptance'])
        self.assertEqual(len(review['fields']), 200)
        originals = {g['frozen_row']: g for g in read('review-candidates.json')}
        accepted = {g['frozen_row']: g for g in read('groups.json')}
        decisions = {(x['sense_id'], x['field']): x for x in review['fields']}
        boundaries = {(x['sense_id'], x['field']): x for x in read('rights.json')['fields']}
        for row, group in accepted.items():
            sense = group['senses'][0]
            original = originals[row]['senses'][0]
            self.assertEqual(sense['independent_review'], 'passed_after_exact_accepted_replacements')
            self.assertEqual(sense['use'], original['use'])
            for name, field in sense['fields'].items():
                with self.subTest(row=row, field=name):
                    decision = decisions[(sense['sense_id'], name)]
                    boundary = boundaries[(sense['sense_id'], name)]
                    self.assertEqual(field['text'], original['fields'][name]['text'])
                    self.assertEqual(field['rights_eligibility_status'], 'passed_at_selected_text_boundary')
                    self.assertEqual(decision['proposed_text'], original['fields'][name]['text'])
                    self.assertEqual(decision['accepted_text'], field['text'])
                    self.assertEqual(decision['accepted_text_quality_status'], 'passed')
                    self.assertEqual(decision['rights_eligibility_status'], 'passed_at_selected_text_boundary')
                    self.assertEqual(boundary['accepted_text'], field['text'])
                    self.assertEqual(boundary['exact_source_span'], field['source_span'])
                    self.assertEqual(boundary['independent_review'], 'passed_at_selected_text_boundary')
                    self.assertEqual(boundary['reviewer'], review['reviewer'])
                    self.assertTrue(decision['review_reason'])
                    self.assertTrue(decision['selected_text_rights_reason'])
                    self.assertTrue(boundary['independent_rights_reason'])
        for context in read('source-context.json')['current_content']:
            self.assertEqual(context['selected_text_rights_status'], 'passed_at_selected_text_boundary')
            self.assertEqual(context['reviewer'], review['reviewer'])
            self.assertTrue(context['independent_selected_boundary_reason'])
        for support in read('rights.json')['construction_sources']:
            self.assertEqual(support['independent_review'], 'passed_at_selected_text_boundary')
            self.assertTrue(support['independent_rights_reason'])
        self.assertEqual(len(review['usage']), 50)
        self.assertEqual(len(review['spelling_relations']), 50)
        for usage in review['usage']:
            self.assertEqual(usage['accepted_usage_quality_status'], 'passed')
            self.assertEqual(usage['accepted_usage'], accepted[usage['frozen_row']]['senses'][0]['use'])
            self.assertTrue(usage['review_reason'])
        for spelling in review['spelling_relations']:
            self.assertEqual(spelling['accepted_spelling_quality_status'], 'passed')
            self.assertEqual(spelling['alias_admission'], 'not_admitted')
            self.assertTrue(spelling['review_reason'])
        self.assertEqual(read('source-context.json')['spelling_relations'], review['spelling_relations'])
        admission = read('admission.json')
        self.assertEqual((admission['independently_accepted_new_groups'], admission['independently_accepted_new_fields']), (50, 200))
        self.assertEqual(admission['pending_independent_review_rows'], [])
        self.assertEqual(admission['pending_independent_review_fields'], 0)
        self.assertEqual(admission['pending_independent_rights_fields'], 0)
        self.assertEqual((admission['accepted_groups'], admission['accepted_fields']), (50, 200))
        self.assertEqual((admission['missing_groups'], admission['missing_fields']), (0, 0))
        if admission['status'] == 'admitted_core_129_v1_windows_x64_development_resource':
            self.assertEqual(admission['newly_admitted_fields'], 200)
            self.assertEqual(admission['resource_admission'], 'passed_combined_conversion_rights_notices_and_native_gates')
        else:
            self.assertEqual(admission['newly_admitted_fields'], 0)
            self.assertEqual(admission['resource_admission'], 'pending_parent_canonical_assembly_and_output_validation')
        # Identical canonical records may overlap after parent assembly.
        canonical = {g['frozen_row']: g for g in json.loads(
            (ROOT / 'resources/core-pilot-79/curated-inputs.json').read_text(encoding='utf-8'))['groups']}
        for row in accepted.keys() & canonical.keys():
            self.assertEqual(accepted[row], canonical[row])

    def test_original_proposals_preserve_exact_frozen_groups_and_authored_counts(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            scope = list(csv.DictReader(handle))
        groups = read("review-candidates.json")
        self.assertEqual([group["frozen_row"] for group in groups], list(range(2151, 2201)))
        for group in groups:
            frozen = scope[group["frozen_row"] - 1]
            self.assertEqual((group["headword"], group["lexique_cgram"]),
                             (frozen["word"], frozen["lexique_cgram"]))
            for frequency in ("subtitle_lemma_per_million", "subtitle_pos_per_million"):
                self.assertEqual(group[frequency], frozen[frequency])
            self.assertEqual(len(group["senses"]), 1)
            sense = group["senses"][0]
            self.assertTrue(sense["use"])
            self.assertIsNone(sense["cefr"])
            self.assertEqual(sense["independent_review"], "pending")
            self.assertEqual(set(sense["fields"]), set(LANGUAGES))
            for name, language in LANGUAGES.items():
                self.assertEqual(sense["fields"][name]["language"], language)
                self.assertTrue(sense["fields"][name]["text"].strip())
        admission = read("admission-author-proposal.json")
        self.assertEqual((admission["proposed_groups"], admission["proposed_fields"]), (50, 200))
        self.assertEqual((admission["reused_groups"], admission["reused_fields"]), (0, 0))
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual(admission["independently_accepted_new_fields"], 0)
        self.assertIsNone(admission["independent_reviewer"])
        self.assertEqual(admission["pending_independent_review_rows"], list(range(2151, 2201)))
        self.assertEqual(admission["missing_authored_fields"], 0)
        self.assertEqual(read("reuse.json")["groups"], [])

    def test_fixed_field_spans_contributors_and_original_rights_are_reviewable(self):
        sources = {item['id']: item for item in read('sources.json')['sources']}
        contexts = {item['id']: item for item in read('source-context.json')['current_content']}
        capture_document = read('source-captures.json')
        captures = {item['source_id']: item for item in capture_document['sources']}
        review = read('field-review.json')
        proposals = {(item['sense_id'], item['field']): item for item in review['fields']}
        excerpts = {(item['sense_id'], item['field']): item for item in read('source-excerpts.json')['fields']}
        rights = read('rights.json')
        boundaries = {(item['sense_id'], item['field']): item for item in rights['fields']}
        self.assertEqual(len(excerpts), 200)
        self.assertEqual(set(excerpts), set(proposals))
        self.assertEqual(set(excerpts), set(boundaries))
        self.assertTrue(review['independent_acceptance'])
        self.assertTrue(review['reviewer'])
        self.assertEqual(rights['independent_rights_fields_accepted'], 200)
        self.assertEqual(rights['independent_review'], 'passed_at_selected_text_boundary')
        self.assertTrue(rights['redistribution_obligations'])
        for authority in rights['authority_captures']:
            self.assertTrue(capture_path(authority['path']).is_file())
        self.assertEqual((EVIDENCE / 'COPYING').read_bytes().rstrip(),
                         (ROOT / 'resources/core-pilot-79/COPYING').read_bytes().rstrip())
        registry = capture_document['contributor_registry']
        self.assertTrue(registry['pagination_complete'])
        names, anonymous, request_sets = {}, {}, {}
        for relative in registry['parts']:
            path = capture_path(relative)
            response = json.loads(path.read_text(encoding='utf-8'))
            edition = 'en' if 'support-en-' in path.name else 'fr'
            prefix, number = re.fullmatch(r'(.*)-(\d+)\.api\.json', path.name).groups()
            request_sets.setdefault(prefix, []).append((int(number), path, response))
            for page in response['query']['pages']:
                key = edition, page['title']
                names.setdefault(key, set()).update(person['name'] for person in page.get('contributors', []))
                if 'anoncontributors' in page:
                    anonymous[key] = page['anoncontributors']
        for parts in request_sets.values():
            parts.sort(key=lambda part: part[0])
            self.assertEqual([part[0] for part in parts], list(range(1, len(parts) + 1)))
            self.assertNotIn('continue', parts[-1][2])
            for (_, _, previous), (_, path, _) in zip(parts, parts[1:]):
                params = parse_qs(urlparse(path.with_suffix('.request.txt').read_text(encoding='utf-8')).query)
                for key, value in previous['continue'].items():
                    self.assertEqual(params[key], [str(value)])
        raw_lines = {}
        self.assertEqual(set(sources), set(captures))
        for source_id, source in sources.items():
            capture, context = captures[source_id], contexts[source_id]
            response = json.loads(capture_path(capture['api_path']).read_text(encoding='utf-8'))
            page = next(item for item in response['query']['pages'] if item['pageid'] == source['pageid'])
            revision = next(item for item in page['revisions'] if item['revid'] == source['revid'])
            self.assertEqual((page['title'], revision['timestamp']), (source['title'], source['timestamp']))
            body = revision['slots']['main']['content']
            self.assertEqual(body.encode('utf-8'), capture_path(capture['raw_path']).read_bytes())
            raw_lines[source_id] = body.splitlines()
            key = source['edition'], source['title']
            self.assertEqual(context['current_contributors'], sorted(names[key]))
            self.assertEqual(context['anonymous_contributors_count'], anonymous.get(key, 0))
            self.assertTrue(source['attribution'])
            for layer in source['original_historical_dictionary_layers']:
                self.assertEqual(layer['conservative_expiry'], '2006-01-01')
                self.assertEqual(layer['original_authors'], 'Académie française')
        for group in read('groups.json'):
            sense = group['senses'][0]
            for name, field in sense['fields'].items():
                with self.subTest(row=group['frozen_row'], field=name):
                    key = sense['sense_id'], name
                    exact, proposal, boundary = excerpts[key], proposals[key], boundaries[key]
                    line = raw_lines[field['source_id']][exact['line_1_based'] - 1]
                    column = exact['column_1_based'] - 1
                    self.assertEqual(line[column:column + len(exact['reviewed_excerpt'])], exact['reviewed_excerpt'])
                    self.assertEqual(field['source_span']['raw_source_excerpt'], exact['reviewed_excerpt'])
                    self.assertEqual(proposal['proposed_text'], field['text'])
                    self.assertEqual(boundary['exact_source_span'], field['source_span'])
                    self.assertEqual(boundary['independent_review'], 'passed_at_selected_text_boundary')
                    self.assertEqual(boundary['reviewer'], review['reviewer'])
                    self.assertTrue(boundary['author_rights_basis'])
                    self.assertTrue(field['modification'])
                    self.assertEqual(field['content_license'], 'CC-BY-SA-4.0')
                    if name == 'ipa':
                        self.assertEqual(field['text'].strip('/'), exact['reviewed_excerpt'])
                    else:
                        self.assertTrue(line.startswith('# '))
            for support in proposals[(sense['sense_id'], 'french_definition')]['supporting_sources']:
                self.assertEqual(support['reviewed_excerpt'], raw_lines[support['source_id']][support['line_1_based'] - 1])
        relations = {item['frozen_row']: item for item in review['spelling_relations']}
        for row in (2182, 2186, 2191, 2195, 2196, 2200):
            self.assertEqual(relations[row]['alias_admission'], 'not_admitted')
            self.assertTrue(relations[row]['proposed_evidence'])


if __name__ == "__main__":
    unittest.main()
