"""Check ticket #141 at the approved resource/admission seam."""
import csv
import json
from pathlib import Path
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs/evidence/issue-141"
MEMBERSHIP = [
    ("absent", "ADJ"), ("douceur", "NOM"), ("massacrer", "VER"),
    ("échantillon", "NOM"), ("recommander", "VER"), ("défoncer", "VER"),
    ("vendeur", "NOM"), ("couvent", "NOM"), ("solitaire", "ADJ"),
    ("ôter", "VER"), ("mineur", "NOM"), ("légal", "ADJ"),
    ("came", "NOM"), ("masse", "NOM"), ("splendide", "ADJ"),
    ("dialogue", "NOM"), ("missile", "NOM"), ("bien-aimé", "NOM"),
    ("dater", "VER"), ("marquis", "NOM"), ("spécialiste", "NOM"),
    ("baleine", "NOM"), ("exceptionnel", "ADJ"), ("habitant", "NOM"),
    ("langage", "NOM"), ("savon", "NOM"), ("polonais", "ADJ"),
    ("tissu", "NOM"), ("do", "NOM"), ("brigade", "NOM"),
    ("poumon", "NOM"), ("tente", "NOM"), ("villa", "NOM"),
    ("adjudant", "NOM"), ("séjour", "NOM"), ("tournage", "NOM"),
    ("las", "ADJ"), ("ski", "NOM"), ("motif", "NOM"),
    ("beau-père", "NOM"), ("dispute", "NOM"), ("gonfler", "VER"),
    ("issue", "NOM"), ("amateur", "NOM"), ("investir", "VER"),
    ("pâle", "ADJ"), ("accoucher", "VER"), ("basket", "NOM"),
    ("pâte", "NOM"), ("séduire", "VER"),
]
LANGUAGES = {"french_definition": "fr", "ipa": "und", "english": "en", "chinese": "zh"}


def read(name):
    return json.loads((EVIDENCE / name).read_text(encoding="utf-8"))


