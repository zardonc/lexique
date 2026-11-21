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


class Batch124Tests(unittest.TestCase):
    def test_exact_frozen_groups_have_four_candidates_and_honest_pending_admission(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(
            encoding="utf-8-sig", newline=""
        ) as handle:
            scope = list(csv.DictReader(handle))
        groups = read("groups.json")
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
        admission = read("admission.json")
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
        self.assertFalse(review['independent_acceptance'])
        self.assertIsNone(review['reviewer'])
        self.assertEqual(rights['independent_rights_fields_accepted'], 0)
        self.assertEqual(rights['independent_review'], 'pending')
        self.assertTrue(rights['redistribution_obligations'])
        for authority in rights['authority_captures']:
            self.assertTrue((ROOT / authority['path']).is_file())
        self.assertEqual((EVIDENCE / 'COPYING').read_bytes().rstrip(),
                         (ROOT / 'resources/core-pilot-79/COPYING').read_bytes().rstrip())
        registry = capture_document['contributor_registry']
        self.assertTrue(registry['pagination_complete'])
        names, anonymous, request_sets = {}, {}, {}
        for relative in registry['parts']:
            path = ROOT / relative
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
            response = json.loads((ROOT / capture['api_path']).read_text(encoding='utf-8'))
            page = next(item for item in response['query']['pages'] if item['pageid'] == source['pageid'])
            revision = next(item for item in page['revisions'] if item['revid'] == source['revid'])
            self.assertEqual((page['title'], revision['timestamp']), (source['title'], source['timestamp']))
            body = revision['slots']['main']['content']
            self.assertEqual(body.encode('utf-8'), (ROOT / capture['raw_path']).read_bytes())
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
                    self.assertEqual(boundary['independent_review'], 'pending')
                    self.assertIsNone(boundary['reviewer'])
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