class Batch141Tests(unittest.TestCase):
    def test_work_crew_meanings_retain_the_certain_trades_restriction(self):
        group = next(g for g in read('groups.json') if g['headword'] == 'brigade')
        fields = group['senses'][0]['fields']
        self.assertIn('certains métiers', fields['french_definition']['text'])
        self.assertIn('certain trades', fields['english']['text'])
        self.assertIn('某些行业', fields['chinese']['text'])

    def test_exact_frozen_groups_have_complete_bound_candidate_content(self):
        with (ROOT / "resources/core-pilot-79/scope-manifest.csv").open(encoding="utf-8-sig") as handle:
            frozen = list(csv.DictReader(handle))[3000:3050]
        self.assertEqual([(r["word"], r["lexique_cgram"]) for r in frozen], MEMBERSHIP)
        groups = read("groups.json")
        self.assertEqual([(g["headword"], g["lexique_cgram"]) for g in groups], MEMBERSHIP)
        self.assertEqual([g["frozen_row"] for g in groups], list(range(3001, 3051)))
        sources = {s["id"]: s for s in read("sources.json")["sources"]}
        for group, row in zip(groups, frozen, strict=True):
            self.assertEqual(group["subtitle_lemma_per_million"], row["subtitle_lemma_per_million"])
            self.assertEqual(group["subtitle_pos_per_million"], row["subtitle_pos_per_million"])
            self.assertGreaterEqual(len(group["senses"]), 1)
            for sense in group["senses"]:
                self.assertTrue(sense["use"])
                self.assertEqual(sense["source_headword"], group["headword"])
                self.assertEqual(set(sense["fields"]), set(LANGUAGES))
                for name, field in sense["fields"].items():
                    self.assertEqual(field["language"], LANGUAGES[name])
                    self.assertTrue(field["text"].strip())
                    self.assertIn(field["source_id"], sources)
                    self.assertTrue(field["source_span"]["raw_source_excerpt"])
                    self.assertTrue(field["modification"])

    def test_accepted_fields_uses_and_rights_match_preserved_proposals(self):
        admission = read("admission.json")
        review = read("field-review.json")
        rights = read("rights.json")
        self.assertEqual(admission["scope_denominator_groups"], 4087)
        self.assertEqual(admission["proposed_fields"], 200)
        self.assertEqual(admission["independently_accepted_new_fields"], 200)
        self.assertEqual(admission["missing_admitted_fields"], 200)
        self.assertTrue(review["independent_acceptance"])
        self.assertIs(review['independence'], True)
        reviewer = review['reviewer']
        self.assertTrue(reviewer)
        fields = {(r["sense_id"], r["field"]): r for r in review["fields"]}
        legal_rows = {(r["sense_id"], r["field"]): r for r in rights["fields"]}
        self.assertEqual(len(fields), 200)
        self.assertEqual(set(fields), set(legal_rows))
        self.assertEqual(len(review["usage"]), 50)
        self.assertEqual(len(review["spelling_relations"]), 50)
        originals = {(s['sense_id'], name): field for g in read('authored/groups.json')
                     for s in g['senses'] for name, field in s['fields'].items()}
        uses = {u['sense_id']: u for u in review['usage']}
        relations = {r['sense_id']: r for r in review['spelling_relations']}
        sources = {s['id']: s for s in read('sources.json')['sources']}
        contexts = {s['id']: s for s in read('source-context.json')['current_content']}
        legal_sources = {s['source_id']: s for s in rights['sources']}
        changed = set()
        for group in read("groups.json"):
            for sense in group["senses"]:
                self.assertEqual(sense["independent_review"], "passed")
                use = uses[sense['sense_id']]
                self.assertEqual((use['headword'], use['pos'], use['frozen_row']),
                                 (group['headword'], group['lexique_cgram'], group['frozen_row']))
                self.assertEqual(use['accepted_usage'], sense['use'])
                self.assertEqual(use['accepted_usage_quality_status'], 'passed')
                self.assertEqual(use['everyday_sense_status'], 'passed')
                self.assertEqual(use['reviewer'], reviewer)
                self.assertTrue(use['review_reason'])
                self.assertTrue(use['selected_text_rights_reason'])
                relation = relations[sense['sense_id']]
                self.assertEqual((relation['frozen_headword'], relation['source_headword']),
                                 (group['headword'], sense['source_headword']))
                self.assertEqual(relation['independent_review'], 'passed')
                self.assertEqual(relation['alias_admission'], 'not_admitted')
                self.assertEqual(relation['reviewer'], reviewer)
                self.assertTrue(relation['review_reason'])
                for name, field in sense["fields"].items():
                    key = sense["sense_id"], name
                    rec, legal = fields[key], legal_rows[key]
                    self.assertEqual(rec['proposed_text'], originals[key]['text'])
                    self.assertEqual(rec['accepted_text'], field['text'])
                    self.assertEqual(legal['accepted_text'], field['text'])
                    self.assertEqual(rec['accepted_text_quality_status'], 'passed')
                    self.assertEqual(rec['rights_eligibility_status'], 'passed_at_selected_text_boundary')
                    for record in (rec, legal, field):
                        self.assertEqual(record['reviewer'], reviewer)
                        self.assertTrue(record['selected_text_rights_reason'])
                    self.assertTrue(rec['review_reason'])
                    self.assertEqual(field['independent_review'], 'passed')
                    source = sources[field['source_id']]
                    self.assertEqual((rec['exact_source']['source_id'], rec['exact_source']['revid'], rec['exact_source']['url']),
                                     (source['id'], source['revid'], source['fixed_url']))
                    self.assertEqual((rec['exact_source']['line_1_based'], rec['exact_source']['column_1_based'], rec['exact_source']['reviewed_excerpt']),
                                     (field['source_span']['line_1_based'], field['source_span']['column_1_based'], field['source_span']['raw_source_excerpt']))
                    self.assertEqual(legal['exact_source_span'], field['source_span'])
                    self.assertEqual(legal['source_revision'], source['revid'])
                    self.assertEqual(legal['fixed_url'], source['fixed_url'])
                    self.assertEqual(legal['modification'], field['modification'])
                    self.assertEqual(legal['source_id'], field['source_id'])
                    self.assertEqual(legal['license'], 'CC-BY-SA-4.0')
                    self.assertEqual(legal['independent_review'], 'passed_at_selected_text_boundary')
                    self.assertTrue(legal['attribution'])
                    if rec['proposed_text'] != rec['accepted_text']:
                        changed.add((group['frozen_row'], name))
        self.assertEqual(changed, {(3030, 'french_definition'), (3030, 'english'), (3030, 'chinese')})
        replacements = read('accepted-replacements.json')
        self.assertEqual({(r['frozen_row'], r['field']) for r in replacements}, changed)
        for replacement in replacements:
            rec = fields[replacement['sense_id'], replacement['field']]
            for key in ('proposed_text', 'accepted_text', 'exact_source'):
                self.assertEqual(replacement[key], rec[key])
        self.assertEqual(set(contexts), set(sources))
        self.assertEqual(set(legal_sources), set(sources))
        for sid, source in sources.items():
            self.assertEqual(source['selected_text_review'], 'passed_at_selected_text_boundary')
            self.assertEqual(contexts[sid]['current_content_revid'], source['revid'])
            self.assertEqual(contexts[sid]['selected_text_rights_status'], 'passed_at_selected_text_boundary')
            self.assertEqual(legal_sources[sid]['independent_review'], 'passed_at_selected_text_boundary')
            for record in (source, contexts[sid], legal_sources[sid]):
                self.assertEqual(record['reviewer'], reviewer)
                self.assertTrue(record['selected_text_rights_reason'])
        for support in rights['construction_sources']:
            self.assertEqual(support['independent_review'], 'passed_at_selected_text_boundary')
            self.assertTrue(support['selected_text_rights_reason'])
        self.assertEqual((admission['pending_independent_review_fields'], admission['pending_independent_rights_fields']), (0, 0))
        self.assertEqual((admission['newly_admitted_fields'], admission['pending_canonical_admission_fields']), (0, 200))
        self.assertEqual(review['unresolved_fields'], [])
        self.assertEqual(rights['independent_rights_fields_accepted'], 200)
        self.assertEqual((EVIDENCE / "COPYING").read_bytes().rstrip(),
                         (ROOT / "resources/core-pilot-79/COPYING").read_bytes().rstrip())
        self.assertTrue((EVIDENCE / "NOTICES.md").is_file())

    def test_originals_are_exact_final_author_files(self):
        snapshot = read('authored/snapshot.json')
        self.assertEqual(snapshot['author_head'], 'beb0ccf9a1996fef355742683f88d3ca1f86427b')
        for name in snapshot['files']:
            body = subprocess.check_output(['git', 'show', snapshot['author_head'] + ':docs/evidence/issue-141/' + name], cwd=ROOT)
            original = EVIDENCE / 'authored' / (name + '.txt' if name.endswith('.md') else name)
            self.assertEqual(original.read_bytes(), body)

    def test_reuse_is_zero_against_the_author_baseline(self):
        reuse = read('reuse.json')
        baseline = json.loads(subprocess.check_output(['git', 'show', reuse['base_commit'] + ':' + reuse['comparison_input']], cwd=ROOT))
        self.assertEqual(baseline['content_version'], reuse['canonical_content_version'])
        self.assertEqual(len(baseline['groups']), reuse['existing_admitted_groups'])
        selected = set(MEMBERSHIP)
        self.assertEqual([g for g in baseline['groups'] if (g['headword'], g['lexique_cgram']) in selected], [])
        self.assertEqual((reuse['reused_groups'], reuse['reused_fields'], reuse['groups']), (0, 0, []))


if __name__ == "__main__":
    unittest.main()
